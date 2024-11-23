/// Various settings for rendering the emulator's output
pub struct Settings {
    // Display OAM debug information on the screen
    pub oam_debug: bool,
    // Use the debug palette
    pub palette_debug: bool,
    // Pause the game
    pub paused: bool,
    // Set the volume multiplyer (between 0 and 1)
    pub volume: f32,
    // Whether to limit each scanline to rendering at most 8 sprites
    // Sprite 0 hit and sprite overflow flag setting behaviour will not be changed, this is only visual
    pub scanline_sprite_limit: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            oam_debug: false,
            palette_debug: false,
            paused: false,
            volume: 0.25,
            scanline_sprite_limit: true,
        }
    }
}
