use crate::{opcodes::*, Cpu};

/// The NES.
pub struct Nes {
    /// CPU of the NES
    cpu: Cpu,
    /// Memory of the NES
    mem: [u8; 0x10000],
}

impl Nes {
    pub fn new() -> Nes {
        Nes {
            cpu: Cpu::new(),
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
    /// nes.decode_and_execute(&[0xAE, 0x34, 0x12]);
    /// // Perform a noop
    /// nes.decode_and_execute(&[0xEA]);
    /// ```
    pub fn decode_and_execute(&mut self, opcode: &[u8]) -> Result<u16, String> {
        match opcode[0] {
            LDA_I => {
                self.cpu.lda(opcode[1]);
                Ok(2)
            }
            LDA_ZP => {
                self.cpu.lda(self.read_zero_page_addr(opcode[1]));
                Ok(2)
            }
            LDA_ZP_X => {
                self.cpu
                    .lda(self.read_zero_page_addr_offset(opcode[1], self.cpu.x));
                Ok(2)
            }
            LDA_ABS => {
                self.cpu.lda(self.read_absolute_addr(&opcode[1..]));
                Ok(3)
            }
            LDA_ABS_X => {
                self.cpu
                    .lda(self.read_absolute_addr_offset(&opcode[1..], self.cpu.x));
                Ok(3)
            }
            LDA_ABS_Y => {
                self.cpu
                    .lda(self.read_absolute_addr_offset(&opcode[1..], self.cpu.y));
                Ok(3)
            }
            LDA_IND_X => {
                self.cpu
                    .lda(self.read_indexed_indirect(opcode[1], self.cpu.x));
                Ok(2)
            }
            LDA_IND_Y => {
                self.cpu
                    .lda(self.read_indirect_indexed(opcode[1], self.cpu.y));
                Ok(2)
            }
            LDX_I => {
                self.cpu.ldx(opcode[1]);
                Ok(2)
            }
            LDX_ZP => {
                self.cpu.ldx(self.read_zero_page_addr(opcode[1]));
                Ok(2)
            }
            LDX_ZP_Y => {
                self.cpu
                    .ldx(self.read_zero_page_addr_offset(opcode[1], self.cpu.y));
                Ok(2)
            }
            LDX_ABS => {
                self.cpu.ldx(self.read_absolute_addr(&opcode[1..]));
                Ok(3)
            }
            LDX_ABS_Y => {
                self.cpu
                    .ldx(self.read_absolute_addr_offset(&opcode[1..], self.cpu.y));
                Ok(3)
            }
            LDY_I => {
                self.cpu.ldy(opcode[1]);
                Ok(2)
            }
            LDY_ZP => {
                self.cpu.ldy(self.read_zero_page_addr(opcode[1]));
                Ok(2)
            }
            LDY_ZP_X => {
                self.cpu
                    .ldy(self.read_zero_page_addr_offset(opcode[1], self.cpu.x));
                Ok(2)
            }
            LDY_ABS => {
                self.cpu.ldy(self.read_absolute_addr(&opcode[1..]));
                Ok(3)
            }
            LDY_ABS_X => {
                self.cpu
                    .ldy(self.read_absolute_addr_offset(&opcode[1..], self.cpu.x));
                Ok(3)
            }
            ADC_I => {
                self.cpu.adc(opcode[1]);
                Ok(2)
            }
            ADC_ZP => {
                self.cpu.adc(self.read_zero_page_addr(opcode[1]));
                Ok(2)
            }
            ADC_ZP_X => {
                self.cpu
                    .adc(self.read_zero_page_addr_offset(opcode[1], self.cpu.x));
                Ok(2)
            }
            ADC_ABS => {
                self.cpu.adc(self.read_absolute_addr(&opcode[1..]));
                Ok(3)
            }
            ADC_ABS_X => {
                self.cpu
                    .adc(self.read_absolute_addr_offset(&opcode[1..], self.cpu.x));
                Ok(3)
            }
            ADC_ABS_Y => {
                self.cpu
                    .adc(self.read_absolute_addr_offset(&opcode[1..], self.cpu.y));
                Ok(3)
            }
            ADC_IND_X => {
                self.cpu
                    .adc(self.read_indexed_indirect(opcode[1], self.cpu.x));
                Ok(2)
            }
            ADC_IND_Y => {
                self.cpu
                    .adc(self.read_indirect_indexed(opcode[1], self.cpu.y));
                Ok(2)
            }
            AND_I => {
                self.cpu.and(opcode[1]);
                Ok(2)
            }
            AND_ZP => {
                self.cpu.and(self.read_zero_page_addr(opcode[1]));
                Ok(2)
            }
            AND_ZP_X => {
                self.cpu
                    .and(self.read_zero_page_addr_offset(opcode[1], self.cpu.x));
                Ok(2)
            }
            AND_ABS => {
                self.cpu.and(self.read_absolute_addr(&opcode[1..]));
                Ok(3)
            }
            AND_ABS_X => {
                self.cpu
                    .and(self.read_absolute_addr_offset(&opcode[1..], self.cpu.x));
                Ok(3)
            }
            AND_ABS_Y => {
                self.cpu
                    .and(self.read_absolute_addr_offset(&opcode[1..], self.cpu.y));
                Ok(3)
            }
            AND_IND_X => {
                self.cpu
                    .and(self.read_indexed_indirect(opcode[1], self.cpu.x));
                Ok(2)
            }
            AND_IND_Y => {
                self.cpu
                    .and(self.read_indirect_indexed(opcode[1], self.cpu.y));
                Ok(2)
            }
            _ => {
                return Err(format!(
                    "Unknown opcode '{:#04X}' at location '{:#04X}'",
                    opcode[0], self.cpu.p_c
                ))
            }
        }
    }

    // Read using zero paging addressing
    fn read_zero_page_addr(&self, addr: u8) -> u8 {
        self.mem[addr as usize]
    }
    // Read using zero page addressing with an offset
    fn read_zero_page_addr_offset(&self, addr: u8, offset: u8) -> u8 {
        self.mem[addr.wrapping_add(offset) as usize]
    }
    // Read using absolute addressing
    fn read_absolute_addr(&self, addr: &[u8]) -> u8 {
        self.mem[addr[0] as usize + ((addr[1] as usize) << 8)]
    }
    // Read using absllute addressing with an offset
    fn read_absolute_addr_offset(&self, addr: &[u8], offset: u8) -> u8 {
        self.mem[(addr[0] as u16 + ((addr[1] as u16) << 8)).wrapping_add(offset as u16) as usize]
    }
    // Read using indexed indirect addressing with an offset.
    // X is added to the value in the opcode and used to read a pointer from memory.
    fn read_indexed_indirect(&self, addr: u8, offset: u8) -> u8 {
        let first_addr = addr.wrapping_add(offset) as usize;
        let second_addr = &self.mem[first_addr..(first_addr + 2)];
        return self.read_absolute_addr(&second_addr);
    }
    // Read using indirect indexed addressing.
    // A pointer is read from the memory using the value in the opcode, and then Y is added to it.
    fn read_indirect_indexed(&self, addr: u8, offset: u8) -> u8 {
        let first_addr = addr as usize;
        let second_addr = (self.mem[first_addr] as u16 + ((self.mem[first_addr + 1] as u16) << 8))
            .wrapping_add(offset as u16);
        return self.mem[second_addr as usize];
    }
}

#[cfg(test)]
mod tests {
    use rand::random;

    use super::Nes;
    use crate::opcodes::*;
    use assert_hex::assert_eq_hex;

    #[test]
    fn test_init() {
        // Should not throw
        Nes::new();
    }
    // Macros used to generate basic test cases
    macro_rules! test_immediate {
        ($opcode: ident) => {
            #[test]
            fn test_immediate() {
                run_test(|nes, v| {
                    assert_eq!(nes.decode_and_execute(&[$opcode, v]), Ok(2));
                })
            }
        };
    }
    macro_rules! test_zp {
        ($opcode: ident) => {
            #[test]
            fn test_zp() {
                run_test(|nes, v| {
                    let addr = set_addr_zp(nes, v);
                    assert_eq!(nes.decode_and_execute(&[$opcode, addr]), Ok(2));
                })
            }
        };
    }
    macro_rules! test_zp_offset {
        ($opcode: ident, $off_reg: ident) => {
            run_test(|nes, v| {
                let addr = random::<u8>();
                nes.cpu.$off_reg = random::<u8>();
                nes.mem[addr.wrapping_add(nes.cpu.$off_reg) as usize] = v;
                assert_eq!(nes.decode_and_execute(&[$opcode, addr]), Ok(2));
            })
        };
    }
    macro_rules! test_zp_x {
        ($opcode: ident) => {
            #[test]
            fn test_zp_x() {
                test_zp_offset!($opcode, x);
            }
        };
    }
    macro_rules! test_zp_y {
        ($opcode: ident) => {
            #[test]
            fn test_zp_y() {
                test_zp_offset!($opcode, y);
            }
        };
    }
    macro_rules! test_absolute {
        ($opcode: ident) => {
            #[test]
            fn test_absolute() {
                run_test(|nes, v| {
                    let addr = set_addr_abs(nes, v);
                    assert_eq!(nes.decode_and_execute(&[$opcode, addr[0], addr[1]]), Ok(3));
                })
            }
        };
    }
    macro_rules! test_absolute_offset {
        ($opcode: ident, $off_reg: ident) => {
            run_test(|nes, v| {
                let addr = random::<u16>();
                nes.cpu.$off_reg = random::<u8>();
                nes.mem[addr.wrapping_add(nes.cpu.$off_reg as u16) as usize] = v;
                assert_eq!(
                    nes.decode_and_execute(&[$opcode, first_byte(addr), second_byte(addr)]),
                    Ok(3)
                );
            })
        };
    }
    macro_rules! test_absolute_x {
        ($opcode: ident) => {
            #[test]
            fn test_absolute_x() {
                test_absolute_offset!($opcode, x);
            }
        };
    }
    macro_rules! test_absolute_y {
        ($opcode: ident) => {
            #[test]
            fn test_absolute_y() {
                test_absolute_offset!($opcode, y);
            }
        };
    }
    macro_rules! test_indexed_indirect {
        ($opcode: ident) => {
            #[test]
            fn test_indexed_indirect() {
                run_test(|nes, v| {
                    let addr = random::<u16>();
                    nes.mem[addr as usize] = v;
                    let mut operand = random::<u8>();
                    nes.cpu.x = random::<u8>();
                    let mut second_addr = operand.wrapping_add(nes.cpu.x);
                    // Avoid collisions
                    if second_addr as u16 == addr || second_addr as u16 == addr.wrapping_sub(1) {
                        second_addr = second_addr.wrapping_add(2);
                        operand = operand.wrapping_add(2);
                    }
                    nes.mem[second_addr as usize] = first_byte(addr);
                    nes.mem[second_addr as usize + 1] = second_byte(addr);
                    assert_eq!(nes.decode_and_execute(&[$opcode, operand]), Ok(2));
                });
            }
        };
    }
    macro_rules! test_indirect_indexed {
        ($opcode: ident) => {
            #[test]
            fn test_indirect_indexed() {
                run_test(|nes, v| {
                    let addr = random::<u16>();
                    nes.cpu.y = random::<u8>();
                    nes.mem[addr.wrapping_add(nes.cpu.y as u16) as usize] = v;
                    let mut operand = random::<u8>();
                    if operand as u16 == addr || operand as u16 == addr.wrapping_sub(1) {
                        operand = operand.wrapping_add(2);
                    }
                    nes.mem[operand as usize] = first_byte(addr);
                    nes.mem[operand as usize + 1] = second_byte(addr);
                    assert_eq!(nes.decode_and_execute(&[$opcode, operand]), Ok(2));
                })
            }
        };
    }
    mod lda {
        use super::*;
        fn run_test<F: Fn(&mut Nes, u8)>(test: F) {
            let mut nes = Nes::new();
            let v = random::<u8>();
            test(&mut nes, v);
            assert_eq_hex!(nes.cpu.a, v);
        }
        test_immediate!(LDA_I);
        test_zp!(LDA_ZP);
        test_zp_x!(LDA_ZP_X);
        test_absolute!(LDA_ABS);
        test_absolute_x!(LDA_ABS_X);
        test_absolute_y!(LDA_ABS_Y);
        test_indexed_indirect!(LDA_IND_X);
        test_indirect_indexed!(LDA_IND_Y);
    }
    mod ldx {
        use super::*;
        fn run_test<F: Fn(&mut Nes, u8)>(test: F) {
            let mut nes = Nes::new();
            let v = random::<u8>();
            test(&mut nes, v);
            assert_eq_hex!(nes.cpu.x, v);
        }
        test_immediate!(LDX_I);
        test_zp!(LDX_ZP);
        test_zp_y!(LDX_ZP_Y);
        test_absolute!(LDX_ABS);
        test_absolute_y!(LDX_ABS_Y);
    }
    mod ldy {
        use super::*;
        fn run_test<F: Fn(&mut Nes, u8)>(test: F) {
            let mut nes = Nes::new();
            let v = random::<u8>();
            test(&mut nes, v);
            assert_eq_hex!(nes.cpu.y, v);
        }
        test_immediate!(LDY_I);
        test_zp!(LDY_ZP);
        test_zp_x!(LDY_ZP_X);
        test_absolute!(LDY_ABS);
        test_absolute_x!(LDY_ABS_X);
    }
    mod adc {
        use super::*;
        fn run_test<F: Fn(&mut Nes, u8)>(test: F) {
            let mut nes = Nes::new();
            nes.cpu.a = random::<u8>();
            let v = random::<u8>();
            let exp = nes.cpu.a.wrapping_add(v);
            test(&mut nes, v);
            assert_eq_hex!(nes.cpu.a, exp);
        }
        test_immediate!(ADC_I);
        test_zp!(ADC_ZP);
        test_zp_x!(ADC_ZP_X);
        test_absolute!(ADC_ABS);
        test_absolute_x!(ADC_ABS_X);
        test_absolute_y!(ADC_ABS_Y);
        test_indexed_indirect!(ADC_IND_X);
        test_indirect_indexed!(ADC_IND_Y);
    }
    mod and {
        use super::*;
        fn run_test<F: Fn(&mut Nes, u8)>(f: F) {
            let mut nes = Nes::new();
            let v_one = random::<u8>();
            nes.cpu.a = v_one;
            let v_two = random::<u8>();
            f(&mut nes, v_two);
            assert_eq_hex!(nes.cpu.a, v_one & v_two);
        }
        test_immediate!(AND_I);
        test_zp!(AND_ZP);
        test_zp_x!(AND_ZP_X);
        test_absolute!(AND_ABS);
        test_absolute_x!(AND_ABS_X);
        test_absolute_y!(AND_ABS_Y);
        test_indexed_indirect!(AND_IND_X);
        test_indirect_indexed!(AND_IND_Y);
    }

    // Utility functions to get some addresses in memory set to the value given
    fn set_addr_zp(nes: &mut Nes, value: u8) -> u8 {
        let addr = random::<u8>();
        nes.mem[addr as usize] = value;
        return addr;
    }
    fn set_addr_abs(nes: &mut Nes, value: u8) -> [u8; 2] {
        let addr = random::<u16>();
        nes.mem[addr as usize] = value;
        return [(addr & 0xFF) as u8, (addr >> 8) as u8];
    }
    fn first_byte(addr: u16) -> u8 {
        (addr & 0xFF) as u8
    }
    fn second_byte(addr: u16) -> u8 {
        (addr >> 8) as u8
    }
}
