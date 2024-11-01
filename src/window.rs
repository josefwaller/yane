use crate::{Audio, Controller, Nes, Screen};
use sdl2::{event::Event::Quit, keyboard::Keycode, video::GLContext, Sdl};

pub struct Window {
    window: sdl2::video::Window,
    event_loop: sdl2::EventPump,
    // Just needs to stay in scope
    _gl_context: GLContext,
    audio: Audio,
    screen: Screen,
}

impl Window {
    pub fn new(nes: &Nes) -> Window {
        let window_width = 256 * 3;
        let window_height = 240 * 3;
        unsafe {
            let sdl = sdl2::init().unwrap();
            // Setup video
            let video = sdl.video().unwrap();
            let gl_attr = video.gl_attr();
            gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
            gl_attr.set_context_version(3, 3);
            let window = video
                .window("Y.A.N.E", window_width, window_height)
                .opengl()
                .resizable()
                .build()
                .unwrap();
            let gl_context = window.gl_create_context().unwrap();
            let gl =
                glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _);

            window.gl_make_current(&gl_context).unwrap();
            let event_loop = sdl.event_pump().unwrap();
            let screen = Screen::new(&nes, gl);
            let audio = Audio::new(&sdl);
            Window {
                window,
                event_loop,
                // Just needs to stay in scope
                _gl_context: gl_context,
                audio,
                screen,
            }
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
        for event in self.event_loop.poll_iter() {
            match event {
                Quit { .. } => {
                    return true;
                }
                _ => {}
            }
        }
        self.screen.render(nes, self.window.size());
        self.window.gl_swap_window();
        false
    }
}
