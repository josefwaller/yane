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
            .wrapping_add(if self.s_r.c { 1 } else { 0 });
        self.s_r.z = if self.a == 0 { true } else { false };
        // Way of checking for (unsigned) overflow
        self.s_r.c = if self.a < value { true } else { false };
        // Way of checking for (signed) overflow
        // If I and value are the same sign (i.e. both positive/negative) but the result is a different sign, overflow has occured
        self.s_r.v = if (i & 0x80) == (value & 0x80) && (i & 0x80) != (self.a & 0x80) {
            true
        } else {
            false
        };
        self.s_r.n = if self.a & 0x80 != 0 { true } else { false };
    }
    /// Perform an AND (`&``) operation between A and some value.
    /// * Z is set if A is 0
    /// * N is set if A is negative (i.e. the MSB is set)
    /// ```
    /// let mut cpu = yane::Cpu::new();
    /// cpu.a = 0xAA;
    /// cpu.and(0x0F);
    /// assert_eq!(cpu.a, 0x0A);
    /// ```
    pub fn and(&mut self, value: u8) {
        self.a &= value;
        if self.a == 0 {
            self.s_r.z = true;
        }
        if self.a & 0x80 != 0 {
            self.s_r.n = true;
        }
    }

    // Set the status register's flags when loading (LDA, LDX, or LDY)
    fn set_load_flags(&mut self, value: u8) {
        if value == 0 {
            self.s_r.z = true;
        }
        if value & 0x80 != 0 {
            self.s_r.n = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Cpu;
    use assert_hex::assert_eq_hex;
    #[derive(PartialEq)]
    enum Flag {
        Carry,
        Zero,
        Interrupt,
        Decimal,
        Break,
        Overflow,
        Negative,
    }

    fn check_flags(cpu: &Cpu, flags: Vec<Flag>) {
        macro_rules! check_flag {
            ($flag:ident, $flag_enum:ident, $flag_str:literal) => {
                assert_eq!(
                    cpu.s_r.$flag,
                    flags.contains(&Flag::$flag_enum),
                    "Expected {} flag to be {}",
                    $flag_str,
                    flags.contains(&Flag::$flag_enum)
                );
            };
        }
        check_flag!(c, Carry, "carry");
        check_flag!(z, Zero, "zero");
        check_flag!(i, Interrupt, "interrupt");
        check_flag!(d, Decimal, "decimal");
        check_flag!(b, Break, "break");
        check_flag!(v, Overflow, "overflow");
        check_flag!(n, Negative, "negative");
    }

    macro_rules! ld_test {
        ($ld:ident) => {
            let mut cpu = Cpu::new();
            // Test loading a number doesn't change flags
            cpu.$ld(0x18);
            check_flags(&cpu, Vec::new());
            // Test loading zero sets zero flag
            cpu.$ld(0x00);
            check_flags(&cpu, vec![Flag::Zero]);
            // Test loading a negative number sets negative flag and doesn't unset zero flag
            cpu.$ld(0x80);
            check_flags(&cpu, vec![Flag::Zero, Flag::Negative]);
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
        check_flags(&cpu, Vec::new());
        cpu.adc(0x45);
        assert_eq_hex!(cpu.a, 0x14 + 0x45);
        check_flags(&cpu, Vec::new());
    }
    #[test]
    fn test_adc_zero() {
        let mut cpu = Cpu::new();
        cpu.adc(0x0);
        assert_eq_hex!(cpu.a, 0x00);
        check_flags(&cpu, vec![Flag::Zero]);
    }
    #[test]
    fn test_adc_negative() {
        let mut cpu = Cpu::new();
        cpu.adc(0x80);
        check_flags(&cpu, vec![Flag::Negative]);
    }
    #[test]
    fn test_adc_unsigned_overflow() {
        let mut cpu = Cpu::new();
        cpu.adc(0x35);
        cpu.adc(0xFF);
        check_flags(&cpu, vec![Flag::Carry]);
    }
    #[test]
    fn test_adc_signed_overflow() {
        let mut cpu = Cpu::new();
        cpu.adc(0x40);
        cpu.adc(0x41);
        check_flags(&cpu, vec![Flag::Overflow, Flag::Negative]);
        cpu.adc(0x81);
        check_flags(&cpu, vec![Flag::Overflow, Flag::Carry]);
    }
    #[test]
    fn test_adc_with_carry() {
        let mut cpu = Cpu::new();
        cpu.a = 0x18;
        cpu.s_r.c = true;
        cpu.adc(0x45);
        assert_eq_hex!(cpu.a, 0x18 + 0x45 + 0x01);
        check_flags(&cpu, vec![]);
        cpu.adc(0x02);
        assert_eq_hex!(cpu.a, 0x18 + 0x45 + 0x01 + 0x02);
        check_flags(&cpu, vec![]);
    }
    #[test]
    fn test_adc_carry_unsigned_overflow() {
        let mut cpu = Cpu::new();
        cpu.a = 0x65;
        cpu.s_r.c = true;
        cpu.adc(0xFF - 0x65);
        assert_eq_hex!(cpu.a, 0x0);
        check_flags(&cpu, vec![Flag::Carry, Flag::Zero]);
    }
    #[test]
    fn test_and() {
        let mut cpu = Cpu::new();
        cpu.a = 0x67;
        cpu.and(0x60);
        assert_eq_hex!(cpu.a, 0x60);
        assert_eq!(cpu.s_r.z, false);
        assert_eq!(cpu.s_r.n, false);
    }
    #[test]
    fn test_and_zero() {
        let mut cpu = Cpu::new();
        cpu.a = 0xFF;
        cpu.and(0x00);
        assert_eq_hex!(cpu.a, 0x00);
        assert_eq!(cpu.s_r.z, true);
        assert_eq!(cpu.s_r.n, false);
    }
    #[test]
    fn test_and_negative() {
        let mut cpu = Cpu::new();
        cpu.a = 0xFF;
        cpu.and(0x85);
        assert_eq_hex!(cpu.a, 0x85);
        assert_eq!(cpu.s_r.z, false);
        assert_eq!(cpu.s_r.n, true);
    }
}
