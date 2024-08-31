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

    #[test]
    fn test_lda_no_flags() {
        let mut cpu = Cpu::new();
        cpu.lda(0x18);
        assert_eq_hex!(cpu.s_r.zero, false);
        assert_eq_hex!(cpu.s_r.negative, false);
    }
    #[test]
    fn test_lda_zero_flag() {
        let mut cpu = Cpu::new();
        cpu.lda(0x00);
        assert_eq_hex!(cpu.s_r.zero, true);
        assert_eq_hex!(cpu.s_r.negative, false);
    }
    #[test]
    fn test_lda_negative_flag() {
        let mut cpu = Cpu::new();
        cpu.lda(0x80);
        assert_eq_hex!(cpu.s_r.zero, false);
        assert_eq_hex!(cpu.s_r.negative, true);
    }
}
