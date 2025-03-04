use std::fmt::Debug;

use serde::{Deserialize, Serialize};

/// The status register of the NES.

#[derive(Clone, Serialize, Deserialize)]
pub struct StatusRegister {
    /// The carry flag, also known as the unsigned overflow flag
    pub c: bool,
    /// The zero flag
    pub z: bool,
    /// The interrupt diable flag
    pub i: bool,
    /// The decimal mode flag
    pub d: bool,
    /// The break command flag
    pub b: bool,
    /// The (signed) overflow flag
    pub v: bool,
    /// The negative flag
    pub n: bool,
}

impl Default for StatusRegister {
    fn default() -> Self {
        Self::new()
    }
}

impl StatusRegister {
    /// Create a new StatusRegister, initialising all flags to [false].
    pub fn new() -> StatusRegister {
        StatusRegister {
            c: false,
            z: false,
            i: true,
            d: false,
            b: false,
            v: false,
            n: false,
        }
    }
    /// Get the status register as a single byte to be written to memory.
    /// ```
    /// let mut s = yane::core::StatusRegister::new();
    /// s.z = true;
    /// s.d = true;
    /// s.i = false;
    /// assert_eq!(s.to_byte(), 0b00101010);
    /// s.n = true;
    /// s.v = true;
    /// assert_eq!(s.to_byte(), 0b11101010);
    /// ```
    pub fn to_byte(&self) -> u8 {
        let mut b = 0x20;
        if self.c {
            b |= 0x01;
        }
        if self.z {
            b |= 0x02;
        }
        if self.i {
            b |= 0x04;
        }
        if self.d {
            b |= 0x08;
        }
        if self.b {
            b |= 0x10;
        }
        if self.v {
            b |= 0x40;
        }
        if self.n {
            b |= 0x80;
        }
        b
    }
    /// Set the status register from a given byte that contains one bit per flag.
    /// ```
    /// let mut s = yane::core::StatusRegister::new();
    /// s.from_byte(0b11001010);
    /// assert_eq!(s.d, true);
    /// assert_eq!(s.v, true);
    /// assert_eq!(s.d, true);
    /// assert_eq!(s.z, true);
    /// ```
    pub fn from_byte(&mut self, byte: u8) {
        self.c = (byte & 0x01) != 0;
        self.z = (byte & 0x02) != 0;
        self.i = (byte & 0x04) != 0;
        self.d = (byte & 0x08) != 0;
        self.v = (byte & 0x40) != 0;
        self.n = (byte & 0x80) != 0;
    }
}

impl Debug for StatusRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        macro_rules! format_flag {
            ($flag: ident, $name: literal) => {
                if self.$flag {
                    format!(" {}", $name)
                } else {
                    format!("!{}", $name)
                }
            };
        }
        write!(
            f,
            "[{} {} {} {} {} {}]",
            format_flag!(c, "C"),
            format_flag!(z, "Z"),
            format_flag!(i, "I"),
            format_flag!(d, "D"),
            format_flag!(v, "V"),
            format_flag!(n, "N")
        )
    }
}
