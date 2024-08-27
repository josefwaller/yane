use crate::{opcodes::*, StatusRegister};

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
            LDA_I => self.lda(opcode[1]),
            LDA_ZP => self.lda(self.mem[opcode[1] as usize]),
            LDA_ZP_X => self.lda(self.mem[opcode[1].wrapping_add(self.x) as usize]),
            LDA_ABS => self.lda(self.read_absolute_addr(&opcode[1..])),
            LDA_ABS_X => self.lda(self.read_absolute_addr_offset(&opcode[1..], self.x)),
            LDA_ABS_Y => self.lda(self.read_absolute_addr_offset(&opcode[1..], self.y)),
            LDA_IND_X => self.lda(self.read_indirect_addr_offset(&opcode[1..], self.x)),
            LDA_IND_Y => self.lda(self.read_indirect_addr_offset(&opcode[1..], self.y)),
            _ => {
                return Err(format!(
                    "Unknown opcode '{:#04X}' at location '{:#04X}'",
                    opcode[0], self.p_c
                ))
            }
        };
        Ok(2)
    }

    // Opcode functions
    fn lda(&mut self, value: u8) {
        self.a = value;
        if self.a == 0 {
            self.s_r.zero = true;
        }
        if self.a & 0x80 != 0 {
            self.s_r.negative = true;
        }
    }

    // Read using absolute addressing
    fn read_absolute_addr(&self, addr: &[u8]) -> u8 {
        self.mem[addr[0] as usize + ((addr[1] as usize) << 8)]
    }
    // Read using absllute addressing with an offset
    fn read_absolute_addr_offset(&self, addr: &[u8], offset: u8) -> u8 {
        self.mem[(addr[0] as u16 + ((addr[1] as u16) << 8)).wrapping_add(offset as u16) as usize]
    }
    // Read using indirect addressing with an offset
    fn read_indirect_addr_offset(&self, addr: &[u8], offset: u8) -> u8 {
        let first_addr = addr[0].wrapping_add(offset) as usize;
        let second_addr = &self.mem[first_addr..(first_addr + 2)];
        return self.read_absolute_addr(&second_addr);
    }
}

#[cfg(test)]
mod tests {
    use rand::{random, seq::SliceRandom, thread_rng};

    use super::Nes;
    use crate::opcodes::*;
    use assert_hex::assert_eq_hex;

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
    #[test]
    fn test_lda_zp_x() {
        all_lda_tests(|nes, v| {
            let addr = random::<u8>();
            nes.x = random::<u8>();
            nes.mem[(addr.wrapping_add(nes.x)) as usize] = v;
            nes.decode_and_execute(&[LDA_ZP_X, addr]).unwrap();
        })
    }
    #[test]
    fn test_lda_abs() {
        all_lda_tests(|nes, v| {
            let addr = random::<u16>();
            nes.mem[addr as usize] = v;
            nes.decode_and_execute(&[LDA_ABS, (addr & 0xFF) as u8, (addr >> 8) as u8])
                .unwrap();
        })
    }
    #[test]
    fn test_lda_abs_x() {
        all_lda_tests(|nes, v| {
            let addr = random::<u16>();
            nes.x = random::<u8>();
            nes.mem[addr.wrapping_add(nes.x as u16) as usize] = v;
            nes.decode_and_execute(&[LDA_ABS_X, first_byte(addr), second_byte(addr)])
                .unwrap();
        })
    }
    #[test]
    fn test_lda_abs_y() {
        all_lda_tests(|nes, v| {
            let addr = random::<u16>();
            nes.y = random::<u8>();
            nes.mem[addr.wrapping_add(nes.y as u16) as usize] = v;
            nes.decode_and_execute(&[LDA_ABS_Y, first_byte(addr), second_byte(addr)])
                .unwrap();
        });
    }
    #[test]
    fn test_lda_ind_x() {
        all_lda_tests(|nes, v| {
            let addr = random::<u16>();
            nes.mem[addr as usize] = v;
            let mut operand = random::<u8>();
            nes.x = random::<u8>();
            let mut second_addr = operand.wrapping_add(nes.x);
            // Avoid collisions
            if second_addr as u16 == addr || second_addr as u16 == addr.wrapping_sub(1) {
                second_addr = second_addr.wrapping_add(2);
                operand = operand.wrapping_add(2);
            }
            nes.mem[second_addr as usize] = first_byte(addr);
            nes.mem[second_addr as usize + 1] = second_byte(addr);
            nes.decode_and_execute(&[LDA_IND_X, operand]).unwrap();
        });
    }
    #[test]
    fn test_lda_ind_y() {
        all_lda_tests(|nes, v| {
            let addr = random::<u16>();
            nes.mem[addr as usize] = v;
            let mut operand = random::<u8>();
            nes.y = random::<u8>();
            let mut second_addr = operand.wrapping_add(nes.y);
            // Avoid collisions
            if second_addr as u16 == addr || second_addr as u16 == addr.wrapping_sub(1) {
                second_addr = second_addr.wrapping_add(2);
                operand = operand.wrapping_add(2);
            }
            nes.mem[second_addr as usize] = first_byte(addr);
            nes.mem[second_addr as usize + 1] = second_byte(addr);
            nes.decode_and_execute(&[LDA_IND_Y, operand]).unwrap();
        });
    }

    fn first_byte(addr: u16) -> u8 {
        (addr & 0xFF) as u8
    }
    fn second_byte(addr: u16) -> u8 {
        (addr >> 8) as u8
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
