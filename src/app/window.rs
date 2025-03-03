use crate::{
    app::{Audio, Config, Screen},
    core::{Controller, Nes},
    utils,
};
use log::*;
use sdl2::{keyboard::Keycode, video::GLContext, Sdl, VideoSubsystem};

use super::{key_map::Key, utils::save_new_savestate};

/// A wrapper around [Screen] that provides an SDL2 GL context, and handles input through SDL2.
pub struct Window {
    window: sdl2::video::Window,
    gl_context: GLContext,
    pub audio: Audio,
    screen: Screen,
    last_keys: Vec<Keycode>,
    game_name: Option<String>,
}
impl Window {
    pub fn new(video: &VideoSubsystem, sdl: &Sdl, game_name: Option<String>) -> Window {
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
            last_keys: Vec::new(),
            game_name,
        }
    }
    fn key_down(k: &Key, keys: &Vec<Keycode>) -> bool {
        keys.contains(&k.code)
    }
    fn key_pressed(&self, k: &Key, keys: &Vec<Keycode>) -> bool {
        Window::key_down(k, keys) && !Window::key_down(k, &self.last_keys)
    }
    fn update_controller(&self, nes: &mut Nes, index: usize, keys: &Vec<Keycode>, config: &Config) {
        let c = &config.key_map.controllers;
        // P1
        let controller = Controller {
            up: Window::key_down(&c[index].up, keys),
            left: Window::key_down(&c[index].left, keys),
            right: Window::key_down(&c[index].right, keys),
            down: Window::key_down(&c[index].down, keys),
            a: Window::key_down(&c[index].a, keys),
            b: Window::key_down(&c[index].b, keys),
            start: Window::key_down(&c[index].start, keys),
            select: Window::key_down(&c[index].select, keys),
        };
        nes.set_input(index, controller);
    }
    pub fn update(&mut self, nes: &mut Nes, keys: &Vec<Keycode>, config: &mut Config) {
        // Update inputs
        self.update_controller(nes, 0, &keys, config);
        self.update_controller(nes, 1, &keys, config);
        let km = &config.key_map;
        if self.key_pressed(&km.pause, keys) {
            config.paused = !config.paused;
        }
        let diff = if self.key_pressed(&km.volume_up, keys) {
            0.1
        } else if self.key_pressed(&km.volume_down, keys) {
            -0.1
        } else {
            0.0
        };
        config.volume = (config.volume + diff).clamp(0.0, 3.0) as f32;
        // Update audio
        self.audio.update_audio(nes, config);

        // Check for quickload
        if self.key_pressed(&km.quicksave, keys) {
            save_new_savestate(nes, config, &self.game_name);
        } else if self.key_pressed(&km.quickload, keys) {
            match &config.quickload_file {
                Some(f) => match std::fs::read(&f) {
                    Ok(data) => match postcard::from_bytes(&data) {
                        Ok(n) => {
                            debug!("Loaded quicksave at {:?}", f);
                            *nes = n;
                        }
                        Err(e) => error!("Unable to deserialize save state {:?}: {}", &f, e),
                    },
                    Err(e) => error!("Unable to read save state {:?}: {}", &f, e),
                },
                None => info!("No save state to quickload from"),
            }
        }

        self.last_keys = keys.clone();
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
