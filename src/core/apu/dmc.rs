use log::*;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::core::Cartridge;

pub const DMC_RATES: [u32; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
];

#[derive(Clone, Serialize, Deserialize)]
/// The DMC register of the NES.
///
/// A delta-modulation sound register in the NES.
/// Takes 1-bit delta encoded samples as input and outputs a value
/// between 0 and 127 to the APU's mixer.
/// Can also trigger CPU interupts when the sample ends, though this is rarely
/// implemented in any games.
pub struct DmcRegister {
    /// Whether the IRQ is enabled
    pub irq_enabled: bool,
    /// The IRQ flag
    pub irq_flag: bool,
    /// Whether to repeat the sample after playing it
    pub repeat: bool,
    /// The DMC rate
    pub rate: u32,
    /// The DMC's timer's value
    pub timer: u32,
    /// The value to reload the timer with when it runs out
    pub time_reload: u32,
    /// Address of the sample, in CPU memory space
    pub sample_addr: usize,
    /// Length of the sample in bytes
    pub sample_len: usize,
    /// Index of the byte currently read from the sample
    /// Set to 0 when reloading
    pub sample_index: usize,
    /// Number of bytes remaining in the sample
    pub bytes_remaining: usize,
    /// Byte of the sample currently in buffer
    pub sample: u8,
    /// Number of bits left in `sample` for the DMC to read
    pub bits_left: u32,
    /// The current output of the DMC
    pub output: u32,
    /// The DMC silent flag
    pub silent: bool,
}
impl Debug for DmcRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "bytes_remaining={:X} silent={} timer={:3X} repeat={} sample_addr={:X} sample_len={:X} IRQ={}, output={}",
            self.bytes_remaining,
            self.silent,
            self.time_reload,
            self.repeat,
            self.sample_addr,
            self.sample_len,
            self.irq_enabled,
            self.output
        )
    }
}
impl Default for DmcRegister {
    fn default() -> Self {
        DmcRegister {
            irq_enabled: false,
            irq_flag: false,
            repeat: false,
            rate: DMC_RATES[0],
            timer: 0,
            time_reload: 0,
            sample_len: 0,
            sample_index: 0,
            bytes_remaining: 0,
            bits_left: 0,
            sample_addr: 0,
            sample: 0,
            output: 0,
            silent: true,
        }
    }
}
impl DmcRegister {
    /// Enable or disable the DMC
    pub fn set_enabled(&mut self, enabled: bool) {
        // Clear IRQ flag
        self.irq_flag = false;
        if enabled {
            if self.bytes_remaining == 0 {
                self.silent = false;
                self.timer = 0;
                self.bits_left = 0;
                self.bytes_remaining = self.sample_len;
                self.sample_index = self.sample_addr;
            }
        } else {
            self.bytes_remaining = 0;
        }
    }
}
impl DmcRegister {
    /// Load a new byte into the DMC's sample buffer
    /// * `cartridge`: The cartridge currently inserted in the NES
    pub fn load_sample(&mut self, cartridge: &mut Cartridge) {
        self.sample = cartridge.read_cpu(self.sample_index);
        if self.sample_index < 0xC000 {
            error!("Invalid sample index {:X}", self.sample);
        }
        self.bits_left = 8;
    }
}
