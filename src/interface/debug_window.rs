use std::cmp::min;

use log::*;

use crate::{check_error, utils::*, NametableArrangement, Nes, Settings, DEBUG_PALETTE};
use glow::{HasContext, NativeProgram, NativeTexture, VertexArray};
use imgui::{Condition::FirstUseEver, TextureId, TreeNodeFlags};
use imgui_glow_renderer::AutoRenderer;
use imgui_sdl2_support::SdlPlatform;
use sdl2::{event::Event, EventPump, Sdl, VideoSubsystem};
// Renders all the CHR ROM (and CHR RAM TBD) in the cartridge for debug purposes
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
    // Cached nametable RAM
    nametable_ram: Vec<u8>,
    nametable_num: usize,

    chr_tex: NativeTexture,
    nametable_tex: NativeTexture,
    reset_nametable_tex: bool,
    nametable_arrangement: NametableArrangement,
}

impl DebugWindow {
    pub fn new(nes: &Nes, video: &VideoSubsystem, sdl: &Sdl) -> DebugWindow {
        // Figure out how many rows/columns
        let num_tiles =
            (nes.cartridge.memory.chr_rom.len() + nes.cartridge.memory.chr_ram.len()) / 0x10;
        let num_columns = 0x10;
        let num_rows = 0x10;
        // Set window size
        let window_width = 512;
        let window_height = 512;

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
                nametable_ram: Vec::new(),
                nametable_num: 0,
                chr_tex,
                nametable_tex,
                reset_nametable_tex: true,
                nametable_arrangement: NametableArrangement::Horizontal,
            }
        }
    }
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
            .map(|(tile_index, pixel_index)| {
                let index = 4 * palette_indices[*tile_index % palette_indices.len()] + *pixel_index;
                self.palette[palette[index % palette.len()] as usize]
            })
            .flatten()
            .collect()
    }
    pub fn render(&mut self, nes: &Nes, event_pump: &EventPump, settings: &mut Settings) {
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
                if settings.use_debug_palette {
                    &DEBUG_PALETTE
                } else {
                    &nes.ppu.palette_ram
                },
                // Todo: cache this somewhere
                // vec![self.palette_index; self.num_columns * self.num_rows],
                vec![0, 1, 2, 3, 4],
            );
            check_error!(gl);
            let chr_tex_num: i32 = 1;
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
            if self.nametable_num != nes.ppu.base_nametable_num()
                || self.nametable_ram != nes.ppu.nametable_ram
                || self.reset_nametable_tex
                || self.nametable_arrangement != nes.cartridge.nametable_arrangement()
            {
                self.reset_nametable_tex = false;
                self.nametable_num = nes.ppu.base_nametable_num();
                self.nametable_ram = nes.ppu.nametable_ram.to_vec();
                self.nametable_arrangement = nes.cartridge.nametable_arrangement();
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
                .map(|n| {
                    let attr_addr = n.map(|i| i + 0x3C0);
                    (0..30)
                        // For every row
                        .map(move |y| {
                            // For every column
                            (0..64).map(move |x| {
                                let (nt, at) = if x < 32 {
                                    (n[0], attr_addr[0])
                                } else {
                                    (n[1], attr_addr[1])
                                };
                                // Get the tile here, which could be in one of two nametables
                                // Since we want the top left and top right ones to be beside each other
                                let tile_addr = nes.ppu.nametable_tile_addr()
                                    + 0x10
                                        * nes.ppu.nametable_ram[nes
                                            .cartridge
                                            .transform_nametable_addr(nt)
                                            + 32 * y
                                            + x] as usize;
                                let palette_shift = 2 * ((((x % 32) / 2) % 2) + 2 * ((y / 2) % 2));
                                let palette_byte_addr = nes.cartridge.transform_nametable_addr(at)
                                    + 0x08 * (y / 4)
                                    + ((x % 32) / 4);
                                let palette_byte = nes.ppu.nametable_ram[palette_byte_addr];
                                let palette = (palette_byte >> palette_shift) & 0x03;
                                (0..0x10).map(move |j| {
                                    (nes.cartridge.read_ppu(tile_addr + j), palette as usize)
                                })
                            })
                        })
                        .flatten()
                        .flatten()
                })
                .flatten()
                .collect::<Vec<(u8, usize)>>();
                let (nt_tiles, nt_palettes): (Vec<u8>, Vec<usize>) = nametables.into_iter().unzip();
                let p: Vec<usize> = nt_palettes.into_iter().step_by(0x10).collect();
                let tex_data = self.transform_chr_data(
                    nt_tiles.as_slice(),
                    64,
                    60,
                    if settings.use_debug_palette {
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
                    8 * 64 as i32,
                    8 * 60 as i32,
                    0,
                    glow::RGB,
                    glow::UNSIGNED_BYTE,
                    Some(&tex_data),
                );
                check_error!(gl);
            }
            gl.clear_color(0.0, 0.0, 0.0, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);

            // Draw imgui
            self.platform
                .prepare_frame(&mut self.imgui, &self.window, event_pump);
            let ui = self.imgui.new_frame();
            if ui.collapsing_header("Settings", TreeNodeFlags::empty()) {
                let p = settings.use_debug_palette;
                ui.checkbox("Debug palette", &mut settings.use_debug_palette);
                if p != settings.use_debug_palette {
                    self.reset_nametable_tex = true;
                }
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
                let s = nes.cartridge.debug_string();
                if !s.is_empty() {
                    ui.text(s);
                }
            }
            if ui.collapsing_header("Previous Instructions", TreeNodeFlags::empty()) {
                nes.previous_states.iter().rev().take(0x20).for_each(|s| {
                    ui.text(format!("{:?}", s));
                });
            }
            if ui.collapsing_header("Graphics Debug", TreeNodeFlags::empty()) {
                if let Some(c) =
                    ui.begin_combo("Palette", format!("Palette {}", self.palette_index))
                {
                    (0..8).for_each(|i| {
                        let label = format!("Palette {}", i);
                        if ui.selectable(label) {
                            self.palette_index = i;
                            self.reset_nametable_tex = true;
                        }
                    });
                    c.end();
                }
                if ui.collapsing_header("CHR ROM/RAM", TreeNodeFlags::empty()) {
                    if let Some(c) = ui.begin_combo("Page", format!("Page {}", self.tile_page)) {
                        (0..(self.num_tiles / (self.num_columns * self.num_rows))).for_each(|i| {
                            if ui.selectable(format!("Page {}", i)) {
                                self.tile_page = i;
                            }
                        });
                        c.end();
                    }
                    ui.slider("Scale", 0.01, 10.0, &mut self.chr_size);
                    let size = [
                        self.chr_size * 8.0 * self.num_columns as f32,
                        self.chr_size * 8.0 * self.num_rows as f32,
                    ];
                    check_error!(gl);
                    let image = imgui::Image::new(TextureId::new(chr_tex_num as usize), size);
                    image.build(&ui);
                }
                if ui.collapsing_header("Nametables", TreeNodeFlags::empty()) {
                    if ui.button("Refresh") {
                        self.reset_nametable_tex = true;
                    }
                    imgui::Image::new(TextureId::new(2), [64.0 * 8.0, 60.0 * 8.0]).build(&ui);
                }
            }
            let draw_data = self.imgui.render();
            self.renderer
                .render(draw_data)
                .expect("Error rendering DearImGui");
        }
        self.window.gl_swap_window();
    }
}
