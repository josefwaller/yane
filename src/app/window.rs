use crate::{
    app::{Config, Screen},
    core::Nes,
    utils,
};
use sdl2::{video::GLContext, VideoSubsystem};

/// A wrapper around [Screen] that provides an SDL2 GL context, and handles input through SDL2.
pub struct Window {
    window: sdl2::video::Window,
    gl_context: GLContext,
    screen: Screen,
}
impl Window {
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

        let screen = Screen::new(gl);
        Window {
            window,
            gl_context,
            screen,
        }
    }
    pub fn render(&mut self, nes: &Nes, config: &Config) {
        self.window.gl_make_current(&self.gl_context).unwrap();
        self.screen.render(nes, self.window.size(), config);
        self.window.gl_swap_window();
    }
    pub fn screen(&mut self) -> &mut Screen {
        &mut self.screen
    }
    pub fn make_gl_current(&mut self) {
        self.window.gl_make_current(&self.gl_context).unwrap()
    }
}
