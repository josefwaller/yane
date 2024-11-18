use crate::{check_error, set_uniform, utils::*, NametableArrangement, Nes};
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

        // Gather information for rendering background
        let nametable = self.get_nametable(nes.ppu.get_base_nametable(), nes);
        let nametable_row_index = scanline / 8;
        let nametable_palettes: [i32; 32] = core::array::from_fn(|i| {
            // Get the X,Y coords of the 4x4 tile area whose palette is controlled by a single byte
            let area_x = i / 4;
            let area_y = nametable_row_index / 4;
            // Get the config byte
            let config_byte = nametable[0x3C0 + (8 * area_y + area_x)];
            // Get the specific 2 bit value that controls this tile's palette
            let x = (i / 2) % 2;
            let y = (nametable_row_index / 2) % 2;
            ((config_byte >> (2 * (2 * y + x))) & 0x03) as i32
        });

        // We build a big table to tiles to render, and then render them all in one draw_arrays_instanced call
        let nametable_pos: Vec<(i32, i32)> = (0..32)
            .map(|i| (8 * i as i32, 8 * nametable_row_index as i32))
            .collect();

        let nametable_patterns: Vec<i32> = (0..32)
            .map(|i| {
                nes.ppu.get_background_pattern_table_addr() as i32 / 0x10
                    + nametable[32 * nametable_row_index + i] as i32
            })
            .collect();

        // Gether OAM to render
        let sprite_height = if nes.ppu.is_8x16_sprites() { 16 } else { 8 };
        let oam_to_render: Vec<&[u8]> = nes
            .ppu
            .oam
            .chunks(4)
            .filter(|obj| obj[0] as usize <= scanline && obj[0] as usize + sprite_height > scanline)
            .take(8)
            .collect();
        let oam_pos: Vec<(i32, i32)> = oam_to_render
            .iter()
            .map(|obj| (obj[3] as i32, obj[0] as i32))
            .collect();
        let oam_patterns: Vec<i32> = oam_to_render
            .iter()
            .map(|obj| {
                if nes.ppu.is_8x16_sprites() {
                    (obj[1] & 0x01) as i32 * 0x100 + (obj[1] & 0xFE) as i32
                } else {
                    nes.ppu.get_spr_pattern_table_addr() as i32 / 0x10 + obj[1] as i32
                }
            })
            .collect();
        let oam_palettes: Vec<i32> = oam_to_render
            .iter()
            .map(|obj| 4 + (obj[2] & 0x03) as i32)
            .collect();
        let background_depths: Vec<f32> = (0..nametable_pos.len()).map(|_| 0.5).collect();
        let background_flip: Vec<i32> = (0..nametable_pos.len()).map(|_| 0).collect();
        let background_heights: Vec<i32> = (0..nametable_pos.len()).map(|_| 1).collect();
        let oam_depths: Vec<f32> = (0..oam_to_render.len()).map(|_| 0.3).collect();
        let oam_flip_horz = oam_to_render
            .iter()
            .map(|obj| if obj[2] & 0x80 != 0 { 1 } else { 0 })
            .collect();
        let oam_flip_vertz = oam_to_render
            .iter()
            .map(|obj| if obj[2] & 0x40 != 0 { 1 } else { 0 })
            .collect();
        let oam_heights: Vec<i32> = (0..oam_to_render.len())
            .map(|_| if nes.ppu.is_8x16_sprites() { 2 } else { 1 })
            .collect();
        // Get palette as RGB values
        let palette = nes.ppu.palette_ram.map(|i| self.palette[i as usize]);
        // Render tiles
        bulk_render_tiles(
            &self.gl,
            self.tile_program,
            self.chr_tex,
            self.tile_vao,
            [nametable_pos, oam_pos].concat(),
            [nametable_patterns, oam_patterns].concat(),
            [nametable_palettes.to_vec(), oam_palettes].concat(),
            &palette,
            scanline,
            [background_flip.clone(), oam_flip_horz].concat(),
            [background_flip, oam_flip_vertz].concat(),
            [background_depths, oam_depths].concat(),
            [background_heights, oam_heights].concat(),
        );

        // Todo: Check for sprite overflow
    }

    pub fn render(&mut self, nes: &Nes, window_size: (u32, u32)) {
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

            self.gl.finish();
            refresh_chr_texture(&self.gl, self.chr_tex, nes);
        }
    }
    fn get_nametable(&self, addr: usize, nes: &Nes) -> Vec<u8> {
        match nes.cartridge.nametable_arrangement {
            NametableArrangement::Horizontal => nes.ppu.nametable_ram[(addr
                % nes.ppu.nametable_ram.len())
                ..((addr % nes.ppu.nametable_ram.len()) + 0x400)]
                .to_vec(),
            NametableArrangement::Vertical => {
                nes.ppu.nametable_ram[(addr % 0x800)..((addr % 0x800) + 0x400)].to_vec()
            }
            _ => unimplemented!(
                "Mapper {:?} not implemented",
                nes.cartridge.nametable_arrangement
            ),
        }
    }
}
