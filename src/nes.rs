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
    /// Returns `(bytes, cycles`, where `bytes` is how much the program counter should be incremented by,
    /// i.e. how many bytes were used by the opcode, and `cycles` is how many cycles the operation needed.
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
    pub fn decode_and_execute(&mut self, opcode: &[u8]) -> Result<(u16, i64), String> {
        match opcode[0] {
            LDA_I => {
                self.cpu.lda(opcode[1]);
                Ok((2, 2))
            }
            LDA_ZP => {
                self.cpu.lda(self.read_zero_page_addr(opcode[1]));
                Ok((2, 3))
            }
            LDA_ZP_X => {
                self.cpu
                    .lda(self.read_zero_page_addr_offset(opcode[1], self.cpu.x));
                Ok((2, 4))
            }
            LDA_ABS => {
                self.cpu.lda(self.read_absolute_addr(&opcode[1..]));
                Ok((3, 4))
            }
            LDA_ABS_X => {
                self.cpu
                    .lda(self.read_absolute_addr_offset(&opcode[1..], self.cpu.x));
                Ok((
                    3,
                    4 + if Nes::page_crossed_abs(&opcode[1..], self.cpu.x) {
                        1
                    } else {
                        0
                    },
                ))
            }
            LDA_ABS_Y => {
                self.cpu
                    .lda(self.read_absolute_addr_offset(&opcode[1..], self.cpu.y));
                Ok((
                    3,
                    4 + if Nes::page_crossed_abs(&opcode[1..], self.cpu.y) {
                        1
                    } else {
                        0
                    },
                ))
            }
            LDA_IDX_IND => {
                self.cpu
                    .lda(self.read_indexed_indirect(opcode[1], self.cpu.x));
                Ok((2, 6))
            }
            LDA_IND_IDX => {
                self.cpu
                    .lda(self.read_indirect_indexed(opcode[1], self.cpu.y));
                Ok((
                    2,
                    5 + if self.page_crossed_ind_idx(&opcode[1..], self.cpu.y) {
                        1
                    } else {
                        0
                    },
                ))
            }
            LDX_I => {
                self.cpu.ldx(opcode[1]);
                Ok((2, 2))
            }
            LDX_ZP => {
                self.cpu.ldx(self.read_zero_page_addr(opcode[1]));
                Ok((2, 3))
            }
            LDX_ZP_Y => {
                self.cpu
                    .ldx(self.read_zero_page_addr_offset(opcode[1], self.cpu.y));
                Ok((2, 4))
            }
            LDX_ABS => {
                self.cpu.ldx(self.read_absolute_addr(&opcode[1..]));
                Ok((3, 4))
            }
            LDX_ABS_Y => {
                self.cpu
                    .ldx(self.read_absolute_addr_offset(&opcode[1..], self.cpu.y));
                Ok((
                    3,
                    4 + if Nes::page_crossed_abs(&opcode[1..], self.cpu.y) {
                        1
                    } else {
                        0
                    },
                ))
            }
            LDY_I => {
                self.cpu.ldy(opcode[1]);
                Ok((2, 2))
            }
            LDY_ZP => {
                self.cpu.ldy(self.read_zero_page_addr(opcode[1]));
                Ok((2, 3))
            }
            LDY_ZP_X => {
                self.cpu
                    .ldy(self.read_zero_page_addr_offset(opcode[1], self.cpu.x));
                Ok((2, 4))
            }
            LDY_ABS => {
                self.cpu.ldy(self.read_absolute_addr(&opcode[1..]));
                Ok((3, 4))
            }
            LDY_ABS_X => {
                self.cpu
                    .ldy(self.read_absolute_addr_offset(&opcode[1..], self.cpu.x));
                Ok((
                    3,
                    4 + if Nes::page_crossed_abs(&opcode[1..], self.cpu.x) {
                        1
                    } else {
                        0
                    },
                ))
            }
            ADC_I => {
                self.cpu.adc(opcode[1]);
                Ok((2, 2))
            }
            ADC_ZP => {
                self.cpu.adc(self.read_zero_page_addr(opcode[1]));
                Ok((2, 3))
            }
            ADC_ZP_X => {
                self.cpu
                    .adc(self.read_zero_page_addr_offset(opcode[1], self.cpu.x));
                Ok((2, 4))
            }
            ADC_ABS => {
                self.cpu.adc(self.read_absolute_addr(&opcode[1..]));
                Ok((3, 4))
            }
            ADC_ABS_X => {
                self.cpu
                    .adc(self.read_absolute_addr_offset(&opcode[1..], self.cpu.x));
                Ok((
                    3,
                    4 + if Nes::page_crossed_abs(&opcode[1..], self.cpu.x) {
                        1
                    } else {
                        0
                    },
                ))
            }
            ADC_ABS_Y => {
                self.cpu
                    .adc(self.read_absolute_addr_offset(&opcode[1..], self.cpu.y));
                Ok((
                    3,
                    4 + if Nes::page_crossed_abs(&opcode[1..], self.cpu.y) {
                        1
                    } else {
                        0
                    },
                ))
            }
            ADC_IDX_IND => {
                self.cpu
                    .adc(self.read_indexed_indirect(opcode[1], self.cpu.x));
                Ok((2, 6))
            }
            ADC_IND_IDX => {
                self.cpu
                    .adc(self.read_indirect_indexed(opcode[1], self.cpu.y));
                Ok((
                    2,
                    5 + if self.page_crossed_ind_idx(&opcode[1..], self.cpu.y) {
                        1
                    } else {
                        0
                    },
                ))
            }
            AND_I => {
                self.cpu.and(opcode[1]);
                Ok((2, 2))
            }
            AND_ZP => {
                self.cpu.and(self.read_zero_page_addr(opcode[1]));
                Ok((2, 3))
            }
            AND_ZP_X => {
                self.cpu
                    .and(self.read_zero_page_addr_offset(opcode[1], self.cpu.x));
                Ok((2, 4))
            }
            AND_ABS => {
                self.cpu.and(self.read_absolute_addr(&opcode[1..]));
                Ok((3, 4))
            }
            AND_ABS_X => {
                self.cpu
                    .and(self.read_absolute_addr_offset(&opcode[1..], self.cpu.x));
                Ok((
                    3,
                    4 + if Nes::page_crossed_abs(&opcode[1..], self.cpu.x) {
                        1
                    } else {
                        0
                    },
                ))
            }
            AND_ABS_Y => {
                self.cpu
                    .and(self.read_absolute_addr_offset(&opcode[1..], self.cpu.y));
                Ok((
                    3,
                    4 + if Nes::page_crossed_abs(&opcode[1..], self.cpu.y) {
                        1
                    } else {
                        0
                    },
                ))
            }
            AND_IDX_IND => {
                self.cpu
                    .and(self.read_indexed_indirect(opcode[1], self.cpu.x));
                Ok((2, 6))
            }
            AND_IND_IDX => {
                self.cpu
                    .and(self.read_indirect_indexed(opcode[1], self.cpu.y));
                Ok((
                    2,
                    5 + if self.page_crossed_ind_idx(&opcode[1..], self.cpu.y) {
                        1
                    } else {
                        0
                    },
                ))
            }
            ASL_A => {
                self.cpu.a = self.cpu.asl(self.cpu.a);
                Ok((1, 2))
            }
            ASL_ZP => {
                let v = self.cpu.asl(self.read_zero_page_addr(opcode[1]));
                self.set_zero_page_addr(opcode[1], v);
                Ok((2, 5))
            }
            ASL_ZP_X => {
                let v = self
                    .cpu
                    .asl(self.read_zero_page_addr_offset(opcode[1], self.cpu.x));
                self.set_zero_page_addr_offset(opcode[1], self.cpu.x, v);
                Ok((2, 6))
            }
            ASL_ABS => {
                let v = self.cpu.asl(self.read_absolute_addr(&opcode[1..]));
                self.write_absolute_addr(&opcode[1..], v);
                Ok((3, 6))
            }
            ASL_ABS_X => {
                let v = self
                    .cpu
                    .asl(self.read_absolute_addr_offset(&opcode[1..], self.cpu.x));
                self.write_absolute_addr_offset(&opcode[1..], self.cpu.x, v);
                Ok((3, 7))
            }
            BCS => Ok((2, self.cpu.branch_if(self.cpu.s_r.c, opcode[1]))),
            BCC => Ok((2, self.cpu.branch_if(!self.cpu.s_r.c, opcode[1]))),
            BEQ => Ok((2, self.cpu.branch_if(self.cpu.s_r.z, opcode[1]))),
            BNE => Ok((2, self.cpu.branch_if(!self.cpu.s_r.z, opcode[1]))),
            BMI => Ok((2, self.cpu.branch_if(self.cpu.s_r.n, opcode[1]))),
            BPL => Ok((2, self.cpu.branch_if(!self.cpu.s_r.n, opcode[1]))),
            BVS => Ok((2, self.cpu.branch_if(self.cpu.s_r.v, opcode[1]))),
            BVC => Ok((2, self.cpu.branch_if(!self.cpu.s_r.v, opcode[1]))),
            BIT_ZP => {
                self.cpu.bit(self.read_zero_page_addr(opcode[1]));
                Ok((2, 3))
            }
            BIT_ABS => {
                self.cpu.bit(self.read_absolute_addr(&opcode[1..]));
                Ok((3, 4))
            }
            _ => {
                return Err(format!(
                    "Unknown opcode '{:#04X}' at location '{:#04X}'",
                    opcode[0], self.cpu.p_c
                ))
            }
        }
    }

    // Zero page addressing
    fn read_zero_page_addr(&self, addr: u8) -> u8 {
        self.mem[addr as usize]
    }
    fn set_zero_page_addr(&mut self, addr: u8, val: u8) {
        self.mem[addr as usize] = val;
    }
    // Read using zero page addressing with an offset
    fn read_zero_page_addr_offset(&self, addr: u8, offset: u8) -> u8 {
        self.mem[addr.wrapping_add(offset) as usize]
    }
    fn set_zero_page_addr_offset(&mut self, addr: u8, offset: u8, value: u8) {
        self.mem[addr.wrapping_add(offset) as usize] = value;
    }
    // Absolute addressing
    fn get_absolute_addr_offset(addr: &[u8], offset: u8) -> usize {
        (addr[0] as u16 + ((addr[1] as u16) << 8)).wrapping_add(offset as u16) as usize
    }
    fn get_absolute_addr(addr: &[u8]) -> usize {
        Nes::get_absolute_addr_offset(addr, 0)
    }
    fn read_absolute_addr(&self, addr: &[u8]) -> u8 {
        self.mem[Nes::get_absolute_addr(addr)]
    }
    fn write_absolute_addr(&mut self, addr: &[u8], value: u8) {
        self.mem[Nes::get_absolute_addr(addr)] = value;
    }
    // Read using absllute addressing with an offset
    fn read_absolute_addr_offset(&self, addr: &[u8], offset: u8) -> u8 {
        self.mem[Nes::get_absolute_addr_offset(addr, offset)]
    }
    fn write_absolute_addr_offset(&mut self, addr: &[u8], offset: u8, value: u8) {
        self.mem[Nes::get_absolute_addr_offset(addr, offset)] = value;
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
    // Return true if a page is crossed by an operation using the absolute address and offset given
    fn page_crossed_abs(addr: &[u8], offset: u8) -> bool {
        255 - addr[1] >= offset
    }
    // Return true if a page is crossed by the indirect indexed address and offset given
    fn page_crossed_ind_idx(&self, addr: &[u8], offset: u8) -> bool {
        255 - self.read_zero_page_addr(addr[0]) >= offset
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
                    assert_eq!(nes.decode_and_execute(&[$opcode, v]), Ok((2, 2)));
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
                    assert_eq!(nes.decode_and_execute(&[$opcode, addr[0]]), Ok((2, 3)));
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
                assert_eq!(nes.decode_and_execute(&[$opcode, addr]), Ok((2, 4)));
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
                    assert_eq!(
                        nes.decode_and_execute(&[$opcode, addr[0], addr[1]]),
                        Ok((3, 4))
                    );
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
                nes.decode_and_execute(&[$opcode, first_byte(addr), second_byte(addr)])
                    .unwrap();
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
                    assert_eq!(nes.decode_and_execute(&[$opcode, operand]), Ok((2, 6)));
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
                    nes.decode_and_execute(&[$opcode, operand]).unwrap();
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
        test_indexed_indirect!(LDA_IDX_IND);
        test_indirect_indexed!(LDA_IND_IDX);
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
        test_indexed_indirect!(ADC_IDX_IND);
        test_indirect_indexed!(ADC_IND_IDX);
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
        test_indexed_indirect!(AND_IDX_IND);
        test_indirect_indexed!(AND_IND_IDX);
    }
    mod asl {
        use super::*;
        use test_case::test_case;

        macro_rules! check_flags {
            ($nes: ident, $zero: expr, $negative: expr, $carry: expr) => {
                assert_eq!(
                    $nes.cpu.s_r.z,
                    $zero,
                    "zero should be {}",
                    if $zero { "set" } else { "unset" }
                );
                assert_eq!(
                    $nes.cpu.s_r.n,
                    $negative,
                    "negative should be {}",
                    if $negative { "set " } else { "unset" }
                );
                assert_eq!(
                    $nes.cpu.s_r.c,
                    $carry,
                    "carry should be {}",
                    if $carry { "set " } else { "unset" }
                );
            };
        }

        #[test_case(0x18, 0x30, false, false, false ; "happy case")]
        #[test_case(0x45, 0x8A, false, true, false ; "negative is set")]
        #[test_case(0x88, 0x10, false, false, true ; "carry is set")]
        #[test_case(0x80, 0x00, true, false, true; "zero is set")]
        fn test_accumulator(value: u8, shifted: u8, zero: bool, negative: bool, carry: bool) {
            let mut nes = Nes::new();
            nes.cpu.a = value;
            assert_eq!(nes.decode_and_execute(&[ASL_A]), Ok((1, 2)));
            assert_eq_hex!(nes.cpu.a, shifted, "shifted is correct");
            check_flags!(nes, zero, negative, carry);
        }
        #[test_case(0x01, 0x02, false, false, false ; "happy case")]
        #[test_case(0x44, 0x88, false, true, false ; "negative is set")]
        #[test_case(0x00, 0x00, true, false, false ; "zero is set")]
        #[test_case(0x8A, 0x14, false, false, true ; "carry is set")]
        fn test_zp(value: u8, shifted: u8, zero: bool, negative: bool, carry: bool) {
            let mut nes = Nes::new();
            let addr = set_addr_zp(&mut nes, value);
            assert_eq!(nes.decode_and_execute(&[ASL_ZP, addr[0]]), Ok((2, 5)));
            assert_eq_hex!(nes.mem[addr[0] as usize], shifted);
            check_flags!(nes, zero, negative, carry);
        }
        #[test_case(0x33, 0x66, false, false, false ; "happy case")]
        #[test_case(0x45, 0x8A, false, true, false ; "negative set")]
        #[test_case(0x8F, 0x1E, false, false, true ; "carry set")]
        #[test_case(0x00, 0x00, true, false, false ; "zero set")]
        fn test_zp_x(value: u8, shifted: u8, zero: bool, negative: bool, carry: bool) {
            let mut nes = Nes::new();
            let x_value = nes.cpu.x;
            nes.cpu.x = x_value;
            let addr = set_addr_zp_offset(&mut nes, value, x_value);
            assert_eq!(nes.decode_and_execute(&[ASL_ZP_X, addr[0]]), Ok((2, 6)));
            assert_eq_hex!(nes.mem[addr[0] as usize], shifted);
            check_flags!(nes, zero, negative, carry);
        }
        #[test_case(0x08, 0x10, false, false, false ; "happy case")]
        #[test_case(0x48, 0x90, false, true, false ; "negative set")]
        #[test_case(0x88, 0x10, false, false, true ; "carry set")]
        #[test_case(0x00, 0x00, true, false, false ; "zero set")]
        fn test_abs(value: u8, shifted: u8, zero: bool, negative: bool, carry: bool) {
            let mut nes = Nes::new();
            let addr = set_addr_abs(&mut nes, value);
            assert_eq!(
                nes.decode_and_execute(&[ASL_ABS, addr[0], addr[1]]),
                Ok((3, 6))
            );
            assert_eq_hex!(nes.mem[addr_from_bytes(addr)], shifted);
            check_flags!(nes, zero, negative, carry);
        }
        #[test_case(0x07, 0x0E, false, false, false ; "happy case")]
        #[test_case(0x00, 0x00, true, false, false ; "zero set")]
        #[test_case(0x45, 0x8A, false, true, false ; "negative set")]
        #[test_case(0x86, 0x0C, false, false, true ; "carry set")]
        fn test_abs_x(value: u8, shifted: u8, zero: bool, negative: bool, carry: bool) {
            let mut nes = Nes::new();
            let x_value = random::<u8>();
            let addr = set_addr_abs_offset(&mut nes, value, x_value);
            assert_eq!(
                nes.decode_and_execute(&[ASL_ABS_X, addr[0], addr[1]]),
                Ok((3, 7))
            );
            assert_eq_hex!(nes.mem[Nes::get_absolute_addr(&addr)], shifted);
            check_flags!(nes, zero, negative, carry);
        }
    }
    macro_rules! branch_tests {
        ($name: ident, $opcode: ident, $flag: ident, $value: expr) => {
            mod $name {
                use super::*;
                use test_case::test_case;
                #[test_case(true, 0x12, 0x34, 0x46, 3 ; "branched")]
                #[test_case(false, 0x12, 0x34, 0x12, 2 ; "doesn't branch")]
                #[test_case(true, 0x18, 0x00, 0x18, 3 ; "branches to same location")]
                #[test_case(true, 0x00ff, 0x05, 0x0104, 5 ; "branches to a different page")]
                fn test_implied(
                    should_branch: bool,
                    pc: u16,
                    operand: u8,
                    new_pc: u16,
                    cycles: i64,
                ) {
                    let mut nes = Nes::new();
                    nes.cpu.p_c = pc;
                    nes.cpu.s_r.$flag = if should_branch { $value } else { !$value };
                    assert_eq!(nes.decode_and_execute(&[$opcode, operand]), Ok((2, cycles)));
                    assert_eq_hex!(nes.cpu.p_c, new_pc);
                }
            }
        };
    }
    branch_tests!(bcs, BCS, c, true);
    branch_tests!(bcc, BCC, c, false);
    branch_tests!(beq, BEQ, z, true);
    branch_tests!(bne, BNE, z, false);
    branch_tests!(bmi, BMI, n, true);
    branch_tests!(bpl, BPL, n, false);
    branch_tests!(bvs, BVS, v, true);
    branch_tests!(bvc, BVC, v, false);
    mod bit {
        use super::*;
        use test_case::test_case;
        macro_rules! bit_test {
            ($name: ident, $opcode: ident, $addr_func: ident, $result: expr) => {
                #[test_case(0x18, 0x27, true, false, false; "should set the zero flag")]
                #[test_case(0x18, 0x1F, false, false, false; "should clear the zero flag")]
                #[test_case(0x12, 0x74, false, true, false; "should set V")]
                #[test_case(0x11, 0x80, true, false, true; "should set N")]
                #[test_case(0x18, 0xFF, false, true, true ; "should set Z and N flag")]
                fn $name(a: u8, value: u8, z: bool, v: bool, n: bool) {
                    let mut nes = Nes::new();
                    nes.cpu.a = a;
                    let addr = $addr_func(&mut nes, value);
                    assert_eq!(
                        nes.decode_and_execute(&prepend_with_opcode($opcode, &addr)),
                        Ok($result)
                    );
                    assert_eq_hex!(nes.cpu.a, a, "A is changed");
                    assert_eq!(nes.cpu.s_r.z, z, "Z is wrong");
                    assert_eq!(nes.cpu.s_r.v, v, "V is wrong");
                    assert_eq!(nes.cpu.s_r.n, n, "N is wrong");
                }
            };
        }
        bit_test!(test_zero_page, BIT_ZP, set_addr_zp, (2, 3));
        bit_test!(test_absolute, BIT_ABS, set_addr_abs, (3, 4));
    }
    // Utility functions to get some addresses in memory set to the value given
    fn set_addr_zp(nes: &mut Nes, value: u8) -> [u8; 1] {
        set_addr_zp_offset(nes, value, 0)
    }
    fn set_addr_zp_offset(nes: &mut Nes, value: u8, offset: u8) -> [u8; 1] {
        let addr = random::<u8>();
        nes.mem[addr.wrapping_add(offset) as usize] = value;
        return [addr];
    }
    fn set_addr_abs_offset(nes: &mut Nes, value: u8, offset: u8) -> [u8; 2] {
        let addr = random::<u16>().wrapping_add(offset as u16);
        nes.mem[addr as usize] = value;
        return [(addr & 0xFF) as u8, (addr >> 8) as u8];
    }
    fn set_addr_abs(nes: &mut Nes, value: u8) -> [u8; 2] {
        set_addr_abs_offset(nes, value, 0)
    }
    fn first_byte(addr: u16) -> u8 {
        (addr & 0xFF) as u8
    }
    fn second_byte(addr: u16) -> u8 {
        (addr >> 8) as u8
    }
    fn addr_from_bytes(addr: [u8; 2]) -> usize {
        ((addr[1] as usize) << 8) + (addr[0] as usize)
    }
    // ten here since all instructions are way less than 10 bytes and extra ones can ust be ignored
    fn prepend_with_opcode(opcode: u8, arr: &[u8]) -> [u8; 10] {
        let mut a: [u8; 10] = [0; 10];
        a[0] = opcode;
        a[1..(arr.len() + 1)].copy_from_slice(arr);
        a
    }
}
