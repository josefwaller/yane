use std::fmt::Debug;

use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Default, Serialize, Deserialize)]
pub struct LengthCounter {
    pub halt: bool,
    pub load: usize,
}
impl LengthCounter {
    pub fn muted(&self) -> bool {
        self.load == 0
    }
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
