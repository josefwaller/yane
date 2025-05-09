use serde::{Deserialize, Serialize};

use super::{envelope::Envelope, length_counter::LengthCounter};
use std::fmt::Debug;

#[derive(Clone, Copy, Serialize, Deserialize)]
/// The APU's Noise register
/// Creates psuedo random noise wave output.
pub struct NoiseRegister {
    pub length_counter: LengthCounter,
    pub enabled: bool,
    /// The current value of the timer, controls the wave's frequency
    pub timer: u32,
    /// The reload value for the timer
    pub timer_reload: u32,
    pub envelope: Envelope,
    /// The mode ([false] = 0, [true] = 1)
    pub mode: bool,
    /// The shift register
    pub shift: u16,
}
impl Debug for NoiseRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "on={} timer={:3X} length=[{:?}]",
            self.enabled, self.timer, self.length_counter
        )
    }
}

impl Default for NoiseRegister {
    fn default() -> Self {
        NoiseRegister {
            length_counter: LengthCounter::default(),
            enabled: false,
            timer: 0,
            timer_reload: 0,
            envelope: Envelope::default(),
            mode: false,
            shift: 1,
        }
    }
}

impl NoiseRegister {
    pub fn muted(&self) -> bool {
        !self.enabled || self.length_counter.muted() || self.shift & 0x01 == 1
    }
    pub fn value(&self) -> u32 {
        if self.muted() {
            0
        } else {
            self.envelope.value()
        }
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !self.enabled {
            self.length_counter.load = 0;
        }
    }
}
