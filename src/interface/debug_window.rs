use log::*;

use crate::{check_error, utils::*, Nes, Settings};
use glow::{HasContext, NativeProgram, NativeTexture, VertexArray};
use imgui::Condition::FirstUseEver;
use imgui_glow_renderer::AutoRenderer;
use imgui_sdl2_support::SdlPlatform;
use sdl2::{event::Event, EventPump, Sdl, VideoSubsystem};
// Renders all the CHR ROM (and CHR RAM TBD) in the cartridge for debug purposes
pub struct DebugWindow {
    window: sdl2::video::Window,
    gl_context: sdl2::video::GLContext,
    palette: [[u8; 3]; 64],
    // Stuff for rendering the single quad texture to screen
    screen_program: NativeProgram,
    screen_vao: VertexArray,
    screen_texture: NativeTexture,
    // Amount of rows/columns of tiles
    num_tiles: usize,
    num_rows: usize,
    num_columns: usize,
    // Imgui stuff
    imgui: imgui::Context,
    platform: SdlPlatform,
    renderer: AutoRenderer,
    // Settings to change through imgui
    palette_index: usize,
    // Index of current page of tiles we are viewing
    tile_page: usize,
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
            let palette_data: &[u8] = include_bytes!("../2C02G_wiki.pal");
            let palette: [[u8; 3]; 64] =
                core::array::from_fn(|i| core::array::from_fn(|j| palette_data[3 * i + j] as u8));
            check_error!(gl);
            let (_screen_fbo, screen_vao, screen_program, screen_texture) =
                create_screen_texture(&gl, (8 * num_columns, 8 * num_rows));

            let platform = SdlPlatform::new(&mut imgui);
            let renderer = AutoRenderer::new(gl, &mut imgui).unwrap();
            DebugWindow {
                window,
                gl_context,
                palette,
                screen_program,
                screen_texture,
                screen_vao,
                num_tiles,
                num_columns,
                num_rows,
                tile_page: 0,
                platform,
                renderer,
                imgui,
                palette_index: 0,
            }
        }
    }
    pub fn handle_event(&mut self, event: &Event) {
        self.platform.handle_event(&mut self.imgui, event);
    }
    pub fn render(&mut self, nes: &Nes, event_pump: &EventPump, settings: &mut Settings) {
        unsafe {
            self.window.gl_make_current(&self.gl_context).unwrap();
            let gl = self.renderer.gl_context();
            // Refresh texture
            let num_tiles_per_page = self.num_rows * self.num_columns;
            let start = 16 * num_tiles_per_page * self.tile_page;
            let end = 16 * num_tiles_per_page * (self.tile_page + 1);
            let data = &nes.cartridge.get_pattern_table()[start..end];
            let tex_data: Vec<u8> = data
                .chunks(16)
                .map(|tile| {
                    (0..64).map(|i| {
                        let x = i % 8;
                        let y = i / 8;
                        let tile_low = tile[y] as usize;
                        let tile_high = (tile[y + 8] as usize) << 1;
                        ((tile_low >> (7 - x)) & 0x01) + ((tile_high >> (7 - x)) & 0x02)
                    })
                })
                .enumerate()
                .fold(
                    vec![Vec::with_capacity(8 * self.num_columns); 8 * self.num_rows],
                    |mut a, (i, e)| {
                        e.enumerate().for_each(|(j, s)| {
                            a[8 * (i / self.num_columns) + j / 8].push(s);
                        });
                        a
                    },
                )
                .iter()
                .flatten()
                .map(|i| self.palette[nes.ppu.palette_ram[4 * self.palette_index + *i] as usize])
                .flatten()
                .collect();
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.use_program(Some(self.screen_program));
            check_error!(gl);
            gl.viewport(
                0,
                0,
                self.window.size().0 as i32,
                self.window.size().1 as i32,
            );
            check_error!(gl);
            // Use FBO texture now
            let tex_num: i32 = 1;
            gl.active_texture(glow::TEXTURE0 + tex_num as u32);
            check_error!(gl);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.screen_texture));
            check_error!(gl);
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGB as i32,
                8 * self.num_columns as i32,
                8 * self.num_rows as i32,
                // 8 * self.num_columns as i32,
                // 8 * self.num_rows as i32,
                0,
                glow::RGB,
                glow::UNSIGNED_BYTE,
                Some(&tex_data),
            );
            check_error!(gl);
            let loc = gl.get_uniform_location(self.screen_program, "renderedTexture");
            gl.uniform_1_i32(loc.as_ref(), tex_num);

            gl.bind_vertex_array(Some(self.screen_vao));
            check_error!(gl);
            gl.draw_arrays(glow::TRIANGLES, 0, 6);
            check_error!(gl);

            // Draw imgui
            self.platform
                .prepare_frame(&mut self.imgui, &self.window, event_pump);
            let ui = self.imgui.new_frame();
            ui.window("Settings")
                .size([600.0, 400.0], FirstUseEver)
                .build(|| {
                    if let Some(c) = ui.begin_combo("Page", format!("Page {}", self.tile_page)) {
                        (0..(self.num_tiles / (self.num_columns * self.num_rows))).for_each(|i| {
                            if ui.selectable(format!("Page {}", i)) {
                                self.tile_page = i;
                            }
                        });
                        c.end();
                    }
                    ui.disabled(settings.use_debug_palette, || {
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
                    ui.checkbox("Debug palette", &mut settings.use_debug_palette);
                    ui.checkbox("Debug OAM", &mut settings.oam_debug);
                    if ui.checkbox("Paused", &mut settings.paused) {
                        info!(
                            "Manual pause {}",
                            if settings.paused {
                                "checked"
                            } else {
                                "unchecked"
                            }
                        );
                    }
                    ui.checkbox("Scanline sprite limit", &mut settings.scanline_sprite_limit);
                    ui.checkbox(
                        "Always draw sprites on top of background",
                        &mut settings.always_sprites_on_top,
                    );
                    ui.slider("Volume", 0.0, 10.0, &mut settings.volume);
                    ui.slider("Speed", 0.1, 3.0, &mut settings.speed);
                    ui.same_line();
                    if ui.button("Reset to 1") {
                        settings.speed = 1.0;
                    }
                    ui.text(format!(
                        "Scroll: ({:3}, {:3})",
                        nes.ppu.scroll_x, nes.ppu.scroll_y
                    ));
                    ui.text(format!(
                        "PC: {:X}, opcode: {:X} {:X} ({:X})",
                        nes.cpu.p_c,
                        nes.last_instructions[0][0],
                        nes.last_instructions[0][1],
                        nes.last_instructions[0][2]
                    ));
                    let s = nes.cartridge.debug_string();
                    if !s.is_empty() {
                        ui.text(s);
                    }
                });
            let draw_data = self.imgui.render();
            self.renderer
                .render(draw_data)
                .expect("Error rendering DearImGui");
        }
        self.window.gl_swap_window();
    }
}
