//! Module that contains constants for every opcode used in the NES.
//! Includes both documented and undocumented opcodes.
/// Load A Immediate
pub const LDA_I: u8 = 0xA9;
/// Load A Zero Page
pub const LDA_ZP: u8 = 0xA5;
/// Load A Zero Page X
pub const LDA_ZP_X: u8 = 0xB5;
/// Load A Absolute
pub const LDA_ABS: u8 = 0xAD;
/// Load A Absolute X
pub const LDA_ABS_X: u8 = 0xBD;
/// Load A Absolute Y
pub const LDA_ABS_Y: u8 = 0xB9;
/// Load A Indirect X
pub const LDA_IND_X: u8 = 0xA1;
/// Load A Indirect Y
pub const LDA_IND_Y: u8 = 0xB1;
