use crate::Settings;

use super::{cartridge, Cartridge, NametableArrangement, DEBUG_PALETTE};
use log::*;

const DOTS_PER_SCANLINE: u32 = 341;
const SCANLINES_PER_FRAME: u32 = 262;

#[derive(Debug)]
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
    /// The PPUSCROLL register, split into its X/Y components
    pub scroll_x: u8,
    pub scroll_y: u8,
    /// The PPUADDR register
    pub addr: u16,
    // Temporary register holding most significant byte of the address
    temp_addr_msb: u8,
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
    // (x, y) coordinate of dot (pixel) being processed
    dot: (u32, u32),
    // t register
    t: u32,
    // v register
    v: u32,
    x: u32,
    // Indices of sprites on the scanline it is currently drawing
    // None means no sprite on that pixel
    scanline_sprites: [Option<(usize, usize)>; 256],
    // Internal screen buffer, olding indices of colours in TV palette
    pub output: [[usize; 256]; 240],
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            oam: [0; 0x100],
            ctrl: 0x00,
            mask: 0,
            status: 0xA0,
            oam_addr: 0,
            scroll_x: 0,
            scroll_y: 0,
            addr: 0,
            temp_addr_msb: 0,
            data: 0,
            oam_dma: None,
            palette_ram: [0; 0x20],
            nametable_ram: [0; 0x800],
            w: true,
            dot: (0, 0),
            t: 0,
            v: 0,
            x: 0,
            scanline_sprites: [None; 256],
            output: [[0; 256]; 240],
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
            4 => self.oam[self.oam_addr as usize % self.oam.len()],
            // SCROLL and ADDR shouldn't be read from
            5 => self.scroll_x,
            6 => self.addr as u8,
            7 => {
                if self.in_vblank() {
                    self.v = if self.v & 0x7000 == 0x7000 {
                        // Note we are checking for 0x3A0 here
                        // Coarse Y wraps at 30, not 32
                        if self.v & 0x3E0 == 0x3A0 {
                            // Switch vertical nametable and reset both coarse and fine Y
                            self.v ^ (0x800 + 0x3A0 + 0x7000)
                        } else {
                            // Reset fine Y and increment coarse Y
                            self.v - 0x7000 + 0x20
                        }
                    } else {
                        // Inc fine Y
                        self.v + 0x1000
                    };
                    // Go to next tile or horizontal nametable
                    self.v = if self.v & 0x1F == 0x1F {
                        self.v ^ 0x41F
                    } else {
                        self.v + 1
                    };
                }
                self.read_vram(cartridge)
            }
            _ => panic!("This should never happen. Addr is {:#X}", addr),
        }
    }

    /// Write a byte to the PPU registers given an address in CPU space
    pub fn write_byte(&mut self, addr: usize, value: u8, cartridge: &mut Cartridge) {
        match addr % 8 {
            0 => {
                self.ctrl = value;
                self.t = (self.t & !0xC00) | (((value & 0x03) as u32) << 10);
            }
            1 => self.mask = value,
            2 => {}
            3 => self.oam_addr = value,
            4 => self.write_oam(0, value),
            5 => {
                if self.w {
                    self.scroll_y = value;
                    self.t = (self.t & 0x0C1F)
                        | (((value & 0x07) as u32) << 12)
                        | (((value & 0x0F8) as u32) << 2);
                } else {
                    self.t = (self.t & 0xFFE0) | (value >> 3) as u32;
                    self.x = (value & 0x07) as u32;
                    self.scroll_x = value
                }
                self.w = !self.w;
            }
            6 => {
                self.write_to_addr(value);
            }
            7 => {
                self.write_vram(value, cartridge);
                if self.in_vblank() {
                    self.fine_y_inc();
                    self.coarse_x_inc();
                }
            }
            _ => panic!("This should never happen. Addr is {:#X}", addr),
        }
    }

    /// Write to OAM using OAM_ADDR and the offset provided
    /// Increments OAM_ADDR
    pub fn write_oam(&mut self, offset: usize, value: u8) {
        self.oam[(self.oam_addr as usize + offset) % self.oam.len()] = value;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    pub fn write_to_addr(&mut self, value: u8) {
        if self.w {
            // Set address
            self.addr = ((self.temp_addr_msb as u16) << 8) + value as u16;
            self.t = (self.t & 0xFF00) | value as u32;
            self.v = self.t;
        } else {
            // Set the most significant byte to the temp register
            self.temp_addr_msb = value;
            self.t = (self.t & 0x00FF) | (value as u32 & 0x3F) << 8;
        }
        self.w = !self.w;
    }
    // Return whether to trigger an NMI
    pub fn advance_dots(
        &mut self,
        dots: u32,
        cartridge: &Cartridge,
        settings_opt: Option<Settings>,
    ) -> bool {
        let settings = settings_opt.unwrap_or_default();
        // Todo: tidy
        let mut to_return = false;
        (0..dots).for_each(|_| {
            self.dot = if self.dot.0 == DOTS_PER_SCANLINE - 1 {
                if self.dot.1 == SCANLINES_PER_FRAME - 1 {
                    (0, 0)
                } else {
                    (0, self.dot.1 + 1)
                }
            } else {
                (self.dot.0 + 1, self.dot.1)
            };
            if self.dot.0 == 0 {
                if self.dot.1 == 0 {
                    self.v = self.t;
                }
                // Refresh scanline sprites
                self.scanline_sprites = [None; 256];
                let sprite_height = if self.is_8x16_sprites() { 16 } else { 8 };
                // Get the 8 objs on the scanline
                let objs: Vec<usize> = self
                    .oam
                    .chunks(4)
                    .enumerate()
                    .filter(|(_i, obj)| {
                        (obj[0] as u32 + 1) <= self.dot.1
                            && obj[0] as u32 + 1 + sprite_height > self.dot.1
                    })
                    .take(if settings.scanline_sprite_limit {
                        8
                    } else {
                        64
                    })
                    .map(|(i, _obj)| i)
                    .collect();
                // Add them to the scanline
                objs.iter().for_each(|i| {
                    let obj = &self.oam[(4 * i)..(4 * i + 4)];
                    let flip_hor = (obj[2] & 0x40) != 0;
                    let flip_vert = (obj[2] & 0x80) != 0;
                    let palette_index = 16 + 4 * (obj[2] & 0x03) as usize;
                    let y_off = if flip_vert {
                        (sprite_height - 1 - (self.dot.1 - (obj[0] as u32 + 1))) as usize
                    } else {
                        (self.dot.1 - (obj[0] as u32 + 1)) as usize
                    };

                    let (mut tile_low, mut tile_high) = if self.is_8x16_sprites() {
                        let tile_addr = 0x1000 * (obj[1] & 0x01) as usize
                            + 16 * (obj[1] & 0xFE) as usize
                            + if y_off > 7 { 16 + y_off % 8 } else { y_off };
                        (
                            cartridge.read_ppu(tile_addr) as usize,
                            cartridge.read_ppu(tile_addr + 8) as usize,
                        )
                    } else {
                        let tile_addr =
                            self.spr_pattern_table_addr() + 16 * obj[1] as usize + y_off;
                        (
                            cartridge.read_ppu(tile_addr) as usize,
                            cartridge.read_ppu(tile_addr + 8) as usize,
                        )
                    };
                    // Optimization - shift tile_high left by one so combining it with tile_low is simply
                    // (tile_high & 0x02) + (tile_lot & 0x01)
                    tile_high <<= 1;
                    let palette = if settings.use_debug_palette {
                        &DEBUG_PALETTE
                    } else {
                        &self.palette_ram
                    };
                    (0..8).for_each(|j| {
                        let pixel_index = (tile_low as usize & 0x01) + (tile_high as usize & 0x02);
                        let x = obj[3] as usize + if flip_hor { j } else { 7 - j };
                        if pixel_index != 0 && x < 256 {
                            self.scanline_sprites[x]
                                .get_or_insert((*i, palette[palette_index + pixel_index] as usize));
                        }
                        tile_low >>= 1;
                        tile_high >>= 1;
                    })
                });
            }
            if self.dot.0 < 256 + 8 {
                if self.dot.1 < 240 {
                    match self.dot.0 % 8 {
                        2 => {
                            // Fetch nametable
                        }
                        4 => {
                            // Fetch attribute table
                        }
                        6 => {
                            // Fetch LSB
                        }
                        7 => {
                            // Fetch MSB
                            // Also set output
                            // Get nametable
                            let nt_addr = cartridge
                                .transform_nametable_addr(0x2000 + (self.v as usize & 0xFFF));
                            let nt_num = self.nametable_ram[nt_addr] as usize;
                            // Get palette index
                            let palette_byte_addr = cartridge.transform_nametable_addr(
                                (0x23C0
                                    + (self.v & 0xC00)
                                    + ((self.v >> 4) & 0x38)
                                    + ((self.v >> 2) & 0x07))
                                    as usize,
                            );
                            let palette_byte = self.nametable_ram[palette_byte_addr];
                            let palette_shift = ((self.v & 0x40) >> 4) + ((self.v & 0x02) >> 0);
                            let palette_index = ((palette_byte >> palette_shift) as usize) & 0x03;
                            // Get high/low byte of tile
                            let fine_y = ((self.v & 0x7000) >> 12) as usize;
                            let mut tile_low = cartridge
                                .read_ppu(self.nametable_tile_addr() + 16 * nt_num + fine_y)
                                as usize;
                            let mut tile_high = cartridge
                                .read_ppu(self.nametable_tile_addr() + 16 * nt_num + 8 + fine_y)
                                as usize;
                            // Add tile data to output
                            tile_high <<= 1;
                            (0..8).for_each(|i| {
                                let palette = if settings.use_debug_palette {
                                    &DEBUG_PALETTE
                                } else {
                                    &self.palette_ram
                                };
                                let x: isize = self.dot.0 as isize - i as isize - self.x as isize;
                                if x >= 0 && x < 256 {
                                    // Initially set output to background
                                    let mut output = if self.is_background_rendering_enabled() {
                                        let index = (tile_low & 0x01) + (tile_high & 0x02);
                                        if index == 0 {
                                            None
                                        } else {
                                            Some(palette[4 * palette_index + index] as usize)
                                        }
                                    } else {
                                        None
                                    };
                                    // Check for sprite
                                    if let Some((j, p)) = self.scanline_sprites[x as usize] {
                                        if self.is_sprite_rendering_enabled() {
                                            // Check for sprite 0 hit
                                            if !self.sprite_zero_hit()
                                                && j == 0
                                                && output.is_some()
                                                && x < 255
                                                && (x > 7
                                                    || (!self.sprite_left_clipping()
                                                        && !self.background_left_clipping()))
                                            {
                                                self.status |= 0x40;
                                            }
                                            if self.oam[4 * j + 2] & 0x20 == 0
                                                || output == None
                                                || settings.always_sprites_on_top
                                            {
                                                output = Some(p);
                                            }
                                        }
                                    }
                                    self.output[self.dot.1 as usize][x as usize] =
                                        output.unwrap_or(self.palette_ram[0] as usize);
                                }
                                tile_low >>= 1;
                                tile_high >>= 1;
                            });
                            self.coarse_x_inc();
                        }
                        _ => {}
                    }
                }
            } else if self.dot.0 == 256 + 8 {
                self.fine_y_inc();
                // Copy horizontal nametable and coarse X
                self.v = (self.v & !0x41F) + (self.t & 0x41F);
            }
            // Passes the timing test
            if self.dot == (13, 241) {
                // Set vblank
                self.status |= 0x80;
                to_return = true;
            } else if self.dot == (1, 261) {
                // Clear VBlank, sprite overflow and sprite 0 hit flags
                self.status &= 0x1F;
            }
        });
        to_return
    }

    // Coarse X increment on V
    fn coarse_x_inc(&mut self) {
        // Go to next tile or horizontal nametable
        self.v = if self.v & 0x1F == 0x1F {
            self.v ^ 0x41F
        } else {
            self.v + 1
        };
    }
    // Fine Y increment on V
    fn fine_y_inc(&mut self) {
        self.v = if self.v & 0x7000 == 0x7000 {
            // Note we are checking for 0x3A0 here
            // Coarse Y wraps at 30, not 32
            if self.v & 0x3E0 == 0x3A0 {
                // Switch vertical nametable and reset both coarse and fine Y
                self.v ^ (0x800 + 0x3A0 + 0x7000)
            } else {
                // Reset fine Y and increment coarse Y
                self.v - 0x7000 + 0x20
            }
        } else {
            // Inc fine Y
            self.v + 0x1000
        };
    }

    pub fn in_vblank(&self) -> bool {
        // self.dot.0 + DOTS_PER_SCANLINE * self.dot.1 > 1 + DOTS_PER_SCANLINE * 241 || self.dot.1 == 0
        self.dot.1 < 1 || self.dot.1 > 240
    }
    /// Write a single byte to VRAM at `PPUADDR`
    /// Increments `PPUADDR` by 1 or by 32 depending `PPUSTATUS`
    fn write_vram(&mut self, value: u8, cartridge: &mut Cartridge) {
        if self.addr < 0x2000 {
            cartridge.write_chr(self.addr as usize, value);
        } else if self.addr < 0x3000 {
            self.nametable_ram[cartridge.transform_nametable_addr(self.addr as usize)] = value;
        } else if self.addr >= 0x3F00 {
            let palette_index = Ppu::get_palette_index(self.addr);
            self.palette_ram[palette_index] = value;
        }
        self.inc_addr();
    }

    /// Read a single byte from VRAM
    fn read_vram(&mut self, cartridge: &Cartridge) -> u8 {
        let addr = self.addr;
        self.inc_addr();
        if self.addr < 0x2000 {
            // Set buffer to cartridge read value and return old buffer
            let b = self.data;
            self.data = cartridge.read_ppu(addr as usize);
            return b;
        }
        if self.addr < 0x3F00 {
            // Update buffer to nametable value and return old buffer
            let b = self.data;
            self.data = self.nametable_ram[cartridge.transform_nametable_addr(addr as usize)];
            return b;
        }
        // Palette ram updates the buffer but also returns the current value
        let palette_index = Ppu::get_palette_index(addr);
        let b = self.palette_ram[palette_index];
        // Read the mirrored nametable byte into memory
        self.data = self.nametable_ram[addr as usize % self.nametable_ram.len()];
        b
    }

    fn get_palette_index(addr: u16) -> usize {
        // The 0th (invisible) colors are shared between background and sprites
        if addr % 4 == 0 {
            addr as usize % 0x10
        } else {
            addr as usize % 0x20
        }
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
    pub fn is_sprite_rendering_enabled(&self) -> bool {
        (self.mask & 0x10) != 0
    }
    /// Return true if background rendering is enabled, and false otherwise
    pub fn is_background_rendering_enabled(&self) -> bool {
        (self.mask & 0x08) != 0
    }
    /// Whether rendering sprites in the 8 leftmost pixels is disabled.
    pub fn sprite_left_clipping(&self) -> bool {
        (self.mask & 0x04) == 0
    }
    /// Whether rendering the background in the 8 leftmost pixels is disabled.
    pub fn background_left_clipping(&self) -> bool {
        (self.mask & 0x02) == 0
    }
    /// Return whether greyscale mode is on
    pub fn is_greyscale_mode_on(&self) -> bool {
        (self.mask & 0x01) != 0
    }
    /// Return where to read the sprite patterns from
    pub fn spr_pattern_table_addr(&self) -> usize {
        if self.ctrl & 0x08 != 0 {
            return 0x1000;
        }
        0x0000
    }
    /// Return where to read the backgronud patterns from
    pub fn nametable_tile_addr(&self) -> usize {
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
    /// Get the index of the scanline currently being drawn.
    /// Between [0, 261]
    pub fn scanline(&self) -> u32 {
        self.dot.1
    }
}
