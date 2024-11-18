use crate::{check_error, utils::*, NametableArrangement, Nes};
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
    // Stuff for rendering the background
    background_program: NativeProgram,
    background_vao: NativeVertexArray,
    // Stuff for rendering a single tile
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

            let background_program = gl.create_program().expect("Unable to create program!");
            compile_and_link_shader(
                &gl,
                glow::VERTEX_SHADER,
                include_str!("../shaders/background.vert"),
                &background_program,
            );
            compile_and_link_shader(
                &gl,
                glow::FRAGMENT_SHADER,
                include_str!("../shaders/tile.frag"),
                &background_program,
            );

            gl.link_program(background_program);
            if !gl.get_program_link_status(background_program) {
                panic!(
                    "Couldn't link program: {}",
                    gl.get_program_info_log(background_program)
                );
            }
            let verts: [[f32; 8]; 64] =
                core::array::from_fn(|_| [0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0]);
            let background_vao = create_f32_slice_vao(&gl, verts.as_flattened(), 2);

            let (screen_fbo, screen_vao, screen_program, screen_texture) =
                create_screen_texture(&gl, (256, 240));
            // Load pallete
            let palette_data: &[u8] = include_bytes!("../2C02G_wiki.pal");
            let palette: [[f32; 3]; 64] = core::array::from_fn(|i| {
                core::array::from_fn(|j| palette_data[3 * i + j] as f32 / 255.0)
            });

            let e = gl.get_error();
            if e != glow::NO_ERROR {
                panic!("Error generated sometime during initialization: {:X?}", e);
            }

            let tile_program = gl.create_program().unwrap();
            check_error!(gl);
            compile_and_link_shader(
                &gl,
                glow::VERTEX_SHADER,
                include_str!("../shaders/tile.vert"),
                &tile_program,
            );
            compile_and_link_shader(
                &gl,
                glow::FRAGMENT_SHADER,
                include_str!("../shaders/tile.frag"),
                &tile_program,
            );
            gl.link_program(tile_program);
            if !gl.get_program_link_status(tile_program) {
                panic!(
                    "Couldn't link program: {}",
                    gl.get_program_info_log(tile_program)
                );
            }
            gl.use_program(Some(tile_program));
            let tile_vao = create_f32_slice_vao(
                &gl,
                [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]].as_flattened(),
                2,
            );

            let scanline_program = gl.create_program().unwrap();
            compile_and_link_shader(
                &gl,
                glow::VERTEX_SHADER,
                include_str!("../shaders/scanline.vert"),
                &scanline_program,
            );
            compile_and_link_shader(
                &gl,
                glow::FRAGMENT_SHADER,
                include_str!("../shaders/color.frag"),
                &scanline_program,
            );
            gl.link_program(scanline_program);
            if !gl.get_program_link_status(scanline_program) {
                panic!(
                    "Couldn't link program: {}",
                    gl.get_program_info_log(scanline_program)
                );
            }
            let scanline_vao =
                create_f32_slice_vao(&gl, [[0.0, 0.0], [1.0, 0.0]].as_flattened(), 2);

            let wireframe_program = gl.create_program().unwrap();
            compile_and_link_shader(
                &gl,
                glow::VERTEX_SHADER,
                include_str!("../shaders/tile.vert"),
                &wireframe_program,
            );
            compile_and_link_shader(
                &gl,
                glow::FRAGMENT_SHADER,
                include_str!("../shaders/color.frag"),
                &wireframe_program,
            );
            gl.link_program(wireframe_program);
            if !gl.get_program_link_status(wireframe_program) {
                panic!(
                    "Couldn't link program: {}",
                    gl.get_program_info_log(wireframe_program)
                );
            }
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
                background_program,
                background_vao,
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
        let loc = self
            .gl
            .get_uniform_location(self.scanline_program, "inColor");
        self.gl.uniform_3_f32_slice(loc.as_ref(), &clear_color);
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

        // Render background
        let nametable = self.get_nametable(nes.ppu.get_base_nametable(), nes);
        let nametable_row_index = scanline / 8;
        self.gl.use_program(Some(self.background_program));
        self.gl.bind_vertex_array(Some(self.background_vao));
        // Set texture
        const TEX_NUM: i32 = 2;
        self.gl.active_texture(glow::TEXTURE0 + TEX_NUM as u32);
        check_error!(self.gl);
        self.gl.bind_texture(glow::TEXTURE_2D, Some(self.chr_tex));
        check_error!(self.gl);
        let loc = self
            .gl
            .get_uniform_location(self.background_program, "chrTex");
        check_error!(self.gl);
        self.gl.uniform_1_i32(loc.as_ref(), TEX_NUM);
        check_error!(self.gl);
        // Set position
        set_int_uniform(
            &self.gl,
            &self.background_program,
            "scanline",
            scanline as i32,
        );
        let palette_indices: [i32; 32] = core::array::from_fn(|i| {
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
        let palette = nes.ppu.palette_ram.map(|i| self.palette[i as usize]);

        // We build a big table to tiles to render, and then render them all in one draw_arrays_instanced call
        let background_pos: Vec<(i32, i32)> = (0..32)
            .map(|i| (8 * i as i32, 8 * nametable_row_index as i32))
            .collect();

        let tiles_pattern_num: Vec<i32> = (0..32)
            .map(|i| {
                nes.ppu.get_background_pattern_table_addr() as i32 / 0x10
                    + nametable[32 * nametable_row_index + i] as i32
            })
            .collect();

        self.gl.use_program(Some(self.tile_program));
        check_error!(self.gl);
        // Set texture
        let loc = self.gl.get_uniform_location(self.tile_program, "chrTex");
        self.gl.uniform_1_i32(loc.as_ref(), TEX_NUM);

        // Render OAM
        let oam_to_render: Vec<&[u8]> = nes
            .ppu
            .oam
            .chunks(4)
            .filter(|obj| obj[0] as usize <= scanline && obj[0] as usize + 8 >= scanline)
            .take(8)
            .collect();
        oam_to_render.iter().take(8).for_each(|obj| {
            // self.render_tile(
            //     obj[3] as usize,
            //     obj[0] as usize,
            //     obj[1] as usize,
            //     4 + (obj[2] & 0x03) as usize,
            //     (obj[2] & 0x80) != 0,
            //     (obj[2] & 0x40) != 0,
            //     &palette,
            // );
        });
        let oam_pos: Vec<(i32, i32)> = oam_to_render
            .iter()
            .map(|obj| (obj[3] as i32, obj[0] as i32))
            .collect();
        let oam_pattern: Vec<i32> = oam_to_render
            .iter()
            .map(|obj| nes.ppu.get_spr_pattern_table_addr() as i32 / 0x10 + obj[1] as i32)
            .collect();
        let oam_palettes: Vec<i32> = oam_to_render
            .iter()
            .map(|obj| 4 + (obj[2] & 0x03) as i32)
            .collect();
        let background_depths: Vec<f32> = (0..background_pos.len()).map(|_| 0.5).collect();
        let background_flip: Vec<i32> = (0..background_pos.len()).map(|_| 0).collect();
        let oam_depths: Vec<f32> = (0..oam_to_render.len()).map(|_| 0.3).collect();
        let oam_flip_horz = oam_to_render
            .iter()
            .map(|obj| if obj[2] & 0x80 != 0 { 1 } else { 0 })
            .collect();
        let oam_flip_vertz = oam_to_render
            .iter()
            .map(|obj| if obj[2] & 0x40 != 0 { 1 } else { 0 })
            .collect();
        self.bulk_render_tiles(
            [background_pos, oam_pos].concat(),
            [tiles_pattern_num, oam_pattern].concat(),
            [palette_indices.to_vec(), oam_palettes].concat(),
            &palette,
            scanline,
            [background_flip.clone(), oam_flip_horz].concat(),
            [background_flip, oam_flip_vertz].concat(),
            [background_depths, oam_depths].concat(),
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
                let loc = self
                    .gl
                    .get_uniform_location(self.wireframe_program, "position");
                self.gl
                    .uniform_2_f32(loc.as_ref(), obj[3] as f32, obj[0] as f32);
                let loc = self
                    .gl
                    .get_uniform_location(self.wireframe_program, "inColor");
                self.gl.uniform_3_f32(loc.as_ref(), 1.0, 0.0, 0.0);
                self.gl.draw_arrays(glow::LINE_LOOP, 0, 4);
            });

            self.gl.finish();
            self.refresh_chr_texture(nes);
        }
    }
    unsafe fn bulk_render_tiles(
        &self,
        pos: Vec<(i32, i32)>,
        pattern_nums: Vec<i32>,
        palette_indices: Vec<i32>,
        palette: &[[f32; 3]; 0x20],
        scanline: usize,
        flip_vert: Vec<i32>,
        flip_horz: Vec<i32>,
        depths: Vec<f32>,
    ) {
        #[cfg(debug_assertions)]
        {
            // Make sure everything is the same length
            assert_eq!(pos.len(), pattern_nums.len());
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
        self.gl.use_program(Some(self.background_program));
        let loc = self
            .gl
            .get_uniform_location(self.background_program, "patternIndices");
        let row = pattern_nums;
        // let row = vec![0; 32];
        self.gl.uniform_1_i32_slice(loc.as_ref(), row.as_slice());
        check_error!(self.gl);

        // Set position
        let loc = self
            .gl
            .get_uniform_location(self.background_program, "positions");
        let temp_pos: Vec<i32> = pos.iter().map(|a| [a.0, a.1]).flatten().collect();
        self.gl
            .uniform_2_i32_slice(loc.as_ref(), temp_pos.as_slice());
        check_error!(self.gl);
        set_int_uniform(
            &self.gl,
            &self.background_program,
            "scanline",
            scanline as i32,
        );
        let loc = self
            .gl
            .get_uniform_location(self.background_program, "paletteIndices");
        self.gl.uniform_1_i32_slice(loc.as_ref(), &palette_indices);
        let loc = self
            .gl
            .get_uniform_location(self.background_program, "palette");
        self.gl
            .uniform_3_f32_slice(loc.as_ref(), palette.as_flattened());
        let loc = self
            .gl
            .get_uniform_location(self.background_program, "depths");
        self.gl.uniform_1_f32_slice(loc.as_ref(), &depths);
        let loc = self
            .gl
            .get_uniform_location(self.background_program, "flipHorizontal");
        self.gl
            .uniform_1_i32_slice(loc.as_ref(), flip_horz.as_slice());
        let loc = self
            .gl
            .get_uniform_location(self.background_program, "flipVertical");
        self.gl
            .uniform_1_i32_slice(loc.as_ref(), flip_vert.as_slice());
        self.gl
            .draw_arrays_instanced(glow::TRIANGLE_STRIP, 0, 4, pos.len() as i32);
    }

    unsafe fn render_tile(
        &self,
        x: usize,
        y: usize,
        tile_addr: usize,
        palette_index: usize,
        flip_vert: bool,
        flip_horz: bool,
        palette: &[[f32; 3]; 0x20],
    ) {
        // Set position
        let loc = self.gl.get_uniform_location(self.tile_program, "position");
        self.gl.uniform_2_f32(loc.as_ref(), x as f32, y as f32);
        // Set address
        set_int_uniform(&self.gl, &self.tile_program, "tileIndex", tile_addr as i32);
        let loc = self.gl.get_uniform_location(self.tile_program, "palette");
        self.gl
            .uniform_3_f32_slice(loc.as_ref(), palette.as_flattened());
        set_bool_uniform(&self.gl, &self.tile_program, "flipVertical", flip_vert);
        set_bool_uniform(&self.gl, &self.tile_program, "flipHorizontal", flip_horz);
        set_int_uniform(
            &self.gl,
            &self.tile_program,
            "oamPaletteIndex",
            palette_index as i32,
        );
        self.gl.bind_vertex_array(Some(self.tile_vao));
        self.gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
    }
    unsafe fn refresh_chr_texture(&mut self, nes: &Nes) {
        let pattern_table = nes.cartridge.get_pattern_table();
        let texture_data: Vec<u8> = pattern_table
            .chunks(16)
            .map(|sprite_data| {
                (0..(8 * 8)).map(|i| {
                    let less_sig = (sprite_data[i / 8] >> (7 - i % 8)) & 0x01;
                    let more_sig = (sprite_data[i / 8 + 8] >> (7 - i % 8)) & 0x01;
                    2 * more_sig + less_sig
                })
            })
            .flatten()
            .collect();
        self.gl.bind_texture(glow::TEXTURE_2D, Some(self.chr_tex));
        check_error!(self.gl);
        // Generate a texture 8 pixels long to use for the CHR ROM/RAM
        let width = 8;
        self.gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::R8 as i32,
            width,
            texture_data.len() as i32 / width,
            0,
            glow::RED,
            glow::UNSIGNED_BYTE,
            Some(&texture_data),
        );
        check_error!(self.gl);
        self.gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::NEAREST as i32,
        );
        check_error!(self.gl);
        self.gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::NEAREST as i32,
        );
        check_error!(self.gl);
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
