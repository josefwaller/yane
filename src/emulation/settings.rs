/// Settings for the emulator.
/// Contain a couple fields that change the visual output of the PPU.
#[derive(Copy, Clone)]
pub struct Settings {
    /// Debugging palette override, assigns each palette a unique colour to quickly show which tiles are using which palettes.
    pub use_debug_palette: bool,
    /// Whether to limit each scanline to rendering at most 8 sprites.
    /// Sprite 0 hit and sprite overflow flag setting behaviour will not be changed, this is only visual
    pub scanline_sprite_limit: bool,
    /// Whether to always draw sprites on top of the background
    pub always_sprites_on_top: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            use_debug_palette: false,
            scanline_sprite_limit: true,
            always_sprites_on_top: false,
        }
    }
}
