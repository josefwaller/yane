use crate::{opcodes, StatusRegister};

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
    /// Memory of the NES
    mem: [u8; 0x10000],
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
            mem: [0x00; 0x10000],
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
            // LDA immediate
            opcodes::LDA_I => self.lda(opcode[1]),
            // LDA zero page
            opcodes::LDA_ZP => self.lda(self.mem[opcode[1] as usize]),
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
    use rand::{random, seq::SliceRandom, thread_rng};

    use assert_hex::assert_eq_hex;

    use super::Nes;

    #[test]
    fn test_init() {
        // Should not throw
        Nes::new();
    }
    #[test]
    fn test_lda_i() {
        all_lda_tests(|nes, v| {
            nes.decode_and_execute(&[0xA9, v]).unwrap();
        });
    }
    #[test]
    fn test_lda_zp() {
        all_lda_tests(|nes, v| {
            let addr = random::<u8>();
            nes.mem[addr as usize] = v;
            nes.decode_and_execute(&[0xA5, addr]).unwrap();
        });
    }

    // Exhaustively test loading all possible 0-255 values into A and then check the flags
    // Used by different tests for immediate, zero page, etc
    fn all_lda_tests<F: FnMut(&mut Nes, u8)>(mut load_into_a: F) {
        test_all_rand_8bit(|v| {
            let mut nes = Nes::new();

            test_all_rand_8bit(|v| {
                let zero = nes.s_r.zero;
                let negative = nes.s_r.negative;
                load_into_a(&mut nes, v);
                assert_eq_hex!(nes.a, v);
                if v == 0 || zero {
                    assert_eq!(nes.s_r.zero, true);
                } else if !zero {
                    assert_eq!(nes.s_r.zero, false);
                }
                if v > 0x7F || negative {
                    assert_eq!(nes.s_r.negative, true);
                } else if !negative {
                    assert_eq!(nes.s_r.negative, false);
                }
            })
        })
    }
    fn test_all_rand_8bit<F: FnMut(u8)>(mut f: F) {
        let mut values: Vec<u8> = (0..255).collect();
        values.shuffle(&mut thread_rng());
        values.iter().for_each(|v| f(*v));
    }
}
