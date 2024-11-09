use crate::{emulation::cartridge::CartridgeMemory, Mapper};
use log::*;

#[derive(Default)]
pub struct NRom {}
impl Mapper for NRom {
    fn read_cpu(&self, addr: usize, mem: &CartridgeMemory) -> u8 {
        if addr < 0x6000 {
            return 0;
        }
        if addr < 0x8000 {
            // Todo: Figure out if this is correct
            if mem.prg_ram.len() == 0 {
                return 0;
            }
            return mem.prg_ram[(addr - 0x6000) % mem.prg_ram.len()];
        }
        mem.prg_rom[(addr - 0x8000) % mem.prg_rom.len()]
    }
    fn write_cpu(&mut self, addr: usize, mem: &mut CartridgeMemory, value: u8) {
        if addr < 0x6000 {
            return;
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
        } else {
            let len = mem.prg_rom.len();
            mem.prg_rom[(addr - 0x8000) % len] = value;
        }
    }
    fn read_ppu(&self, ppu_addr: usize, mem: &CartridgeMemory) -> u8 {
        unimplemented!("Read PPU")
    }
    fn write_ppu(&mut self, ppu_addr: usize, mem: &mut CartridgeMemory, value: u8) {
        unimplemented!("Write PPU")
    }
}
