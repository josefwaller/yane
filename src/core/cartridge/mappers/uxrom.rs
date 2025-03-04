use std::fmt::{Debug, Display};

use crate::core::{
    cartridge::{
        mapper::{bank_addr, num_banks},
        CartridgeMemory,
    },
    Mapper,
};
use log::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
/// UxROM cartridge mapper and variants (mapper 2)
pub struct UxRom {
    bank: usize,
}

const BANK_SIZE: usize = 0x4000;
#[typetag::serde]
impl Mapper for UxRom {
    fn mapper_num(&self) -> u32 {
        2
    }
    fn read_cpu(&self, cpu_addr: usize, mem: &CartridgeMemory) -> u8 {
        if cpu_addr < 0x8000 {
            warn!("Reading PRG RAM when there is none (ADDR = {:X})", cpu_addr);
            0
        } else if cpu_addr >= 0xC000 {
            // Fixed to last bank
            mem.read_prg_rom(bank_addr(
                BANK_SIZE,
                num_banks(BANK_SIZE, &mem.prg_rom) - 1,
                cpu_addr,
            ))
        } else {
            mem.prg_rom[bank_addr(BANK_SIZE, self.bank, cpu_addr)]
        }
    }
    fn write_cpu(&mut self, _cpu_addr: usize, _mem: &mut CartridgeMemory, value: u8) {
        self.bank = value as usize;
    }
    fn read_ppu_debug(&self, ppu_addr: usize, mem: &CartridgeMemory) -> u8 {
        // No switching
        if mem.chr_ram.is_empty() {
            return mem.chr_rom[ppu_addr % mem.chr_rom.len()];
        }
        mem.chr_ram[ppu_addr % mem.chr_ram.len()]
    }
    fn write_ppu(&mut self, ppu_addr: usize, mem: &mut CartridgeMemory, value: u8) {
        if mem.chr_ram.is_empty() {
            warn!(
                "Tring to write to CHR RAM when there is none (Address = {:X})",
                ppu_addr
            );
        } else {
            let l = mem.chr_ram.len();
            mem.chr_ram[ppu_addr % l] = value;
        }
    }
}

impl Display for UxRom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UxROM")
    }
}
impl Debug for UxRom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UxROM bank={}", self.bank)
    }
}
