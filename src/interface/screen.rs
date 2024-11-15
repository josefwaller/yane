use crate::{check_error, interface::window, utils::*, NametableArrangement, Nes};
use glow::*;
use log::*;

// Renders the PPU
pub struct Screen {
    gl: Context,
    sprite_program: NativeProgram,
    vao_array: [NativeVertexArray; 64],
    // Stuff for rendering to screen
    screen_fbo: NativeFramebuffer,
    screen_texture: NativeTexture,
    screen_program: NativeProgram,
    screen_vao: NativeVertexArray,
    palette: [[f32; 3]; 0x40],
    background_program: NativeProgram,
    background_vao: NativeVertexArray,
    tile_program: NativeProgram,
    tile_vao: NativeVertexArray,
    scanline_program: NativeProgram,
    scanline_vao: NativeVertexArray,
    chr_tex: NativeTexture,
}
impl Screen {
    // TODO: Rename
    pub fn new(nes: &Nes, gl: Context) -> Screen {
        unsafe {
            gl.disable(glow::DEPTH_TEST);
            // Send CHR ROM/RAM data
            let chr_tex = gl.create_texture().unwrap();

            // Create program for rendering sprites to texture
            let sprite_program = gl.create_program().expect("Unable to create program");
            compile_and_link_shader(
                &gl,
                glow::VERTEX_SHADER,
                include_str!("../shaders/pass_through.vert"),
                &sprite_program,
            );
            compile_and_link_shader(
                &gl,
                glow::GEOMETRY_SHADER,
                include_str!("../shaders/oam.geom"),
                &sprite_program,
            );
            compile_and_link_shader(
                &gl,
                glow::FRAGMENT_SHADER,
                include_str!("../shaders/old_tile.frag"),
                &sprite_program,
            );

            gl.link_program(sprite_program);
            if !gl.get_program_link_status(sprite_program) {
                panic!(
                    "Couldn't link program: {}",
                    gl.get_program_info_log(sprite_program)
                );
            }

            let vao_array = core::array::from_fn(|i| {
                // Our "vertice" is a 1-D vector with the OAM ID in it
                let vertices = [i as i32];
                buffer_data_slice(&gl, &sprite_program, &vertices)
            });

            let background_program = gl.create_program().expect("Unable to create program!");
            compile_and_link_shader(
                &gl,
                glow::VERTEX_SHADER,
                include_str!("../shaders/pass_through.vert"),
                &background_program,
            );
            compile_and_link_shader(
                &gl,
                glow::GEOMETRY_SHADER,
                include_str!("../shaders/background.geom"),
                &background_program,
            );
            compile_and_link_shader(
                &gl,
                glow::FRAGMENT_SHADER,
                include_str!("../shaders/old_tile.frag"),
                &background_program,
            );

            gl.link_program(background_program);
            if !gl.get_program_link_status(background_program) {
                panic!(
                    "Couldn't link program: {}",
                    gl.get_program_info_log(background_program)
                );
            }
            let verts: [i32; 2 * 32 * 30] = core::array::from_fn(|i| i as i32);
            let background_vao = buffer_data_slice(&gl, &background_program, &verts);

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
            if !gl.get_program_link_status(tile_program) {
                panic!(
                    "Couldn't link program: {}",
                    gl.get_program_info_log(tile_program)
                );
            }
            let scanline_vao =
                create_f32_slice_vao(&gl, [[0.0, 0.0], [1.0, 0.0]].as_flattened(), 2);

            Screen {
                gl,
                sprite_program,
                vao_array,
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
            }
        }
    }

    pub unsafe fn render_scanline(&mut self, nes: &Nes, window_size: (u32, u32), scanline: usize) {
        self.gl.enable(glow::STENCIL_TEST);
        self.gl.viewport(0, 0, 256, 240);
        self.gl.bind_texture(glow::TEXTURE_2D, Some(self.chr_tex));

        self.gl
            .bind_framebuffer(glow::FRAMEBUFFER, Some(self.screen_fbo));
        // Set stencil buffer to only the one line
        self.gl.stencil_mask(0x00);
        self.gl.clear(glow::STENCIL_BUFFER_BIT);
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
        check_error!(self.gl);

        // Set pattern table
        self.gl.use_program(Some(self.tile_program));
        check_error!(self.gl);
        let palette = nes.ppu.palette_ram.map(|i| self.palette[i as usize]);
        // Render background
        let nametable = self.get_nametable(nes.ppu.get_base_nametable(), nes);
        let y = scanline / 8;
        (0..32).for_each(|x| {
            let tile_index = 32 * y + x;
            let palette_byte = nametable[0x3C0 + (x / 4) + 8 * (y / 4)];
            let pal_x = (x % 4) / 2;
            let pal_y = (y % 4) / 2;
            let palette_index = ((palette_byte >> (2 * (2 * pal_y + pal_x))) & 0x03) as usize;
            self.render_tile(
                8 * x,
                8 * y,
                nes.ppu.get_background_pattern_table_addr() / 0x10 + nametable[tile_index] as usize,
                palette_index,
                false,
                false,
                &palette,
            );
        });
        // Render OAM
        let oam_to_render: Vec<&[u8]> = nes
            .ppu
            .oam
            .chunks(4)
            .filter(|obj| obj[0] as usize <= scanline && obj[0] as usize + 8 > scanline)
            .collect();
        oam_to_render.iter().take(8).for_each(|obj| {
            self.render_tile(
                obj[3] as usize,
                obj[0] as usize,
                obj[1] as usize,
                4 + (obj[2] & 0x03) as usize,
                (obj[2] & 0x80) != 0,
                (obj[2] & 0x40) != 0,
                &palette,
            );
        });
        // Todo: Check for sprite overflow
    }

