use log::*;
use std::cmp::{max, min};

use crate::{check_error, set_uniform, utils::*, Cartridge, Nes};
use glow::{HasContext, NativeFramebuffer, NativeProgram, NativeTexture, VertexArray};
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
    screen_fbo: NativeFramebuffer,
    screen_program: NativeProgram,
    screen_vao: VertexArray,
    screen_texture: NativeTexture,
    chr_tex: NativeTexture,
    // Amount of rows/columns of tiles
    num_rows: usize,
    num_columns: usize,
    // Imgui stuff
    imgui: imgui::Context,
    platform: SdlPlatform,
    renderer: AutoRenderer,
    // Settings to change through imgui
    palette_index: usize,
    debug_palette: bool,
    paused: bool,
    volume: f32,
}

impl DebugWindow {
    pub fn new(nes: &Nes, video: &VideoSubsystem, sdl: &Sdl) -> DebugWindow {
        // Figure out how many rows/columns
        let num_tiles =
            (nes.cartridge.memory.chr_rom.len() + nes.cartridge.memory.chr_ram.len()) / 0x10;
        let num_columns = 0x10;
        // let num_rows = max(num_tiles / num_columns, 1);
        let num_rows = 0x10;
        // Set window size
        let window_width = 4 * 8 * num_columns as u32;
        let window_height = 4 * 8 * num_rows as u32;

        let (window, gl_context, gl) = create_window(video, "CHR ROM", window_width, window_height);

        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);
        imgui.set_log_filename(None);
        imgui
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

        unsafe {
            check_error!(gl);
            let program = gl.create_program().unwrap();
            check_error!(gl);
            // let chr_tex = create_data_texture(&gl, &[]);
            let chr_tex = gl.create_texture().unwrap();
            compile_and_link_shader(
                &gl,
                glow::VERTEX_SHADER,
                include_str!("../shaders/chr.vert"),
                &program,
            );
            compile_and_link_shader(
                &gl,
                glow::FRAGMENT_SHADER,
                include_str!("../shaders/tile.frag"),
                &program,
            );
            gl.link_program(program);
            check_error!(gl);
            if !gl.get_program_link_status(program) {
                panic!(
                    "Couldn't link program: {}",
                    gl.get_program_info_log(program)
                );
            }

            gl.use_program(Some(program));

            let vao = create_f32_slice_vao(
                &gl,
                [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]].as_flattened(),
                2,
            );
            let palette_data: &[u8] = include_bytes!("../2C02G_wiki.pal");
            let palette: [[f32; 3]; 64] = core::array::from_fn(|i| {
                core::array::from_fn(|j| palette_data[3 * i + j] as f32 / 255.0)
            });
            check_error!(gl);
            let (screen_fbo, screen_vao, screen_program, screen_texture) =
                create_screen_texture(&gl, (8 * num_columns, 8 * num_rows));

            let platform = SdlPlatform::new(&mut imgui);
            let renderer = AutoRenderer::new(gl, &mut imgui).unwrap();
            DebugWindow {
                window,
                gl_context,
                vao,
                palette,
                program,
                screen_fbo,
                screen_program,
                screen_texture,
                screen_vao,
                chr_tex,
                num_columns,
                num_rows,
                platform,
                renderer,
                imgui,
                palette_index: 0,
                debug_palette: false,
                paused: false,
                volume: 0.00,
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
            refresh_chr_texture(&gl, self.chr_tex, nes);
            // Render onto framebuffer
            gl.use_program(Some(self.program));
            check_error!(gl);
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.screen_fbo));
            check_error!(gl);
            gl.viewport(0, 0, 8 * self.num_columns as i32, 8 * self.num_rows as i32);
            check_error!(gl);

            let clear_color = self.palette[(nes.ppu.palette_ram[0] & 0x3F) as usize];
            gl.clear_color(clear_color[0], clear_color[1], clear_color[2], 1.0);
            check_error!(gl);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::STENCIL_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            check_error!(gl);

            // Set uniforms
            let mut palette: Vec<f32> = if self.debug_palette {
                [
                    [0.0, 0.0, 0.0],
                    [0.6, 0.0, 0.1],
                    [0.1, 0.6, 0.1],
                    [0.3, 0.3, 1.0],
                ]
                .as_flattened()
                .to_vec()
            } else {
                nes.ppu
                    .palette_ram
                    .chunks(4)
                    .nth(self.palette_index)
                    .unwrap()
                    .iter()
                    .map(|p| self.palette[*p as usize])
                    .collect::<Vec<[f32; 3]>>()
                    .as_flattened()
                    .to_vec()
            };
            // Pad with 0s
            palette.resize(3 * 4 * 4 * 2, 0.0);
            set_uniform!(
                gl,
                self.program,
                "palette",
                uniform_3_f32_slice,
                palette.as_slice()
            );
            set_uniform!(
                gl,
                self.program,
                "numColumns",
                uniform_1_i32,
                self.num_columns as i32
            );
            set_uniform!(
                gl,
                self.program,
                "numRows",
                uniform_1_i32,
                self.num_rows as i32
            );
            // Set CHR data
            const TEX_NUM: i32 = 2;
            gl.use_program(Some(self.program));
            gl.bind_texture(glow::TEXTURE_2D, Some(self.chr_tex));
            gl.active_texture(glow::TEXTURE0 + TEX_NUM as u32);
            check_error!(gl);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.chr_tex));
            check_error!(gl);
            set_uniform!(gl, self.program, "chrTex", uniform_1_i32, TEX_NUM);
            // Draw sprites
            gl.bind_vertex_array(Some(self.vao));
            check_error!(gl);
            gl.draw_arrays_instanced(
                glow::TRIANGLE_STRIP,
                0,
                4,
                (self.num_rows * self.num_columns) as i32,
            );
            check_error!(gl);
            // Render onto screen
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.use_program(Some(self.screen_program));
            gl.viewport(
                0,
                0,
                self.window.size().0 as i32,
                self.window.size().1 as i32,
            );
            // Use FBO texture now
            let tex_num: i32 = 1;
            gl.active_texture(glow::TEXTURE0 + tex_num as u32);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.screen_texture));
            set_uniform!(
                gl,
                self.screen_program,
                "renderedTexture",
                uniform_1_i32,
                tex_num
            );

            gl.bind_vertex_array(Some(self.screen_vao));
            check_error!(gl);
            gl.draw_arrays(glow::TRIANGLES, 0, 6);
            check_error!(gl);

            // Draw imgui
            self.platform
                .prepare_frame(&mut self.imgui, &self.window, event_pump);
            let ui = self.imgui.new_frame();
            ui.window("Settings")
                .size([200.0, 200.0], FirstUseEver)
                .build(|| {
                    ui.disabled(self.debug_palette, || {
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
                    });
                    ui.checkbox("Debug palette", &mut self.debug_palette);
                    if ui.button(if self.paused { "Unpause" } else { "Pause" }) {
                        self.paused = !self.paused;
                    }
                    ui.slider("Volume", 0.0, 1.0, &mut self.volume);
                });
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
