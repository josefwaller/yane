use std::fmt::{Debug, Display};

use crate::{
    emulation::cartridge::mapper::{bank_addr, num_banks},
    Mapper, NametableArrangement,
};
use log::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
/// PxROM cartridge mapper and variants (mapper 9)
pub struct PxRom {
    prg_bank: usize,
    // The CHR banks, indexed first by address (0 = 0x0000-0x0FFF, 1 = 0x1000-0x1FFF)
    // and then by mode (0 = latch is FD, 1 = latch is FE)
    chr_banks: [[usize; 2]; 2],
    // latches
    latches: [usize; 2],
    nametable_arrangement: NametableArrangement,
}
impl Default for PxRom {
    fn default() -> Self {
        PxRom {
            prg_bank: 0,
            chr_banks: [[0; 2]; 2],
            latches: [0xFD; 2],
            nametable_arrangement: NametableArrangement::Horizontal,
        }
    }
}
#[typetag::serde]
impl Mapper for PxRom {
    fn mapper_num(&self) -> u32 {
        9
    }
    fn read_cpu(&self, cpu_addr: usize, mem: &crate::CartridgeMemory) -> u8 {
        if cpu_addr < 0x6000 {
            warn!("Invalid CPU addr {:X}", cpu_addr);
            0
        } else if cpu_addr < 0x8000 {
            // PRG RAM
            mem.prg_ram[cpu_addr % mem.prg_ram.len()]
        } else if cpu_addr < 0xA000 {
            // Switchable bank
            mem.prg_rom[bank_addr(0x2000, self.prg_bank, cpu_addr)]
        } else {
            let n = num_banks(0x2000, &mem.chr_rom);
            let bank_num = if cpu_addr < 0xC000 {
                // Fixed to third last bank
                n - 3
            } else if cpu_addr < 0xE000 {
                // Second last bank
                n - 2
            } else {
                // Last bank
                n - 1
            };
            mem.prg_rom[bank_addr(0x2000, bank_num, cpu_addr)]
        }
    }
    fn read_ppu_debug(&self, ppu_addr: usize, mem: &crate::CartridgeMemory) -> u8 {
        let bank_num = if ppu_addr < 0x1000 {
            if self.latches[0] == 0xFD {
                self.chr_banks[0][0]
            } else if self.latches[0] == 0xFE {
                self.chr_banks[0][1]
            } else {
                error!("Invalid latches value {:X?}", self.latches);
                self.chr_banks[0][0]
            }
        } else {
            if self.latches[1] == 0xFD {
                self.chr_banks[1][0]
            } else if self.latches[1] == 0xFE {
                self.chr_banks[1][1]
            } else {
                error!("Invalid latches value {:X?}", self.latches);
                self.chr_banks[1][0]
            }
        };
        mem.chr_rom[bank_addr(0x1000, bank_num, ppu_addr)]
    }
    fn read_ppu(&mut self, ppu_addr: usize, mem: &crate::CartridgeMemory) -> u8 {
        let v = self.read_ppu_debug(ppu_addr, mem);
        if ppu_addr == 0x0FD8 {
            self.latches[0] = 0xFD;
        } else if ppu_addr == 0x0FE8 {
            self.latches[0] = 0xFE;
        } else if (0x1FD8..=0x1FDF).contains(&ppu_addr) {
            self.latches[1] = 0xFD
        } else if (0x1FE8..=0x1FEF).contains(&ppu_addr) {
            self.latches[1] = 0xFE;
        }
        v
    }
    fn write_cpu(&mut self, cpu_addr: usize, mem: &mut crate::CartridgeMemory, value: u8) {
        let bank = (value & 0x1F) as usize;
        if cpu_addr < 0xA000 {
        } else if cpu_addr < 0xB000 {
            self.prg_bank = bank & 0x0F;
        } else if cpu_addr < 0xC000 {
            self.chr_banks[0][0] = bank;
        } else if cpu_addr < 0xD000 {
            self.chr_banks[0][1] = bank;
        } else if cpu_addr < 0xE000 {
            self.chr_banks[1][0] = bank;
        } else if cpu_addr < 0xF000 {
            self.chr_banks[1][1] = bank;
        } else {
            self.nametable_arrangement = if (value & 0x01) == 0 {
                NametableArrangement::Horizontal
            } else {
                NametableArrangement::Vertical
            };
        }
    }
    fn write_ppu(&mut self, ppu_addr: usize, mem: &mut crate::CartridgeMemory, value: u8) {}
    fn nametable_arrangement(&self, _: &crate::CartridgeMemory) -> NametableArrangement {
        self.nametable_arrangement
    }
}

impl Display for PxRom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PxROM")
    }
}
impl Debug for PxRom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PxROM, prg_bank={} chr_banks={:?} latches={:?} NT arrangement={:?}",
            self.prg_bank, self.chr_banks, self.latches, self.nametable_arrangement
        )
    }
}
