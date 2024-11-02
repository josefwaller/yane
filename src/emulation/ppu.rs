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
    /// The PPUDATA register
    pub data: u8,
    /// The OAMDMA register
    pub oam_dma: u8,
    /// VRAM
    pub palette_ram: [u8; 0x20],
    pub nametable_ram: [u8; 0x400],
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
            nametable_ram: [0; 0x400],
            w: true,
        }
    }

    /// Read a byte from the PPU register given an address in CPU space
    pub fn read_byte(&mut self, addr: usize) -> u8 {
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
            7 => self.data,
            _ => panic!("This should never happen. Addr is {:#X}", addr),
        }
    }

    /// Write a byte to the PPU registers given an address in CPU space
    pub fn write_byte(&mut self, addr: usize, value: u8) {
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
            7 => self.write_to_vram(value),
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
    pub fn write_to_vram(&mut self, value: u8) {
        if self.addr >= 0x3F00 {
            self.palette_ram[(self.addr - 0x3F00) as usize % 0x020] = value;
        } else if self.addr >= 0x2000 {
            self.nametable_ram[(self.addr - 0x2000) as usize % 0x400] = value
        }
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
}
