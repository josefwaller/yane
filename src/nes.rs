use rand::thread_rng;

use crate::StatusRegister;

/// The NES.
pub struct Nes {
    /// Accumulator
    a: u8,
    /// X index registers
    x: u8,
    /// Y index registers
    y: u8,
    /// Program counter
    p_c: u16,
    /// Stack pointer
    s_p: u8,
    /// Status register
    s_r: StatusRegister,
}

impl Nes {
    pub fn new() -> Nes {
        Nes {
            a: 0x0,
            x: 0x0,
            y: 0x0,
            p_c: 0x00,
            s_p: 0x0,
            s_r: StatusRegister::new(),
        }
    }

    /// Decode and then execute first byte of `opcode` as an NES opcode.
    /// Returns how much the program counter should be incremented by, i.e. how many bytes were used by the opcode.
    /// Does not change the program counter.
    ///
    /// # Examples
    /// ```
    /// use yane::Nes;
    /// let mut nes = Nes::new();
    /// // Load 0x18 into A
    /// nes.decode_and_execute(&[0xA9, 0x18]);
    /// // Load the memory at 0x1234 into A
    /// nes.decode_and_execute(&[0xAE, 0x12, 0x34]);
    /// // Perform a noop
    /// nes.decode_and_execute(&[0xEA]);
    /// ```
    pub fn decode_and_execute(&mut self, opcode: &[u8]) -> Result<u16, String> {
        match opcode[0] {
            0xA9 => self.lda(opcode[1]),
            _ => {
                return Err(format!(
                    "Unknown opcode '{:#04X}' at location '{:#04X}'",
                    opcode[0], self.p_c
                ))
            }
        };
        Ok(2)
    }

    fn lda(&mut self, value: u8) {
        self.a = value;
        if self.a == 0 {
            self.s_r.zero = true;
        }
        if self.a & 0x80 != 0 {
            self.s_r.negative = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::{seq::SliceRandom, thread_rng};

    use assert_hex::assert_eq_hex;

    use super::Nes;

    #[test]
    fn test_init() {
        // Should not throw
        Nes::new();
    }
    #[test]
    fn test_lda_i() {
        let mut values: Vec<u8> = (0..255).collect();
        let mut nes = Nes::new();
        let mut has_set_zero = false;
        let mut has_set_negative = false;

        values.shuffle(&mut thread_rng());
        values.iter().for_each(|v| {
            nes.decode_and_execute(&[0xA9, *v]).unwrap();
            assert_eq_hex!(*v, nes.a);
            if *v == 0 || has_set_zero {
                assert_eq!(nes.s_r.zero, true);
                has_set_zero = true;
            } else if !has_set_zero {
                assert_eq!(nes.s_r.zero, false);
            }
            if *v > 0x7F || has_set_negative {
                assert_eq!(nes.s_r.negative, true);
                has_set_negative = true;
            } else if !has_set_negative {
                assert_eq!(nes.s_r.negative, false);
            }
        });
    }
}
