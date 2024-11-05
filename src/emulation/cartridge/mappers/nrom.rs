use crate::{emulation::cartridge::CartridgeMemory, Mapper};

#[derive(Default)]
pub struct NRom {}
impl Mapper for NRom {
    // /// Read a byte from the cartridge given an address in the CPU's memory space
    // pub fn read_byte(&self, addr: usize) -> u8 {
    //     if addr < 0x8000 {
    //         return self.prg_ram[(addr - 0x6000) % self.prg_ram.len()];
    //     }
    //     self.prg_rom[(addr - 0x8000) as usize % self.prg_rom.len()]
    // }
    // /// Write a byte from the cartridge given an address in the CPU's memory space
    // pub fn write_byte(&mut self, addr: usize, value: u8) {
    //     self.prg_ram[addr - 0x6000] = value
    // }
    fn read_cpu(&self, addr: usize, mem: &CartridgeMemory) -> u8 {
        if addr < 0x8000 {
            return mem.prg_ram[(addr - 0x6000) % mem.prg_ram.len()];
        }
        mem.prg_rom[(addr - 0x8000) % mem.prg_rom.len()]
    }
    fn write_cpu(&mut self, addr: usize, mem: &mut CartridgeMemory, value: u8) {
        if addr < 0x8000 {
            let len = mem.prg_ram.len();
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
