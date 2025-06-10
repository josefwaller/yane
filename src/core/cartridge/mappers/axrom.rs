use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

use crate::core::{cartridge::mapper::bank_addr, CartridgeMemory, Mapper, NametableArrangement};

#[derive(Default, Serialize, Deserialize)]
pub struct AxRom {
    prg_bank: usize,
    vram_select: usize,
}

impl Display for AxRom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AxRom")
    }
}

impl Debug for AxRom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AxRom")
    }
}

#[typetag::serde]
impl Mapper for AxRom {
    fn read_cpu(&self, cpu_addr: usize, mem: &crate::core::CartridgeMemory) -> u8 {
        mem.read_prg_rom(bank_addr(0x8000, self.prg_bank, cpu_addr))
    }
    fn write_cpu(&mut self, cpu_addr: usize, mem: &mut crate::core::CartridgeMemory, value: u8) {
        self.prg_bank = (value & 0x07) as usize;
        self.vram_select = ((value & 0x10) >> 4) as usize
    }
    fn read_ppu_debug(&self, ppu_addr: usize, mem: &crate::core::CartridgeMemory) -> u8 {
        mem.read_chr(ppu_addr)
    }
    fn write_ppu(&mut self, ppu_addr: usize, mem: &mut crate::core::CartridgeMemory, value: u8) {
        mem.write_chr(ppu_addr, value);
    }
    fn mapper_num(&self) -> u32 {
        7
    }
    fn nametable_arrangement(&self, _mem: &CartridgeMemory) -> NametableArrangement {
        NametableArrangement::Custom
    }
    fn transform_nametable_addr(&self, addr: usize) -> usize {
        self.vram_select * 0x400 + (addr % 0x400)
    }
}
