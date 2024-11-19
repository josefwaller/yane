use super::{cartridge, Cartridge, NametableArrangement};
use log::*;

pub struct Ppu {
    /// The Object Access Memory, or OAM
    pub oam: [u8; 0x100],
    /// The PPUCTRL register
    pub ctrl: u8,
    /// The PPUMASK register
    pub mask: u8,
    /// The PPUSTATUS register
    pub status: u8,
    /// The OAMADDR register
    pub oam_addr: u8,
    /// The OAMDATA register
    pub oam_data: u8,
    /// The PPUSCROLL register, split into its X/Y components
    pub scroll_x: u8,
    pub scroll_y: u8,
    /// The PPUADDR register
    pub addr: u16,
    /// The PPUDATA register (actually the read buffer)
    pub data: u8,
    /// The OAMDMA register
    pub oam_dma: u8,
    /// VRAM
    pub palette_ram: [u8; 0x20],
    pub nametable_ram: [u8; 0x800],
    // W register
    w: bool,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            oam: [0; 0x100],
            ctrl: 0x00,
            mask: 0,
            status: 0xA0,
            oam_addr: 0,
            oam_data: 0,
            scroll_x: 0,
            scroll_y: 0,
            addr: 0,
            data: 0,
            oam_dma: 0,
            palette_ram: [0; 0x20],
            nametable_ram: [0; 0x800],
            w: true,
        }
    }

    /// Read a byte from the PPU register given an address in CPU space
    pub fn read_byte(&mut self, addr: usize, cartridge: &Cartridge) -> u8 {
        match addr % 8 {
            // Zero out some bits in control
            0 => self.ctrl & 0xBF,
            1 => self.mask,
            2 => {
                // VBLANK is cleared on read
                let status = self.status;
                self.status &= 0x7F;
                status
            }
            3 => self.oam_addr,
            4 => self.oam_data,
            // SCROLL and ADDR shouldn't be read from
            5 => self.scroll_x,
            6 => self.addr as u8,
            7 => {
                let t = self.data;
                self.data = self.read_vram(cartridge);
                t
            }
            _ => panic!("This should never happen. Addr is {:#X}", addr),
        }
    }

    /// Write a byte to the PPU registers given an address in CPU space
    pub fn write_byte(&mut self, addr: usize, value: u8, cartridge: &mut Cartridge) {
        match addr % 8 {
            0 => self.ctrl = value,
            1 => self.mask = value,
            2 => self.status = value,
            3 => self.oam_addr = value,
            4 => self.oam_data = value,
            5 => {
                if self.w {
                    self.scroll_x = value
                } else {
                    self.scroll_y = value;
                }
                self.w = !self.w;
            }
            6 => self.write_to_addr(value),
            7 => {
                self.write_vram(value, cartridge);
                // self.data = value;
            }
            _ => panic!("This should never happen. Addr is {:#X}", addr),
        }
    }

    pub fn write_to_addr(&mut self, value: u8) {
        if self.w {
            self.addr = (self.addr & 0x00FF) + ((value as u16) << 8);
        } else {
            self.addr = (self.addr & 0x3F00) + value as u16;
        }
        self.w = !self.w;
    }

    /// Write a single byte to VRAM at `PPUADDR`
    /// Increments `PPUADDR` by 1 or by 32 depending `PPUSTATUS`
    fn write_vram(&mut self, value: u8, cartridge: &mut Cartridge) {
        if self.addr < 0x2000 {
            cartridge.write_chr(self.addr as usize, value);
        } else if self.addr < 0x3000 {
            self.nametable_ram[cartridge.transform_nametable_addr(self.addr as usize)] = value;
        } else if self.addr >= 0x3F00 {
            self.palette_ram[(self.addr - 0x3F00) as usize % 0x020] = value;
        }
        self.inc_addr();
    }

    /// Read a single byte from VRAM
    fn read_vram(&mut self, cartridge: &Cartridge) -> u8 {
        let addr = self.addr;
        self.inc_addr();
        if self.addr < 0x2000 {
            return cartridge.read_ppu(addr as usize);
        }
        if self.addr < 0x3F00 {
            return self.nametable_ram[cartridge.transform_nametable_addr(self.addr as usize)];
        }
        self.palette_ram[(addr - 0x3F00) as usize % 0x020]
    }

    fn inc_addr(&mut self) {
        self.addr = self
            .addr
            .wrapping_add(if self.ctrl & 0x04 == 0 { 1 } else { 32 });
    }

    pub fn is_8x16_sprites(&self) -> bool {
        (self.ctrl & 0x20) != 0
    }
    /// Return true if sprite endering is enabled
    pub fn is_sprite_enabled(&self) -> bool {
        (self.mask & 0x10) != 0
    }
    /// Return true if background rendering is enabledf
    pub fn is_background_enabled(&self) -> bool {
        (self.mask & 0x08) != 0
    }
    /// Return whether to hide the left 8 pixels when drawing sprites
    pub fn should_hide_leftmost_sprites(&self) -> bool {
        (self.mask & 0x04) == 0
    }
    /// Return whether to hide the left 8 pixels when drwaing background
    pub fn should_hide_leftmost_background(&self) -> bool {
        (self.mask & 0x08) == 0
    }
    /// Return whether greyscale mode is on
    pub fn is_greyscale_mode_on(&self) -> bool {
        (self.mask & 0x01) != 0
    }
    /// Return where to read the sprite patterns from
    pub fn get_spr_pattern_table_addr(&self) -> usize {
        if self.ctrl & 0x08 != 0 {
            return 0x1000;
        }
        0x0000
    }
    /// Return where to read the backgronud patterns from
    pub fn get_background_pattern_table_addr(&self) -> usize {
        if self.ctrl & 0x10 != 0 {
            return 0x1000;
        }
        0x0000
    }
    // Return whether the red tint is active
    pub fn is_red_tint_on(&self) -> bool {
        (self.mask & 0x20) != 0
    }
    // Return whether the blue tint is active
    pub fn is_blue_tint_on(&self) -> bool {
        (self.mask & 0x40) != 0
    }
    // Return whether the green tint is active
    pub fn is_green_tint_on(&self) -> bool {
        (self.mask & 0x80) != 0
    }
    /// Return whether the NMI is enabled
    pub fn get_nmi_enabled(&self) -> bool {
        return self.ctrl & 0x80 != 0;
    }
    // TODO: maybe remove, just do this in nes
    pub fn on_vblank(&mut self) {
        self.status |= 0x80;
    }
    pub fn get_nametable_addr(&self) -> usize {
        return 0x2000 + (self.status & 0x03) as usize * 0x400;
    }
    pub fn get_base_nametable(&self) -> usize {
        0x400 * (self.ctrl as usize & 0x03)
    }
    /// Returns the base nametable number, a nubmer between 0 and 3.
    /// * 0 means that the base nametable is top left (0x2000)
    /// * 1 means that the base nametable is top right (0x2400)
    /// * 2 means that the base nametable is bot left (0x2800)
    /// * 3 means that the base nametable is bot right (0x2C00)
    ///
    /// The base nametable address can then be found by calculating `0x2000 + 0x400 * ppu.base_nametable_num()`
    pub fn base_nametable_num(&self) -> usize {
        (self.ctrl as usize) & 0x03
    }
    /// Get the address of the nametable at the top left of the current tilemap.
    pub fn top_left_nametable_addr(&self) -> usize {
        return 0x2000 + self.base_nametable_num() * 0x400;
    }
    /// Get the address of the nametable at the top right of the current tilemap.
    pub fn top_right_nametable_addr(&self) -> usize {
        match self.base_nametable_num() {
            0 => 0x2400,
            1 => 0x2000,
            2 => 0x2C00,
            3 => 0x2800,
            _ => panic!("Invalid nametable num {}", self.base_nametable_num()),
        }
    }
    /// Get the address of the nametable at the bottom left of the current tilemap.
    pub fn bot_left_nametable_addr(&self) -> usize {
        match self.base_nametable_num() {
            0 => 0x2800,
            1 => 0x2C00,
            2 => 0x2000,
            3 => 0x2400,
            _ => panic!("Invalid nametable num {}", self.base_nametable_num()),
        }
    }
    /// Get the address of the nametable at the bottom right of the current tilemap.
    pub fn bot_right_nametable_addr(&self) -> usize {
        match self.base_nametable_num() {
            0 => 0x2C00,
            1 => 0x2800,
            2 => 0x2400,
            3 => 0x2000,
            _ => panic!("Invalid nametable num {}", self.base_nametable_num()),
        }
    }
}
