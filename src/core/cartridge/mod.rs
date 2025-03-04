// mod cartridge;
// pub use cartridge::{Cartridge, CartridgeMemory, NametableArrangement};
mod mapper;
pub use mapper::Mapper;
pub mod mappers;

use crate::core::cartridge::mapper::get_mapper;
use log::*;
use serde::{Deserialize, Serialize};
use std::{
    cmp::max,
    fmt::{Debug, Display},
};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
/// The various nametable arrangements a cartridge can have.
///
/// Determines how the 2 screens of `VRAM` are mirrored to create the 4 screens of potential outputs.
/// Note that this is the nametable ARRANGEMENT - [NametableArrangement::Horizontal] means that the
/// nametables are using VERTICAL mirroring.
pub enum NametableArrangement {
    OneScreen,
    Horizontal,
    Vertical,
}

/// Contains all memory in the cartridge that isn't mapper-specific.
///
/// Contains PRG/CHR ROM/RAM.
/// Does not contain any latches, banks, or dividers used by mappers.
#[derive(Clone, Serialize, Deserialize)]
pub struct CartridgeMemory {
    /// Program RAM (PRG RAM) of the cartridge
    pub prg_ram: Vec<u8>,
    /// Program ROM (PRG ROM) of the cartridge
    pub prg_rom: Vec<u8>,
    /// Character RAM (CHR RAM) of the cartridge
    pub chr_ram: Vec<u8>,
    /// Character ROM (CHR ROM) of the cartridge
    pub chr_rom: Vec<u8>,
    /// Nametable arrangement of the cartridge, read when parsing the file.
    /// May be changed by the mapper, use [Mapper::nametable_arrangement] to get the current nametable arrangement being used
    pub nametable_arrangement: NametableArrangement,
}
impl CartridgeMemory {
    /// Read a byte from CHR ROM or (if CHR ROM is empty) CHR RAM.
    ///
    /// This is really a convience function, since most cartridges either are all CHR ROM or CHR RAM.
    /// So this could be interpreted as just "Read CHR from whatever format the cartridge using"
    pub fn read_chr(&self, addr: usize) -> u8 {
        if self.chr_rom.is_empty() {
            self.chr_ram[addr % self.chr_ram.len()]
        } else {
            self.chr_rom[addr % self.chr_rom.len()]
        }
    }
    /// Write a byte to CHR RAM, if present.
    pub fn write_chr(&mut self, addr: usize, value: u8) {
        if !self.chr_ram.is_empty() {
            let i = addr % self.chr_ram.len();
            self.chr_ram[i] = value;
        }
    }
}

/// An NES cartridge.
///
/// Contains the cartridge's RAM and ROM in [CartridgeMemory] and a [Mapper] responsible for mapping addresses to data.
#[derive(Serialize, Deserialize)]
pub struct Cartridge {
    /// The memory in the cartridge
    pub memory: CartridgeMemory,
    /// The mapper the cartridge is using
    pub mapper: Box<dyn Mapper>,
    // Whether the cartridge has battery backed RAM and should be saved
    has_battery_ram: bool,
}

