mod app_settings;
mod emulation;
mod interface;
mod utils;
pub use app_settings::AppSettings;
pub use emulation::*;
pub use interface::*;
// Todo remove
pub const CPU_CYCLES_PER_SCANLINE: f32 = 113.67;
pub const CPU_CYCLES_PER_VBLANK: i64 = 2260;
pub const CPU_CYCLES_PER_OAM: u32 = 513;
