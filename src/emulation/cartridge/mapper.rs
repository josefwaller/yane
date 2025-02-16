use serde::{de::Visitor, Deserialize, Deserializer, Serialize};

use super::{
    mappers::{CnRom, NRom, PxRom, SxRom, TxRom, UxRom},
    CartridgeMemory, NametableArrangement,
};
#[typetag::serde(tag = "mapper")]
pub trait Mapper {
    // Read/write a byte using various memory spaces
    // Used by games, for debug reading use read_ppu_debug
    fn read_cpu(&self, cpu_addr: usize, mem: &CartridgeMemory) -> u8;
    fn write_cpu(&mut self, cpu_addr: usize, mem: &mut CartridgeMemory, value: u8);
    fn read_ppu(&mut self, ppu_addr: usize, mem: &CartridgeMemory) -> u8 {
        self.read_ppu_debug(ppu_addr, mem)
    }
    fn write_ppu(&mut self, ppu_addr: usize, mem: &mut CartridgeMemory, value: u8);
    // Read PPU for debug purposes only, not changing the cartridge state at all
    fn read_ppu_debug(&self, ppu_addr: usize, mem: &CartridgeMemory) -> u8;
    fn nametable_arrangement(&self) -> Option<NametableArrangement> {
        None
    }
    fn get_debug_string(&self) -> String {
        "".to_string()
    }
    fn advance_cpu_cycles(&mut self, _cycles: u32) {}
    fn set_addr_value(&mut self, _addr: u32) {}
    // Get the address if the CPU has generated an IRQ, or None if it hasn't
    fn irq_addr(&mut self) -> Option<usize> {
        None
    }
    fn mapper_num(&self) -> u32;
}
pub fn get_mapper(mapper_id: usize) -> Box<dyn Mapper> {
    match mapper_id {
        0 => Box::new(NRom::default()),
        1 => Box::new(SxRom::default()),
        2 => Box::new(UxRom::default()),
        3 => Box::new(CnRom::default()),
        4 => Box::new(TxRom::default()),
        9 => Box::new(PxRom::default()),
        _ => panic!("Unsupported mapper: {}", mapper_id),
    }
}

pub fn bank_addr(bank_size: usize, bank_num: usize, offset: usize) -> usize {
    bank_size * bank_num + (offset % bank_size)
}
// Get the number of banks in a given section of memory
pub fn num_banks(bank_size: usize, mem: &[u8]) -> usize {
    (mem.len() - 1) / bank_size + 1
}
