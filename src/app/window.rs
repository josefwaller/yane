use crate::{
    app::Config,
    core::Nes,
    utils::{
        self, check_error, create_f32_slice_vao, create_program, create_screen_texture, set_uniform,
    },
};
use glow::{Context, HasContext, NativeProgram, NativeTexture, NativeVertexArray};
use sdl2::{video::GLContext, VideoSubsystem};

/// The window of the emulator app.
///
/// Responsible for the actual graphical output of the emualator.
/// Acts as the bridge between an [Nes]'s RGB output array, and an SDL window.
/// Takes the RGB output, pipes that data into on OpenGL texture, and renders that texture
/// to the screen of the window.
/// See [Window::render].
pub struct Window {
    window: sdl2::video::Window,
    gl_context: GLContext,
    // screen: Screen,
    gl: Context,
    // Stuff for rendering to screen
    screen_texture: NativeTexture,
    screen_program: NativeProgram,
    screen_vao: NativeVertexArray,
    // Program for rendering a primitve using wireframe
    wireframe_program: NativeProgram,
    wireframe_vao: NativeVertexArray,
    // Screen data buffer
    screen_buffer: [[[u8; 3]; 256]; 240],
}
impl Window {
    /// Create a new [Window] from an SDL video subsystem
    pub fn from_sdl_video(video: &mut VideoSubsystem) -> Window {
        // Set up openGL
        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(3, 3);
        // Init SDL window
        let window_width = 256 * 3;
        let window_height = 240 * 3;
        let (window, gl_context, gl) =
            utils::create_window(video, "Y.A.N.E.", window_width, window_height);

        unsafe {
            let (screen_vao, screen_program, screen_texture) =
                create_screen_texture(&gl, (256, 240));
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
            Window {
                window,
                gl_context,
                // screen,
                gl,
                screen_program,
                screen_texture,
                screen_vao,
                wireframe_program,
                wireframe_vao,
                screen_buffer: [[[0; 3]; 256]; 240],
            }
        }
    }
    /// Render the [Nes]'s video output to the window.
    pub fn render(&mut self, nes: &Nes, config: &Config) {
        self.window.gl_make_current(&self.gl_context).unwrap();
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
            // Copy output
            nes.ppu.rgb_output_buf(&mut self.screen_buffer);
            // Flatten to a single slice
            let texture_data: &[u8] = self.screen_buffer.as_flattened().as_flattened();
            // Pipe to texture
            self.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGB as i32,
                256,
                240,
                0,
                glow::RGB,
                glow::UNSIGNED_BYTE,
                Some(texture_data),
            );
            set_uniform!(
                self.gl,
                self.screen_program,
                "screenSize",
                uniform_2_f32,
                config.screen_size.0 as f32,
                config.screen_size.1 as f32
            );
            check_error!(self.gl);
            let loc = self
                .gl
                .get_uniform_location(self.screen_program, "renderedTexture");
            self.gl.uniform_1_i32(loc.as_ref(), TEX_NUM);
            check_error!(self.gl);
            self.gl.viewport(
                0,
                0,
                self.window.size().0 as i32,
                self.window.size().1 as i32,
            );
            check_error!(self.gl);
            self.gl.bind_vertex_array(Some(self.screen_vao));
            check_error!(self.gl);
            self.gl.draw_arrays(glow::TRIANGLES, 0, 6);
            check_error!(self.gl);

            // Render wireframe box around each OAM object
            if config.oam_debug {
                self.gl.use_program(Some(self.wireframe_program));
                self.gl.bind_vertex_array(Some(self.wireframe_vao));
                check_error!(self.gl);
                set_uniform!(
                    self.gl,
                    self.wireframe_program,
                    "screenSize",
                    uniform_2_f32,
                    config.screen_size.0 as f32,
                    config.screen_size.1 as f32
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
        self.window.gl_swap_window();
    }
    /// Get a mutable reference to the SDL [Window][sdl2::video::Window]
    ///
    /// Allows configuration of the underlying SDL window such as changing title, icon, etc
    pub fn sdl_window(&mut self) -> &mut sdl2::video::Window {
        &mut self.window
    }
}
