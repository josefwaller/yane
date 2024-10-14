use crate::Nes;
use glow::*;
use sdl2::event::{
    Event::{Quit, Window},
    WindowEvent::Resized,
};
use std::mem::size_of;

pub struct Gui {
    gl: glow::Context,
    _gl_context: sdl2::video::GLContext,
    window: sdl2::video::Window,
    event_loop: sdl2::EventPump,
    program: NativeProgram,
    texture_program: NativeProgram,
    tex_vao: NativeVertexArray,
    vao_array: [NativeVertexArray; 64],
    texture_buffer: NativeFramebuffer,
    palette: [[f32; 3]; 0x40],
}
impl Gui {
    pub fn new() -> Gui {
        let window_width = 800;
        let window_height = 600;
        unsafe {
            // Create SDL2 Window
            let sdl = sdl2::init().unwrap();
            let video = sdl.video().unwrap();
            let gl_attr = video.gl_attr();
            gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
            gl_attr.set_context_version(3, 3);
            let window = video
                .window("Y.A.N.E", window_width, window_height)
                .opengl()
                .resizable()
                .build()
                .unwrap();
            let gl_context = window.gl_create_context().unwrap();
            let gl =
                glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _);
            window.gl_make_current(&gl_context).unwrap();
            let event_loop = sdl.event_pump().unwrap();

            gl.enable(glow::BLEND);
            gl.blend_func(glow::BLEND_SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            // Create program for rendering sprites to texture
            let program = gl.create_program().expect("Unable to create program");
            compile_and_link_shader(
                &gl,
                glow::VERTEX_SHADER,
                include_str!("./shaders/vertex_shader.vert"),
                &program,
            );
            compile_and_link_shader(
                &gl,
                glow::GEOMETRY_SHADER,
                include_str!("./shaders/geometry_shader.geom"),
                &program,
            );
            compile_and_link_shader(
                &gl,
                glow::FRAGMENT_SHADER,
                include_str!("./shaders/fragment_shader.frag"),
                &program,
            );

            let vao_array = core::array::from_fn(|i| {
                // Our "vertice" is a 1-D vector with the OAM ID in it
                let vertices_u8: &[u8] =
                    core::slice::from_raw_parts([i].as_ptr() as *const u8, size_of::<i32>());
                let vbo = gl.create_buffer().unwrap();
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &vertices_u8, glow::STATIC_DRAW);

                let vao = gl
                    .create_vertex_array()
                    .expect("Could not create VertexArray");
                gl.bind_vertex_array(Some(vao));
                gl.enable_vertex_attrib_array(0);
                // Describe the data as a single int
                gl.vertex_attrib_pointer_i32(0, 1, glow::INT, size_of::<i32>() as i32, 0);
                vao
            });

            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                panic!(
                    "Couldn't link program: {}",
                    gl.get_program_info_log(program)
                );
            }
            let texture_program = gl.create_program().unwrap();
            compile_and_link_shader(
                &gl,
                glow::VERTEX_SHADER,
                include_str!("./shaders/quad_shader.vert"),
                &texture_program,
            );
            compile_and_link_shader(
                &gl,
                glow::FRAGMENT_SHADER,
                include_str!("./shaders/quad_shader.frag"),
                &texture_program,
            );
            gl.link_program(texture_program);
            gl.use_program(Some(texture_program));
            let quad_verts: &[f32] = [
                [-1.0, -1.0, 0.0],
                [-1.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [-1.0, -1.0, 0.0],
                [1.0, -1.0, 0.0],
                [1.0, 1.0, 0.0],
            ]
            .as_flattened();
            let quad_verts_u8 = core::slice::from_raw_parts(
                quad_verts.as_ptr() as *const u8,
                quad_verts.len() * size_of::<f32>(),
            );
            let tex_buf = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::VERTEX_ARRAY, Some(tex_buf));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &quad_verts_u8, glow::STATIC_DRAW);

            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 3 * size_of::<f32>() as i32, 0);

            gl.link_program(texture_program);
            let tex_vao = vao;

            let texture_buffer = gl.create_framebuffer().unwrap();
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(texture_buffer));
            let render_texture = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(render_texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                256,
                240,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                None,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );
            gl.framebuffer_texture(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                Some(render_texture),
                0,
            );
            gl.draw_buffers(&[glow::COLOR_ATTACHMENT0]);
            if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
                panic!("Error creating frame buffer");
            }
            // Load pallete
            let palette_data: &[u8] = include_bytes!("./2C02G_wiki.pal");
            let palette: [[f32; 3]; 64] = core::array::from_fn(|i| {
                core::array::from_fn(|j| palette_data[3 * i + j] as f32 / 255.0)
            });
            Gui {
                gl,
                window,
                event_loop,
                // This just needs to stay in scope
                _gl_context: gl_context,
                program,
                texture_program,
                vao_array,
                tex_vao,
                texture_buffer,
                palette,
            }
        }
    }
    pub fn render(&mut self, nes: &mut Nes) -> bool {
        unsafe {
            self.gl.use_program(Some(self.program));
            self.gl
                .bind_framebuffer(glow::FRAMEBUFFER, Some(self.texture_buffer));
            // Set clear color
            let clear_color = self.palette[(nes.ppu.vram[0] & 0x3F) as usize];
            self.gl
                .clear_color(clear_color[0], clear_color[1], clear_color[2], 1.0);
            self.gl.viewport(0, 0, 256, 240);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
            // Pipe OAM data to GLSL
            let oam_uni = self.gl.get_uniform_location(self.program, "oamData");
            let oam_data: [u32; 4 * 64] = core::array::from_fn(|i| nes.ppu.oam[i] as u32);
            self.gl.uniform_1_u32_slice(oam_uni.as_ref(), &oam_data);
            // Map VV HHHH colors to RGB colors
            let palette_colors: Vec<[f32; 3]> = nes.ppu.vram[0..0x100]
                .iter()
                .map(|b| self.palette[(b & 0x3F) as usize])
                .collect();
            // Set colors matrix
            let color_uni = self.gl.get_uniform_location(self.program, "palettes");
            self.gl.uniform_3_f32_slice(
                color_uni.as_ref(),
                &palette_colors.as_slice().as_flattened(),
            );
            // Draw sprites at points
            // GLSL Shaders add pixels
            for (i, vao) in self.vao_array.iter().enumerate() {
                self.gl.bind_vertex_array(Some(*vao));
                let sprite_addr = (((nes.ppu.status & 0x08) as usize) << 12)
                    + (nes.ppu.oam[4 * i + 1] as usize) * 16;
                let sprite: [i32; 128] = core::array::from_fn(|i| {
                    let byte = i / 8;
                    let bit = i % 8;
                    ((nes.cartridge.chr_rom[sprite_addr + byte] >> (7 - bit)) & 0x01) as i32
                });
                let sprite_uni = self.gl.get_uniform_location(self.program, "sprite");
                self.gl.uniform_1_i32_slice(sprite_uni.as_ref(), &sprite);
                self.gl.draw_arrays(glow::POINTS, 0, 1);
            }
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            self.gl.use_program(Some(self.texture_program));
            self.gl.viewport(
                0,
                0,
                self.window.size().0 as i32,
                self.window.size().1 as i32,
            );
            self.gl.bind_vertex_array(Some(self.tex_vao));
            self.gl.draw_arrays(glow::TRIANGLES, 0, 6);
        }
        self.window.gl_swap_window();
        for event in self.event_loop.poll_iter() {
            match event {
                Quit { .. } => return true,
                _ => {}
            }
        }
        false
    }
}

unsafe fn compile_and_link_shader(
    gl: &Context,
    shader_type: u32,
    shader_src: &str,
    program: &NativeProgram,
) {
    let shader = gl.create_shader(shader_type).expect("Cannot create shader");
    gl.shader_source(shader, shader_src);
    gl.compile_shader(shader);
    if !gl.get_shader_compile_status(shader) {
        panic!(
            "Failed to compile shader with source {}: {}",
            shader_src,
            gl.get_shader_info_log(shader)
        );
    }
    gl.attach_shader(*program, shader);
    gl.delete_shader(shader);
}
