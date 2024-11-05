use crate::{emulation::cartridge::CartridgeMemory, Mapper};

#[derive(Default)]
pub struct UxRom {
    bank: usize,
}

const BANK_SIZE: usize = 0x4000;
impl Mapper for UxRom {
    fn read_cpu(&self, cpu_addr: usize, mem: &CartridgeMemory) -> u8 {
        if cpu_addr < 0x8000 {
            println!(
                "Warning - trying to read PRG RAM when there is none (ADDR = {:X})",
                cpu_addr
            );
            return 0;
        }
        // Fixed to last bank
        if cpu_addr >= 0xC000 {
            let last_bank = &mem.prg_rom[(mem.prg_rom.len() - BANK_SIZE)..(mem.prg_rom.len())];
            return last_bank[(cpu_addr - 0xC000) % last_bank.len()];
        }
        let final_addr = (cpu_addr - 0x8000 + self.bank * BANK_SIZE);
        mem.prg_rom[final_addr % mem.prg_rom.len()]
    }
    fn write_cpu(&mut self, cpu_addr: usize, mem: &mut CartridgeMemory, value: u8) {
        self.bank = value as usize;
    }
    fn read_ppu(&self, ppu_addr: usize, mem: &CartridgeMemory) -> u8 {
        0
    }
    fn write_ppu(&mut self, ppu_addr: usize, mem: &mut CartridgeMemory, value: u8) {}
}