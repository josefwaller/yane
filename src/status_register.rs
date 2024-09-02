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
            i: false,
            d: false,
            b: false,
            v: false,
            n: false,
        }
    }
}
