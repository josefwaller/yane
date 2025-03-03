//! The application code, used when running Yane as a standalone emulator.
//!
//! A Nintendo Entertainment System emulator.
//! Uses OpenGL and SDL to create a multiplatform window.
//! Allows for customizing controls, screen size, volume, speed, and others (see [Config]).
//! All of the actual NES emulation is done by importing from [core]
mod screen;
pub use screen::Screen;
mod audio;
pub use audio::Audio;
mod window;
pub use window::Window;
mod debug_window;
pub use debug_window::DebugWindow;
mod key_map;
pub use key_map::KeyMap;
mod config;
pub mod utils;
pub use config::Config;
