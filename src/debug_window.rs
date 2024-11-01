use crate::{utils::*, Nes};
use glow::{HasContext, NativeProgram, VertexArray, VERTEX_ARRAY};
use sdl2::VideoSubsystem;
// Renders all the CHR ROM (and CHR RAM TBD) in the cartridge for debug purposes
pub struct DebugWindow {
    window: sdl2::video::Window,
    gl_context: sdl2::video::GLContext,
    gl: glow::Context,
    vao: VertexArray,
    palette: [[f32; 3]; 64],
    program: NativeProgram,
}

impl DebugWindow {
    pub fn new(nes: &Nes, video: &VideoSubsystem) -> DebugWindow {
        // Render 8 * 8 pixel tiles 16 tiles across across with a 1px border inbetween
        let window_width = 256;
        // let window_height = nes.cartridge.chr_rom.len() as u32 / 16;
        let window_height = 240;

        let (window, gl_context, gl) = create_window(video, "CHR ROM", window_width, window_height);

        unsafe {
            let program = gl.create_program().unwrap();
            gl.use_program(Some(program));
            let chr_rom_tex = create_data_texture(&gl, nes.cartridge.chr_rom.as_slice());
            compile_and_link_shader(
                &gl,
                glow::VERTEX_SHADER,
                include_str!("./shaders/pass_through.vert"),
                &program,
            );
            compile_and_link_shader(
                &gl,
                glow::GEOMETRY_SHADER,
                include_str!("./shaders/chr_rom_debug.geom"),
                &program,
            );
            compile_and_link_shader(
                &gl,
                glow::FRAGMENT_SHADER,
                include_str!("./shaders/tile.frag"),
                &program,
            );
            gl.link_program(program);

            let verts: Vec<i32> = (0..(nes.cartridge.chr_rom.len() / 0x10))
                .map(|i| i as i32)
                .collect();
            let vao = buffer_data_slice(&gl, &program, verts.as_slice());
            let palette_data: &[u8] = include_bytes!("./2C02G_wiki.pal");
            let palette: [[f32; 3]; 64] = core::array::from_fn(|i| {
                core::array::from_fn(|j| palette_data[3 * i + j] as f32 / 255.0)
            });
            DebugWindow {
                window,
                gl_context,
                gl,
                vao,
                palette,
                program,
            }
        }
    }
    pub fn render(&self, nes: &Nes) {
        unsafe {
            self.window.gl_make_current(&self.gl_context).unwrap();
            self.gl.use_program(Some(self.program));
            // let clear_color = self.palette[(nes.ppu.palette_ram[0] & 0x3F) as usize];
            let clear_color = self.palette[0];
            self.gl
                .clear_color(clear_color[0], clear_color[1], clear_color[2], 1.0);
            self.gl.viewport(
                0,
                0,
                self.window.size().0 as i32,
                self.window.size().1 as i32,
            );
            self.gl.clear(glow::COLOR_BUFFER_BIT);

            let palette: Vec<i32> = nes.ppu.palette_ram.iter().map(|p| *p as i32).collect();
            let palette_uni = self.gl.get_uniform_location(self.program, "palettes");
            self.gl
                .uniform_1_i32_slice(palette_uni.as_ref(), palette.as_slice());
            // Set colors
            let colors = self.palette.as_flattened();
            let color_uni = self.gl.get_uniform_location(self.program, "colors");
            self.gl.uniform_3_f32_slice(color_uni.as_ref(), colors);
            // Set tint uniforms
            set_bool_uniform(&self.gl, &self.program, "redTint", nes.ppu.is_red_tint_on());
            set_bool_uniform(
                &self.gl,
                &self.program,
                "blueTint",
                nes.ppu.is_blue_tint_on(),
            );
            set_bool_uniform(
                &self.gl,
                &self.program,
                "greenTint",
                nes.ppu.is_green_tint_on(),
            );
            // Set greyscale mode
            set_bool_uniform(
                &self.gl,
                &self.program,
                "greyscaleMode",
                nes.ppu.is_greyscale_mode_on(),
            );

            self.gl.bind_vertex_array(Some(self.vao));
            self.gl
                .draw_arrays(glow::POINTS, 0, nes.cartridge.chr_rom.len() as i32 / 0x10);
        }
        self.window.gl_swap_window();
    }
}
