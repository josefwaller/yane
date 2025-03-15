use std::{cmp::min, collections::VecDeque};

use crate::core::Settings;

use super::{Cartridge, DEBUG_PALETTE, HV_TO_RGB};
use log::*;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

/// Number of dots per scanline
const DOTS_PER_SCANLINE: u32 = 341;
/// Number of scanlines per frame
const SCANLINES_PER_FRAME: u32 = 262;
/// Index of the prerender scanline
const PRERENDER_SCANLINE: u32 = SCANLINES_PER_FRAME - 1;
/// Number of render scanlines (scanlines during rendering)
const RENDER_SCANLINES: u32 = 240;
/// Visible dots per scanline
const RENDER_DOTS: u32 = 256;
const DOTS_PER_OPEN_BUS_DECAY: u32 = 1_789_000 / 3;

fn zeros() -> Box<[[usize; 256]; 240]> {
    Box::new([[0; 256]; 240])
}

#[derive(Debug, Serialize, Deserialize)]
/// The picture processing unit of the NES.
///
/// Responsible for computing the picture output of the console.
/// The PPU provides the video output as either the raw hue/value byte per pixel computed
/// by the NES through [Ppu::hv_output], or an easier-to-use RGB value per pixel through
/// [Ppu::rgb_output] or [Ppu::rgb_output_buf].
pub struct Ppu {
    /// The Object Access Memory, or OAM
    #[serde(with = "BigArray")]
    pub oam: [u8; 0x100],
    /// The PPUCTRL register
    pub ctrl: u8,
    /// The PPUMASK register
    pub mask: u8,
    /// The PPUSTATUS register
    pub status: u8,
    /// The OAMADDR register
    pub oam_addr: u8,
    /// The PPUDATA register (actually the read buffer)
    pub data: u8,
    /// The OAMDMA register
    ///
    /// This register is usually [None], and is only set to [Some] when written to.
    /// It is then reset when the DMA is executed.
    pub oam_dma: Option<u8>,
    /// VRAM
    pub palette_ram: [u8; 0x20],
    #[serde(with = "BigArray")]
    pub nametable_ram: [u8; 0x800],
    // W register, false = 0
    w: bool,
    // (x, y) coordinate of dot (pixel) being processed
    pub dot: (u32, u32),
    // t register
    t: u32,
    // v register
    v: u32,
    x: u32,
    // Indices of sprites on the scanline it is currently drawing
    // None means no sprite on that pixel
    #[serde(with = "BigArray")]
    scanline_sprites: [Option<(usize, usize)>; 256],
    // Screen buffer, storing 1 byte HV values per pixel
    #[serde(skip, default = "zeros")]
    output: Box<[[usize; 256]; 240]>,
    // Open bus output
    open_bus: u8,
    // Cycles since open bus was written
    open_bus_dots: u32,
    // Cycles since status byte was read
    status_dots: u32,
    // Tile buffer, emulates both the 2 16bit shift registers for the tile data
    // and the 8bit shift register for the attribute data.
    // First entry is the tile data (index of the pixel in the palette), second is the palette index
    tile_buffer: VecDeque<(usize, usize)>,
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}

