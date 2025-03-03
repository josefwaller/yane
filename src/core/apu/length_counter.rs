use std::fmt::Debug;

use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Default, Serialize, Deserialize)]
/// A length counter.
/// Simple divider that mutes an APU register when it hits 0.
pub struct LengthCounter {
    /// The halt flag, pauses the counter when true
    pub halt: bool,
    /// The current value
    pub load: usize,
}
impl LengthCounter {
    /// Return `true` if the counter should be muting the register
    pub fn muted(&self) -> bool {
        self.load == 0
    }
    /// Clock the length counter
    pub fn clock(&mut self) {
        if !self.halt && self.load > 0 {
            self.load -= 1;
        }
    }
}
impl Debug for LengthCounter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "halt={} load={:X}", self.halt, self.load)
    }
}
