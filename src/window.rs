use crate::{utils, Audio, Controller, Nes, Screen};
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
    video::GLContext,
    Sdl, VideoSubsystem,
};

pub struct Window {
    window: sdl2::video::Window,
    event_loop: sdl2::EventPump,
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

        let event_loop = sdl.event_pump().unwrap();

        let screen = Screen::new(&nes, gl);
        let audio = Audio::new(&sdl);
        Window {
            window,
            event_loop,
            gl_context,
            audio,
            screen,
        }
    }
    pub fn update(&mut self, nes: &mut Nes) {
        // Update inputs
        let keys: Vec<Keycode> = self
            .event_loop
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();
        // P1
        let controller = Controller {
            up: keys.contains(&Keycode::W),
            left: keys.contains(&Keycode::A),
            right: keys.contains(&Keycode::D),
            down: keys.contains(&Keycode::S),
            a: keys.contains(&Keycode::Q),
            b: keys.contains(&Keycode::E),
            start: keys.contains(&Keycode::R),
            select: keys.contains(&Keycode::F),
        };
        nes.set_input(0, controller);
        // P2
        let controller = Controller {
            up: keys.contains(&Keycode::I),
            left: keys.contains(&Keycode::J),
            right: keys.contains(&Keycode::L),
            down: keys.contains(&Keycode::K),
            a: keys.contains(&Keycode::U),
            b: keys.contains(&Keycode::O),
            start: keys.contains(&Keycode::Y),
            select: keys.contains(&Keycode::H),
        };
        nes.set_input(1, controller);

        // Update audio
        self.audio.update_audio(nes);
    }
    pub fn render(&mut self, nes: &mut Nes) -> bool {
        self.window.gl_make_current(&self.gl_context).unwrap();
        for event in self.event_loop.poll_iter() {
            match event {
                Event::Quit { .. } => return true,
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::Close => return true,
                    _ => {}
                },
                _ => {}
            }
        }
        self.screen.render(nes, self.window.size());
        self.window.gl_swap_window();
        false
    }
}
