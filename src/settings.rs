/// Various settings for rendering the emulator's output
#[derive(Clone)]
pub struct Settings {
    // Display OAM debug information on the screen
    pub oam_debug: bool,
    // Debugging palette override
    pub use_debug_palette: bool,
    // Pause the game
    pub paused: bool,
    // Set the volume multiplyer (between 0 and 1)
    pub volume: f32,
    // Set the speed multiplyer
    pub speed: f32,
    // Whether to limit each scanline to rendering at most 8 sprites
    // Sprite 0 hit and sprite overflow flag setting behaviour will not be changed, this is only visual
    pub scanline_sprite_limit: bool,
    // Whether to always draw sprites on top of the background
    pub always_sprites_on_top: bool,
    // Whether to record audio
    pub record_audio: bool,
    // The file to record the audio samples to
    pub record_audio_filename: String,
    // Screen output size, only used by window
    pub screen_size: (u32, u32),
    // Whether to verbosely log a lot of things
    // Mostly just used in development
    pub verbose_logging: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            oam_debug: false,
            use_debug_palette: false,
            paused: false,
            volume: 1.0,
            speed: 1.0,
            scanline_sprite_limit: true,
            always_sprites_on_top: false,
            record_audio: false,
            record_audio_filename: "sample".to_string(),
            screen_size: (256, 240),
            verbose_logging: false,
        }
    }
}
