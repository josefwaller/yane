use crate::{check_error, utils::*, NametableArrangement, Nes};
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
    chr_tex: NativeTexture,
}
impl Screen {
    // TODO: Rename
    pub fn new(nes: &Nes, gl: Context) -> Screen {
        unsafe {
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
            let tile_buf = gl.create_buffer().unwrap();
            check_error!(gl);
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(tile_buf));
            check_error!(gl);
            let verts: &[f32] = [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]].as_flattened();
            let verts_u8 = core::slice::from_raw_parts(
                verts.as_ptr() as *const u8,
                verts.len() * size_of::<f32>(),
            );
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &verts_u8, glow::STATIC_DRAW);
            check_error!(gl);
            // Describe the format of the data
            let tile_vao = gl.create_vertex_array().unwrap();
            check_error!(gl);
            gl.bind_vertex_array(Some(tile_vao));
            gl.enable_vertex_attrib_array(0);
            check_error!(gl);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 2 * size_of::<f32>() as i32, 0);
            check_error!(gl);

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
            }
        }
    }

    pub fn render(&mut self, nes: &Nes, window_size: (u32, u32)) {
        unsafe {
            self.refresh_chr_texture(nes);
            self.gl.use_program(Some(self.sprite_program));
            self.gl
                .bind_framebuffer(glow::FRAMEBUFFER, Some(self.screen_fbo));
            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.chr_tex));
            // Set clear color
            let clear_color = self.palette[(nes.ppu.palette_ram[0] & 0x3F) as usize];
            self.gl
                .clear_color(clear_color[0], clear_color[1], clear_color[2], 1.0);
            self.gl.viewport(0, 0, 256, 240);
            self.gl.clear(glow::COLOR_BUFFER_BIT);

            // Set pattern table
            self.gl.use_program(Some(self.tile_program));
            check_error!(self.gl);
            // Render OAM
            self.gl.viewport(0, 0, 256, 240);
            nes.ppu.oam.chunks(4).for_each(|obj| {
                let position_loc = self.gl.get_uniform_location(self.tile_program, "position");
                self.gl
                    .uniform_2_f32(position_loc.as_ref(), obj[3] as f32, obj[0] as f32);
                self.gl.bind_vertex_array(Some(self.tile_vao));
                self.gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
            });
            check_error!(self.gl);
            // Behind background
            // if nes.ppu.is_sprite_enabled() {
            //     self.render_sprites(nes, 1);
            // }
            // if nes.ppu.is_background_enabled() {
            //     self.render_background(nes);
            // }
            // // In front of background
            // if nes.ppu.is_sprite_enabled() {
            //     self.render_sprites(nes, 0);
            // }

            self.gl.finish();
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
        }
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
            glow::RED as i32,
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
