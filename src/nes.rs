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
    /// nes.decode_and_execute(&[0xAE, 0x12, 0x34]);
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
                    .lda(self.read_indirect_addr_offset(&opcode[1..], self.cpu.x));
                Ok(2)
            }
            LDA_IND_Y => {
                self.cpu
                    .lda(self.read_indirect_addr_offset(&opcode[1..], self.cpu.y));
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
    // Read using indirect addressing with an offset
    fn read_indirect_addr_offset(&self, addr: &[u8], offset: u8) -> u8 {
        let first_addr = addr[0].wrapping_add(offset) as usize;
        let second_addr = &self.mem[first_addr..(first_addr + 2)];
        return self.read_absolute_addr(&second_addr);
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
    macro_rules! ld_abs_test {
        ($reg: ident, $opcode: expr) => {
            let mut nes = Nes::new();
            let addr = random::<u16>();
            nes.mem[addr as usize] = 0x18;
            nes.decode_and_execute(&[$opcode, (addr & 0xFF) as u8, (addr >> 8) as u8])
                .unwrap();
            assert_eq_hex!(nes.cpu.$reg, 0x18);
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
        ld_ind_offset_test!(a, LDA_IND_X, x);
    }
    #[test]
    fn test_lda_ind_y() {
        ld_ind_offset_test!(a, LDA_IND_Y, y);
    }

    fn first_byte(addr: u16) -> u8 {
        (addr & 0xFF) as u8
    }
    fn second_byte(addr: u16) -> u8 {
        (addr >> 8) as u8
    }
}