impl Cartridge {
    /// Create a new cartridge from the contents of an iNes (.nes) file.
    ///
    /// * `bytes` The contents of the iNes file.
    /// * `savedata` The battery backed static RAM on the cartridge, used to initialise the PRG RAM if present.
    pub fn from_ines(bytes: &[u8], savedata: Option<Vec<u8>>) -> Cartridge {
        if cfg!(debug_assertions) {
            assert_eq!(bytes[0], b'N');
            assert_eq!(bytes[1], b'E');
            assert_eq!(bytes[2], b'S');
            assert_eq!(bytes[3], 0x1A);
        }
        let prg_rom_size = 0x4000 * bytes[4] as usize;
        let chr_rom_size = 0x2000 * bytes[5] as usize;
        let prg_ram_size = max(bytes[8] as usize * 0x2000, 0x2000);
        let mut chr_ram_size = if chr_rom_size == 0 { 0x2000 } else { 0x0 };
        debug!("Cartridge header: {:X?}", &bytes[0..16]);
        let has_battery_ram = (bytes[6] & 0x02) != 0;
        let has_trainer = (bytes[6] & 0x04) != 0;
        let alt_nametable_layout = (bytes[6] & 0x08) != 0;
        debug!(
            "Trainer: {}, alternate nametable: {}, battery backed ram: {}",
            has_trainer, alt_nametable_layout, has_battery_ram
        );
        // Detect type of iNes file
        let total_file_size = 16 + if has_trainer { 512 } else { 0 } + prg_rom_size + chr_rom_size;
        debug!(
            "Total data size: {:X} bytes. File size: {:X}",
            total_file_size,
            bytes.len()
        );
        let file_type = if bytes[7] & 0x0C == 0x08 && bytes.len() >= total_file_size {
            debug!("iNES 2.0 detected");
            0
        } else if bytes[7] & 0x0C == 0x04 {
            debug!("Archaic iNES detected");
            chr_ram_size = 0;
            1
        } else if bytes[7] & 0x0C == 0x00 {
            debug!("iNES detected");
            2
        } else {
            debug!("Archaic iNES probably detected");
            1
        };
        debug!(
            "Detected as {}, ignoring.",
            if bytes[9] & 0x01 != 0 { "PAL" } else { "NTSC" }
        );
        debug!(
            "{:X} bytes PRG ROM, {:X} bytes CHR ROM, {:X} bytes PRG RAM, {:X} bytes CHR RAM",
            prg_rom_size, chr_rom_size, prg_ram_size, chr_ram_size
        );
        // Todo
        let mapper_id = (bytes[6] >> 4) + if file_type != 1 { bytes[7] & 0xF0 } else { 0 };
        let nametable_arrangement = if (bytes[6] & 0x01) == 0 {
            NametableArrangement::Vertical
        } else {
            NametableArrangement::Horizontal
        };
        debug!(
            "Cartridge is using a {:?} nametable arrangment",
            nametable_arrangement
        );
        debug!(
            "Cartridge {} using an alternative nametable arrangement",
            if (bytes[6] & 0x08) == 0 {
                "isn't"
            } else {
                "is"
            }
        );
        debug!(
            "Cartridge is using {} mapper (0x{:X})",
            mapper_id, mapper_id
        );
        let mapper = get_mapper(mapper_id as usize);
        let mut start = 16 + if has_trainer { 512 } else { 0 };
        let mut end = start + prg_rom_size;
        let prg_rom = bytes[start..end].to_vec();
        start = end;
        end += chr_rom_size;
        debug!("Reading CHR ROM at {:#X}", start);
        let chr_rom = bytes[start..end].to_vec();
        // Load PRG RAM from savedata if we have some
        let prg_ram = match savedata {
            Some(data) => {
                assert_eq!(data.len(), prg_ram_size);
                data
            }
            None => vec![0; prg_ram_size],
        };
        Cartridge {
            memory: CartridgeMemory {
                prg_rom,
                chr_rom,
                prg_ram,
                chr_ram: vec![0; chr_ram_size],
                nametable_arrangement,
            },
            mapper,
            has_battery_ram,
        }
    }
    /// Read a byte from the cartridge's memory given an address in CPU memory space
    pub fn read_cpu(&self, addr: usize) -> u8 {
        self.mapper.read_cpu(addr, &self.memory)
    }
    /// Write a byte in the cartridge's memory given an address in CPU memory space
    pub fn write_cpu(&mut self, addr: usize, value: u8) {
        self.mapper.write_cpu(addr, &mut self.memory, value);
    }
    /// Read a byte in the cartridge's memory given an address in PPU memory space
    pub fn read_ppu(&mut self, addr: usize) -> u8 {
        self.mapper.read_ppu(addr, &self.memory)
    }
    /// Write a byte of data to CHR ROM/RAM in PPU memory space.
    pub fn write_ppu(&mut self, addr: usize, value: u8) {
        self.mapper.write_ppu(addr, &mut self.memory, value);
    }
    /// Get all of the CHR data as bytes.
    ///
    /// Only used for debug purposes, the PPU should use [Cartridge::read_ppu] to allow the mapper to transform the address.
    pub fn get_pattern_table(&self) -> &[u8] {
        if self.memory.chr_ram.is_empty() {
            return self.memory.chr_rom.as_slice();
        }
        &self.memory.chr_ram
    }
    /// Transform a given nametable address to a valid address in the PPU's VRAM.
    ///
    /// The NES needs to show four full screens of nametable data (top left, top right, bottom left, and bottom right),
    /// but only has enough memory to store 2 full screens of nametable data.
    /// So two of the screens are mirrored by transforming the addresses when reading nametable data.
    /// See [the NESDEV wiki](https://www.nesdev.org/wiki/PPU_nametables).
    pub fn transform_nametable_addr(&self, addr: usize) -> usize {
        let nametable = self.mapper.nametable_arrangement(&self.memory);
        match nametable {
            NametableArrangement::OneScreen => addr % 0x400,
            NametableArrangement::Horizontal => {
                // 0x2000 = 0x2800, 0x2400 = 0x2C00
                (addr - 0x2000) % 0x800
            }
            NametableArrangement::Vertical => {
                // 0x2000 = 0x2400, 0x2800 = 0x2C00
                if addr < 0x2800 {
                    addr % 0x400
                } else {
                    (addr % 0x400) + 0x400
                }
            }
        }
    }
    /// [true] if the cartridge has battery backed RAM (i.e. save data), [false] otherwise
    pub fn has_battery_backed_ram(&self) -> bool {
        self.has_battery_ram
    }
    /// Get the nametable arrangement the cartridge is currently using
    pub fn nametable_arrangement(&self) -> NametableArrangement {
        self.mapper.nametable_arrangement(&self.memory)
    }
    /// Advance the cartridge by a certain number of CPU cycles
    pub fn advance_cpu_cycles(&mut self, cycles: u32) {
        self.mapper.advance_cpu_cycles(cycles);
    }
}

impl Display for Cartridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.mapper, f)
    }
}
impl Debug for Cartridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.mapper, f)
    }
}
