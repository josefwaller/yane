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
    /// Load some valie into Y.
    /// Sets the zero flag is Y = 0 and the negative flag if Y =
    ///
    /// ```
    /// let mut cpu = yane::Cpu::new();
    /// cpu.ldy(0x18);
    /// assert_eq!(cpu.y, 0x18);
    /// ```
    pub fn ldy(&mut self, value: u8) {
        self.y = value;
        self.set_load_flags(self.y);
    }
    /// Add some value with A and the carry bit in the status register.
    /// * Zero is set if A = 0 after the operation
    /// * Overflow is set if overflow occurs
    /// * Negative flag is set if the seventh flag is set
    pub fn adc(&mut self, value: u8) {
        let i = self.a;
        self.a = self
            .a
            .wrapping_add(value)
            .wrapping_add(if self.s_r.carry { 1 } else { 0 });
        self.s_r.zero = if self.a == 0 { true } else { false };
        // Way of checking for (unsigned) overflow
        self.s_r.carry = if self.a < value { true } else { false };
        // Way of checking for (signed) overflow
        // If I and value are the same sign (i.e. both positive/negative) but the result is a different sign, overflow has occured
        self.s_r.overflow = if (i & 0x80) == (value & 0x80) && (i & 0x80) != (self.a & 0x80) {
            true
        } else {
            false
        };
        self.s_r.negative = if self.a & 0x80 != 0 { true } else { false };
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
    #[test]
    fn test_ldy() {
        ld_test!(ldy);
    }
    #[test]
    fn test_adc() {
        let mut cpu = Cpu::new();
        cpu.adc(0x14);
        assert_eq_hex!(cpu.a, 0x14);
        cpu.adc(0x45);
        assert_eq_hex!(cpu.a, 0x14 + 0x45);
        assert_eq!(cpu.s_r.zero, false);
        assert_eq!(cpu.s_r.carry, false);
        assert_eq!(cpu.s_r.overflow, false);
        assert_eq!(cpu.s_r.negative, false);
    }
    #[test]
    fn test_adc_zero() {
        let mut cpu = Cpu::new();
        assert_eq!(cpu.s_r.zero, false);
        cpu.adc(0x0);
        assert_eq_hex!(cpu.a, 0x00);
        assert_eq!(cpu.s_r.zero, true);
        assert_eq!(cpu.s_r.carry, false);
        assert_eq!(cpu.s_r.overflow, false);
        assert_eq!(cpu.s_r.negative, false);
    }
    #[test]
    fn test_adc_negative() {
        let mut cpu = Cpu::new();
        assert_eq!(cpu.s_r.negative, false);
        cpu.adc(0x80);
        assert_eq!(cpu.s_r.negative, true);
        assert_eq!(cpu.s_r.overflow, false);
        assert_eq!(cpu.s_r.zero, false);
    }
    #[test]
    fn test_adc_overflow() {
        let mut cpu = Cpu::new();
        assert_eq!(cpu.s_r.overflow, false);
        cpu.adc(0x35);
        cpu.adc(0xFF);
        assert_eq!(cpu.s_r.carry, true);
        assert_eq!(cpu.s_r.zero, false);
        assert_eq!(cpu.s_r.negative, false);
    }
    #[test]
    fn test_adc_with_carry() {
        let mut cpu = Cpu::new();
        cpu.a = 0x18;
        cpu.s_r.carry = true;
        cpu.adc(0x45);
        assert_eq_hex!(cpu.a, 0x18 + 0x45 + 0x01);
        cpu.s_r.carry = false;
        cpu.adc(0x02);
        assert_eq_hex!(cpu.a, 0x18 + 0x45 + 0x01 + 0x02);
    }
    #[test]
    fn test_adc_carry_unsigned_overflow() {
        let mut cpu = Cpu::new();
        cpu.a = 0x65;
        cpu.s_r.carry = true;
        cpu.adc(0xFF - 0x65);
        assert_eq_hex!(cpu.a, 0x0);
        assert_eq!(cpu.s_r.carry, true);
    }
}