impl Ppu {
    /// Initialise a new PPU.
    ///
    /// Zero out all memory, set all registers to their initial value, and set
    /// the dot position to `(0, 0)` (The top left pixel of the screen).
    pub fn new() -> Ppu {
        Ppu {
            oam: [0; 0x100],
            ctrl: 0x00,
            mask: 0,
            status: 0xA0,
            oam_addr: 0,
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
            output: Box::new([[0; 256]; 240]),
            open_bus: 0,
            open_bus_dots: 0,
            status_dots: 0,
            tile_buffer: VecDeque::from([(0, 0); 16]),
        }
    }
    /// Read a byte from the PPU register given an address in CPU space.
    ///
    /// Requires the cartridge currently inserted in the NES.
    pub fn read_byte(&mut self, addr: usize, cartridge: &mut Cartridge) -> u8 {
        match addr % 8 {
            2 => {
                // VBLANK is cleared on read
                let status = self.status;
                self.status &= 0x7F;
                // Clear W
                self.w = false;
                self.status_dots = 0;
                (status & 0xE0) | (self.open_bus & 0x1F)
            }
            4 => {
                // 0 out some bits on the OAM attribute byte (byte 2)
                let v = self.oam[self.oam_addr as usize % self.oam.len()]
                    & if self.oam_addr % 4 == 2 { 0xE3 } else { 0xFF };
                self.open_bus = v;
                v
            }
            7 => {
                // Set decay value to what's read
                let v = self.read_vram(cartridge);
                self.open_bus = v;
                v
            }
            _ => self.open_bus,
        }
    }
    /// Write a byte to the PPU registers given an address in CPU space.
    ///
    /// Requires the cartridge currently inserted in the NES.
    pub fn write_byte(&mut self, addr: usize, value: u8, cartridge: &mut Cartridge) {
        self.open_bus = value;
        self.open_bus_dots = 0;
        match addr % 8 {
            // PPUCTRL
            0 => {
                self.ctrl = value;
                self.t = (self.t & !0x0C00) | (((value & 0x03) as u32) << 10);
            }
            // PPUMASK
            1 => self.mask = value,
            // PPUSTATUS
            2 => self.w = false,
            // OAMADDR
            3 => self.oam_addr = value,
            // OAMDATA
            4 => self.write_oam(0, value),
            // PPUSCROLL
            5 => {
                if self.w {
                    // Second write (Y)
                    self.t = (self.t & 0x0C1F)
                        | (((value & 0x07) as u32) << 12)
                        | (((value & 0x0F8) as u32) << 2);
                } else {
                    // First write (X)
                    self.t = (self.t & 0xFFE0) | (value >> 3) as u32;
                    self.x = (value & 0x07) as u32;
                }
                self.w = !self.w;
            }
            // PPUADDR
            6 => {
                if self.w {
                    // Second write (LSB)
                    self.t = (self.t & 0xFF00) | value as u32;
                    self.v = self.t;
                    // Refresh controller ADDR pin values
                    cartridge.mapper.set_addr_value(self.v);
                } else {
                    // First write (MSB)
                    self.t = (self.t & 0x00FF) | (value as u32 & 0x3F) << 8;
                }
                self.w = !self.w;
            }
            // PPUDATA
            7 => {
                self.write_vram(value, cartridge);
            }
            _ => panic!("This should never happen. Addr is {:#X}", addr),
        }
    }
    /// Write a single byte to OAM using the OAM_ADDR register and the offset provided
    ///
    /// Increments OAM_ADDR after writing.
    pub fn write_oam(&mut self, offset: usize, value: u8) {
        self.oam[(self.oam_addr as usize + offset) % self.oam.len()] = value;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    // Refresh scanline_sprites by fetching the first 8 sprites on the scanline given
    // May set the overflow flag.
    // Note that this is done at the end of the scanline, so these sprites will show up on the next scanline (and thus will appear at Y + 1)
    fn refresh_scanline_sprites(
        &mut self,
        scanline: u32,
        cartridge: &mut Cartridge,
        settings: &Settings,
    ) {
        // Refresh scanline sprites
        self.scanline_sprites = [None; 256];
        if scanline < RENDER_SCANLINES || scanline == PRERENDER_SCANLINE {
            let sprite_height = if self.is_8x16_sprites() { 16 } else { 8 };
            // Get the 8 objs on the scanline (actually on the next scanline, since sprites will be draw on the next one)
            let objs: Vec<usize> = self
                .oam
                .chunks(4)
                .enumerate()
                .filter(|(_i, obj)| {
                    (obj[0] as u32) <= scanline && obj[0] as u32 + sprite_height > scanline
                })
                .map(|(i, _obj)| i)
                .collect();
            // Check for sprite overflow
            if objs.len() > 8 {
                // Check in an incorrectly implemented fashion
                // Where instead of checking the coordinates horizontally, we start diagonally right-down from
                // the last sprite on the scanline
                let last_obj = &self.oam[(4 * objs[7])..(4 * objs[7] + 4)];
                (objs[8]..64).enumerate().for_each(|(i, obj_i)| {
                    let x = last_obj[3] as u32 + i as u32;
                    let y = last_obj[0] as u32 + i as u32;
                    if x < 256
                        && y < 240
                        && self.oam[4 * obj_i] == last_obj[0].wrapping_add(i as u8)
                        && self.oam[4 * obj_i + 3] == last_obj[3].wrapping_add(i as u8)
                    {
                        self.status |= 0x20;
                    }
                });
            }
            // Add them to the scanline
            objs.iter()
                .take(if settings.scanline_sprite_limit {
                    8
                } else {
                    64
                })
                .for_each(|i| {
                    let obj = &self.oam[(4 * i)..(4 * i + 4)];
                    let flip_hor = (obj[2] & 0x40) != 0;
                    let flip_vert = (obj[2] & 0x80) != 0;
                    let palette_index = 16 + 4 * (obj[2] & 0x03) as usize;
                    let y_off = if flip_vert {
                        (sprite_height - 1 - (scanline - (obj[0] as u32))) as usize
                    } else {
                        (scanline - (obj[0] as u32)) as usize
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
                        let pixel_index = (tile_low & 0x01) + (tile_high & 0x02);
                        let x = obj[3] as usize + if flip_hor { j } else { 7 - j };
                        if pixel_index != 0 && x < 256 {
                            self.scanline_sprites[x]
                                .get_or_insert((*i, palette[palette_index + pixel_index] as usize));
                        }
                        tile_low >>= 1;
                        tile_high >>= 1;
                    })
                });
            // We now do dummy fetches to 0xFF for however many spriets we have left
            // This is required for the MMC3 interupts to work
            (0..(8 - min(objs.len(), 8))).for_each(|_| {
                cartridge.read_ppu(if self.is_8x16_sprites() {
                    0x10FE
                } else {
                    self.spr_pattern_table_addr() + 0xFF
                });
            });
        }
    }
    /// Advance the PPU a certain number of dots.
    ///
    /// Write a new pixel of output for every dot processed.
    /// Update the PPU's state accordingly, may set the VBlank flag.
    /// Return [true] if an NMI is triggered by a VBlank, and [false] otherwise.
    pub fn advance_dots(
        &mut self,
        dots: u32,
        cartridge: &mut Cartridge,
        settings: &Settings,
    ) -> bool {
        self.open_bus_dots += dots;
        if self.open_bus_dots >= DOTS_PER_OPEN_BUS_DECAY && self.open_bus != 0 {
            self.open_bus = 0;
        }
        // Todo: tidy
        let mut to_return = false;
        // Dots 0-239 are the visible scanlines, 261 is the pre-render scanline
        (0..dots).for_each(|_| {
            self.status_dots = self.status_dots.saturating_add(1);
            self.dot = if self.dot.0 == DOTS_PER_SCANLINE - 1 {
                if self.dot.1 == SCANLINES_PER_FRAME - 1 {
                    (0, 0)
                } else {
                    (0, self.dot.1 + 1)
                }
            } else {
                (self.dot.0 + 1, self.dot.1)
            };
            // Set output if we are in the visible picture
            self.set_output(settings);
            // Load tile data
            if self.is_background_rendering_enabled() || self.is_sprite_rendering_enabled() {
                if self.dot == (280, PRERENDER_SCANLINE) {
                    // Copy vertical component from T to V
                    self.v = (self.v & 0x041F) | (self.t & !0x041F);
                }
                // IF we are in the visible picture
                if self.dot.1 < RENDER_SCANLINES || self.dot.1 == PRERENDER_SCANLINE {
                    // Fetch sprites to render at dot 263
                    if self.dot.0 == 264 {
                        // Refresh scanline sprites
                        self.refresh_scanline_sprites(self.dot.1, cartridge, settings);
                    }
                    // Check if we should fetch a tile
                    if self.dot.0 < 256 && self.dot.0 % 8 == 7 {
                        self.read_tile_to_buffer(cartridge);
                        self.coarse_x_inc();
                    } else if [328, 336].contains(&self.dot.0) {
                        // Fetch tiles for next line
                        self.read_tile_to_buffer(cartridge);
                        self.coarse_x_inc();
                    }
                }
                if self.dot.0 == 256 && !self.can_access_vram() {
                    self.fine_y_inc();
                    // Copy horizontal nametable and coarse X
                    self.v = (self.v & !0x41F) | (self.t & 0x41F);
                }
            }
            if self.dot == (1, 241) {
                // Set vblank
                self.status |= 0x80;
                // Skip NMI if we read VBlank recently
                if self.status_dots > 3 {
                    to_return = true;
                }
            } else if self.dot == (1, PRERENDER_SCANLINE) {
                // Clear VBlank, sprite overflow and sprite 0 hit flags
                self.status &= 0x1F;
            }
        });
        to_return
    }
    /// Get the output of the PPU as RGB triplets, in a new array
    ///
    /// Get the output of the PPU (i.e. the pixels on the screen) as RGB values.
    /// This will allocate a new buffer every call - it is recomended to allocate a screen
    /// buffer once and call [Ppu::rgb_output_buf] instead.
    pub fn rgb_output(&self) -> [[[u8; 3]; 256]; 240] {
        core::array::from_fn(|y| core::array::from_fn(|x| self.get_rgb(self.output[y][x])))
    }
    /// Copy the current output of the PPU as RGB values into the given buffer
    ///
    /// Copy the PPU output into the buffer provided, as RGB values.
    /// This will overwrite any data in `buf`.
    /// If you want to create a new array instead, use [Ppu::rgb_output].
    pub fn rgb_output_buf(&self, buf: &mut [[[u8; 3]; 256]; 240]) {
        buf.iter_mut().enumerate().for_each(|(y, row)| {
            row.iter_mut()
                .enumerate()
                .for_each(|(x, pixel)| *pixel = self.get_rgb(self.output[y][x]))
        });
    }
    /// Get the current output of the PPU as hue-value bytes
    ///
    /// The NES's video output is a single value for each pixel, representing a
    /// hue/value combination. This output can be paired with an
    /// [NES palette file \(.PAL\)](https://www.nesdev.org/wiki/PPU_palettes#Palettes) to generate
    /// an RGB value for each pixel. [Ppu::rgb_output] will do this automatically.
    pub fn hv_output(&self) -> &[[usize; 256]; 240] {
        &self.output
    }
    /// Transform an HV value into an RGB value
    fn get_rgb(&self, hv_byte: usize) -> [u8; 3] {
        let v = HV_TO_RGB[hv_byte];
        // Check for red/green/blue emphasis
        if !(self.is_red_tint_on() || self.is_green_tint_on() || self.is_blue_tint_on()) {
            v
        } else {
            const M: f32 = 0.5;
            let should_dim = [
                self.is_green_tint_on() || self.is_blue_tint_on(),
                self.is_red_tint_on() || self.is_blue_tint_on(),
                self.is_red_tint_on() || self.is_green_tint_on(),
            ];
            core::array::from_fn(|i| {
                (v[i] as f32 * if should_dim[i] { M } else { 1.0 }).floor() as u8
            })
        }
    }
    /// Compute the output at the current dot, and set it in [Ppu::output]
    fn set_output(&mut self, settings: &Settings) {
        // If we are in the render window
        if self.dot.0 < RENDER_DOTS && self.dot.1 < RENDER_SCANLINES {
            let palette = if settings.use_debug_palette {
                &DEBUG_PALETTE
            } else {
                &self.palette_ram
            };
            // Initially set output to background
            let mut output = if self.is_background_rendering_enabled()
                && !(self.dot.0 < 8 && self.background_left_clipping())
            {
                let (index, palette_index) = match self.tile_buffer.get(self.x as usize) {
                    Some(t) => *t,
                    None => {
                        error!(
                            "Ppu::tile_buffer is too small (len={:}, fine x={:}, dot={:?})",
                            self.tile_buffer.len(),
                            self.x,
                            self.dot
                        );
                        (0, 0)
                    }
                };
                if index == 0 {
                    None
                } else {
                    Some(palette[4 * palette_index + index] as usize)
                }
            } else {
                None
            };
            // Check for sprite
            if self.is_sprite_rendering_enabled()
                && !(self.dot.0 < 8 && self.sprite_left_clipping())
            {
                if let Some((j, p)) = self.scanline_sprites[self.dot.0 as usize] {
                    // Check for sprite 0 hit
                    if !self.sprite_zero_hit()
                        && j == 0
                        && output.is_some()
                        && self.dot.1 > 0
                        && self.dot.0 < 255
                        && (self.dot.0 > 7
                            || (!self.sprite_left_clipping() && !self.background_left_clipping()))
                    {
                        // debug!("Hit {:?}", self.dot);
                        self.status |= 0x40;
                    }
                    if self.oam[4 * j + 2] & 0x20 == 0
                        || output.is_none()
                        || settings.always_sprites_on_top
                    {
                        output = Some(p);
                    }
                }
            }
            // Set output to background/sprite or palette 0
            self.output[self.dot.1 as usize][self.dot.0 as usize] =
                output.unwrap_or(self.palette_ram[0] as usize);
        }
        // Shift tile and attribute registers
        if self.dot.0 < 337 {
            self.tile_buffer.pop_front();
            self.tile_buffer.push_back((0, 0));
        }
    }

