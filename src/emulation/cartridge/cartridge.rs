use crate::{emulation::cartridge::mapper::get_mapper, Mapper};
use log::*;
use std::cmp::max;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum NametableArrangement {
    OneScreen,
    Horizontal,
    Vertical,
}

/// Holds all the memory in the cartridge
// Todo: Maybe rename (get rid of cartridge)
pub struct CartridgeMemory {
    pub prg_ram: Vec<u8>,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub chr_ram: Vec<u8>,
}
impl CartridgeMemory {
    // Read from CHR ROM or CHR RAM, if CHR ROM is empty
    // Used for cartridges that don't use both CHR ROM and CHR RAM
    pub fn read_chr(&self, addr: usize) -> u8 {
        if self.chr_rom.len() == 0 {
            self.chr_ram[addr % self.chr_ram.len()]
        } else {
            self.chr_rom[addr % self.chr_rom.len()]
        }
    }
}

/// An NES cartridge, or perhaps more accurately, an iNES file.
/// Contains all the ROM and information encoded in the header.
pub struct Cartridge {
    pub memory: CartridgeMemory,
    /// Nametable mirroring arrangement
    nametable_arrangement: NametableArrangement,
    // Mapper
    mapper: Box<dyn Mapper>,
}

impl Cartridge {
    pub fn new(bytes: &[u8]) -> Cartridge {
        if cfg!(debug_assertions) {
            assert_eq!(bytes[0], 'N' as u8);
            assert_eq!(bytes[1], 'E' as u8);
            assert_eq!(bytes[2], 'S' as u8);
            assert_eq!(bytes[3], 0x1A);
        }
        let prg_rom_size = 0x4000 * bytes[4] as usize;
        let chr_rom_size = 0x2000 * bytes[5] as usize;
        let mut prg_ram_size = max(bytes[8] as usize, 1) * 8000;
        let chr_ram_size = if chr_rom_size == 0 { 0x2000 } else { 0x0 };
        if bytes[7] & 0x0C == 0x08 {
            warn!("iNES 2.0 file detected");
        } else {
            info!("iNES file detected");
            info!("Header: {:X?}", &bytes[0..16]);
            prg_ram_size = max(bytes[8] as usize * 8000, 8000);
            info!(
                "Detected as {}, ignoring.",
                if bytes[9] & 0x01 != 0 { "PAL" } else { "NTSC" }
            );
        }
        info!(
            "{:X} bytes PRG ROM, {:X} bytes CHR ROM, {:X} bytes PRG RAM, {:X} bytes CHR RAM",
            prg_rom_size, chr_rom_size, prg_ram_size, chr_ram_size
        );
        // Todo
        let mapper_id = (bytes[6] >> 4) + (bytes[7] & 0xF0);
        let nametable_arrangement = if (bytes[6] & 0x01) != 0 {
            NametableArrangement::Horizontal
        } else {
            NametableArrangement::Vertical
        };
        info!("Cartridge is using {:?} nametable", nametable_arrangement);
        info!("Cartridge is using {} mapper", mapper_id);
        let mapper = get_mapper(mapper_id as usize);
        let has_trainer = bytes[6] & 0x04 != 0;
        info!(
            "Cartridge {} trainer",
            if has_trainer { "has" } else { "does not have" },
        );
        // TODO: Add CHR_RAM
        let mut start = 16 + if has_trainer { 512 } else { 0 };
        let mut end = start + prg_rom_size;
        let prg_rom = bytes[start..end].to_vec();
        start = end;
        end += chr_rom_size;
        info!("Reading CHRROM at {:#X}", start);
        let chr_rom = bytes[start..end].to_vec();
        Cartridge {
            memory: CartridgeMemory {
                prg_rom,
                chr_rom,
                prg_ram: vec![0; prg_ram_size],
                chr_ram: vec![0; chr_ram_size],
            },
            nametable_arrangement,
            mapper,
        }
    }
    /// Read a byte from the cartridge's memory given an address in CPU memory space
    /// Usually reads from PRG ROM/RAM.
    pub fn read_cpu(&self, addr: usize) -> u8 {
        self.mapper.read_cpu(addr, &self.memory)
    }
    /// Write a byte in the cartridge's memory given an address in CPU memory space
    /// Usually reads from PRG RAM.
    pub fn write_cpu(&mut self, addr: usize, value: u8) {
        self.mapper.write_cpu(addr, &mut self.memory, value);
    }
    /// Read a byte in the cartridge's memory given an address in PPU memory space
    /// Usually reads from CHR ROM/RAM.
    pub fn read_ppu(&self, addr: usize) -> u8 {
        self.mapper.read_ppu(addr, &self.memory)
    }
    /// Write a byte to the cartridge's memory given an address in PPU memory space
    /// Usually writes tot CHR RAM.
    pub fn write_chr(&mut self, addr: usize, value: u8) {
        // TBA - only do this if there is CHR RAM
        if addr < self.memory.chr_ram.len() {
            self.memory.chr_ram[addr] = value;
        }
    }
    pub fn get_pattern_table(&self) -> &[u8] {
        if self.memory.chr_ram.len() == 0 {
            return self.memory.chr_rom.as_slice();
        }
        return &self.memory.chr_ram;
    }
    pub fn transform_nametable_addr(&self, addr: usize) -> usize {
        let nametable = self
            .mapper
            .nametable_arrangement()
            .unwrap_or(self.nametable_arrangement);
        match nametable {
            NametableArrangement::OneScreen => addr % 0x400,
            NametableArrangement::Horizontal => {
                // 0x2000 = 0x2800, 0x2400 = 0x2C00
                (addr - 0x2000) % 0x800
            }
            NametableArrangement::Vertical => {
                // 0x2000 = 0x2400, 0x2800 = 0x2C00
                if addr < 0x2400 {
                    addr % 0x400
                } else if addr < 0x2800 {
                    addr % 0x400
                } else if addr < 0x2C00 {
                    (addr % 0x400) + 0x400
                } else {
                    (addr % 0x400) + 0x400
                }
            }
        }
    }
    // This always return 8x16 sprites
    pub fn get_tile(&self, tile_num: usize) -> &[u8] {
        &self.get_pattern_table()[(16 * tile_num)..(16 * (tile_num + 1))]
    }
}
