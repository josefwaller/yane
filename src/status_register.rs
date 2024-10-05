/// The status register of the NES.
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

impl StatusRegister {
    /// Create a new StatusRegister, initialising all flags to `false`.
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
    /// let mut s = yane::StatusRegister::new();
    /// s.z = true;
    /// s.d = true;
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
    /// let mut s = yane::StatusRegister::new();
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
        //self.b = (byte & 0x10) != 0;
        self.v = (byte & 0x40) != 0;
        self.n = (byte & 0x80) != 0;
    }
}
