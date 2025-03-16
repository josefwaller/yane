use copypasta::{ClipboardContext, ClipboardProvider};
use log::*;

use crate::{
    app::Config,
    core::{Cartridge, Nes, Ppu, DEBUG_PALETTE},
    utils::*,
};
use glow::{HasContext, NativeTexture};
use imgui::{FontId, TextureId, TreeNodeFlags};
use imgui_glow_renderer::AutoRenderer;
use imgui_sdl2_support::SdlPlatform;
use sdl2::{event::Event, EventPump, VideoSubsystem};

use super::utils::{quickload, quicksave};

/// Debug window for the emulator
///
/// The debug window that spawns when the debug argument is passed.
/// Allows the user to change various settings of the emulator.
pub struct DebugWindow {
    window: sdl2::video::Window,
    gl_context: sdl2::video::GLContext,
    palette: [[u8; 3]; 64],
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
    // Size of CHR texture
    chr_size: f32,
    // Update nametable every 6 "ticks" (should be around 10 Hz)
    nametable_timer: u32,
    // Fonts
    small_font: FontId,

    chr_tex: NativeTexture,
    nametable_tex: NativeTexture,
}

impl DebugWindow {
    /// Spawn a new [DebugWindow]
    pub fn new(nes: &Nes, video: &VideoSubsystem) -> DebugWindow {
        // Figure out how many rows/columns
        let num_tiles =
            (nes.cartridge.memory.chr_rom.len() + nes.cartridge.memory.chr_ram.len()) / 0x10;
        let num_columns = 0x10;
        let num_rows = 0x10;
        // Set window size
        let window_width = 600;
        let window_height = 1200;

        let (window, gl_context, gl) = create_window(
            video,
            "Y.A.N.E. - Debug Settings",
            window_width,
            window_height,
        );

        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);
        imgui.set_log_filename(None);
        imgui
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        let small_config = imgui::FontConfig {
            size_pixels: 9.0,
            ..imgui::FontConfig::default()
        };
        let small_font = imgui
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData {
                config: Some(small_config),
            }]);

        unsafe {
            let palette_data: &[u8] = include_bytes!("../2C02G_wiki.pal");
            let palette: [[u8; 3]; 64] =
                core::array::from_fn(|i| core::array::from_fn(|j| palette_data[3 * i + j]));
            check_error!(gl);
            let chr_tex = create_texture(&gl);
            let nametable_tex = create_texture(&gl);

            let platform = SdlPlatform::new(&mut imgui);
            let renderer = AutoRenderer::new(gl, &mut imgui).unwrap();
            DebugWindow {
                window,
                gl_context,
                palette,
                num_tiles,
                num_columns,
                num_rows,
                tile_page: 0,
                platform,
                renderer,
                imgui,
                palette_index: 0,
                chr_size: 4.0,
                chr_tex,
                nametable_tex,
                nametable_timer: 0,
                small_font,
            }
        }
    }
    /// Process an event
    pub fn handle_event(&mut self, event: &Event) {
        self.platform.handle_event(&mut self.imgui, event);
    }
    /// Transform some tile data (i.e. sections of CHR ROM/RAM in the cartridge) to RGB triplets that can be
    /// piped to an OpenGL texture.
    /// `width` and `height` are how many tiles wide/high the texture should be.
    /// The resulting texture will have dimensions `(8 * width, 8 * height)`.
    fn transform_chr_data(
        &self,
        data: &[u8],
        width: usize,
        height: usize,
        palette: &[u8],
        palette_indices: Vec<usize>,
    ) -> Vec<u8> {
        data.chunks(16)
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
                vec![Vec::with_capacity(8 * width); 8 * height],
                |mut a, (i, e)| {
                    e.enumerate().for_each(|(j, s)| {
                        a[8 * (i / width) + j / 8].push((i, s));
                    });
                    a
                },
            )
            .iter()
            .flatten()
            .flat_map(|(tile_index, pixel_index)| {
                let index = 4 * palette_indices[*tile_index % palette_indices.len()] + *pixel_index;
                self.palette[palette[index % palette.len()] as usize % self.palette.len()]
            })
            .collect()
    }
    /// Render the debug window, and update the [Config] and [Nes] if any of the imgui buttons are pressed.
    pub fn render(&mut self, nes: &mut Nes, event_pump: &EventPump, config: &mut Config) {
        let chr_tex_num: i32 = 1;
        unsafe {
            self.window.gl_make_current(&self.gl_context).unwrap();
            let gl = self.renderer.gl_context();
            // Refresh texture
            let num_tiles_per_page = self.num_rows * self.num_columns;
            let start = 0x10 * num_tiles_per_page * self.tile_page;
            let end = 0x10 * num_tiles_per_page * (self.tile_page + 1);
            let data = &nes.cartridge.get_pattern_table()[start..end];
            let tex_data: Vec<u8> = self.transform_chr_data(
                data,
                self.num_columns,
                self.num_rows,
                if config.emu_settings.use_debug_palette {
                    &DEBUG_PALETTE
                } else {
                    &nes.ppu.palette_ram
                },
                // Todo: cache this somewhere
                vec![self.palette_index],
            );
            check_error!(gl);
            gl.active_texture(glow::TEXTURE0 + chr_tex_num as u32);
            check_error!(gl);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.chr_tex));
            check_error!(gl);
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGB as i32,
                8 * self.num_columns as i32,
                8 * self.num_rows as i32,
                0,
                glow::RGB,
                glow::UNSIGNED_BYTE,
                Some(&tex_data),
            );
            check_error!(gl);
            // Set up nametable texture
            // Only update texture if there is a change since this is a fairly costly method
            self.nametable_timer = (self.nametable_timer + 1) % 6;
            if self.nametable_timer == 0 {
                // Pass a reference to the reference so that the closures can take ownership
                let nes = &nes;
                let nametables = [
                    [
                        nes.ppu.top_left_nametable_addr(),
                        nes.ppu.top_right_nametable_addr(),
                    ],
                    [
                        nes.ppu.bot_left_nametable_addr(),
                        nes.ppu.bot_right_nametable_addr(),
                    ],
                ]
                .iter()
                // For every left-right nametable pair
                .flat_map(|n| {
                    let attr_addr = n.map(|i| i + 0x3C0);
                    // For every row
                    (0..30)
                        .flat_map(move |y| {
                            // For every column
                            (0..64).map(move |x| {
                                // Get the tile here, which could be in one of two nametables
                                // Since we want the top left and top right ones to be beside each other
                                let (nt, at) = if x < 32 {
                                    (n[0], attr_addr[0])
                                } else {
                                    (n[1], attr_addr[1])
                                };
                                let tile_addr = nes.ppu.nametable_tile_addr()
                                    + 0x10
                                        * nes.ppu.nametable_ram[nes
                                            .cartridge
                                            .transform_nametable_addr(nt)
                                            + 32 * y
                                            + x % 32]
                                            as usize;
                                // Get palette info
                                let palette_shift = 2 * ((((x % 32) / 2) % 2) + 2 * ((y / 2) % 2));
                                let palette_byte_addr = nes.cartridge.transform_nametable_addr(at)
                                    + 0x08 * (y / 4)
                                    + ((x % 32) / 4);
                                let palette_byte = nes.ppu.nametable_ram[palette_byte_addr];
                                let palette = (palette_byte >> palette_shift) & 0x03;
                                (0..0x10).map(move |j| {
                                    (
                                        // This needs to not go through the cartridge in order to avoid triggering interrupts through counting
                                        nes.cartridge
                                            .mapper
                                            .read_ppu_debug(tile_addr + j, &nes.cartridge.memory),
                                        palette as usize,
                                    )
                                })
                            })
                        })
                        .flatten()
                })
                .collect::<Vec<(u8, usize)>>();
                let (nt_tiles, nt_palettes): (Vec<u8>, Vec<usize>) = nametables.into_iter().unzip();
                let p: Vec<usize> = nt_palettes.into_iter().step_by(0x10).collect();
                let tex_data = self.transform_chr_data(
                    nt_tiles.as_slice(),
                    64,
                    60,
                    if config.emu_settings.use_debug_palette {
                        &DEBUG_PALETTE
                    } else {
                        &nes.ppu.palette_ram
                    },
                    p,
                );
                let tex_num: i32 = 2;
                gl.active_texture(glow::TEXTURE0 + tex_num as u32);
                check_error!(gl);
                gl.bind_texture(glow::TEXTURE_2D, Some(self.nametable_tex));
                check_error!(gl);
                gl.tex_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    glow::RGB as i32,
                    8 * 64_i32,
                    8 * 60_i32,
                    0,
                    glow::RGB,
                    glow::UNSIGNED_BYTE,
                    Some(&tex_data),
                );
                check_error!(gl);
            }
            gl.clear_color(0.0, 0.0, 0.0, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }

        // Draw imgui
        self.platform
            .prepare_frame(&mut self.imgui, &self.window, event_pump);
        let ui = self.imgui.new_frame();
        ui.window("Debug Settings")
            .position([0.0, 0.0], imgui::Condition::Always)
            .size(
                [self.window.size().0 as f32, self.window.size().1 as f32],
                imgui::Condition::Always,
            )
            .build(|| {
                if ui.button("Reset") {
                    nes.reset();
                }
                ui.checkbox("Debug OAM", &mut config.oam_debug);
                if ui.checkbox("Paused", &mut config.paused) {
                    info!(
                        "Manual pause {}",
                        if config.paused {
                            "checked"
                        } else {
                            "unchecked"
                        }
                    );
                }
                ui.disabled(!config.paused, || {
                    if ui.button("Advance instruction") {
                        if let Err(e) = nes.advance_instruction(&config.emu_settings) {
                            error!("Error while advancing instruction: {:?}", e)
                        }
                    }
                    if ui.button("Advance end of vblank") {
                        while nes.ppu.in_vblank() {
                            if let Err(e) = nes.advance_instruction(&config.emu_settings) {
                                error!("Error while advancing instruction: {:?}", e)
                            }
                        }
                    }
                    if ui.button("Advance frame") {
                        if let Err(e) = nes.advance_frame(&config.emu_settings) {
                            error!("Error while advancing frame: {:?}", e)
                        }
                    }
                });
                ui.checkbox(
                    "Scanline sprite limit",
                    &mut config.emu_settings.scanline_sprite_limit,
                );
                ui.checkbox(
                    "Always draw sprites on top of background",
                    &mut config.emu_settings.always_sprites_on_top,
                );
                ui.slider("Volume", 0.0, 10.0, &mut config.volume);
                ui.slider("Speed", 0.1, 3.0, &mut config.speed);
                ui.same_line();
                if ui.button("Reset to 1") {
                    config.speed = 1.0;
                }
                ui.checkbox("Verbose Logging", &mut config.verbose_logging);
                ui.checkbox(
                    "Restrict controller input",
                    &mut config.restrict_controller_directions,
                );
                if ui.button("Quick save") {
                    quicksave(nes, config);
                }
                ui.same_line();
                if ui.button("Quick load") {
                    match quickload(config) {
                        Some(n) => *nes = n,
                        None => error!("Encountered an error while quickloading, aborting"),
                    };
                }
                if let Some(c) = ui.begin_combo(
                    "Screen Size",
                    format!("{}x{}px", config.screen_size.0, config.screen_size.1),
                ) {
                    [(256, 240), (256, 224), (240, 212), (224, 192)]
                        .iter()
                        .for_each(|res| {
                            if ui.selectable(format!("{}x{}px", res.0, res.1)) {
                                config.screen_size = *res;
                            }
                        });
                    c.end();
                }
                ui.text(format!("{:?}", &nes.cartridge));
                if ui.collapsing_header("Previous Instructions", TreeNodeFlags::empty()) {
                    nes.previous_states.iter().rev().take(0x20).for_each(|s| {
                        ui.text(format!("{:?}", s));
                    });
                }
                if ui.collapsing_header("Graphics Debug", TreeNodeFlags::empty()) {
                    ui.checkbox("Debug palette", &mut config.emu_settings.use_debug_palette);
                    if ui.collapsing_header("CHR ROM/RAM", TreeNodeFlags::empty()) {
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
                        if let Some(c) = ui.begin_combo("Page", format!("Page {}", self.tile_page))
                        {
                            (0..(self.num_tiles / (self.num_columns * self.num_rows))).for_each(
                                |i| {
                                    if ui.selectable(format!("Page {}", i)) {
                                        self.tile_page = i;
                                    }
                                },
                            );
                            c.end();
                        }
                        ui.slider("Scale", 0.01, 10.0, &mut self.chr_size);
                        let size = [
                            self.chr_size * 8.0 * self.num_columns as f32,
                            self.chr_size * 8.0 * self.num_rows as f32,
                        ];
                        let image = imgui::Image::new(TextureId::new(chr_tex_num as usize), size);
                        image.build(ui);
                    }
                    if ui.collapsing_header("Nametables", TreeNodeFlags::empty()) {
                        if ui.button("Copy snapshot to keyboard") {
                            if let Ok(mut ctx) = ClipboardContext::new() {
                                if ctx
                                    .set_contents(format!("{:?}", nes.ppu.nametable_ram))
                                    .is_err()
                                {
                                    error!("Unable to set contents of clipboard");
                                }
                            }
                        }
                        let f = ui.push_font(self.small_font);
                        ui.text(DebugWindow::format_nametable_text(&nes.ppu, &nes.cartridge));
                        f.pop();
                        imgui::Image::new(TextureId::new(2), [64.0 * 8.0, 60.0 * 8.0]).build(ui);
                    }
                }
                if ui.collapsing_header("Audio", TreeNodeFlags::empty()) {
                    ui.input_text(
                        "Recording file name (.wav)",
                        &mut config.record_audio_filename,
                    )
                    .build();
                    ui.checkbox("Record?", &mut config.record_audio);
                    ui.text(format!("Pulse 0 : {:?}", nes.apu.pulse_registers[0]));
                    ui.text(format!("Pulse 1 : {:?}", nes.apu.pulse_registers[1]));
                    ui.text(format!("Triangle: {:?}", nes.apu.triangle_register));
                    ui.text(format!("Noise   : {:?}", nes.apu.noise_register));
                    ui.text(format!("DMC     : {:?}", nes.apu.dmc_register));
                }
            });
        let draw_data = self.imgui.render();
        self.renderer
            .render(draw_data)
            .expect("Error rendering DearImGui");
        self.window.gl_swap_window();
    }
    fn format_nametable_text(ppu: &Ppu, cartridge: &Cartridge) -> String {
        [
            [
                ppu.top_left_nametable_addr(),
                ppu.top_right_nametable_addr(),
            ],
            [
                ppu.bot_left_nametable_addr(),
                ppu.bot_right_nametable_addr(),
            ],
        ]
        .into_iter()
        .map(|nts| {
            nts.map(|nt| {
                (0..30)
                    .map(|y| {
                        (0..32)
                            .map(|x| {
                                ppu.nametable_ram
                                    [cartridge.transform_nametable_addr(nt + 32 * y + x)]
                            })
                            .collect::<Vec<u8>>()
                    })
                    // Collect into a vector since we still need to merge the left and right
                    .collect::<Vec<Vec<u8>>>()
            })
            .into_iter()
            .fold(vec![Vec::<u8>::new(); 30], |mut a, e| {
                // Combine left and right rows into full row
                e.into_iter().enumerate().for_each(|(i, row)| {
                    let l = a.len();
                    a[i % l].extend_from_slice(row.as_slice());
                });
                a
            })
        })
        // Combine the two halves into one big image
        .fold(String::new(), |a, e| {
            // Combine top and bottom nametables
            format!(
                "{}{}",
                a,
                e.into_iter().fold(String::new(), |a, e| format!(
                    "{}{}\n",
                    a,
                    e.into_iter()
                        .fold(String::new(), |a, e| format!("{}{:2X}", a, e))
                ))
            )
        })
    }
}
