mod app_settings;
mod emulation;
mod interface;
mod utils;
pub use app_settings::AppSettings;
pub use emulation::*;
pub use interface::*;
pub const CPU_CLOCK_SPEED: u32 = 1_789_000;
