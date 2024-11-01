pub enum NametableArrangement {
    Horizontal,
    Vertical,
}

/// An NES cartridge, or perhaps more accurately, an iNES file.
/// Contains all the ROM and information encoded in the header.
pub struct Cartridge {
    prg_ram: Vec<u8>,
    prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    chr_ram: Vec<u8>,
    /// Nametable mirroring arrangement
    nametable_arrangement: NametableArrangement,
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
        // Todo
        let prg_ram_size = 8000;
        let chr_ram_size = 0x00;
        let mapper = (bytes[6] >> 4) + (bytes[7] & 0xF0);
        let nametable_arrangement = if (bytes[6] & 0x01) != 0 {
            NametableArrangement::Horizontal
        } else {
            NametableArrangement::Vertical
        };
        if mapper != 0 {
            panic!("Unsupported mapper {}", mapper);
        }
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
            prg_rom,
            chr_rom,
            prg_ram: vec![0; prg_ram_size],
            chr_ram: vec![0; chr_ram_size],
            nametable_arrangement,
        }
    }
    /// Read a byte from the cartridge given an address in the CPU's memory space
    pub fn read_byte(&self, addr: usize) -> u8 {
        if addr < 0x8000 {
            return self.prg_ram[(addr - 0x6000) % self.prg_ram.len()];
        }
        self.prg_rom[(addr - 0x8000) as usize % self.prg_rom.len()]
    }
    /// Write a byte from the cartridge given an address in the CPU's memory space
    pub fn write_byte(&mut self, addr: usize, value: u8) {
        self.prg_ram[addr - 0x6000] = value
    }
}
