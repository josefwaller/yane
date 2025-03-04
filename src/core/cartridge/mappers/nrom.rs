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
            // Todo: Figure out if this is correct
            if mem.prg_ram.is_empty() {
                return 0;
            }
            return mem.prg_ram[(addr - 0x6000) % mem.prg_ram.len()];
        }
        mem.prg_rom[(addr - 0x8000) % mem.prg_rom.len()]
    }
    fn write_cpu(&mut self, addr: usize, mem: &mut CartridgeMemory, value: u8) {
        if addr < 0x6000 {
        } else if addr < 0x8000 {
            let len = mem.prg_ram.len();
            if len == 0 {
                info!(
                    "Tried to write PRG RAM that doesn't exist (ADDR = {:X})",
                    addr
                );
                return;
            }
            mem.prg_ram[(addr - 0x6000) % len] = value;
        }
    }
    fn read_ppu_debug(&self, ppu_addr: usize, mem: &CartridgeMemory) -> u8 {
        if mem.chr_ram.is_empty() {
            return mem.chr_rom[ppu_addr % mem.chr_rom.len()];
        }
        mem.chr_ram[ppu_addr % mem.chr_ram.len()]
    }
    fn write_ppu(&mut self, ppu_addr: usize, mem: &mut CartridgeMemory, value: u8) {
        if !mem.chr_ram.is_empty() {
            let len = mem.chr_ram.len();
            mem.chr_ram[ppu_addr % len] = value;
        } else {
            warn!("Trying to write to CHR RAM but there is none present");
        }
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
