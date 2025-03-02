//! The actual emulation code, provided as an out-of-the-box library.
//!
//! A library for emulating the behaviour of the Nintendo Entertainment System.
//! Contains the entire state of the machine,and updates it accordingly as the NES is advanced.
//! Stores the CRT TV output as a 2D array.
//! ```
//! use yane::core::{Nes, Controller, Settings};
//! // The actual state of the NES
//! let mut nes = Nes::new();
//! // Various configurable settings for how to run the emulator
//! let settings = Settings::default();
//! // Advance the NES by 1 instruction
//! nes.step();
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
//! let output = nes.ppu.output;
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

pub const DEBUG_PALETTE: [u8; 32] = [
    0x1D, 0x01, 0x11, 0x21, 0x1D, 0x05, 0x15, 0x25, 0x1D, 0x09, 0x19, 0x29, 0x1D, 0x06, 0x16, 0x26,
    0x1D, 0x13, 0x23, 0x33, 0x1D, 0x17, 0x27, 0x37, 0x1D, 0x1B, 0x2B, 0x3B, 0x1D, 0x18, 0x28, 0x38,
];

pub const CPU_CLOCK_SPEED: u32 = 1_789_000;
pub const CARTRIDGE_IRQ_ADDR: usize = 0xFFFE;
pub const RESET_IRQ_ADDR: usize = 0xFFFC;
pub const NMI_IRQ_ADDR: usize = 0xFFFA;
