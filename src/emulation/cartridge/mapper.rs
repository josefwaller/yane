use crate::emulation::cartridge::{mappers::NRom, CartridgeMemory};

use super::mappers::{ScRom, UxRom};
pub trait Mapper {
    // Read/write a byte using various memory spaces
    fn read_cpu(&self, cpu_addr: usize, mem: &CartridgeMemory) -> u8;
    fn write_cpu(&mut self, cpu_addr: usize, mem: &mut CartridgeMemory, value: u8);
    fn read_ppu(&self, ppu_addr: usize, mem: &CartridgeMemory) -> u8;
    fn write_ppu(&mut self, ppu_addr: usize, mem: &mut CartridgeMemory, value: u8);
}

pub fn get_mapper(mapper_id: usize) -> Box<dyn Mapper> {
    match mapper_id {
        0 => Box::new(NRom::default()),
        1 => Box::new(ScRom::default()),
        2 => Box::new(UxRom::default()),
        _ => panic!("Unsupported mapper: {}", mapper_id),
    }
}

pub fn bank_addr(bank_size: usize, bank_num: usize, offset: usize) -> usize {
    bank_size * bank_num + (offset % bank_size)
}
