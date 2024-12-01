use crate::{check_error, set_uniform, utils::*, Nes, Settings};
use glow::*;
use log::*;

/// An NES rendering implementation that uses OpenGL 3.3.
/// Uses the `glow` library to render, so requires a `glow` `Context`.
/// Can be paired with a `Window` to render to an SDL2 window.
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
    pub fn new(gl: Context) -> Screen {
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

    pub unsafe fn render_scanline(&mut self, nes: &Nes, scanline: usize, settings: &Settings) {
        // Enable stencil test
        self.gl.enable(glow::STENCIL_TEST);
        self.gl.enable(glow::DEPTH_TEST);
        // Initially set depth testing to always to write the scanline value
        self.gl.depth_func(glow::ALWAYS);
        self.gl.depth_mask(true);
        check_error!(self.gl);
        // Render the background as a line, while also setting initial depth value and stencil value
        self.gl.viewport(0, 0, 256, 240);
        self.gl
            .bind_framebuffer(glow::FRAMEBUFFER, Some(self.screen_fbo));
        // Set stencil buffer to only the one line
        self.gl
            .clear(glow::STENCIL_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        check_error!(self.gl);
        self.gl.stencil_func(glow::ALWAYS, 0x03, 0xFF);
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

        // Enable depth testing
        self.gl.depth_func(glow::LESS);
        check_error!(self.gl);

        // Get palette as RGB values
        let palette = &nes.ppu.palette_ram.map(|i| self.palette[i as usize]);
        // Gether OAM to render
        if nes.ppu.is_oam_rendering_enabled() {
            let sprite_height = if nes.ppu.is_8x16_sprites() { 16 } else { 8 };
            let sprite_limit = if settings.scanline_sprite_limit {
                8
            } else {
                64
            };
            let oam_to_render: Vec<&[u8]> = nes
                .ppu
                .oam
                .chunks(4)
                .filter(|obj| {
                    // Sprite rendering is offset by 1px
                    obj[0] as usize + 1 <= scanline
                        && obj[0] as usize + 1 + sprite_height > scanline
                })
                .take(sprite_limit)
                .collect();
            let positions = oam_to_render
                .iter()
                .map(|obj| [obj[3] as i32, obj[0] as i32 + 1])
                .flatten()
                .collect();
            let tile_index = oam_to_render
                .iter()
                .map(|obj| {
                    if nes.ppu.is_8x16_sprites() {
                        (obj[1] & 0x01) as i32 * 0x100 + (obj[1] & 0xFE) as i32
                    } else {
                        nes.ppu.get_spr_pattern_table_addr() as i32 / 0x10 + obj[1] as i32
                    }
                })
                .collect();
            let palette_indices = oam_to_render
                .iter()
                .map(|obj| 4 + (obj[2] & 0x03) as i32)
                .collect();
            let depths = oam_to_render
                .iter()
                .enumerate()
                .map(|(i, obj)| {
                    return if obj[2] & 0x20 != 0 && !settings.always_sprites_on_top {
                        // Behind background
                        0.7
                    } else {
                        // In front of background
                        0.3
                    };
                    //+ ((64.0 - i as f32) / 64.0) / 10.0;
                })
                .collect();
            let flip_x = oam_to_render
                .iter()
                .map(|obj| if obj[2] & 0x80 != 0 { 1 } else { 0 })
                .collect();
            let flip_y = oam_to_render
                .iter()
                .map(|obj| if obj[2] & 0x40 != 0 { 1 } else { 0 })
                .collect();
            // Now we use the 1st (not 0th) bit of the stencil buffer for sprite priority
            // If the sprites are behind the background but have priority over other sprites, we want to draw the background
            // So we draw all the sprites, using bit 1 of the stencil buffer as a mask to make sure we don't draw over existing sprites
            // And then use depth testing to draw the background in front/behind.
            self.gl.stencil_func(glow::EQUAL, 3, 0x02);
            check_error!(self.gl);
            // On pass, write 0 but only write to the 1st bit
            // So essentially clear the 1st bit
            self.gl.stencil_op(glow::KEEP, glow::KEEP, glow::ZERO);
            self.gl.stencil_mask(0x02);
            check_error!(self.gl);
            // Render tiles
            self.bulk_render_tiles(
                positions,
                tile_index,
                palette_indices,
                palette,
                scanline,
                flip_x,
                flip_y,
                depths,
                if nes.ppu.is_8x16_sprites() { 2 } else { 1 },
                settings,
            );
        }
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
            let positions = (0..NUM_NAMETABLE_TILES)
                .map(|i| [8 * i as i32 - (nes.ppu.scroll_x % 8) as i32, pos_y])
                .flatten()
                .collect();
            // Add background patterns
            let tile_index = (0..NUM_NAMETABLE_TILES)
                .map(|i| {
                    let x = nes.ppu.scroll_x as usize / 8 + i;
                    let (nametable, index) = if x < 32 {
                        (&left_nametable, x)
                    } else {
                        (&right_nametable, x - 32)
                    };
                    nes.ppu.get_background_pattern_table_addr() as i32 / 0x10
                        + nametable[32 * nametable_row_index + index] as i32
                })
                .collect();
            // Add palette indexes
            let palette_indices = (0..NUM_NAMETABLE_TILES)
                .map(|i| {
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
                    ((config_byte >> (2 * (2 * y + x))) & 0x03) as i32
                })
                .collect();
            // Add misc, simple settings
            let depths = vec![0.5; NUM_NAMETABLE_TILES];
            // We never flip background tiles either horizontally or vertically
            let flip = vec![0; NUM_NAMETABLE_TILES];
            // Use the 0th bit as the mask for the background
            // This bit should be set on this scanline and unaffected by the sprite masking stuff we do above
            self.gl.stencil_func(glow::EQUAL, 1, 0x01);
            self.gl.stencil_mask(0xFF);
            self.gl.stencil_op(glow::KEEP, glow::KEEP, glow::KEEP);
            self.bulk_render_tiles(
                positions,
                tile_index,
                palette_indices,
                palette,
                scanline,
                flip.clone(),
                flip,
                depths,
                8,
                settings,
            );
        }
    }

    pub fn render(&mut self, nes: &Nes, window_size: (u32, u32), settings: &Settings) {
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
            if settings.oam_debug {
                self.gl.use_program(Some(self.wireframe_program));
                self.gl.bind_vertex_array(Some(self.wireframe_vao));
                check_error!(self.gl);
                nes.ppu.oam.chunks(4).enumerate().for_each(|(i, obj)| {
                    set_uniform!(
                        self.gl,
                        self.wireframe_program,
                        "position",
                        uniform_2_f32,
                        obj[3] as f32,
                        obj[0] as f32 + 1.0
                    );
                    let color = if i == 0 {
                        [0.0, 1.0, 0.0]
                    } else {
                        [1.0, 0.0, 0.0]
                    };
                    set_uniform!(
                        self.gl,
                        self.wireframe_program,
                        "inColor",
                        uniform_3_f32,
                        color[0],
                        color[1],
                        color[2]
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
            let chr: Vec<u8> = (0..0x2000).map(|i| nes.cartridge.read_ppu(i)).collect();
            refresh_chr_texture(&self.gl, self.chr_tex, nes, chr);
        }
    }
    /// Bulk render a bunch of tiles in one single draw_arrays_instanced call
    unsafe fn bulk_render_tiles(
        &self,
        // The tiles' positions (each should have 2, X then Y)
        pos: Vec<i32>,
        // The tiles' index, i.e. the index of the tile in the pattern table to draw
        pattern_nums: Vec<i32>,
        // The tiles' palette indices (each should have 1)
        palette_indices: Vec<i32>,
        palette: &[[f32; 3]; 0x20],
        scanline: usize,
        // Whether to flip each tile along the X or Y axis
        flip_vert: Vec<i32>,
        flip_horz: Vec<i32>,
        // The tiles' depths, i.e. their Z index
        // Used to draw sprites on top of background or vice versa
        depths: Vec<f32>,
        // The sprite height (1 = 8px, 2 = 16px)
        height: i32,
        settings: &Settings,
    ) {
        #[cfg(debug_assertions)]
        {
            // Make sure everything is the same length
            assert_eq!(pos.len(), 2 * pattern_nums.len());
            assert_eq!(pattern_nums.len(), palette_indices.len());
            assert_eq!(palette_indices.len(), flip_vert.len());
            assert_eq!(flip_vert.len(), flip_horz.len());
            // Make sure all the booleans are valid
            [&flip_horz, &flip_vert].iter().for_each(|v| {
                v.iter().for_each(|val| {
                    assert!(
                        *val == 0 || *val == 1,
                        "Invalid boolean value passed to bulk_render_tiles"
                    )
                })
            });
        }
        self.gl.use_program(Some(self.tile_program));
        self.gl.bind_vertex_array(Some(self.tile_vao));
        // Set texture
        const TEX_NUM: i32 = 2;
        self.gl.active_texture(glow::TEXTURE0 + TEX_NUM as u32);
        check_error!(self.gl);
        self.gl.bind_texture(glow::TEXTURE_2D, Some(self.chr_tex));
        check_error!(self.gl);
        set_uniform!(self.gl, self.tile_program, "chrTex", uniform_1_i32, TEX_NUM);
        set_uniform!(
            self.gl,
            self.tile_program,
            "patternIndices",
            uniform_1_i32_slice,
            pattern_nums.as_slice()
        );

        // Set position
        set_uniform!(
            self.gl,
            self.tile_program,
            "positions",
            uniform_2_i32_slice,
            pos.as_slice()
        );
        set_int_uniform(&self.gl, &self.tile_program, "scanline", scanline as i32);
        set_uniform!(
            self.gl,
            self.tile_program,
            "paletteIndices",
            uniform_1_i32_slice,
            palette_indices.as_slice()
        );
        let debug_pal: Vec<f32> = (0..8)
            .map(|_| debug_palette().into_flattened())
            .flatten()
            .collect();
        let final_palette = if settings.palette_debug {
            debug_pal.as_slice()
        } else {
            palette.as_flattened()
        };

        set_uniform!(
            self.gl,
            self.tile_program,
            "palette",
            uniform_3_f32_slice,
            final_palette
        );
        set_uniform!(
            self.gl,
            self.tile_program,
            "depths",
            uniform_1_f32_slice,
            depths.as_slice()
        );
        set_uniform!(
            self.gl,
            self.tile_program,
            "flipHorizontal",
            uniform_1_i32_slice,
            flip_horz.as_slice()
        );
        set_uniform!(
            self.gl,
            self.tile_program,
            "flipVertical",
            uniform_1_i32_slice,
            flip_vert.as_slice()
        );
        set_uniform!(self.gl, self.tile_program, "height", uniform_1_i32, height);
        self.gl
            .draw_arrays_instanced(glow::TRIANGLE_STRIP, 0, 4, pattern_nums.len() as i32);
    }
    // This function should probably be moved into the PPU somewhere
    // Or perhaps the cartridge
    fn get_nametable(&self, addr: usize, nes: &Nes) -> Vec<u8> {
        nes.ppu.nametable_ram[nes.cartridge.transform_nametable_addr(addr)
            ..=nes.cartridge.transform_nametable_addr(addr + 0x3FF)]
            .to_vec()
    }
}
