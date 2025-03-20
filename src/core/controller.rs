use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm-bindgen")]
extern crate wasm_bindgen;

/// An NES controller
///
/// Used to represent the controller's state in the emulator.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "wasm-bindgen", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Controller {
    pub up: bool,
    pub left: bool,
    pub right: bool,
    pub down: bool,
    pub start: bool,
    pub select: bool,
    pub a: bool,
    pub b: bool,
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}

impl Controller {
    pub fn new() -> Controller {
        Controller {
            up: false,
            left: false,
            right: false,
            down: false,
            start: false,
            select: false,
            a: false,
            b: false,
        }
    }
}
