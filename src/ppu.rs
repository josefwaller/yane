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
    /// The PPUSCROLL register
    pub scroll: u8,
    /// The PPUADDR register
    pub addr: u16,
    /// The PPUDATA register
    pub data: u8,
    /// The OAMDMA register
    pub oam_dma: u8,
    /// VRAM
    pub vram: [u8; 0x100],
    // W register
    w: bool,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            oam: [0; 0x100],
            ctrl: 0,
            mask: 0,
            status: 0xA0,
            oam_addr: 0,
            oam_data: 0,
            scroll: 0,
            addr: 0,
            data: 0,
            oam_dma: 0,
            vram: [0; 0x100],
            w: true,
        }
    }

    pub fn write_to_addr(&mut self, value: u8) {
        if self.w {
            self.addr = (self.addr & 0x00FF) + (value as u16) << 8;
        } else {
            self.addr = (self.addr & 0x3F00) + value as u16;
        }
        self.w = !self.w;
        println!("{:X}", self.addr);
    }

    /// Get the tile at a certain address
    pub fn get_tile_at(&self, addr: usize) {}

    /// Write a single byte to VRAM at `PPUADDR`
    /// Increments `PPUADDR` by 1 or by 32 depending `PPUSTATUS`
    pub fn write_to_vram(&mut self, value: u8) {
        if self.addr > 0x3F1F {
            return;
        }
        // println!("Writing VRAM {:X} = {:X}", self.addr, value);
        if self.addr > 0x3EFF {
            self.vram[(self.addr - 0x3F00) as usize % 0x100] = value;
            self.addr = self
                .addr
                .wrapping_add(if self.ctrl & 0x04 == 0 { 1 } else { 32 });
        } else {
        }
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
    pub fn get_spr_pattern_table_addr(&self) -> usize {
        if self.status & 0x08 != 0 {
            return 0x1000;
        }
        0x000
    }
}
