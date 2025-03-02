use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize)]
pub struct Envelope {
    /// Constant volume flag
    pub constant: bool,
    /// Volume value (either the volume or the volume reload value)
    pub volume: usize,
    /// Current value of the volume divider
    pub divider: usize,
    /// Current value of the volume decay
    pub decay: usize,
}
impl Envelope {
    pub fn clock(&mut self, restart: bool) {
        // Clock volume divider
        if self.divider == 0 {
            self.divider = self.volume;
            // Clock volume decay
            if self.decay == 0 {
                // Reset if loop flag is set
                if restart {
                    self.decay = 0xF;
                }
            } else {
                self.decay -= 1;
            }
        } else {
            self.divider -= 1;
        }
    }
    pub fn value(&self) -> u32 {
        if self.constant {
            self.volume as u32
        } else {
            self.decay as u32
        }
    }
}
