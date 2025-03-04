use std::fmt::{Debug, Display};

use crate::core::{cartridge::CartridgeMemory, Mapper};
use log::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
/// NROM cartridge mapper (mapper 0)
pub struct NRom {}
#[typetag::serde]
impl Mapper for NRom {
    fn mapper_num(&self) -> u32 {
        0
    }
    fn read_cpu(&self, addr: usize, mem: &CartridgeMemory) -> u8 {
        if addr < 0x6000 {
            return 0;
        }
        if addr < 0x8000 {
            return mem.read_prg_ram(addr - 0x6000);
        }
        mem.read_prg_rom(addr - 0x8000)
    }
    fn write_cpu(&mut self, addr: usize, mem: &mut CartridgeMemory, value: u8) {
        if (0x6000..0x8000).contains(&addr) {
            mem.write_prg_ram(addr - 0x6000, value);
        }
    }
    fn read_ppu_debug(&self, ppu_addr: usize, mem: &CartridgeMemory) -> u8 {
        mem.read_chr(ppu_addr)
    }
    fn write_ppu(&mut self, ppu_addr: usize, mem: &mut CartridgeMemory, value: u8) {
        mem.write_chr(ppu_addr, value)
    }
}

impl Display for NRom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NROM")
    }
}
impl Debug for NRom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}
