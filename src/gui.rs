use crate::Nes;
use glow::*;
use std::mem::size_of;

pub struct Gui {
    gl: glow::Context,
    gl_context: sdl2::video::GLContext,
    window: sdl2::video::Window,
    event_loop: sdl2::EventPump,
    program: NativeProgram,
    vao_array: [NativeVertexArray; 64],
}
impl Gui {
    pub fn new() -> Gui {
        let window_width = 256;
        let window_height = 240;
        let gl: glow::Context;
        let window: sdl2::video::Window;
        let event_loop: sdl2::EventPump;
        let gl_context: sdl2::video::GLContext;
        let program: Program;
        let vao_array: [NativeVertexArray; 64];
        unsafe {
            // Create SDL2 Window
            let sdl = sdl2::init().unwrap();
            let video = sdl.video().unwrap();
            let gl_attr = video.gl_attr();
            gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
            gl_attr.set_context_version(3, 3);
            window = video
                .window("Y.A.N.E", window_width, window_height)
                .opengl()
                .resizable()
                .build()
                .unwrap();
            gl_context = window.gl_create_context().unwrap();
            gl = glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _);
            window.gl_make_current(&gl_context).unwrap();
            event_loop = sdl.event_pump().unwrap();

            program = gl.create_program().expect("Unable to create program");
            compile_and_link_shader(
                &gl,
                glow::VERTEX_SHADER,
                include_str!("./vertex_shader.vert"),
                &program,
            );
            compile_and_link_shader(
                &gl,
                glow::GEOMETRY_SHADER,
                include_str!("./geometry_shader.geom"),
                &program,
            );
            compile_and_link_shader(
                &gl,
                glow::FRAGMENT_SHADER,
                include_str!("./fragment_shader.frag"),
                &program,
            );

            vao_array = core::array::from_fn(|i| {
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
            gl.use_program(Some(program));
            gl.clear_color(0.3, 0.0, 0.0, 1.0);
        }
        // Create triangles
        Gui {
            gl,
            window,
            event_loop,
            gl_context,
            program,
            vao_array,
        }
    }
    pub fn render(&mut self, nes: &mut Nes) -> bool {
        unsafe {
            self.gl.use_program(Some(self.program));
            self.gl.clear(glow::COLOR_BUFFER_BIT);
            // 3x3 Matrix per each 64 sprites
            let position_matrices: [[f32; 3 * 3]; 64] = core::array::from_fn(|i| {
                core::array::from_fn(|j| {
                    let row = (j / 3) % 3;
                    let col = j % 3;
                    if row as i32 == col as i32 {
                        return 1.0;
                    }
                    if col == 2 {
                        // X position
                        if row == 0 {
                            return (-128.0 + nes.ppu.oam[i + 3] as f32) / 128.0;
                        }
                        // Y Coord
                        if row == 1 {
                            return (-120.0 + nes.ppu.oam[i] as f32) / 120.0;
                        }
                    }
                    0.0
                })
            });
            let pos_uni = self
                .gl
                .get_uniform_location(self.program, "positionMatrices");
            self.gl.uniform_matrix_3_f32_slice(
                pos_uni.as_ref(),
                false,
                &position_matrices.as_flattened(),
            );
            let colors = [
                [0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 0.0],
            ]
            .as_flattened();
            let color_uni = self.gl.get_uniform_location(self.program, "colors");
            self.gl.uniform_3_f32_slice(color_uni.as_ref(), &colors);
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
        }
        self.window.gl_swap_window();
        for event in self.event_loop.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => return true,
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
