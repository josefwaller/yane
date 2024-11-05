use crate::{emulation::cartridge::mapper::get_mapper, Mapper};
use std::cmp::{max, min};

pub enum NametableArrangement {
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
            println!("iNES 2.0 file detected");
        } else {
            println!("iNES file detected");
            println!("Header: {:X?}", &bytes[0..16]);
            prg_ram_size = min(bytes[8] as usize * 8000, 8000);
            println!(
                "Detected as {}, ignoring.",
                if bytes[9] & 0x01 != 0 { "PAL" } else { "NTSC" }
            );
        }
        println!(
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

        let mapper = get_mapper(mapper_id as usize);
        // TODO: Check for trainer and offset by 512 bytes if present
        // TODO: Add CHR_RAM
        let mut start = 16;
        let mut end = 16 + prg_rom_size;
        let prg_rom = bytes[start..end].to_vec();
        start = end;
        end += chr_rom_size;
        println!("Reading CHRROM at {:#X}", start);
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
    pub fn read_byte(&self, addr: usize) -> u8 {
        self.mapper.read_cpu(addr, &self.memory)
    }
    pub fn write_byte(&mut self, addr: usize, value: u8) {
        self.mapper.write_cpu(addr, &mut self.memory, value);
    }
    pub fn write_chr(&mut self, addr: usize, value: u8) {
        // TBA - only do this if there is CHR RAM
        self.memory.chr_ram[addr] = value;
    }
    pub fn get_pattern_table(&self) -> &[u8] {
        if self.memory.chr_ram.len() == 0 {
            return self.memory.chr_rom.as_slice();
        }
        return &self.memory.chr_ram;
    }
}
