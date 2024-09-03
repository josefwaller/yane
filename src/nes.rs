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
    // Macro for loading an immediate into a register
    macro_rules! ld_i_test {
        ($reg: ident, $opcode: expr) => {
            let mut nes = Nes::new();
            nes.decode_and_execute(&[$opcode, 0x18]).unwrap();
            assert_eq_hex!(nes.cpu.$reg, 0x18);
        };
    }
    #[test]
    fn test_lda_i() {
        ld_i_test!(a, LDA_I);
    }
    #[test]
    fn test_ldx_i() {
        ld_i_test!(x, LDX_I);
    }
    #[test]
    fn test_ldy_i() {
        ld_i_test!(y, LDY_I);
    }
    // Macro for generating a test from loading from zero page
    macro_rules! ld_zp_test {
        ($reg: ident, $opcode:expr) => {
            let mut nes = Nes::new();
            let addr = random::<u8>();
            nes.mem[addr as usize] = 0x34;
            nes.decode_and_execute(&[$opcode, addr]).unwrap();
            assert_eq_hex!(nes.cpu.$reg, 0x34);
        };
    }
    #[test]
    fn test_lda_zp() {
        ld_zp_test!(a, LDA_ZP);
    }
    #[test]
    fn test_ldx_zp() {
        ld_zp_test!(x, LDX_ZP);
    }
    #[test]
    fn test_ldy_zp() {
        ld_zp_test!(y, LDY_ZP);
    }
    // Macro for generating a test for loading from zero page with an offset
    macro_rules! ld_zp_offset_test {
        ($reg: ident, $opcode:expr, $off_reg: ident) => {
            let mut nes = Nes::new();
            let addr = random::<u8>();
            nes.cpu.$off_reg = random::<u8>();
            nes.mem[(addr.wrapping_add(nes.cpu.$off_reg)) as usize] = 0x34;
            nes.decode_and_execute(&[$opcode, addr]).unwrap();
            assert_eq_hex!(nes.cpu.$reg, 0x34);
        };
    }
    #[test]
    fn test_lda_zp_x() {
        ld_zp_offset_test!(a, LDA_ZP_X, x);
    }
    #[test]
    fn test_ldx_zp_y() {
        ld_zp_offset_test!(x, LDX_ZP_Y, y);
    }
    #[test]
    fn test_ldy_zp_x() {
        ld_zp_offset_test!(y, LDY_ZP_X, x);
    }
    macro_rules! ld_abs_test {
        ($reg: ident, $opcode: expr) => {
            let mut nes = Nes::new();
            let val = 0x18;
            let addr = set_addr_abs(&mut nes, val);
            nes.decode_and_execute(&[$opcode, addr[0], addr[1]])
                .unwrap();
            assert_eq_hex!(nes.cpu.$reg, val);
        };
    }
    #[test]
    fn test_lda_abs() {
        ld_abs_test!(a, LDA_ABS);
    }
    #[test]
    fn test_ldx_abs() {
        ld_abs_test!(x, LDX_ABS);
    }
    #[test]
    fn test_ldy_abs() {
        ld_abs_test!(y, LDY_ABS);
    }
    macro_rules! ld_abs_offset_test {
        ($reg:ident, $opcode:expr, $off_reg: ident) => {
            let mut nes = Nes::new();
            let addr = random::<u16>();
            nes.cpu.$off_reg = random::<u8>();
            nes.mem[addr.wrapping_add(nes.cpu.$off_reg as u16) as usize] = 0x18;
            nes.decode_and_execute(&[$opcode, first_byte(addr), second_byte(addr)])
                .unwrap();
            assert_eq_hex!(nes.cpu.$reg, 0x18);
        };
    }
    #[test]
    fn test_lda_abs_x() {
        ld_abs_offset_test!(a, LDA_ABS_X, x);
    }
    #[test]
    fn test_lda_abs_y() {
        ld_abs_offset_test!(a, LDA_ABS_Y, y);
    }
    #[test]
    fn test_ldx_abs_y() {
        ld_abs_offset_test!(x, LDX_ABS_Y, y);
    }
    #[test]
    fn test_ldy_abs_x() {
        ld_abs_offset_test!(y, LDY_ABS_X, x);
    }
    macro_rules! ld_ind_offset_test {
        ($reg:ident, $opcode: expr, $off_reg: ident) => {
            let mut nes = Nes::new();
            let v = 0x18;
            let addr = random::<u16>();
            nes.mem[addr as usize] = v;
            let mut operand = random::<u8>();
            nes.cpu.$off_reg = random::<u8>();
            let mut second_addr = operand.wrapping_add(nes.cpu.$off_reg);
            // Avoid collisions
            if second_addr as u16 == addr || second_addr as u16 == addr.wrapping_sub(1) {
                second_addr = second_addr.wrapping_add(2);
                operand = operand.wrapping_add(2);
            }
            nes.mem[second_addr as usize] = first_byte(addr);
            nes.mem[second_addr as usize + 1] = second_byte(addr);
            nes.decode_and_execute(&[$opcode, operand]).unwrap();
            assert_eq_hex!(nes.cpu.$reg, v);
        };
    }
    #[test]
    fn test_lda_ind_x() {
        let mut nes = Nes::new();
        let v = random::<u8>();
        let x = random::<u8>();
        nes.cpu.x = x;
        let addr = set_addr_ind_offset(&mut nes, v, x, 0);
        nes.decode_and_execute(&[LDA_IND_X, addr]).unwrap();
        assert_eq_hex!(nes.cpu.a, v);
    }
    #[test]
    fn test_lda_ind_y() {
        let mut nes = Nes::new();
        let v = random::<u8>();
        let y = random::<u8>();
        nes.cpu.y = y;
        let addr = set_addr_ind_offset(&mut nes, v, 0, y);
        nes.decode_and_execute(&[LDA_IND_Y, addr]).unwrap();
        assert_eq_hex!(nes.cpu.a, v);
    }
    fn adc_test<F: Fn(&mut Nes, u8)>(f: F) {
        let mut nes = Nes::new();
        let init_val = random::<u8>();
        nes.cpu.a = init_val;
        let v = random::<u8>();
        f(&mut nes, v);
        assert_eq_hex!(nes.cpu.a, init_val.wrapping_add(v));
    }
    #[test]
    fn test_adc_i() {
        adc_test(|nes, v| {
            nes.decode_and_execute(&[ADC_I, v]).unwrap();
        });
    }
    #[test]
    fn test_adc_zp() {
        adc_test(|nes, v| {
            let addr = set_addr_zp(nes, v);
            nes.decode_and_execute(&[ADC_ZP, addr]).unwrap();
        });
    }
    #[test]
    fn test_adc_zp_x() {
        adc_test(|nes, v| {
            let addr = set_addr_zp_x(nes, v);
            nes.decode_and_execute(&[ADC_ZP_X, addr]).unwrap();
        })
    }
    #[test]
    fn test_adc_abs() {
        adc_test(|nes, v| {
            let addr = set_addr_abs(nes, v);
            nes.decode_and_execute(&[ADC_ABS, addr[0], addr[1]])
                .unwrap();
        })
    }
    #[test]
    fn test_adc_abs_x() {
        adc_test(|nes, v| {
            let addr = set_addr_abs_offset_x(nes, v);
            nes.decode_and_execute(&[ADC_ABS_X, addr[0], addr[1]])
                .unwrap();
        })
    }
    #[test]
    fn test_adc_abs_y() {
        adc_test(|nes, v| {
            let addr = set_addr_abs_offset_y(nes, v);
            nes.decode_and_execute(&[ADC_ABS_Y, addr[0], addr[1]])
                .unwrap();
        })
    }
    #[test]
    fn test_adc_ind_x() {
        adc_test(|nes, v| {
            let x = random::<u8>();
            nes.cpu.x = x;
            let addr = set_addr_ind_offset(nes, v, x, 0);
            nes.decode_and_execute(&[ADC_IND_X, addr]).unwrap();
        });
    }
    #[test]
    fn test_adc_ind_y() {
        adc_test(|nes, v| {
            let y = random::<u8>();
            nes.cpu.y = y;
            let addr = set_addr_ind_offset(nes, v, 0, y);
            nes.decode_and_execute(&[ADC_IND_Y, addr]).unwrap();
        })
    }
    // Utility functions to get some addresses in memory set to the value given
    // Set addr and return zero page
    fn set_addr_zp(nes: &mut Nes, value: u8) -> u8 {
        let addr = random::<u8>();
        nes.mem[addr as usize] = value;
        return addr;
    }
    // Set addr and return zero page with x offset
    fn set_addr_zp_x(nes: &mut Nes, value: u8) -> u8 {
        nes.cpu.x = random::<u8>();
        let addr = random::<u8>();
        nes.mem[addr.wrapping_add(nes.cpu.x) as usize] = value;
        return addr;
    }
    fn set_addr_abs(nes: &mut Nes, value: u8) -> [u8; 2] {
        let addr = random::<u16>();
        nes.mem[addr as usize] = value;
        return [(addr & 0xFF) as u8, (addr >> 8) as u8];
    }
    macro_rules! set_addr_abs_offset {
        ($nes: ident, $reg: ident, $value: ident) => {
            let addr = random::<u16>();
            $nes.cpu.$reg = random::<u8>();
            $nes.mem[addr.wrapping_add($nes.cpu.$reg as u16) as usize] = $value;
            return [first_byte(addr), second_byte(addr)];
        };
    }
    fn set_addr_abs_offset_x(nes: &mut Nes, value: u8) -> [u8; 2] {
        set_addr_abs_offset!(nes, x, value);
    }
    fn set_addr_abs_offset_y(nes: &mut Nes, value: u8) -> [u8; 2] {
        set_addr_abs_offset!(nes, y, value);
    }
    fn set_addr_ind_offset(nes: &mut Nes, value: u8, first_offset: u8, second_offset: u8) -> u8 {
        let pointer = random::<u16>();
        nes.mem[(pointer.wrapping_add(second_offset as u16)) as usize] = value;
        let mut addr = random::<u8>();
        if addr as u16 == pointer.wrapping_add(second_offset as u16) as u16
            || addr as u16 == pointer.wrapping_add(second_offset as u16).wrapping_sub(1) as u16
        {
            addr = addr.wrapping_add(2);
        }
        nes.mem[(addr.wrapping_add(first_offset)) as usize] = first_byte(pointer);
        nes.mem[(addr.wrapping_add(first_offset)) as usize + 1] = second_byte(pointer);
        return addr;
    }
    fn first_byte(addr: u16) -> u8 {
        (addr & 0xFF) as u8
    }
    fn second_byte(addr: u16) -> u8 {
        (addr >> 8) as u8
    }
}
