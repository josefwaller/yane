use serde::{Deserialize, Serialize};

use super::length_counter::LengthCounter;
use std::fmt::Debug;

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
pub struct TriangleRegister {
    pub length_counter: LengthCounter,
    pub linear_counter: usize,
    // Linear counter reload value
    pub linear_counter_reload: usize,
    pub reload_flag: bool,
    pub timer_reload: u32,
    pub enabled: bool,
    pub sequencer: u32,
    pub timer: u32,
}
impl Debug for TriangleRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "on={} timer={:3X} length=[{:?}] linear={:X}",
            self.enabled, self.timer_reload, self.length_counter, self.linear_counter
        )
    }
}
impl TriangleRegister {
    pub fn muted(&self) -> bool {
        !self.enabled
            || self.length_counter.muted()
            || self.timer_reload < 2
            || self.linear_counter == 0
    }
    pub fn value(&self) -> u32 {
        if self.sequencer <= 15 {
            self.sequencer
        } else {
            31 - self.sequencer
        }
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if self.enabled {
            self.timer = 0;
        } else {
            self.length_counter.load = 0;
        }
    }
}
