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
    // IRQ stuff
    irq_enable: bool,
    irq_reload: bool,
    irq_counter: u32,
    irq_latch: u32,
    last_ppu_addr: u32,
    generate_irq: bool,
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
            irq_enable: false,
            irq_reload: false,
            irq_counter: 0,
            irq_latch: 0,
            generate_irq: false,
            last_ppu_addr: 0,
            bank_select: 0,
            nametable: NametableArrangement::Horizontal,
        }
    }
}

impl Mapper for TxRom {
    fn read_cpu(&self, cpu_addr: usize, mem: &crate::CartridgeMemory) -> u8 {
        if cpu_addr < 0x6000 {
            warn!("Trying to read PRG RAM where there is none: {:X}", cpu_addr);
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
            if cpu_addr % 2 == 0 {
                self.irq_latch = value as u32;
            } else {
                self.irq_reload = true;
            }
        } else {
            if cpu_addr % 2 == 0 {
                if self.irq_enable {}
                self.irq_enable = false;
                self.generate_irq = false;
            } else {
                if !self.irq_enable {}
                self.irq_enable = true;
            }
        }
    }
    fn read_ppu(&mut self, ppu_addr: usize, mem: &crate::CartridgeMemory) -> u8 {
        // Refresh controller ADDR pin values
        self.set_addr_value(ppu_addr as u32);
        self.read_ppu_debug(ppu_addr, mem)
    }
    fn read_ppu_debug(&self, ppu_addr: usize, mem: &crate::CartridgeMemory) -> u8 {
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
    fn write_ppu(&mut self, _ppu_addr: usize, _mem: &mut crate::CartridgeMemory, _value: u8) {}
    fn set_addr_value(&mut self, ppu_addr: u32) {
        // Update IRQ
        if self.last_ppu_addr == 0 && ppu_addr & 0x1000 != 0 {
            // Check for reload or decrement
            if self.irq_counter == 0 || self.irq_reload {
                self.irq_counter = self.irq_latch;
                self.irq_reload = false;
            } else {
                self.irq_counter -= 1;
            }
            // Check for interrupt
            if self.irq_counter == 0 {
                if self.irq_enable {
                    self.generate_irq = true;
                }
            }
        }
        self.last_ppu_addr = ppu_addr & 0x1000;
    }
    fn nametable_arrangement(&self) -> Option<NametableArrangement> {
        Some(self.nametable)
    }
    fn get_debug_string(&self) -> String {
        format!(
            "Counter enabled = {:}, reload = {:}",
            self.irq_enable, self.irq_reload
        )
    }
    fn irq_addr(&mut self) -> Option<usize> {
        if self.generate_irq {
            self.generate_irq = false;
            Some(0xFFFE)
        } else {
            None
        }
    }
}
