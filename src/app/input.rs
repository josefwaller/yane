use crate::{
    app::Config,
    core::{Controller, Nes},
};
use log::*;

use super::{key_map::Key, utils::save_new_savestate};
use sdl2::keyboard::Keycode;

/// Handles updating the input in the emulator
///
/// Basically takes the SDL keyboard state, conputes the equivalent NES controller states and
/// updates them in the emulator.
/// Also responsible for updating the app specific settings, such as increasing/decreasing volume.
pub struct Input {
    last_keys: Vec<Keycode>,
}

impl Input {
    pub fn new() -> Input {
        Input {
            last_keys: Vec::new(),
        }
    }
    fn key_down(k: &Key, keys: &[Keycode]) -> bool {
        keys.contains(&k.code)
    }
    fn key_pressed(&self, k: &Key, keys: &[Keycode]) -> bool {
        Input::key_down(k, keys) && !Input::key_down(k, &self.last_keys)
    }
    fn update_controller(&self, nes: &mut Nes, index: usize, keys: &[Keycode], config: &Config) {
        let c = &config.key_map.controllers;
        // P1
        let controller = Controller {
            up: Input::key_down(&c[index].up, keys),
            left: Input::key_down(&c[index].left, keys),
            right: Input::key_down(&c[index].right, keys),
            down: Input::key_down(&c[index].down, keys),
            a: Input::key_down(&c[index].a, keys),
            b: Input::key_down(&c[index].b, keys),
            start: Input::key_down(&c[index].start, keys),
            select: Input::key_down(&c[index].select, keys),
        };
        nes.set_input(index, controller);
    }
    pub fn update(&mut self, nes: &mut Nes, keys: &[Keycode], config: &mut Config) {
        // Update inputs
        self.update_controller(nes, 0, keys, config);
        self.update_controller(nes, 1, keys, config);
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
        config.volume = (config.volume + diff).clamp(0.0, 3.0);

        // Check for quickload
        if self.key_pressed(&km.quicksave, keys) {
            // save_new_savestate(nes, config, &self.game_name);
        } else if self.key_pressed(&km.quickload, keys) {
            match &config.quickload_file {
                Some(f) => match std::fs::read(f) {
                    Ok(data) => match postcard::from_bytes(&data) {
                        Ok(n) => {
                            info!("Loaded quicksave at {:?}", f);
                            *nes = n;
                        }
                        Err(e) => error!("Unable to deserialize save state {:?}: {}", &f, e),
                    },
                    Err(e) => error!("Unable to read save state {:?}: {}", &f, e),
                },
                None => info!("No save state to quickload from"),
            }
        }

        self.last_keys = keys.to_owned();
    }
}