    pub fn render(&mut self, nes: &Nes, window_size: (u32, u32)) {
        unsafe {
            self.gl.disable(glow::STENCIL_TEST);
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            check_error!(self.gl);
            self.gl.use_program(Some(self.screen_program));
            self.gl
                .bind_texture(glow::TEXTURE_2D, Some(self.screen_texture));
            check_error!(self.gl);
            self.gl
                .viewport(0, 0, window_size.0 as i32, window_size.1 as i32);
            check_error!(self.gl);
            self.gl.bind_vertex_array(Some(self.screen_vao));
            check_error!(self.gl);
            self.gl.draw_arrays(glow::TRIANGLES, 0, 6);
            check_error!(self.gl);
            self.gl.finish();
            self.refresh_chr_texture(nes);
        }
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
        self.gl.uniform_3_f32_slice(
            loc.as_ref(),
            palette[(4 * palette_index)..(4 * palette_index + 4)].as_flattened(),
        );
        set_bool_uniform(&self.gl, &self.tile_program, "flipVertical", flip_vert);
        set_bool_uniform(&self.gl, &self.tile_program, "flipHorizontal", flip_horz);
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
        self.gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
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
    unsafe fn render_background(&mut self, nes: &Nes) {
        self.gl.use_program(Some(self.background_program));
        self.gl.bind_vertex_array(Some(self.background_vao));
        self.setup_render_uniforms(&self.background_program, nes);
        // Set nametable
        let nametable_uni = self
            .gl
            .get_uniform_location(self.background_program, "nametable");
        let base_nametable = nes.ppu.get_base_nametable();
        // Build the two nametables
        let n: Vec<i32> = [
            // First nametable
            self.get_nametable(base_nametable, nes),
            // Second nametable
            self.get_nametable(base_nametable + 0x400, nes),
        ]
        .concat()[0..0x800]
            // Pack nametable tightly
            .chunks(4)
            .map(|b| {
                ((b[3] as i32) << 24) + ((b[2] as i32) << 16) + ((b[1] as i32) << 8) + b[0] as i32
            })
            .collect();
        self.gl
            .uniform_1_i32_slice(nametable_uni.as_ref(), n.as_slice());
        set_int_uniform(
            &self.gl,
            &self.background_program,
            "backgroundPatternLocation",
            nes.ppu.get_background_pattern_table_addr() as i32,
        );
        set_bool_uniform(
            &self.gl,
            &self.background_program,
            "hideLeftmostBackground",
            nes.ppu.should_hide_leftmost_background(),
        );
        self.gl.finish();
        self.gl.draw_arrays(glow::POINTS, 0, 2 * 30 * 32);
    }

    unsafe fn render_sprites(&mut self, nes: &Nes, priority: i32) {
        self.gl.use_program(Some(self.sprite_program));
        check_error!(self.gl);
        self.setup_render_uniforms(&self.sprite_program, nes);
        // Pipe OAM data to GLSL
        let oam_uni = self.gl.get_uniform_location(self.sprite_program, "oamData");
        let oam_data: [u32; 4 * 64] = core::array::from_fn(|i| nes.ppu.oam[i] as u32);
        self.gl.uniform_1_u32_slice(oam_uni.as_ref(), &oam_data);
        check_error!(self.gl);
        // Set various flags
        set_bool_uniform(
            &self.gl,
            &self.sprite_program,
            "hide_left_sprites",
            nes.ppu.should_hide_leftmost_sprites(),
        );
        set_bool_uniform(
            &self.gl,
            &self.sprite_program,
            "tallSprites",
            nes.ppu.is_8x16_sprites(),
        );
        set_int_uniform(
            &self.gl,
            &self.sprite_program,
            "spritePatternLocation",
            nes.ppu.get_spr_pattern_table_addr() as i32,
        );
        set_int_uniform(&self.gl, &self.sprite_program, "priority", priority);
        // Draw sprites as points
        // GLSL Shaders add pixels to form the full 8x8 sprite
        // Todo maybe: batch this
        self.gl.finish();
        self.vao_array.iter().for_each(|vao| {
            self.gl.bind_vertex_array(Some(*vao));
            self.gl.draw_arrays(glow::POINTS, 0, 1);
        });
    }

    unsafe fn setup_render_uniforms(&self, program: &NativeProgram, nes: &Nes) {
        // Set pallete
        let palette: Vec<i32> = nes.ppu.palette_ram.iter().map(|p| *p as i32).collect();
        let palette_uni = self.gl.get_uniform_location(*program, "palettes");
        check_error!(self.gl);
        self.gl
            .uniform_1_i32_slice(palette_uni.as_ref(), palette.as_slice());
        check_error!(self.gl);
        // Set colors
        let colors = self.palette.as_flattened();
        let color_uni = self.gl.get_uniform_location(*program, "colors");
        check_error!(self.gl);
        self.gl.uniform_3_f32_slice(color_uni.as_ref(), colors);
        check_error!(self.gl);
        // Set tint uniforms
        set_bool_uniform(&self.gl, program, "redTint", nes.ppu.is_red_tint_on());
        set_bool_uniform(&self.gl, program, "blueTint", nes.ppu.is_blue_tint_on());
        set_bool_uniform(&self.gl, program, "greenTint", nes.ppu.is_green_tint_on());
        // Set greyscale mode
        set_bool_uniform(
            &self.gl,
            program,
            "greyscaleMode",
            nes.ppu.is_greyscale_mode_on(),
        );
        // Set scroll
        set_int_uniform(&self.gl, program, "scrollX", nes.ppu.scroll_x as i32);
        set_int_uniform(&self.gl, program, "scrollY", nes.ppu.scroll_y as i32);
    }
}
