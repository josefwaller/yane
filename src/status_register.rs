/// The status register of the NES.
pub struct StatusRegister {
    /// The carry flag, also known as the unsigned overflow flag
    pub carry: bool,
    /// The zero flag
    pub zero: bool,
    /// The interrupt diable flag
    pub interrupt_disable: bool,
    /// The decimal mode flag
    pub decimal_mode: bool,
    /// The break command flag
    pub break_command: bool,
    /// The (signed) overflow flag
    pub overflow: bool,
    /// The negative flag
    pub negative: bool,
}

impl StatusRegister {
    /// Create a new StatusRegister, initialising all flags to `false`.
    pub fn new() -> StatusRegister {
        StatusRegister {
            carry: false,
            zero: false,
            interrupt_disable: false,
            decimal_mode: false,
            break_command: false,
            overflow: false,
            negative: false,
        }
    }
}
