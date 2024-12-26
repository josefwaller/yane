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
#[derive(Clone)]
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
    // Whether the cartridge has battery backed RAM and should be saved
    has_battery_ram: bool,
}

impl Cartridge {
    pub fn new(bytes: &[u8], savedata: Option<Vec<u8>>) -> Cartridge {
        if cfg!(debug_assertions) {
            assert_eq!(bytes[0], 'N' as u8);
            assert_eq!(bytes[1], 'E' as u8);
            assert_eq!(bytes[2], 'S' as u8);
            assert_eq!(bytes[3], 0x1A);
        }
        let prg_rom_size = 0x4000 * bytes[4] as usize;
        let chr_rom_size = 0x2000 * bytes[5] as usize;
        let prg_ram_size = max(bytes[8] as usize * 0x2000, 0x2000);
        let mut chr_ram_size = if chr_rom_size == 0 { 0x2000 } else { 0x0 };
        info!("Header: {:X?}", &bytes[0..16]);
        let has_battery_ram = (bytes[6] & 0x02) != 0;
        let has_trainer = (bytes[6] & 0x04) != 0;
        let alt_nametable_layout = (bytes[6] & 0x08) != 0;
        info!(
            "Trainer: {}, alternate nametable: {}, battery backed ram: {}",
            has_trainer, alt_nametable_layout, has_battery_ram
        );
        // Detect type of iNes file
        let total_file_size = 16 + if has_trainer { 512 } else { 0 } + prg_rom_size + chr_rom_size;
        info!(
            "Total data size: {:X} bytes. File size: {:X}",
            total_file_size,
            bytes.len()
        );
        let file_type = if bytes[7] & 0x0C == 0x08 && bytes.len() >= total_file_size {
            info!("iNES 2.0 detected");
            0
        } else if bytes[7] & 0x0C == 0x04 {
            info!("Archaic iNES detected");
            chr_ram_size = 0;
            1
        } else if bytes[7] & 0x0C == 0x00 {
            info!("iNES detected");
            2
        } else {
            info!("Archaic iNES probably detected");
            1
        };
        info!(
            "Detected as {}, ignoring.",
            if bytes[9] & 0x01 != 0 { "PAL" } else { "NTSC" }
        );
        info!(
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
        info!(
            "Cartridge is using a {:?} nametable arrangment",
            nametable_arrangement
        );
        info!(
            "Cartridge is using {} mapper (0x{:X})",
            mapper_id, mapper_id
        );
        let mapper = get_mapper(mapper_id as usize);
        let mut start = 16 + if has_trainer { 512 } else { 0 };
        let mut end = start + prg_rom_size;
        let prg_rom = bytes[start..end].to_vec();
        start = end;
        end += chr_rom_size;
        info!("Reading CHRROM at {:#X}", start);
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
            },
            nametable_arrangement,
            mapper,
            has_battery_ram,
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
    /// Todo: Use mapper here?
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
    /// Transform nametable address to index in VRAM array in PPU
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
    pub fn debug_string(&self) -> String {
        self.mapper.get_debug_string()
    }
    pub fn has_battery_backed_ram(&self) -> bool {
        self.has_battery_ram
    }
    pub fn nametable_arrangement(&self) -> NametableArrangement {
        self.mapper
            .nametable_arrangement()
            .unwrap_or(self.nametable_arrangement)
    }
    pub fn advance_cpu_cycles(&mut self, cycles: u32) {
        self.mapper.advance_cpu_cycles(cycles);
    }
}
