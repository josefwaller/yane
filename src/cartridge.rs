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
    chr_rom: Vec<u8>,
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
        let prg_rom_size = 0x10 * 0x400 * bytes[4] as usize;
        let chr_rom_size = 0x2000 * bytes[5] as usize;
        let prg_ram_size = 0x00;
        let chr_ram_size = 0x00;
        let mapper = (bytes[6] >> 4) + (bytes[7] & 0xF0);
        println!(
            "Reading an NES file with {:#X} KiB PRG ROM, {:#X} KiB CHG ROM, mapper {:}",
            prg_rom_size, chr_rom_size, mapper
        );
        // TODO: Check for trainer and offset by 512 bytes if present
        let mut start = 16;
        let mut end = 16 + prg_rom_size;
        let prg_rom = bytes[start..end].to_vec();
        start = end;
        end += chr_ram_size;
        let chr_rom = bytes[start..end].to_vec();
        Cartridge {
            prg_rom,
            chr_rom,
            prg_ram: vec![0; prg_ram_size],
            chr_ram: vec![0; chr_ram_size],
        }
    }
    pub fn read_byte(&self, addr: usize) -> u8 {
        if addr < 0x8000 {
            return self.prg_ram[(addr - 0x6000) % self.prg_ram.len()];
        }
        self.prg_rom[(addr - 0x8000) as usize % self.prg_rom.len()]
    }
    pub fn write_byte(&mut self, addr: usize, value: u8) {
        self.prg_ram[addr - 0x6000] = value
    }
}
