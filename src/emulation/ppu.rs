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
    /// This register is usually None, and is only set to Some(n) when written to.
    /// It is then reset when the DMA is executed.
    pub oam_dma: Option<u8>,
    /// VRAM
    pub palette_ram: [u8; 0x20],
    pub nametable_ram: [u8; 0x800],
    // W register, false = 0
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
            oam_dma: None,
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
                // Clear W
                self.w = false;
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
                    self.scroll_y = value;
                } else {
                    self.scroll_x = value
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
            // Set LSB
            self.addr = (self.addr & 0x3F00) + value as u16;
        } else {
            // Set MSB
            self.addr = (self.addr & 0x00FF) + ((value as u16) << 8);
        }
        self.w = !self.w;
    }

    /// Set the sprite zero hit flag and the sprite overflow flag
    /// when rendering the scanline.
    pub fn on_scanline(&mut self, cartridge: &Cartridge, scanline: usize) {
        if scanline == 0 {
            // Clear sprite zero and sprite overflow flags
            self.status &= 0b1001_1111;
        }
        // Check for sprite 0 hit
        if !self.sprite_zero_hit()
            && self.is_background_rendering_enabled()
            && self.is_oam_rendering_enabled()
        {
            let y = self.oam[0] as usize;
            let sprite_height = if self.is_8x16_sprites() { 16 } else { 8 };
            if y <= scanline && y + sprite_height > scanline {
                // Get the tile
                let tile_num = if self.is_8x16_sprites() {
                    ((self.oam[1] as usize & 0x01) << 8) + (self.oam[1] as usize & 0xFE)
                } else {
                    self.get_spr_pattern_table_addr() + self.oam[1] as usize
                };
                let slice_index = scanline - y;
                let slice = if self.is_8x16_sprites() && slice_index > 7 {
                    // Get the next tile if we are in the bottom half of an 8x16 tile
                    let tile = cartridge.get_tile(tile_num + 1);
                    tile[slice_index - 8] | tile[slice_index]
                } else {
                    let tile = cartridge.get_tile(tile_num);
                    tile[slice_index] | tile[slice_index + 8]
                };
                if slice != 0 {
                    // Check all 8 pixels
                    for i in 0..8 {
                        if (slice >> i) & 0x01 != 0 {
                            // Check if this pixel intersects the background
                            if !self.background_pixel_is_transparent(
                                self.oam[3] as usize + (7 - i),
                                y,
                                cartridge,
                            ) {
                                self.status |= 0x40;
                                break;
                            }
                        }
                    }
                }
            }
        }
        // Check for sprite overflow
        if self
            .oam
            .chunks(4)
            .filter(|obj| {
                let y = obj[0] as usize;
                y <= scanline && y < scanline + if self.is_8x16_sprites() { 8 } else { 16 }
            })
            .count()
            > 8
        {
            self.status |= 0x20;
        } else {
            self.status &= !0x20;
        }
    }

    // Return whether the background pixel index given an (x, y) coordinate in screen space is transparent
    fn background_pixel_is_transparent(&self, x: usize, y: usize, cartridge: &Cartridge) -> bool {
        // Get tile X Y coordinates
        let tile_x = x / 8;
        let tile_y = y / 8;
        // Get nametable tile address by figuring out what nametable we're in
        let tile_addr = if tile_x < 32 {
            if tile_y < 30 {
                self.top_left_nametable_addr() + 32 * tile_y + tile_x
            } else {
                self.bot_left_nametable_addr() + 32 * (tile_y - 30) + tile_x
            }
        } else {
            if tile_y < 30 {
                self.bot_left_nametable_addr() + 32 * tile_y + (tile_x - 32)
            } else {
                self.bot_right_nametable_addr() + 32 * (tile_y - 30) + (tile_x - 32)
            }
        };
        let final_tile_addr = cartridge.transform_nametable_addr(tile_addr);
        // Get tile
        let tile_num = self.get_background_pattern_table_addr() / 0x10
            + self.nametable_ram[final_tile_addr] as usize;
        let tile = &cartridge.get_tile(tile_num);
        // Get slice at this y index
        let slice = tile[y % 8] | tile[8 + y % 8];
        // Check pixel at slice
        (slice >> (7 - (x % 8))) == 0
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
    /// Return true if OAM rendering is enabled, and false otherwise
    pub fn is_oam_rendering_enabled(&self) -> bool {
        (self.mask & 0x10) != 0
    }
    /// Return true if background rendering is enabled, and false otherwise
    pub fn is_background_rendering_enabled(&self) -> bool {
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
    /// TODO: Rename
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
    pub fn sprite_zero_hit(&self) -> bool {
        (self.status & 0x40) != 0
    }
    pub fn sprite_overflow(&self) -> bool {
        (self.status & 0x20) != 0
    }
    // TODO: maybe remove, just do this in nes
    pub fn on_vblank(&mut self) {
        // Set VBlank flag
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
