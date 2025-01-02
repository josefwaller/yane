use crate::{
    emulation::cartridge::mapper::bank_addr, CartridgeMemory, Mapper, NametableArrangement,
};
use log::*;

pub struct SxRom {
    shift: usize,
    chr_bank_0: usize,
    chr_bank_1: usize,
    prg_bank: usize,
    control: usize,
    // Whether something has been written this CPU cycle, and thus further writes should be blocked
    has_written: bool,
}

impl Default for SxRom {
    fn default() -> SxRom {
        SxRom {
            shift: 0x10,
            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0,
            control: 0,
            has_written: false,
        }
    }
}

impl Mapper for SxRom {
    fn read_cpu(&self, cpu_addr: usize, mem: &CartridgeMemory) -> u8 {
        if cpu_addr < 0x8000 {
            if cpu_addr < 0x6000 {
                warn!("Reading to {:X}", cpu_addr);
                return 0;
            }
            mem.prg_ram[(cpu_addr - 0x6000) % mem.prg_ram.len()]
        } else {
            let mode = (self.control & 0x0C) >> 2;
            let addr = match mode {
                0 | 1 => {
                    let bank_num = (self.prg_bank & 0x0E) as usize >> 1;
                    // Switch 32 KiB mode
                    bank_addr(0x8000, bank_num, cpu_addr)
                }
                2 => {
                    let bank_num = self.prg_bank & 0x0F;
                    if cpu_addr < 0xC000 {
                        // First 16 KiB bank only
                        // cpu_addr % 0x4000
                        bank_addr(0x4000, 0, cpu_addr)
                    } else {
                        // Switchable 16 KiB bank
                        bank_addr(0x4000, bank_num, cpu_addr)
                    }
                }
                3 => {
                    let bank_num = self.prg_bank & 0x0F;
                    if cpu_addr < 0xC000 {
                        // Switch 16 KiB bank
                        bank_addr(0x4000, bank_num, cpu_addr)
                    } else {
                        // Last 16 KiB bank
                        let last_bank_num = (mem.prg_rom.len() - 1) / 0x4000;
                        bank_addr(0x4000, last_bank_num, cpu_addr)
                    }
                }
                _ => panic!("Should never happen"),
            };
            mem.prg_rom[addr % mem.prg_rom.len()]
        }
    }
    fn read_ppu(&self, ppu_addr: usize, mem: &CartridgeMemory) -> u8 {
        let mode = (self.control & 0x10) >> 4;
        let addr = if mode == 0 {
            bank_addr(0x2000, (self.chr_bank_0 & 0x1E) >> 1, ppu_addr)
        } else {
            if ppu_addr < 0x1000 {
                bank_addr(0x1000, self.chr_bank_0, ppu_addr)
            } else {
                bank_addr(0x1000, self.chr_bank_1, ppu_addr)
            }
        };
        mem.read_chr(addr)
    }
    fn write_cpu(&mut self, cpu_addr: usize, mem: &mut CartridgeMemory, value: u8) {
        if !self.has_written {
            self.has_written = true;
            if cpu_addr < 0x8000 {
                if cpu_addr < 0x6000 {
                    warn!("Writing to {:X}", cpu_addr);
                } else {
                    let max = mem.prg_ram.len();
                    mem.prg_ram[(cpu_addr - 0x6000) % max] = value;
                }
            } else {
                // If value high bit is not set
                if value & 0x80 == 0 {
                    // Check if shift register is full
                    let new_shift = (self.shift >> 1) | ((value as usize & 0x01) << 4);
                    if (self.shift & 0x01) != 0 {
                        // Set register based on address
                        if cpu_addr < 0xA000 {
                            // Set control
                            self.control = new_shift;
                        } else if cpu_addr < 0xC000 {
                            // Set first character bank
                            self.chr_bank_0 = new_shift;
                        } else if cpu_addr < 0xE000 {
                            // Set second character bank
                            self.chr_bank_1 = new_shift;
                        } else {
                            // Set program bank
                            self.prg_bank = new_shift;
                        }
                        self.shift = 0x10;
                    } else {
                        // Add bit to shift register
                        self.shift = new_shift;
                    }
                } else {
                    // Reset shift and set control
                    self.shift = 0x10;
                    self.control = self.control | 0x0C;
                }
            }
        }
    }
    // Todo: dedup
    fn write_ppu(&mut self, ppu_addr: usize, mem: &mut CartridgeMemory, value: u8) {
        let mode = (self.control & 0x10) >> 4;
        let addr = if mode == 0 {
            bank_addr(0x2000, (self.chr_bank_0 & 0x1E) >> 1, ppu_addr)
        } else {
            if ppu_addr < 0x1000 {
                bank_addr(0x1000, self.chr_bank_0, ppu_addr)
            } else {
                bank_addr(0x1000, self.chr_bank_1, ppu_addr)
            }
        };
        mem.write_chr(addr, value);
    }
    fn nametable_arrangement(&self) -> Option<NametableArrangement> {
        Some(match self.control & 0x03 {
            0 | 1 => NametableArrangement::OneScreen,
            2 => NametableArrangement::Horizontal,
            3 => NametableArrangement::Vertical,
            _ => panic!("Should never happen"),
        })
    }
    fn get_debug_string(&self) -> String {
        format!(
            "Mode: {:X}, Control: {:X}, shift: {:X}",
            (self.control & 0x0C) >> 2,
            self.control,
            self.shift
        )
    }
    fn advance_cpu_cycles(&mut self, _cycles: u32) {
        self.has_written = false;
    }
}
