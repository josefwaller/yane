use crate::{check_error, set_uniform, utils::*, Nes, Settings};
use glow::*;
use log::*;

/// An NES rendering implementation that uses OpenGL 3.3.
/// Uses the `glow` library to render, so requires a `glow` `Context`.
/// Can be paired with a `Window` to render to an SDL2 window.
pub struct Screen {
    gl: Context,
    // palette: [[f32; 3]; 0x40],
    palette: [[u8; 3]; 0x40],
    // Stuff for rendering to screen
    screen_texture: NativeTexture,
    screen_program: NativeProgram,
    screen_vao: NativeVertexArray,
    // Program for rendering a primitve using wireframe
    wireframe_program: NativeProgram,
    wireframe_vao: NativeVertexArray,
    // Render/misc settings
    settings: Settings,
}
impl Screen {
    // TODO: Rename
    pub fn new(gl: Context) -> Screen {
        unsafe {
            let (screen_vao, screen_program, screen_texture) =
                create_screen_texture(&gl, (256, 240));
            // Load pallete data and convert to RGB values
            let palette_data: &[u8] = include_bytes!("../2C02G_wiki.pal");
            let palette: [[u8; 3]; 64] =
                core::array::from_fn(|i| core::array::from_fn(|j| palette_data[3 * i + j]));
            let wireframe_program = create_program(
                &gl,
                include_str!("../shaders/wireframe.vert"),
                include_str!("../shaders/color.frag"),
            );
            let wireframe_vao = create_f32_slice_vao(
                &gl,
                [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]].as_flattened(),
                2,
            );

            Screen {
                gl,
                screen_program,
                screen_texture,
                screen_vao,
                palette,
                wireframe_program,
                wireframe_vao,
                settings: Settings::default(),
            }
        }
    }
    pub fn render(&mut self, nes: &Nes, window_size: (u32, u32), settings: &Settings) {
        unsafe {
            self.gl.disable(glow::STENCIL_TEST);
            self.gl.disable(glow::DEPTH_TEST);

            self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
            check_error!(self.gl);
            self.gl.use_program(Some(self.screen_program));
            const TEX_NUM: i32 = 1;
            self.gl.active_texture(glow::TEXTURE0 + TEX_NUM as u32);
            self.gl
                .bind_texture(glow::TEXTURE_2D, Some(self.screen_texture));
            let texture_data: Vec<u8> = nes
                .ppu
                .output
                .as_flattened()
                .iter()
                .map(|i| self.palette[*i & 0x3F])
                .flatten()
                .collect();
            self.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGB as i32,
                256,
                240,
                0,
                glow::RGB,
                glow::UNSIGNED_BYTE,
                Some(&texture_data),
            );
            set_uniform!(
                self.gl,
                self.screen_program,
                "screenSize",
                uniform_2_f32,
                settings.screen_size.0 as f32,
                settings.screen_size.1 as f32
            );
            check_error!(self.gl);
            let loc = self
                .gl
                .get_uniform_location(self.screen_program, "renderedTexture");
            self.gl.uniform_1_i32(loc.as_ref(), TEX_NUM);
            check_error!(self.gl);
            self.gl
                .viewport(0, 0, window_size.0 as i32, window_size.1 as i32);
            check_error!(self.gl);
            self.gl.bind_vertex_array(Some(self.screen_vao));
            check_error!(self.gl);
            self.gl.draw_arrays(glow::TRIANGLES, 0, 6);
            check_error!(self.gl);

            // Render wireframe box around each OAM object
            if settings.oam_debug {
                self.gl.use_program(Some(self.wireframe_program));
                self.gl.bind_vertex_array(Some(self.wireframe_vao));
                check_error!(self.gl);
                set_uniform!(
                    self.gl,
                    self.wireframe_program,
                    "screenSize",
                    uniform_2_f32,
                    settings.screen_size.0 as f32,
                    settings.screen_size.1 as f32
                );
                check_error!(self.gl);
                nes.ppu.oam.chunks(4).enumerate().for_each(|(i, obj)| {
                    set_uniform!(
                        self.gl,
                        self.wireframe_program,
                        "position",
                        uniform_2_f32,
                        obj[3] as f32,
                        obj[0] as f32 + 1.0
                    );
                    let color = if i == 0 {
                        [0.0, 1.0, 0.0]
                    } else {
                        [1.0, 0.0, 0.0]
                    };
                    set_uniform!(
                        self.gl,
                        self.wireframe_program,
                        "inColor",
                        uniform_3_f32,
                        color[0],
                        color[1],
                        color[2]
                    );
                    set_uniform!(
                        self.gl,
                        self.wireframe_program,
                        "sizes",
                        uniform_2_f32,
                        1.0,
                        if nes.ppu.is_8x16_sprites() { 2.0 } else { 1.0 }
                    );
                    self.gl.draw_arrays(glow::LINE_LOOP, 0, 4);
                });
            }
        }
    }
    pub fn set_settings(&mut self, settings: Settings) {
        self.settings = settings;
    }
}
