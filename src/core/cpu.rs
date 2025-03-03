use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::core::StatusRegister;

/// The CPU of the NES.
///
/// Contains all registers and is responsible for changing the flags when the values are set/unset.
#[derive(Clone, Serialize, Deserialize)]
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

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
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
    /// ```
    /// let mut cpu = yane::core::Cpu::new();
    /// cpu.lda(0x18);
    /// assert_eq!(cpu.a, 0x18);
    /// ```
    pub fn lda(&mut self, value: u8) {
        self.a = value;
        self.set_load_flags(self.a);
    }

    /// Load some value into X.
    ///
    /// ```
    /// let mut cpu = yane::core::Cpu::new();
    /// cpu.ldx(0x18);
    /// assert_eq!(cpu.x, 0x18);
    /// ```
    pub fn ldx(&mut self, value: u8) {
        self.x = value;
        self.set_load_flags(self.x);
    }
    /// Load some value into Y.
    ///
    /// ```
    /// let mut cpu = yane::core::Cpu::new();
    /// cpu.ldy(0x18);
    /// assert_eq!(cpu.y, 0x18);
    /// ```
    pub fn ldy(&mut self, value: u8) {
        self.y = value;
        self.set_load_flags(self.y);
    }
    /// Add some value with A and the carry bit in the status register.
    ///
    /// * Zero is set if A = 0 after the operation
    /// * Overflow is set if overflow occurs
    /// * Negative flag is set if the seventh flag is set
    pub fn adc(&mut self, value: u8) {
        let i = self.a;
        self.a = self
            .a
            .wrapping_add(value)
            .wrapping_add(if self.s_r.c { 1 } else { 0 });
        self.s_r.z = self.a == 0;
        // Way of checking for (unsigned) overflow
        self.s_r.c = self.a < i || (self.a == i && value > 0);
        // Way of checking for (signed) overflow
        // If I and value are the same sign (i.e. both positive/negative) but the result is a different sign, overflow has occured
        self.s_r.v = (i & 0x80) == (value & 0x80) && (i & 0x80) != (self.a & 0x80);
        self.s_r.n = self.a & 0x80 != 0;
    }
    /// Perform an AND (`&``) operation between A and some value.
    ///
    /// * Z is set if A is 0
    /// * N is set if A is negative (i.e. the MSB is set)
    /// ```
    /// let mut cpu = yane::core::Cpu::new();
    /// cpu.a = 0xAA;
    /// cpu.and(0x0F);
    /// assert_eq!(cpu.a, 0x0A);
    /// ```
    pub fn and(&mut self, value: u8) {
        self.a &= value;
        self.s_r.z = self.a == 0;
        self.s_r.n = (self.a & 0x80) != 0;
    }
    /// Perform an arithmatic shift left on some value.
    ///
    /// Essentially multiply it by 2
    /// * C is set to the carry bit (i.e. the MSB before the shift).
    /// * Z is set if `value` is 0 after the shift.
    /// * N is set if the MSB of `value` is set after the shift.
    /// ```
    /// let mut cpu = yane::core::Cpu::new();
    /// assert_eq!(cpu.asl(0x98), 0x30);
    /// assert_eq!(cpu.s_r.z, false);
    /// assert_eq!(cpu.s_r.c, true);
    /// ```
    pub fn asl(&mut self, value: u8) -> u8 {
        self.s_r.z = value & 0x7F == 0;
        self.s_r.c = value & 0x80 != 0;
        self.s_r.n = value & 0x40 != 0;
        value << 1
    }
    /// Perform a branch to `value` relatively if `param == true`.
    ///
    /// Updates the PC accordingly.
    /// Return how many cycles are needed by the branching operation.
    pub fn branch_if(&mut self, param: bool, value: u8) -> i64 {
        if param {
            // PC if we don't take the branch
            let pc = self.p_c.wrapping_add(2);
            // Value is signed here, so we need to convert to signed values first and then convert back to unsigned
            self.p_c = (self.p_c as i16).wrapping_add((value as i8) as i16) as u16;
            // Wrapping add here since we will add 2 bytes after the current instruction
            return 3 + if (pc & 0xFF00) != (self.p_c.wrapping_add(2) & 0xFF00) {
                1
            } else {
                0
            };
        }
        2
    }
    /// Perform a bitwise test by ANDing A with `value`.
    ///
    /// Does not store the result, but uses it to set some flags
    /// * Z is set if the result is 0.
    /// * V is set to bit 6 of `value`.
    /// * N is set to bit 7 of `value`.
    /// ```
    /// let mut cpu = yane::core::Cpu::new();
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
    ///
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
            self.s_r.to_byte() | 0x10,
        ];
        self.s_r.i = true;
        self.p_c = location;
        to_stack
    }
    /// "Compare" the two values given and set the status register accordingly
    ///
    /// * C is set to `u >= v`
    /// * Z is set to `u == v``
    /// * N is set to the MSB of `u - v`
    /// ```
    /// let mut cpu = yane::core::Cpu::new();
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
    ///
    /// Shorthand for [Cpu::compare] with [Cpu::a] and `v`.
    pub fn cmp(&mut self, v: u8) {
        self.compare(self.a, v);
    }
    /// Compare a value with X.
    ///
    /// Shorthand for [Cpu::compare] with [Cpu::a] and `v`.
    pub fn cpx(&mut self, v: u8) {
        self.compare(self.x, v);
    }
    /// Comapre a value with Y.
    /// Shorthand for `cpu.compare(cpu.y, v)`
    pub fn cpy(&mut self, v: u8) {
        self.compare(self.y, v);
    }
    /// Decrement some value and set the flags accordingly.
    ///
    /// * Z is set if the return value is `0`
    /// * N is set if the MSB of the return value is set.
    /// ```
    /// let mut cpu = yane::core::Cpu::new();
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
        r
    }
    /// Perform an exclusive OR on A.
    ///
    /// Sets A to the result of A ^ `value`.
    /// * Z is set to A == 0
    /// * N is set to the MSB of A
    ///```
    /// let mut cpu = yane::core::Cpu::new();
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
    ///
    /// Return the value after incrementation, wrapping if needed.
    /// * Z is set if the result is 0
    /// * N is set if the result is negative
    pub fn inc(&mut self, value: u8) -> u8 {
        let v = value.wrapping_add(1);
        self.s_r.z = v == 0;
        self.s_r.n = (v & 0x80) != 0;
        v
    }
    /// Logically shift the value right and set the flags accordingly.
    ///
    /// Return the value after shifting.
    /// * C is set to bit 0 of the value before shifting.
    /// * Z is set if the result is 0.
    /// ```
    /// let mut cpu = yane::core::Cpu::new();
    /// let value = cpu.lsr(0x81);
    /// assert_eq!(value, 0x40);
    /// assert_eq!(cpu.s_r.c, true);
    /// assert_eq!(cpu.s_r.z, false);
    /// ```
    pub fn lsr(&mut self, value: u8) -> u8 {
        self.s_r.c = (value & 0x01) != 0;
        let v = value >> 1;
        self.s_r.z = v == 0;
        self.s_r.n = (v & 0x80) != 0;
        v
    }
    /// Perform a bitwise OR with A and `value`.
    ///
    /// Modifies A and sets teh status register accordinly.
    /// * Z is set if A == 0
    /// * N is set if A is negative
    /// ```
    /// let mut cpu = yane::core::Cpu::new();
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
    ///
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
    ///
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
    ///
    /// If C is 0, then it will subtract `value` + 1.
    /// * C is cleared if there is overflow
    /// * Z is set if A is 0
    /// * V is set if the sign bit is incorrect (i.e. if signed overflow has occurred)
    /// * N is set if the MSB is set
    pub fn sbc(&mut self, value: u8) {
        // Two's complement addition
        self.adc(value ^ 0xFF)
    }
    /// Shorthand for LDA then TAX
    ///
    /// Used only in unofficial opcodes
    pub fn lax(&mut self, value: u8) {
        self.lda(value);
        self.x = self.a;
    }
    /// Shorthand for DEC then CMP
    pub fn dcp(&mut self, value: u8) -> u8 {
        let v = self.dec(value);
        self.cmp(v);
        v
    }
    /// Shorthand for INC then SBC
    pub fn isc(&mut self, value: u8) -> u8 {
        let v = self.inc(value);
        self.sbc(v);
        v
    }
    /// Shorthand for ROL then AND
    pub fn rla(&mut self, value: u8) -> u8 {
        let v = self.rol(value);
        self.and(v);
        v
    }
    /// Shorthand for ROR then ADC
    pub fn rra(&mut self, value: u8) -> u8 {
        let v = self.ror(value);
        self.adc(v);
        v
    }
    /// Shorthand for ASL then ORA
    pub fn slo(&mut self, value: u8) -> u8 {
        let v = self.asl(value);
        self.ora(v);
        v
    }
    /// Shorthand for LSR then EOR
    pub fn sre(&mut self, value: u8) -> u8 {
        let v = self.lsr(value);
        self.eor(v);
        v
    }
    // Set the status register's flags when loading (LDA, LDX, or LDY)
    fn set_load_flags(&mut self, value: u8) {
        self.s_r.z = value == 0;
        self.s_r.n = (value & 0x80) != 0;
    }
}

impl Debug for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[PC={:4X} A={:2X} X={:2X} Y={:2X} SP={:2X} SR={:?}]",
            self.p_c, self.a, self.x, self.y, self.s_p, self.s_r
        )
    }
}
