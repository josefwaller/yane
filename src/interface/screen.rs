use crate::{utils::*, Nes};
use glow::*;

// Renders the PPU
pub struct Screen {
    gl: Context,
    sprite_program: NativeProgram,
    texture_program: NativeProgram,
    texture_vao: NativeVertexArray,
    vao_array: [NativeVertexArray; 64],
    texture_buffer: NativeFramebuffer,
    palette: [[f32; 3]; 0x40],
    background_program: NativeProgram,
    background_vao: NativeVertexArray,
}
impl Screen {
    // TODO: Rename
    pub fn new(nes: &Nes, gl: Context) -> Screen {
        unsafe {
            // Send CHR ROM data
            // TODO: Handle cartridges mapping this differently
            let data: &[u8] = nes.cartridge.memory.chr_rom.as_slice();
            let chr_rom_tex = create_data_texture(&gl, data);

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
                include_str!("../shaders/tile.frag"),
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
            let verts: [i32; 32 * 60] = core::array::from_fn(|i| i as i32);
            let background_vao = buffer_data_slice(&gl, &background_program, &verts);

            let (texture_buffer, texture_vao, texture_program) =
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
            Screen {
                gl,
                sprite_program,
                vao_array,
                texture_program,
                texture_buffer,
                texture_vao,
                palette,
                background_program,
                background_vao,
            }
        }
    }

    pub fn render(&mut self, nes: &Nes, window_size: (u32, u32)) {
        unsafe {
            self.gl.use_program(Some(self.sprite_program));
            self.gl
                .bind_framebuffer(glow::FRAMEBUFFER, Some(self.texture_buffer));
            // Set clear color
            let clear_color = self.palette[(nes.ppu.palette_ram[0] & 0x3F) as usize];
            self.gl
                .clear_color(clear_color[0], clear_color[1], clear_color[2], 1.0);
            self.gl.viewport(0, 0, 256, 240);
            self.gl.clear(glow::COLOR_BUFFER_BIT);

            self.render_background(nes);
            if nes.ppu.is_sprite_enabled() {
                self.render_sprites(nes);
            }

            self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            self.gl.use_program(Some(self.texture_program));
            self.gl
                .viewport(0, 0, window_size.0 as i32, window_size.1 as i32);
            self.gl.bind_vertex_array(Some(self.texture_vao));
            self.gl.draw_arrays(glow::TRIANGLES, 0, 6);
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
        // Pack nametable tightly
        let n: Vec<i32> = nes
            .ppu
            .nametable_ram
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
        self.gl.draw_arrays(glow::POINTS, 0, 30 * 32);
    }

    unsafe fn render_sprites(&mut self, nes: &Nes) {
        self.gl.use_program(Some(self.sprite_program));
        // Pipe OAM data to GLSL
        let oam_uni = self.gl.get_uniform_location(self.sprite_program, "oamData");
        let oam_data: [u32; 4 * 64] = core::array::from_fn(|i| nes.ppu.oam[i] as u32);
        self.gl.uniform_1_u32_slice(oam_uni.as_ref(), &oam_data);
        self.setup_render_uniforms(&self.sprite_program, nes);
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
        // Draw sprites as points
        // GLSL Shaders add pixels to form the full 8x8 sprite
        self.vao_array.iter().for_each(|vao| {
            self.gl.bind_vertex_array(Some(*vao));
            self.gl.draw_arrays(glow::POINTS, 0, 1);
        });
    }

    unsafe fn setup_render_uniforms(&self, program: &NativeProgram, nes: &Nes) {
        // Set pallete
        let palette: Vec<i32> = nes.ppu.palette_ram.iter().map(|p| *p as i32).collect();
        let palette_uni = self.gl.get_uniform_location(*program, "palettes");
        self.gl
            .uniform_1_i32_slice(palette_uni.as_ref(), palette.as_slice());
        // Set colors
        let colors = self.palette.as_flattened();
        let color_uni = self.gl.get_uniform_location(*program, "colors");
        self.gl.uniform_3_f32_slice(color_uni.as_ref(), colors);
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
        set_int_uniform(
            &self.gl,
            &self.background_program,
            "scrollX",
            0, //nes.ppu.scroll_x as i32,
        );
        set_int_uniform(
            &self.gl,
            &self.background_program,
            "scrollY",
            0, //nes.ppu.scroll_y as i32,
        );
    }
}