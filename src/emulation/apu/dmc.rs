use log::*;
use std::fmt::Debug;

use crate::Cartridge;

pub const DMC_RATES: [u32; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
];

#[derive(Clone)]
pub struct DmcRegister {
    pub irq_enabled: bool,
    pub irq_flag: bool,
    pub repeat: bool,
    pub rate: u32,
    pub timer: u32,
    pub time_reload: u32,
    // Address of the sample, in CPU memory space
    pub sample_addr: usize,
    // Length of hte sample in bytes
    pub sample_len: usize,
    // Index of the byte currently read from the sample
    // Set to 0 when reloading
    pub sample_index: usize,
    // Number of bytes remaining in the sample
    pub bytes_remaining: usize,
    // Byte of the sample currently in buffer
    pub sample: u8,
    pub bits_left: u32,
    pub output: u32,
    pub silent: bool,
}
impl Debug for DmcRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "bytes_remaining={:X} silent={} timer={:3X} repeat={} sample_addr={:X} sample_len={:X} IRQ={}",
            self.bytes_remaining,
            self.silent,
            self.time_reload,
            self.repeat,
            self.sample_addr,
            self.sample_len,
            self.irq_enabled,
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
    pub fn muted(&self) -> bool {
        false
    }
    pub fn value(&self) -> u32 {
        self.output
    }
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
    pub fn load_sample(&mut self, cartridge: &mut Cartridge) {
        self.sample = cartridge.read_cpu(self.sample_index);
        if self.sample_index < 0xC000 {
            error!("Invalid sample index {:X}", self.sample);
        }
        self.bits_left = 8;
    }
}