    fn read_tile_to_buffer(&mut self, cartridge: &mut Cartridge) {
        // Get nametable
        let nt_addr = cartridge.transform_nametable_addr(0x2000 + (self.v as usize & 0x0FFF));
        let nt_num = self.nametable_ram[nt_addr] as usize;
        // Get palette index
        let palette_byte_addr = cartridge.transform_nametable_addr(
            (0x23C0 + (self.v & 0xC00) + ((self.v >> 4) & 0x38) + ((self.v >> 2) & 0x07)) as usize,
        );
        let palette_byte = self.nametable_ram[palette_byte_addr];
        let palette_shift = ((self.v & 0x40) >> 4) + (self.v & 0x02);
        let palette_index = ((palette_byte >> palette_shift) as usize) & 0x03;
        // Get high/low byte of tile
        let fine_y = ((self.v & 0x7000) >> 12) as usize;
        let tile_low =
            cartridge.read_ppu(self.nametable_tile_addr() + 16 * nt_num + fine_y) as usize;
        // This is initially shifted right by 1 so that we can just read the second-last bit when combining it with tile_low
        let tile_high = (cartridge.read_ppu(self.nametable_tile_addr() + 16 * nt_num + 8 + fine_y)
            as usize)
            << 1;
        // Write to the last 8 entries in the 16 bit shift register
        // Which for us is the last 8 elements in the queue
        self.tile_buffer.truncate(8);
        (0..8).for_each(|i| {
            self.tile_buffer.push_back((
                ((tile_low >> (7 - i)) & 0x01) + ((tile_high >> (7 - i)) & 0x02),
                palette_index,
            ))
        });
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
            } else if self.v & 0x3E0 == 0x3E0 {
                self.v ^ (0x7000 | 0x3E0)
            } else {
                // Reset fine Y and increment coarse Y
                self.v - 0x7000 + 0x20
            }
        } else {
            // Inc fine Y
            self.v + 0x1000
        };
    }
    /// Whether the PPU is currently in VBlank
    pub fn in_vblank(&self) -> bool {
        self.dot.1 >= 240
    }
    /// Whether it is safe for the CPU to access VRAM (i.e. the PPU is not currently accessing VRAM).
    ///
    /// Note this does not stop the CPU from accessing VBlank.
    /// Rather, trying to access VRAM when it is not safe to do so but can cause unexpected behaviour.
    pub fn can_access_vram(&self) -> bool {
        self.in_vblank()
            || (!self.is_background_rendering_enabled() && !self.is_sprite_rendering_enabled())
    }
    /// Write a single byte to VRAM at `PPUADDR` .
    /// Increments `PPUADDR` by 1 or by 32 depending on `PPUSTATUS`
    fn write_vram(&mut self, value: u8, cartridge: &mut Cartridge) {
        let addr = self.v & 0x3FFF;
        if addr < 0x2000 {
            cartridge.write_ppu(addr as usize, value);
        } else if addr < 0x3000 {
            self.nametable_ram[cartridge.transform_nametable_addr(addr as usize)] = value;
        } else if addr >= 0x3F00 {
            let palette_index = Ppu::get_palette_index(addr as u16);
            self.palette_ram[palette_index] = value;
        }
        if self.can_access_vram() {
            self.inc_addr(cartridge);
        } else {
            self.coarse_x_inc();
            self.fine_y_inc();
        }
    }

    /// Read a single byte from VRAM using the v register
    fn read_vram(&mut self, cartridge: &mut Cartridge) -> u8 {
        let addr = self.v & 0x3FFF;
        if self.can_access_vram() {
            self.inc_addr(cartridge);
        } else {
            self.coarse_x_inc();
            self.fine_y_inc();
        }
        if addr < 0x2000 {
            // Set buffer to cartridge read value and return old buffer
            let b = self.data;
            self.data = cartridge.read_ppu(addr as usize);
            return b;
        }
        if addr < 0x3F00 {
            // Update buffer to nametable value and return old buffer
            let b = self.data;
            self.data = self.nametable_ram[cartridge.transform_nametable_addr(addr as usize)];
            return b;
        }
        // Palette ram updates the buffer but also returns the current value
        let palette_index = Ppu::get_palette_index(addr as u16);
        let b = (self.open_bus & 0xC0) | (self.palette_ram[palette_index] & 0x3F);
        // Read the mirrored nametable byte into memory
        self.data = self.nametable_ram[cartridge.transform_nametable_addr(addr as usize)];
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

    fn inc_addr(&mut self, cartridge: &mut Cartridge) {
        // V is 14 bits long, not 16
        self.v = (self.v + if self.ctrl & 0x04 == 0 { 1 } else { 32 }) % 0x3FFF;
        cartridge.mapper.set_addr_value(self.v);
    }
    /// Returns [true] if the NES is in 8x16 sprite mode
    pub fn is_8x16_sprites(&self) -> bool {
        (self.ctrl & 0x20) != 0
    }
    /// Returns [true] if OAM rendering is enabled
    pub fn is_sprite_rendering_enabled(&self) -> bool {
        (self.mask & 0x10) != 0
    }
    /// Return [true]` if background rendering is enabled
    pub fn is_background_rendering_enabled(&self) -> bool {
        (self.mask & 0x08) != 0
    }
    /// Returns [true] if  rendering sprites in the 8 leftmost pixels is disabled
    pub fn sprite_left_clipping(&self) -> bool {
        (self.mask & 0x04) == 0
    }
    /// Returns [true] if rendering the background in the 8 leftmost pixels is disabled
    pub fn background_left_clipping(&self) -> bool {
        (self.mask & 0x02) == 0
    }
    /// Returns [true] if greyscale mode is on
    pub fn is_greyscale_mode_on(&self) -> bool {
        (self.mask & 0x01) != 0
    }
    /// Returns the address in PPU memory space to read the sprite pattern data from
    pub fn spr_pattern_table_addr(&self) -> usize {
        if self.ctrl & 0x08 != 0 {
            return 0x1000;
        }
        0x0000
    }
    /// Return the address in PPU memory space to read the background pattern data from
    pub fn nametable_tile_addr(&self) -> usize {
        if self.ctrl & 0x10 != 0 {
            return 0x1000;
        }
        0x0000
    }
    // Return [true] if the red tint is active
    pub fn is_red_tint_on(&self) -> bool {
        (self.mask & 0x20) != 0
    }
    // Return [true] if the blue tint is active
    pub fn is_blue_tint_on(&self) -> bool {
        (self.mask & 0x40) != 0
    }
    // Return [true] if the green tint is active
    pub fn is_green_tint_on(&self) -> bool {
        (self.mask & 0x80) != 0
    }
    /// Return [true] if the NMI is enabled
    pub fn get_nmi_enabled(&self) -> bool {
        self.ctrl & 0x80 != 0
    }
    /// Return [true] if the sprite 0 hit bit is set
    pub fn sprite_zero_hit(&self) -> bool {
        (self.status & 0x40) != 0
    }
    /// Return [true] if the sprite overflow flag is set
    pub fn sprite_overflow(&self) -> bool {
        (self.status & 0x20) != 0
    }
    /// Returns the base nametable number, a nubmer between 0 and 3.
    ///
    /// * 0 means that the base nametable is top left (0x2000)
    /// * 1 means that the base nametable is top right (0x2400)
    /// * 2 means that the base nametable is bot left (0x2800)
    /// * 3 means that the base nametable is bot right (0x2C00)
    ///
    /// The base nametable address can then be found by calculating `0x2000 + 0x400 * `[Ppu::base_nametable_num]
    pub fn base_nametable_num(&self) -> usize {
        (self.ctrl as usize) & 0x03
    }
    /// Get the address of the nametable at the top left of the current tilemap.
    pub fn top_left_nametable_addr(&self) -> usize {
        0x2000 + self.base_nametable_num() * 0x400
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
