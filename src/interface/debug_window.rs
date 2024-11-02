use crate::{utils::*, Nes};
use glow::{HasContext, NativeFramebuffer, NativeProgram, VertexArray};
use imgui::Condition::FirstUseEver;
use imgui_glow_renderer::AutoRenderer;
use imgui_sdl2_support::SdlPlatform;
use sdl2::{event::Event, EventPump, Sdl, VideoSubsystem};
// Renders all the CHR ROM (and CHR RAM TBD) in the cartridge for debug purposes
pub struct DebugWindow {
    window: sdl2::video::Window,
    gl_context: sdl2::video::GLContext,
    vao: VertexArray,
    palette: [[f32; 3]; 64],
    program: NativeProgram,
    // Stuff for rendering the single quad texture to screen
    texture_program: NativeProgram,
    texture_vao: VertexArray,
    texture_framebuffer: NativeFramebuffer,
    // Amount of rows/columns of tiles
    num_rows: usize,
    num_columns: usize,
    // Imgui stuff
    imgui: imgui::Context,
    platform: SdlPlatform,
    renderer: AutoRenderer,
    // Settings to change through imgui
    palette_index: usize,
    paused: bool,
    volume: f32,
}

impl DebugWindow {
    pub fn new(nes: &Nes, video: &VideoSubsystem, sdl: &Sdl) -> DebugWindow {
        // Figure out how many rows/columns
        let num_columns = 0x20;
        let num_rows = (nes.cartridge.chr_rom.len() / 0x10) / num_columns;
        // Set window size
        let window_width = 4 * 8 * num_columns as u32;
        let window_height = 4 * 8 * num_rows as u32;

        let (window, gl_context, gl) = create_window(video, "CHR ROM", window_width, window_height);
        // let event_pump = sdl.event_pump().unwrap();

        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);
        imgui.set_log_filename(None);
        imgui
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

        unsafe {
            let program = gl.create_program().unwrap();
            gl.use_program(Some(program));
            let _chr_rom_tex = create_data_texture(&gl, nes.cartridge.chr_rom.as_slice());
            compile_and_link_shader(
                &gl,
                glow::VERTEX_SHADER,
                include_str!("../shaders/pass_through.vert"),
                &program,
            );
            compile_and_link_shader(
                &gl,
                glow::GEOMETRY_SHADER,
                include_str!("../shaders/chr_rom_debug.geom"),
                &program,
            );
            compile_and_link_shader(
                &gl,
                glow::FRAGMENT_SHADER,
                include_str!("../shaders/tile.frag"),
                &program,
            );
            gl.link_program(program);

            let verts: Vec<i32> = (0..(nes.cartridge.chr_rom.len() / 0x10))
                .map(|i| i as i32)
                .collect();
            let vao = buffer_data_slice(&gl, &program, verts.as_slice());
            let palette_data: &[u8] = include_bytes!("../2C02G_wiki.pal");
            let palette: [[f32; 3]; 64] = core::array::from_fn(|i| {
                core::array::from_fn(|j| palette_data[3 * i + j] as f32 / 255.0)
            });
            let (texture_framebuffer, texture_vao, texture_program) =
                create_screen_texture(&gl, (8 * num_columns, 8 * num_rows));

            let platform = SdlPlatform::new(&mut imgui);
            let renderer = AutoRenderer::new(gl, &mut imgui).unwrap();
            DebugWindow {
                window,
                gl_context,
                vao,
                palette,
                program,
                texture_program,
                texture_vao,
                texture_framebuffer,
                num_columns,
                num_rows,
                platform,
                renderer,
                imgui,
                palette_index: 0,
                paused: false,
                volume: 0.25,
            }
        }
    }
    pub fn handle_event(&mut self, event: &Event) {
        self.platform.handle_event(&mut self.imgui, event);
    }
    pub fn render(&mut self, nes: &Nes, event_pump: &EventPump) {
        unsafe {
            self.window.gl_make_current(&self.gl_context).unwrap();
            let gl = self.renderer.gl_context();
            // Render onto framebuffer
            gl.use_program(Some(self.program));
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.texture_framebuffer));
            gl.viewport(0, 0, 8 * self.num_columns as i32, 8 * self.num_rows as i32);
            let clear_color = self.palette[(nes.ppu.palette_ram[0] & 0x3F) as usize];
            gl.clear_color(clear_color[0], clear_color[1], clear_color[2], 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);

            let palette: Vec<i32> = nes.ppu.palette_ram.iter().map(|p| *p as i32).collect();
            let palette_uni = gl.get_uniform_location(self.program, "palettes");
            gl.uniform_1_i32_slice(palette_uni.as_ref(), palette.as_slice());
            // Set colors
            let colors = self.palette.as_flattened();
            let color_uni = gl.get_uniform_location(self.program, "colors");
            gl.uniform_3_f32_slice(color_uni.as_ref(), colors);
            // Set tint uniforms
            set_bool_uniform(&gl, &self.program, "redTint", false);
            set_bool_uniform(&gl, &self.program, "blueTint", false);
            set_bool_uniform(&gl, &self.program, "greenTint", false);
            // Set greyscale mode
            set_bool_uniform(&gl, &self.program, "greyscaleMode", false);
            // Set number of columns
            set_int_uniform(&gl, &self.program, "numColumns", self.num_columns as i32);
            set_int_uniform(&gl, &self.program, "numRows", self.num_rows as i32);
            set_int_uniform(
                &gl,
                &self.program,
                "globalPaletteIndex",
                self.palette_index as i32,
            );

            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays(glow::POINTS, 0, nes.cartridge.chr_rom.len() as i32 / 0x10);

            // Render onto screen
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.use_program(Some(self.texture_program));
            gl.viewport(
                0,
                0,
                self.window.size().0 as i32,
                self.window.size().1 as i32,
            );
            gl.bind_vertex_array(Some(self.texture_vao));
            gl.draw_arrays(glow::TRIANGLES, 0, 6);

            // Draw imgui
            self.platform
                .prepare_frame(&mut self.imgui, &self.window, event_pump);
            let ui = self.imgui.new_frame();
            // ui.text("Hello world");
            ui.window("Settings")
                .size([200.0, 200.0], FirstUseEver)
                .build(|| {
                    if let Some(c) =
                        ui.begin_combo("Palette", format!("Palette {}", self.palette_index))
                    {
                        (0..8).for_each(|i| {
                            let label = format!("Palette {}", i);
                            if ui.selectable(label) {
                                self.palette_index = i;
                            }
                        });
                        c.end();
                    }
                    if ui.button(if self.paused { "Unpause" } else { "Pause" }) {
                        self.paused = !self.paused;
                    }
                    ui.slider("Volume", 0.0, 1.0, &mut self.volume);
                });
            // ui.show_demo_window(&mut true);
            let draw_data = self.imgui.render();
            self.renderer
                .render(draw_data)
                .expect("Error rendering DearImGui");
        }
        self.window.gl_swap_window();
    }

    pub fn paused(&self) -> bool {
        self.paused
    }
    pub fn volume(&self) -> f32 {
        self.volume
    }
}
