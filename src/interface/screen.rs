use crate::{check_error, set_uniform, utils::*, Nes};
use glow::*;
use log::*;

// Renders the PPU
pub struct Screen {
    gl: Context,
    palette: [[f32; 3]; 0x40],
    chr_tex: NativeTexture,
    // Stuff for rendering to screen
    screen_fbo: NativeFramebuffer,
    screen_texture: NativeTexture,
    screen_program: NativeProgram,
    screen_vao: NativeVertexArray,
    // Stuff for rendering a bunch of tiles at once
    tile_program: NativeProgram,
    tile_vao: NativeVertexArray,
    // Stuff for rendering a scanline
    scanline_program: NativeProgram,
    scanline_vao: NativeVertexArray,
    // Program for rendering a primitve using wireframe
    wireframe_program: NativeProgram,
    wireframe_vao: NativeVertexArray,
}
impl Screen {
    // TODO: Rename
    pub fn new(nes: &Nes, gl: Context) -> Screen {
        unsafe {
            // Send CHR ROM/RAM data
            let chr_tex = gl.create_texture().unwrap();
            // Create tile program
            let tile_program = create_program(
                &gl,
                include_str!("../shaders/tiles.vert"),
                include_str!("../shaders/tile.frag"),
            );
            // Create tile vertices
            let tile_vao = create_f32_slice_vao(
                &gl,
                [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]].as_flattened(),
                2,
            );

            let (screen_fbo, screen_vao, screen_program, screen_texture) =
                create_screen_texture(&gl, (256, 240));
            // Load pallete data and convert to RGB values
            let palette_data: &[u8] = include_bytes!("../2C02G_wiki.pal");
            let palette: [[f32; 3]; 64] = core::array::from_fn(|i| {
                core::array::from_fn(|j| palette_data[3 * i + j] as f32 / 255.0)
            });
            // Create scanline program
            let scanline_program = create_program(
                &gl,
                include_str!("../shaders/scanline.vert"),
                include_str!("../shaders/color.frag"),
            );
            let scanline_vao =
                create_f32_slice_vao(&gl, [[0.0, 0.0], [1.0, 0.0]].as_flattened(), 2);

            let wireframe_program = create_program(
                &gl,
                include_str!("../shaders/wireframe.vert"),
                include_str!("../shaders/color.frag"),
            );
            let wireframe_vao = create_f32_slice_vao(
                &gl,
                [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]].as_flattened(),
                2,
            );

