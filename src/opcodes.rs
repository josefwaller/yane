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
/// Load A Indexed Indirect
pub const LDA_IDX_IND: u8 = 0xA1;
/// Load A Indirect Indexed
pub const LDA_IND_IDX: u8 = 0xB1;
/// Load X Immediate
pub const LDX_I: u8 = 0xA2;
/// Load X Zero Page
pub const LDX_ZP: u8 = 0xA6;
/// Load X Zero Page Y
pub const LDX_ZP_Y: u8 = 0xB6;
/// Load X Absolute
pub const LDX_ABS: u8 = 0xAE;
/// Load X Absolute Y
pub const LDX_ABS_Y: u8 = 0xBE;
/// Load Y Immediate
pub const LDY_I: u8 = 0xA0;
/// Load Y Zero Page
pub const LDY_ZP: u8 = 0xA4;
/// Load Y Zero Page X
pub const LDY_ZP_X: u8 = 0xB4;
/// Load Y Absolute
pub const LDY_ABS: u8 = 0xAC;
/// Load Y Absolute X
pub const LDY_ABS_X: u8 = 0xBC;
/// Add with Carry Immediate
pub const ADC_I: u8 = 0x69;
/// Add with Carry Zero Page
pub const ADC_ZP: u8 = 0x65;
/// Add with Carry Zero Page X
pub const ADC_ZP_X: u8 = 0x75;
/// Add with Carry Absolute
pub const ADC_ABS: u8 = 0x6D;
/// Add with Carry Absolute X
pub const ADC_ABS_X: u8 = 0x7D;
/// Add with Carry Absolute Y
pub const ADC_ABS_Y: u8 = 0x79;
/// Add with Carry Indexed Indirect
pub const ADC_IDX_IND: u8 = 0x61;
/// Add with Carry Indirect Indexed
pub const ADC_IND_IDX: u8 = 0x71;
/// And Immediate
pub const AND_I: u8 = 0x29;
/// And Zero Page
pub const AND_ZP: u8 = 0x25;
/// And Zero Page X
pub const AND_ZP_X: u8 = 0x35;
/// And Absolute
pub const AND_ABS: u8 = 0x2D;
/// And Absolute X
pub const AND_ABS_X: u8 = 0x3D;
/// And Absolute Y
pub const AND_ABS_Y: u8 = 0x39;
/// And Indexed Indirect
pub const AND_IND_IDX: u8 = 0x21;
/// And Indirect Indexed
pub const AND_IDX_IND: u8 = 0x31;
/// Arithmatic Shift Left Accumulator
pub const ASL_A: u8 = 0x0A;
/// Arithmatic Shift Left Zero Page
pub const ASL_ZP: u8 = 0x06;
/// Arithmatic Shift Left Zero Page X
pub const ASL_ZP_X: u8 = 0x16;
pub const ASL_ABS: u8 = 0x0E;
pub const ASL_ABS_X: u8 = 0x1E;
