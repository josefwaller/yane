use crate::{utils, Audio, Controller, Nes, Screen, Settings};
use sdl2::{keyboard::Keycode, video::GLContext, Sdl, VideoSubsystem};

/// A wrapper around `Screen` that provides an SDL2 GL context, and handles input through SDL2.
pub struct Window {
    window: sdl2::video::Window,
    gl_context: GLContext,
    audio: Audio,
    screen: Screen,
}

impl Window {
    pub fn new(video: &VideoSubsystem, sdl: &Sdl) -> Window {
        // Init SDL window
        let window_width = 256 * 3;
        let window_height = 240 * 3;
        let (window, gl_context, gl) =
            utils::create_window(video, "Y.A.N.E.", window_width, window_height);

        let screen = Screen::new(gl);
        let audio = Audio::new(&sdl);
        Window {
            window,
            gl_context,
            audio,
            screen,
        }
    }
    pub fn update(&mut self, nes: &mut Nes, pressed_keys: Vec<Keycode>, settings: &Settings) {
        // Update inputs

        // P1
        let controller = Controller {
            up: pressed_keys.contains(&Keycode::W),
            left: pressed_keys.contains(&Keycode::A),
            right: pressed_keys.contains(&Keycode::D),
            down: pressed_keys.contains(&Keycode::S),
            a: pressed_keys.contains(&Keycode::SPACE),
            b: pressed_keys.contains(&Keycode::M),
            start: pressed_keys.contains(&Keycode::R),
            select: pressed_keys.contains(&Keycode::F),
        };
        nes.set_input(0, controller);
        // P2
        let controller = Controller {
            up: pressed_keys.contains(&Keycode::I),
            left: pressed_keys.contains(&Keycode::J),
            right: pressed_keys.contains(&Keycode::L),
            down: pressed_keys.contains(&Keycode::K),
            a: pressed_keys.contains(&Keycode::U),
            b: pressed_keys.contains(&Keycode::O),
            start: pressed_keys.contains(&Keycode::Y),
            select: pressed_keys.contains(&Keycode::H),
        };
        nes.set_input(1, controller);

        // Update audio
        self.audio.update_audio(nes, settings);
    }
    pub fn render(&mut self, nes: &Nes, settings: &Settings) {
        self.window.gl_make_current(&self.gl_context).unwrap();
        self.screen.render(nes, self.window.size(), settings);
        self.window.gl_swap_window();
    }
    pub fn screen(&mut self) -> &mut Screen {
        &mut self.screen
    }
    pub fn make_gl_current(&mut self) {
        self.window.gl_make_current(&self.gl_context).unwrap()
    }
}
