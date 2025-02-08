use serde::{Deserialize, Serialize};

use crate::Settings as EmuSettings;
use std::path::PathBuf;

use crate::KeyMap;

/// Settings used when running Yane as an application.
/// Contains all the settings for running Yane as an emulator, as well as
/// other fields such as quicksave locations, volume, speed, etc.
#[derive(Clone, Serialize, Deserialize)]
pub struct AppSettings {
    // Display OAM debug information on the screen
    pub oam_debug: bool,
    // Pause the game
    #[serde(skip)]
    pub paused: bool,
    // Set the volume multiplyer (between 0 and 1)
    pub volume: f32,
    // Set the speed multiplyer
    pub speed: f32,
    // Whether to record audio
    pub record_audio: bool,
    // The file to record the audio samples to
    pub record_audio_filename: String,
    // Screen output size, only used by window
    pub screen_size: (u32, u32),
    // Whether to verbosely log a lot of things
    // Mostly just used in development
    pub verbose_logging: bool,
    // Whether to disallow pressing two opposite directions on the controller at the same time
    // Can cause glitches in some games (i.e. Zelda II)
    pub restrict_controller_directions: bool,
    /// Game and setting controls
    #[serde(skip)]
    pub key_map: KeyMap,
    /// File of most recent quicksave
    /// Used for quickloading
    #[serde(skip)]
    pub quickload_file: Option<PathBuf>,
    /// The emulator settings
    #[serde(skip)]
    pub emu_settings: EmuSettings,
}

impl Default for AppSettings {
    fn default() -> AppSettings {
        AppSettings {
            paused: false,
            oam_debug: false,
            volume: 1.0,
            speed: 1.0,
            record_audio: false,
            record_audio_filename: "sample".to_string(),
            screen_size: (256, 240),
            verbose_logging: false,
            restrict_controller_directions: true,
            key_map: KeyMap::default(),
            quickload_file: None,
            emu_settings: EmuSettings::default(),
        }
    }
}
