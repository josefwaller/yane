use std::fmt::{Debug, Display};

use crate::core::{cartridge::mapper::bank_addr, CartridgeMemory, Mapper};
use log::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
/// CnROM cartridge mapper (mapper 3)
pub struct CnRom {
    chr_bank_select: usize,
}

#[typetag::serde]
impl Mapper for CnRom {
    fn mapper_num(&self) -> u32 {
        3
    }
    fn read_cpu(&self, cpu_addr: usize, mem: &CartridgeMemory) -> u8 {
        if cpu_addr < 0x8000 {
            warn!("Invalid read at address {:X}", cpu_addr);
            0
        } else {
            mem.read_prg_rom(cpu_addr)
        }
    }
    fn write_cpu(&mut self, cpu_addr: usize, _mem: &mut CartridgeMemory, value: u8) {
        if cpu_addr >= 0x8000 {
            self.chr_bank_select = (value & 0x03) as usize;
        }
    }
    fn read_ppu_debug(&self, ppu_addr: usize, mem: &CartridgeMemory) -> u8 {
        mem.read_chr(bank_addr(0x2000, self.chr_bank_select, ppu_addr))
    }
    fn write_ppu(&mut self, _ppu_addr: usize, _mem: &mut CartridgeMemory, _value: u8) {
        // Does nothing
    }
}

impl Display for CnRom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CnROM")
    }
}
impl Debug for CnRom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} chr_bank_select={}", self, self.chr_bank_select)
    }
}
