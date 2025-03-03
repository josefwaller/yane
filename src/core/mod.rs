//! The actual emulation code, provided as a library.
//!
//! A library for emulating the behaviour of the Nintendo Entertainment System.
//! Contains the entire state of the machine, and updates it accordingly as the NES is advanced.
//! Stores the visual output as a 2D array representing the screen, and the audio output
//! as a [VecDeque][std::collections::VecDeque] of samples.
//! ```
//! use yane::core::{Nes, Controller, Settings, HV_TO_RGB};
//! // The actual state of the NES
//! let mut nes = Nes::new();
//! // Various configurable settings for how to run the emulator
//! let settings = Settings::default();
//! // Advance the NES by 1 instruction
//! nes.advance_instruction(&settings);
//! // Advance the NES by 1 frame (continue advancing until a VBlank interval is triggered)
//! nes.advance_frame(&settings);
//! // Press the A button on player 1's controller
//! nes.set_input(0, Controller {
//!   up: false,
//!   left: false,
//!   right: false,
//!   down: false,
//!   a: false,
//!   b: false,
//!   start: false,
//!   select: false
//! });
//! // Read the screen output
//! let screen_output = nes.ppu.output;
//! // Get the RGB value of the top-left pixel
//! let rgb_output = HV_TO_RGB[screen_output[0][0]];
//! println!("Top left pixel is R={} B={} G={}", rgb_output[0], rgb_output[1], rgb_output[2]);
//! // Get the audio output as a vector of samples
//! let audio_output = nes.apu.sample_queue();
//! println!("Read {} audio samples", audio_output.len());
//! ```
mod nes;
pub use nes::{Nes, NesState};
mod cpu;
pub use cpu::Cpu;
mod apu;
pub use apu::Apu;
mod status_register;
pub use status_register::StatusRegister;
mod cartridge;
pub use cartridge::*;
pub mod opcodes;
mod ppu;
pub use ppu::Ppu;
mod controller;
pub use controller::Controller;
mod settings;
pub use settings::Settings;

/// The debug palette, used instead of the palette ram if [Settings::use_debug_palette] is `true`.
pub const DEBUG_PALETTE: [u8; 32] = [
    0x1D, 0x01, 0x11, 0x21, 0x1D, 0x05, 0x15, 0x25, 0x1D, 0x09, 0x19, 0x29, 0x1D, 0x06, 0x16, 0x26,
    0x1D, 0x13, 0x23, 0x33, 0x1D, 0x17, 0x27, 0x37, 0x1D, 0x1B, 0x2B, 0x3B, 0x1D, 0x18, 0x28, 0x38,
];

/// The approximate clock speed of an NES, in hertz.
pub const CPU_CLOCK_SPEED: u32 = 1_789_000;
/// The location of the cartridge's interrupt vector.
pub const CARTRIDGE_IRQ_ADDR: usize = 0xFFFE;
/// The location of the reset interrupt vector.
pub const RESET_IRQ_ADDR: usize = 0xFFFC;
/// The location of the non-maskable interrupt's vector.
pub const NMI_IRQ_ADDR: usize = 0xFFFA;
const PALETTE_DATA: &[u8; 1536] = include_bytes!("../2C02G_wiki.pal");
/// Map of the console's Hue/Value output to RGB values
pub const HV_TO_RGB: &[[u8; 3]] =
    // This is unsafe but it should only be evaluated at compile time
    unsafe { core::slice::from_raw_parts(PALETTE_DATA.as_ptr() as *const [u8; 3], 3 * 64) };
