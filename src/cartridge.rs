// pub struct Cartridge {
//     read_fn: fn(u16) -> u8,
//     write_fn: fn(u16),
// }

// impl Cartridge {
//     pub fn new(bytes: &[u8]) -> Cartridge {
//         Cartridge { read_fn: read_0 }
//     }
// }

// fn read_0(addr: u16, mem: &[u8]) -> u8 {
//     return mem[addr as usize];
// }

pub struct Cartridge {
    prg_ram: Vec<u8>,
    prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    chr_ram: Vec<u8>,
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
        let prg_ram_size = 0x00;
        let chr_ram_size = 0x00;
        let mapper = (bytes[6] >> 4) + (bytes[7] & 0xF0);
        assert_eq!(mapper, 0);
        // TODO: Check for trainer and offset by 512 bytes if present
        // TODO: Add CHR_RAM
        let mut start = 16;
        let mut end = 16 + prg_rom_size;
        let prg_rom = bytes[start..end].to_vec();
        start = end; //0x44b1; //0x4376; //end;
        end += chr_rom_size;
        println!("Reading CHRROM at {:#X}", start);
        let chr_rom = bytes[start..end].to_vec();
        Cartridge {
            prg_rom,
            chr_rom,
            prg_ram: vec![0; prg_ram_size],
            chr_ram: vec![0; chr_ram_size],
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
