use crate::StatusRegister;

/// The CPU of the NES.
/// Contains all registers and is responsible for changing the flags when the values are set/unset.
pub struct Cpu {
    /// Accumulator
    pub a: u8,
    /// X index register
    pub x: u8,
    /// Y index register
    pub y: u8,
    /// Program counter
    pub p_c: u16,
    /// Stack pointer
    pub s_p: u8,
    /// Status register
    pub s_r: StatusRegister,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            p_c: 0,
            s_p: 0,
            s_r: StatusRegister::new(),
        }
    }
    /// Load some value into A.
    /// Sets the status register accordingly.
    ///
    /// ```
    /// let mut cpu = yane::Cpu::new();
    /// cpu.lda(0x18);
    /// assert_eq!(cpu.a, 0x18);
    /// ```
    pub fn lda(&mut self, value: u8) {
        self.a = value;
        self.set_load_flags(self.a);
    }

    /// Load some value into X.
    /// Sets the status register accordingly.
    ///
    /// ```
    /// let mut cpu = yane::Cpu::new();
    /// cpu.ldx(0x18);
    /// assert_eq!(cpu.x, 0x18);
    /// ```
    pub fn ldx(&mut self, value: u8) {
        self.x = value;
        self.set_load_flags(self.x);
    }

    // Set the status register's flags when loading (LDA, LDX, or LDY)
    fn set_load_flags(&mut self, value: u8) {
        if value == 0 {
            self.s_r.zero = true;
        }
        if value & 0x80 != 0 {
            self.s_r.negative = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Cpu;
    use assert_hex::assert_eq_hex;

    macro_rules! ld_test {
        ($ld:ident) => {
            let mut cpu = Cpu::new();
            // Test loading a number doesn't change flags
            cpu.$ld(0x18);
            assert_eq_hex!(cpu.s_r.zero, false);
            assert_eq_hex!(cpu.s_r.negative, false);
            // Test loading zero sets zero flag
            cpu.$ld(0x00);
            assert_eq_hex!(cpu.s_r.zero, true);
            assert_eq_hex!(cpu.s_r.negative, false);
            // Test loading a negative number sets negative flag and doesn't unset zero flag
            cpu.$ld(0x80);
            assert_eq_hex!(cpu.s_r.zero, true);
            assert_eq_hex!(cpu.s_r.negative, true);
        };
    }

    #[test]
    fn test_lda() {
        ld_test!(lda);
    }
    #[test]
    fn test_ldx() {
        ld_test!(ldx);
    }
}
