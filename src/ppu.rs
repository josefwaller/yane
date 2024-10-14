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
    pub addr: u8,
    /// The PPUDATA register
    pub data: u8,
    /// The OAMDMA register
    pub oam_dma: u8,
    /// VRAM
    pub vram: [u8; 0x100],
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
        }
    }

    /// Write a single byte to VRAM at `PPUADDR`
    /// Increments `PPUADDR` by 1 or by 32 depending `PPUSTATUS`
    pub fn write_to_vram(&mut self, value: u8) {
        self.vram[self.addr as usize] = value;
        self.addr = self
            .addr
            .wrapping_add(if self.status & 0x04 == 0 { 1 } else { 32 });
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
        (self.mask & 0x04) != 0
    }
}
