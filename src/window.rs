use crate::{utils, Audio, Controller, Nes, Screen};
use sdl2::{keyboard::Keycode, video::GLContext, Sdl, VideoSubsystem};

pub struct Window {
    window: sdl2::video::Window,
    // event_loop: sdl2::EventPump,
    gl_context: GLContext,
    audio: Audio,
    screen: Screen,
}

impl Window {
    pub fn new(nes: &Nes, video: &VideoSubsystem, sdl: &Sdl) -> Window {
        // Init SDL window
        let window_width = 256 * 3;
        let window_height = 240 * 3;
        let (window, gl_context, gl) =
            utils::create_window(video, "Y.A.N.E.", window_width, window_height);

        // let event_loop = sdl.event_pump().unwrap();

        let screen = Screen::new(&nes, gl);
        let audio = Audio::new(&sdl);
        Window {
            window,
            // event_loop,
            gl_context,
            audio,
            screen,
        }
    }
    pub fn update(&mut self, nes: &mut Nes, pressed_keys: Vec<Keycode>) {
        // Update inputs

        // P1
        let controller = Controller {
            up: pressed_keys.contains(&Keycode::W),
            left: pressed_keys.contains(&Keycode::A),
            right: pressed_keys.contains(&Keycode::D),
            down: pressed_keys.contains(&Keycode::S),
            a: pressed_keys.contains(&Keycode::Q),
            b: pressed_keys.contains(&Keycode::E),
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
        self.audio.update_audio(nes);
    }
    pub fn render(&mut self, nes: &mut Nes) {
        self.window.gl_make_current(&self.gl_context).unwrap();
        self.screen.render(nes, self.window.size());
        self.window.gl_swap_window();
    }
}