            Screen {
                gl,
                screen_fbo,
                screen_program,
                screen_texture,
                screen_vao,
                palette,
                chr_tex,
                tile_program,
                tile_vao,
                scanline_program,
                scanline_vao,
                wireframe_program,
                wireframe_vao,
            }
        }
    }

    pub unsafe fn render_scanline(&mut self, nes: &Nes, scanline: usize) {
        // Enable stencil test
        self.gl.enable(glow::STENCIL_TEST);
        self.gl.enable(glow::DEPTH_TEST);
        self.gl.depth_func(glow::ALWAYS);
        self.gl.depth_mask(true);
        check_error!(self.gl);
        self.gl.viewport(0, 0, 256, 240);
        self.gl
            .bind_framebuffer(glow::FRAMEBUFFER, Some(self.screen_fbo));
        // Set stencil buffer to only the one line
        self.gl.stencil_mask(0xFF);
        self.gl
            .clear(glow::STENCIL_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        check_error!(self.gl);
        self.gl.stencil_func(glow::ALWAYS, 1, 0xFF);
        self.gl.stencil_mask(0xFF);
        check_error!(self.gl);
        self.gl.stencil_op(glow::KEEP, glow::KEEP, glow::REPLACE);
        check_error!(self.gl);

        self.gl.use_program(Some(self.scanline_program));
        check_error!(self.gl);
        set_int_uniform(
            &self.gl,
            &self.scanline_program,
            "scanline",
            scanline as i32,
        );
        let clear_color = self.palette[(nes.ppu.palette_ram[0] & 0x3F) as usize];
        set_uniform!(
            self.gl,
            self.scanline_program,
            "inColor",
            uniform_3_f32_slice,
            &clear_color
        );
        self.gl.bind_vertex_array(Some(self.scanline_vao));
        check_error!(self.gl);
        self.gl.draw_arrays(glow::LINES, 0, 2);
        check_error!(self.gl);

        check_error!(self.gl);
        self.gl.enable(glow::STENCIL_TEST);
        self.gl.stencil_func(glow::EQUAL, 1, 0xFF);
        self.gl.stencil_op(glow::KEEP, glow::KEEP, glow::KEEP);
        self.gl.depth_func(glow::LESS);
        check_error!(self.gl);

        // We build a big table to tiles to render, and then render them all in one draw_arrays_instanced call
        const MAX_NUM_TILES: usize = 33 + 8;
        // The tiles' positions (each should have 2, X then Y)
        let mut positions: Vec<i32> = Vec::with_capacity(MAX_NUM_TILES);
        // The tiles' palette indices (each should have 1)
        let mut palette_indices: Vec<i32> = Vec::with_capacity(MAX_NUM_TILES);
        // The tiles' index, i.e. the index of the tile in the pattern table to draw
        let mut tile_index: Vec<i32> = Vec::with_capacity(MAX_NUM_TILES);
        // The tiles' depths, i.e. their Z index
        // Used to draw sprites on top of background or vice versa
        let mut depths: Vec<f32> = Vec::with_capacity(MAX_NUM_TILES);
        // Whether to flip each tile along the X or Y axis
        let mut flip_x: Vec<i32> = Vec::with_capacity(MAX_NUM_TILES);
        let mut flip_y: Vec<i32> = Vec::with_capacity(MAX_NUM_TILES);
        // The sprite height (1 = 8px, 2 = 16px)
        // We can't have this be constant across all tiles since even in 8x16 mode, the background tiles are 8x8
        let mut heights: Vec<i32> = Vec::with_capacity(MAX_NUM_TILES);
        // Gather information for rendering background
        if nes.ppu.is_background_rendering_enabled() {
            const NUM_NAMETABLE_TILES: usize = 33;
            let y = (scanline + nes.ppu.scroll_y as usize) / 8;
            let (left_nametable_addr, right_nametable_addr) = if y < 30 {
                (
                    nes.ppu.top_left_nametable_addr(),
                    nes.ppu.top_right_nametable_addr(),
                )
            } else {
                (
                    nes.ppu.bot_left_nametable_addr(),
                    nes.ppu.bot_right_nametable_addr(),
                )
            };
            let nametable_row_index = if y < 30 { y } else { y - 30 };
            // Get the left and right nametables
            // We can account for the scroll Y offset when selecting the nametables
            // But we need both in order to account for the scroll X
            let left_nametable = self.get_nametable(left_nametable_addr, nes);
            let right_nametable = self.get_nametable(right_nametable_addr, nes);
            // Add background positions
            // We need to wrap around the current scanline
            // Basically draw the tile somewhere in [scanline - 7, scanline), wherever a tile aligned on the grid would be
            let actual_tile_pos = 8 * (scanline as i32 / 8);
            let fine_scroll_y = nes.ppu.scroll_y as i32 % 8;
            let pos_y = if actual_tile_pos - fine_scroll_y < scanline as i32 - 7 {
                actual_tile_pos + 8 - fine_scroll_y
            } else {
                actual_tile_pos - fine_scroll_y
            };
            (0..NUM_NAMETABLE_TILES)
                .map(|i| [8 * i as i32 - (nes.ppu.scroll_x % 8) as i32, pos_y])
                .flatten()
                .for_each(|p| positions.push(p));
            // Add background patterns
            (0..NUM_NAMETABLE_TILES).for_each(|i| {
                let x = nes.ppu.scroll_x as usize / 8 + i;
                let (nametable, index) = if x < 32 {
                    (&left_nametable, x)
                } else {
                    (&right_nametable, x - 32)
                };
                tile_index.push(
                    nes.ppu.get_background_pattern_table_addr() as i32 / 0x10
                        + nametable[32 * nametable_row_index + index] as i32,
                );
            });
            // Add pattern nums
            (0..NUM_NAMETABLE_TILES).for_each(|i| {
                let x = nes.ppu.scroll_x as usize / 8 + i;
                // Get the config byte
                let (nametable, index) = if x < 32 {
                    (&left_nametable, x)
                } else {
                    (&right_nametable, x - 32)
                };
                // Get the X,Y coords of the 4x4 tile area whose palette is controlled by a single byte
                let area_x = index / 4;
                let area_y = nametable_row_index / 4;
                let config_byte = nametable[0x3C0 + 8 * area_y + area_x];
                // Get the specific 2 bit value that controls this tile's palette
                let x = (index / 2) % 2;
                let y = (nametable_row_index / 2) % 2;
                palette_indices.push(((config_byte >> (2 * (2 * y + x))) & 0x03) as i32);
            });
            // Add misc, simple settings
            (0..NUM_NAMETABLE_TILES).for_each(|_| {
                depths.push(0.5);
                flip_x.push(0);
                flip_y.push(0);
                heights.push(1);
            });
        }
        // Gether OAM to render
        if nes.ppu.is_oam_rendering_enabled() {
            let sprite_height = if nes.ppu.is_8x16_sprites() { 16 } else { 8 };
            let oam_to_render: Vec<&[u8]> = nes
                .ppu
                .oam
                .chunks(4)
                .filter(|obj| {
                    obj[0] as usize <= scanline && obj[0] as usize + sprite_height > scanline
                })
                .take(8)
                .collect();
            oam_to_render
                .iter()
                .map(|obj| [obj[3] as i32, obj[0] as i32])
                .flatten()
                .for_each(|p| positions.push(p));
            oam_to_render.iter().for_each(|obj| {
                tile_index.push(if nes.ppu.is_8x16_sprites() {
                    (obj[1] & 0x01) as i32 * 0x100 + (obj[1] & 0xFE) as i32
                } else {
                    nes.ppu.get_spr_pattern_table_addr() as i32 / 0x10 + obj[1] as i32
                });
            });
            oam_to_render
                .iter()
                .for_each(|obj| palette_indices.push(4 + (obj[2] & 0x03) as i32));
            oam_to_render.iter().enumerate().for_each(|(i, obj)| {
                depths.push(if obj[2] & 0x20 != 0 { 0.7 } else { 0.3 } + (i as f32 / 64.0) / 100.0);
                flip_x.push(if obj[2] & 0x80 != 0 { 1 } else { 0 });
                flip_y.push(if obj[2] & 0x40 != 0 { 1 } else { 0 });
                heights.push(if nes.ppu.is_8x16_sprites() { 2 } else { 1 });
            });
        }
        // Get palette as RGB values
        let palette = nes.ppu.palette_ram.map(|i| self.palette[i as usize]);
        // Render tiles
        bulk_render_tiles(
            &self.gl,
            self.tile_program,
            self.chr_tex,
            self.tile_vao,
            positions,
            tile_index,
            palette_indices,
            &palette,
            scanline,
            flip_x,
            flip_y,
            depths,
            heights,
        );

        // Todo: Check for sprite overflow
    }

    pub fn render(&mut self, nes: &Nes, window_size: (u32, u32), debug_oam: bool) {
        unsafe {
            self.gl.disable(glow::STENCIL_TEST);
            self.gl.disable(glow::DEPTH_TEST);

            self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
            check_error!(self.gl);
            self.gl.use_program(Some(self.screen_program));
            const TEX_NUM: i32 = 1;
            self.gl.active_texture(glow::TEXTURE0 + TEX_NUM as u32);
            self.gl
                .bind_texture(glow::TEXTURE_2D, Some(self.screen_texture));
            let loc = self
                .gl
                .get_uniform_location(self.screen_program, "renderedTexture");
            self.gl.uniform_1_i32(loc.as_ref(), TEX_NUM);
            check_error!(self.gl);
            self.gl
                .viewport(0, 0, window_size.0 as i32, window_size.1 as i32);
            check_error!(self.gl);
            self.gl.bind_vertex_array(Some(self.screen_vao));
            check_error!(self.gl);
            self.gl.draw_arrays(glow::TRIANGLES, 0, 6);
            check_error!(self.gl);

            // Render wireframe box around each OAM object
            if debug_oam {
                self.gl.use_program(Some(self.wireframe_program));
                self.gl.bind_vertex_array(Some(self.wireframe_vao));
                check_error!(self.gl);
                nes.ppu.oam.chunks(4).for_each(|obj| {
                    set_uniform!(
                        self.gl,
                        self.wireframe_program,
                        "position",
                        uniform_2_f32,
                        obj[3] as f32,
                        obj[0] as f32
                    );
                    set_uniform!(
                        self.gl,
                        self.wireframe_program,
                        "inColor",
                        uniform_3_f32,
                        1.0,
                        0.0,
                        0.0
                    );
                    set_uniform!(
                        self.gl,
                        self.wireframe_program,
                        "sizes",
                        uniform_2_f32,
                        1.0,
                        if nes.ppu.is_8x16_sprites() { 2.0 } else { 1.0 }
                    );
                    self.gl.draw_arrays(glow::LINE_LOOP, 0, 4);
                });
            }
            refresh_chr_texture(&self.gl, self.chr_tex, nes);
        }
    }
    // This function should probably be moved into the PPU somewhere
    // Or perhaps the cartridge
    fn get_nametable(&self, addr: usize, nes: &Nes) -> Vec<u8> {
        nes.ppu.nametable_ram[nes.cartridge.transform_nametable_addr(addr)
            ..=nes.cartridge.transform_nametable_addr(addr + 0x3FF)]
            .to_vec()
    }
}
