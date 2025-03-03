use serde::{Deserialize, Serialize};

use super::{envelope::Envelope, length_counter::LengthCounter};
use std::fmt::Debug;

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
/// The APU's pulse registers.
/// Outputs a pulse (rectangle) wave.
pub struct PulseRegister {
    /// The index of the duty to use
    pub duty: u32,
    /// The period of the pulse wave
    pub timer: usize,
    // The amount to reload the timer with when it hits 0
    pub timer_reload: usize,
    /// The envelope
    pub envelope: Envelope,
    pub length_counter: LengthCounter,
    // Sweep enabled flag
    pub sweep_enabled: bool,
    /// Sweep period
    pub sweep_period: usize,
    pub sweep_target_period: usize,
    /// Sweep divider
    pub sweep_divider: usize,
    /// Sweep negate flag
    pub sweep_negate: bool,
    /// Sweep shift amount
    pub sweep_shift: usize,
    // Whether the register is enabled
    pub enabled: bool,
    // Sequencer, i.e. the index of the pulse value currently being sent ot the mixer
    pub sequencer: usize,
}
const DUTY_CYCLES: [[u32; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1],
];
impl PulseRegister {
    pub fn muted(&self) -> bool {
        // Conditions for register being disabled
        !self.enabled
            || self.sweep_target_period > 0x7FF
            || self.length_counter.muted()
            || self.timer_reload < 8
    }
    pub fn value(&self) -> u32 {
        if self.muted() || DUTY_CYCLES[self.duty as usize][self.sequencer] == 0 {
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
impl Debug for PulseRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "on={} timer={:3X} target_period={:X} divider={:X} duty={:X} length=[{:?}] sweep=[on={} shift={:X}]",
            self.enabled,
            self.timer_reload,
            self.sweep_target_period,
            self.sweep_divider,
            self.duty,
            self.length_counter,
            self.sweep_enabled,
            self.sweep_shift
        )
    }
}
