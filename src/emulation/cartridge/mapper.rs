use super::{
    mappers::{CnRom, NRom, PxRom, SxRom, TxRom, UxRom},
    CartridgeMemory, NametableArrangement,
};
use std::fmt::{Debug, Display};
#[typetag::serde(tag = "mapper")]
/// Interface for the various cartridge mappers.
/// Reading and writing bytes will go through these functions,
/// which may transform the address depending on the mapper's state.
pub trait Mapper: Debug + Display {
    /// Read a byte given an address in CPU memory space
    fn read_cpu(&self, cpu_addr: usize, mem: &CartridgeMemory) -> u8;
    /// Write a byte given the address in CPU memory space
    fn write_cpu(&mut self, cpu_addr: usize, mem: &mut CartridgeMemory, value: u8);
    /// Reach a byte given the address in PPU memory space
    /// Used by the emulator and may change the cartridge's state, for debug reading use read_ppu_debug
    fn read_ppu(&mut self, ppu_addr: usize, mem: &CartridgeMemory) -> u8 {
        self.read_ppu_debug(ppu_addr, mem)
    }
    /// Write a byte given the address in PPU memory space
    fn write_ppu(&mut self, ppu_addr: usize, mem: &mut CartridgeMemory, value: u8);
    /// Read a byte in PPU memory space, transforming the address as it usually would, but not changing the cartridge's
    /// state at all.
    /// Used for debug purposes.
    fn read_ppu_debug(&self, ppu_addr: usize, mem: &CartridgeMemory) -> u8;
    /// Get the nametable arrangement the cartridge is currently using
    fn nametable_arrangement(&self, mem: &CartridgeMemory) -> NametableArrangement {
        mem.nametable_arrangement
    }
    /// Advance the cartridge's state a certain number of CPU cycles
    fn advance_cpu_cycles(&mut self, _cycles: u32) {}
    /// Set the value on the PPUADDR pins going into the cartridge.
    /// Must be updated since some cartridges use this value to clock an interrupt timer.
    fn set_addr_value(&mut self, _addr: u32) {}
    /// Return `true` if the cartridge is triggering an IRQ, and false otherwise
    fn irq(&mut self) -> bool {
        false
    }
    /// Get the iNes mapper number of this mapper
    fn mapper_num(&self) -> u32;
}
/// Get an implementation of `Mapper` given a certain mapper number
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
