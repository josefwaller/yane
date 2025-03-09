#![doc = include_str!(concat!("../", std::env!("CARGO_PKG_README")))]
#[cfg(feature = "sdl")]
pub mod app;
pub mod core;
#[cfg(feature = "sdl")]
pub(crate) mod utils;
