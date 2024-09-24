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
            s_p: 0xFF,
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
    /// Load some value into Y.
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
        self.s_r.c = if self.a < i || (self.a == i && value > 0) {
            true
        } else {
            false
        };
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
    /// Perform an arithmatic shift left on some value.
    /// Essentially multiply it by 2
    /// * C is set to the carry bit (i.e. the MSB before the shift).
    /// * Z is set if `value` is 0 after the shift.
    /// * N is set if the MSB of `value` is set after the shift.
    /// ```
    /// let mut cpu = yane::Cpu::new();
    /// assert_eq!(cpu.asl(0x98), 0x30);
    /// assert_eq!(cpu.s_r.z, false);
    /// assert_eq!(cpu.s_r.c, true);
    /// ```
    pub fn asl(&mut self, value: u8) -> u8 {
        self.s_r.z = value & 0x7F == 0;
        self.s_r.c = value & 0x80 != 0;
        self.s_r.n = value & 0x40 != 0;
        return value << 1;
    }
    /// Perform a branch to `value` relatively if `param == true`.
    /// Updates the PC accordingly.
    /// Return how many cycles are needed by the branching operation.
    pub fn branch_if(&mut self, param: bool, value: u8) -> i64 {
        if param {
            let pc = self.p_c;
            self.p_c = self.p_c.wrapping_add(value as u16);
            return 3 + if (pc & 0xFF00) != (self.p_c & 0xFF00) {
                2
            } else {
                0
            };
        }
        2
    }
    /// Perform a bitwise test by ANDing A with `value`.
    /// Does not store the result, but uses it to set some flags
    /// * Z is set if the result is 0.
    /// * V is set to bit 6 of `value`.
    /// * N is set to bit 7 of `value`.
    /// ```
    /// let mut cpu = yane::Cpu::new();
    /// cpu.a = 0x18;
    /// cpu.bit(0xE0);
    /// // A is not affected
    /// assert_eq!(cpu.a, 0x18);
    /// assert_eq!(cpu.s_r.z, true);
    /// assert_eq!(cpu.s_r.v, true);
    /// assert_eq!(cpu.s_r.n, true);
    /// ```
    pub fn bit(&mut self, value: u8) {
        let result = self.a & value;
        self.s_r.z = result == 0;
        self.s_r.v = (value & 0x40) != 0;
        self.s_r.n = (value & 0x80) != 0;
    }
    /// Perform an interrupt to the location specified.
    /// Basically just sets the program counter to `location` and sets the interrupt flag.
    /// Returns an array representing the the program counter and status register, to be pushed to the stack.
    /// Return array is in the form `[LSB(PC), MSB(PC), SR]`, where:
    /// * `LSB(PC)` is the least significant byte of the program counter
    /// * `MSB(PC)` is the most significant byte of the program counter
    /// * SR is the status register
    pub fn brk(&mut self, location: u16) -> [u8; 3] {
        let to_stack: [u8; 3] = [
            (self.p_c & 0xFF) as u8,
            (self.p_c >> 8) as u8,
            self.s_r.to_byte(),
        ];
        self.s_r.i = true;
        self.p_c = location;
        return to_stack;
    }
    /// "Compare" the two values given and set the status register accordingly
    /// * C is set to `u >= v`
    /// * Z is set to `u == v``
    /// * N is set to the MSB of `u - v`
    /// ```
    /// let mut cpu = yane::Cpu::new();
    /// cpu.compare(0x18, 0x18);
    /// assert_eq!(cpu.s_r.z, true);
    /// cpu.compare(0x19, 0x18);
    /// assert_eq!(cpu.s_r.c, true);
    /// cpu.compare(0x18, 0x19);
    /// assert_eq!(cpu.s_r.n, true);
    /// ```
    pub fn compare(&mut self, u: u8, v: u8) {
        self.s_r.c = u >= v;
        self.s_r.z = u == v;
        self.s_r.n = (u.wrapping_sub(v) & 0x80) != 0;
    }
    /// Compare a value with A.
    /// Shorthand for `cpu.compare(cpu.a, v)`.
    pub fn cmp(&mut self, v: u8) {
        self.compare(self.a, v);
    }
    /// Compare a value with X.
    /// Shorthand for `cpu.compare(cpu.x, v)`
    pub fn cpx(&mut self, v: u8) {
        self.compare(self.x, v);
    }
    /// Comapre a value with Y.
    /// Shorthand for `cpu.compare(cpu.y, v)`
    pub fn cpy(&mut self, v: u8) {
        self.compare(self.y, v);
    }
    /// Decrement some value and set the flags accordingly.
    /// * Z is set if the return value is `0`
    /// * N is set if the MSB of the return value is set.
    /// ```
    /// let mut cpu = yane::Cpu::new();
    /// let a = cpu.dec(0x81);
    /// assert_eq!(a, 0x80);
    /// // Result is not zero
    /// assert_eq!(cpu.s_r.z, false);
    /// // Result is negative
    /// assert_eq!(cpu.s_r.n, true);
    /// ```
    pub fn dec(&mut self, v: u8) -> u8 {
        self.s_r.z = v == 1;
        let r = v.wrapping_sub(1);
        self.s_r.n = (r & 0x80) != 0;
        return r;
    }
    /// Perform an exclusive OR on A.
    /// Sets A to the result of A ^ `value`.
    /// * Z is set to A == 0
    /// * N is set to the MSB of A
    ///```
    /// let mut cpu = yane::Cpu::new();
    /// cpu.a = 0xFF;
    /// cpu.eor(0x77);
    /// assert_eq!(cpu.a, 0x88);
    /// assert_eq!(cpu.s_r.z, false);
    /// assert_eq!(cpu.s_r.n, true);
    /// ```
    pub fn eor(&mut self, value: u8) {
        self.a ^= value;
        self.s_r.z = self.a == 0;
        self.s_r.n = (self.a & 0x80) != 0;
    }
    /// Increment the value given and sets flag accordingly.
    /// Return the value after incrementation, wrapping if needed.
    /// * Z is set if the result is 0
    /// * N is set if the result is negative
    pub fn inc(&mut self, value: u8) -> u8 {
        let v = value.wrapping_add(1);
        self.s_r.z = v == 0;
        self.s_r.n = (v & 0x80) != 0;
        return v;
    }
    /// Logically shift the value right and set the flags accordingly.
    /// Return the value after shifting.
    /// * C is set to bit 0 of the value before shifting.
    /// * Z is set if the result is 0.
    /// ```
    /// let mut cpu = yane::Cpu::new();
    /// let value = cpu.lsr(0x81);
    /// assert_eq!(value, 0x40);
    /// assert_eq!(cpu.s_r.c, true);
    /// assert_eq!(cpu.s_r.z, false);
    /// ```
    pub fn lsr(&mut self, value: u8) -> u8 {
        self.s_r.c = (value & 0x01) != 0;
        let v = value >> 1;
        self.s_r.z = v == 0;
        return v;
    }
    /// Perform a bitwise OR with A and `value`.
    /// Modifies A and sets teh status register accordinly.
    /// * Z is set if A == 0
    /// * N is set if A is negative
    /// ```
    /// let mut cpu = yane::Cpu::new();
    /// cpu.ora(0x18);
    /// assert_eq!(cpu.a, 0x18);
    /// assert_eq!(cpu.s_r.n, false);
    /// assert_eq!(cpu.s_r.z, false);
    /// cpu.ora(0x81);
    /// assert_eq!(cpu.a, 0x99);
    /// assert_eq!(cpu.s_r.n, true);
    /// assert_eq!(cpu.s_r.z, false);
    /// ```
    pub fn ora(&mut self, value: u8) {
        self.a |= value;
        self.s_r.z = self.a == 0;
        self.s_r.n = (self.a & 0x80) != 0;
    }
    /// Rotate a byte left and set the flags accordingly
    /// * C is set to the MSB of the value given
    /// * Z is set if the result is 0
    /// * N is set to the MSB of the result
    pub fn rol(&mut self, value: u8) -> u8 {
        let mut new_val = value << 1;
        if self.s_r.c {
            new_val |= 0x01;
        }
        self.s_r.c = (value & 0x80) != 0;
        self.s_r.z = new_val == 0;
        self.s_r.n = (new_val & 0x80) != 0;
        new_val
    }
    /// Rotate a byte right and set the flags accordingly
    /// * C is set the the LSB of the value given (i.e. the value that is lost)
    /// * Z is set is the result is 0
    /// * N is set if the result is negative
    pub fn ror(&mut self, value: u8) -> u8 {
        let mut new_val = value >> 1;
        if self.s_r.c {
            new_val |= 0x80;
        }
        self.s_r.c = (value & 0x01) != 0;
        self.s_r.z = new_val == 0;
        self.s_r.n = (new_val & 0x80) != 0;
        new_val
    }
    /// Subtract a number from the accumulator with the NOT of the C bit.
    /// If C is 0, then it will subtract `value` + 1.
    /// * C is cleared if there is overflow
    /// * Z is set if A is 0
    /// * V is set if the sign bit is incorrect (i.e. if signed overflow has occurred)
    /// * N is set if the MSB is set
    pub fn sbc(&mut self, value: u8) {
        // Two's complement addition
        self.adc(value ^ 0xFF)
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
    use test_case::test_case;
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
    #[test_case(0x12, 0x24, false, false, false ; "happy case")]
    #[test_case(0x85, 0x0A, false, false, true ; "carry is set")]
    #[test_case(0x00, 0x00, true, false, false ; "zero is set")]
    #[test_case(0x80, 0x00, true, false, true ; "zero and carry is set")]
    #[test_case(0x45, 0x8A, false, true, false ; "negative is set")]
    #[test_case(0xF0, 0xE0, false, true, true ; "negative and carry are set")]
    fn test_asl(value: u8, shifted: u8, zero: bool, negative: bool, carry: bool) {
        let mut cpu = Cpu::new();
        assert_eq_hex!(cpu.asl(value), shifted, "should shift correctly");
        assert_eq!(cpu.s_r.z, zero, "zero is correct");
        assert_eq!(cpu.s_r.c, carry, "carry is correct");
        assert_eq!(cpu.s_r.n, negative, "negative is correct");
    }
}
