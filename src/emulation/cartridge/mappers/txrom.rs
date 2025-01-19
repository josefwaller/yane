use crate::{
    emulation::cartridge::mapper::{bank_addr, num_banks},
    Mapper, NametableArrangement,
};
use log::*;

pub struct TxRom {
    prg_banks: [u32; 2],
    chr_banks: [u32; 6],
    prg_mode: u32,
    chr_mode: u32,
    // Bank select, i.e. which bank we are setting
    // 0-5: Editing CHR bank
    // 6-7: Editing PRG bank seelct
    bank_select: u32,
    nametable: NametableArrangement,
}

impl Default for TxRom {
    fn default() -> Self {
        TxRom {
            prg_banks: [0; 2],
            chr_banks: [0; 6],
            prg_mode: 0,
            chr_mode: 0,
            bank_select: 0,
            nametable: NametableArrangement::Horizontal,
        }
    }
}

impl Mapper for TxRom {
    fn read_cpu(&self, cpu_addr: usize, mem: &crate::CartridgeMemory) -> u8 {
        if cpu_addr < 0x6000 {
            warn!("Invalid address {:X}", cpu_addr);
            0
        } else if cpu_addr < 0x8000 {
            if mem.prg_ram.len() == 0 {
                warn!("Trying to read PRG RAM but there is none on this cartridge");
                0
            } else {
                mem.prg_ram[(cpu_addr - 0x6000) % mem.prg_ram.len()]
            }
        } else if cpu_addr < 0xA000 {
            if self.prg_mode == 0 {
                // Switchable 8Kb bank
                mem.prg_rom[bank_addr(0x2000, self.prg_banks[0] as usize, cpu_addr)]
            } else {
                // Fixed to second last bank
                mem.prg_rom[bank_addr(
                    0x2000,
                    num_banks(0x2000, mem.prg_rom.as_slice()) - 2,
                    cpu_addr,
                )]
            }
        } else if cpu_addr < 0xC000 {
            // Switchable 8Kb bank
            mem.prg_rom[bank_addr(0x2000, self.prg_banks[1] as usize, cpu_addr)]
        } else if cpu_addr < 0xE000 {
            if self.prg_mode == 1 {
                // Switchable 8Kb bank
                mem.prg_rom[bank_addr(0x2000, self.prg_banks[0] as usize, cpu_addr)]
            } else {
                // Fixed to second last bank
                mem.prg_rom[bank_addr(
                    0x2000,
                    num_banks(0x2000, mem.prg_rom.as_slice()) - 2,
                    cpu_addr,
                )]
            }
        } else {
            mem.prg_rom[bank_addr(0x2000, num_banks(0x2000, &mem.prg_rom) - 1, cpu_addr)]
        }
    }
    fn write_cpu(&mut self, cpu_addr: usize, mem: &mut crate::CartridgeMemory, value: u8) {
        if cpu_addr < 0x8000 {
            mem.prg_ram[cpu_addr - 0x6000] = value;
        } else if cpu_addr < 0xA000 {
            if cpu_addr % 2 == 0 {
                // Choose bank select
                self.bank_select = (value & 0x07) as u32;

                // Set PRG bank mode
                self.prg_mode = ((value & 0x40) >> 6) as u32;
                // Set CHR bank mode (CHR inversion)
                self.chr_mode = ((value & 0x80) >> 7) as u32;
            } else {
                // Bank data
                if self.bank_select < 6 {
                    self.chr_banks[self.bank_select as usize] = value as u32;
                } else if self.bank_select < 8 {
                    self.prg_banks[self.bank_select as usize - 6] = value as u32
                } else {
                    error!("Invalid bank select value: {:X}", self.bank_select);
                }
            }
        } else if cpu_addr < 0xC000 {
            if cpu_addr % 2 == 0 {
                // Set nametable mirroring
                self.nametable = if value & 0x01 == 0 {
                    NametableArrangement::Horizontal
                } else {
                    NametableArrangement::Vertical
                }
            } else {
                // PRG RAM protection (todo)
            }
        } else if cpu_addr < 0xE000 {
        } else {
            if cpu_addr % 2 == 0 {
                debug!("IRQ disable");
            } else {
                debug!("IRQ enable");
            }
        }
    }
    fn read_ppu(&self, ppu_addr: usize, mem: &crate::CartridgeMemory) -> u8 {
        let (bank_size, bank_num) = if self.chr_mode == 0 {
            if ppu_addr < 0x1000 {
                (0x800, self.chr_banks[ppu_addr / 0x800] / 2)
            } else {
                (0x400, self.chr_banks[(ppu_addr - 0x1000) / 0x400 + 2])
            }
        } else {
            if ppu_addr < 0x1000 {
                (0x400, self.chr_banks[ppu_addr / 0x400 + 2])
            } else {
                (0x800, self.chr_banks[(ppu_addr - 0x1000) / 0x800] / 2)
            }
        };
        mem.read_chr(bank_addr(bank_size, bank_num as usize, ppu_addr))
    }
    fn write_ppu(&mut self, ppu_addr: usize, mem: &mut crate::CartridgeMemory, value: u8) {}
    fn nametable_arrangement(&self) -> Option<NametableArrangement> {
        Some(self.nametable)
    }
}
