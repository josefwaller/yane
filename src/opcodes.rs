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
pub const LDA_IND_X: u8 = 0xA1;
/// Load A Indirect Indexed
pub const LDA_IND_Y: u8 = 0xB1;
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
pub const ADC_IND_X: u8 = 0x61;
/// Add with Carry Indirect Indexed
pub const ADC_IND_Y: u8 = 0x71;
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
pub const AND_IND_Y: u8 = 0x21;
/// And Indirect Indexed
pub const AND_IND_X: u8 = 0x31;
/// Arithmetic Shift Left Accumulator
pub const ASL_A: u8 = 0x0A;
/// Arithmetic Shift Left Zero Page
pub const ASL_ZP: u8 = 0x06;
/// Arithmetic Shift Left Zero Page X
pub const ASL_ZP_X: u8 = 0x16;
/// Arithmetic Shift Left Absolute
pub const ASL_ABS: u8 = 0x0E;
/// Arithmetic Shift Left Absolute X
pub const ASL_ABS_X: u8 = 0x1E;
/// Branch if Carry Clear
pub const BCC: u8 = 0x90;
/// Branch if Carry Set
pub const BCS: u8 = 0xB0;
/// Branch if equal (branch if zero flag is set)
pub const BEQ: u8 = 0xF0;
/// Branch if not equal (branch if zero flag is cleared)
pub const BNE: u8 = 0xD0;
/// Branch if minus (branch if negative flag is set)
pub const BMI: u8 = 0x30;
/// Branch if positive (branch if negative flag is cleared)
pub const BPL: u8 = 0x10;
/// Branch if overflow cleared
pub const BVC: u8 = 0x50;
/// Branch if overflow set
pub const BVS: u8 = 0x70;
/// Bit test zero page
pub const BIT_ZP: u8 = 0x24;
/// Bit test absolute
pub const BIT_ABS: u8 = 0x2C;
/// Force interrupt
pub const BRK: u8 = 0x00;
/// Clear carry flag
pub const CLC: u8 = 0x18;
/// Clear decimal flag
pub const CLD: u8 = 0xD8;
/// Clear interrupt disable flag
pub const CLI: u8 = 0x58;
/// Clear overflow flag
pub const CLV: u8 = 0xB8;
/// Compare with A register Immediate
pub const CMP_I: u8 = 0xC9;
/// Compare with A Zero Page
pub const CMP_ZP: u8 = 0xC5;
/// Compare with A Zero Page X
pub const CMP_ZP_X: u8 = 0xD5;
/// Compare with A Absolute
pub const CMP_ABS: u8 = 0xCD;
/// Compare with A Absolute X
pub const CMP_ABS_X: u8 = 0xDD;
/// Compare with A Absolute Y
pub const CMP_ABS_Y: u8 = 0xD9;
/// Compare with A Indexed Indirect
pub const CMP_IND_X: u8 = 0xC1;
/// Compare with A Indirect Indexed
pub const CMP_IND_Y: u8 = 0xD1;
/// Compare with X Immediate
pub const CPX_I: u8 = 0xE0;
/// Compare with X Zero Page
pub const CPX_ZP: u8 = 0xE4;
/// Compare with X Absolute
pub const CPX_ABS: u8 = 0xEC;
/// Compare with Y Immediate
pub const CPY_I: u8 = 0xC0;
/// Compare with Y Zero Page
pub const CPY_ZP: u8 = 0xC4;
/// Compare with Y Absolute
pub const CPY_ABS: u8 = 0xCC;
/// Decrement Memory Zero Page
pub const DEC_ZP: u8 = 0xC6;
/// Decrement Memory Zero Page X
pub const DEC_ZP_X: u8 = 0xD6;
/// Decrement Absolute
pub const DEC_ABS: u8 = 0xCE;
/// Decrement Absolute X
pub const DEC_ABS_X: u8 = 0xDE;
/// Decrement X
pub const DEX: u8 = 0xCA;
/// Decrement Y
pub const DEY: u8 = 0x88;
/// Exclusive OR Immediate
pub const EOR_I: u8 = 0x49;
/// Exclusive OR Zero Page
pub const EOR_ZP: u8 = 0x45;
/// Exclusive OR Zero Page X
pub const EOR_ZP_X: u8 = 0x55;
/// Exclusive OR Absolute
pub const EOR_ABS: u8 = 0x4D;
/// Exclusive OR Absolute X
pub const EOR_ABS_X: u8 = 0x5D;
/// Exclusive OR Absoluve Y
pub const EOR_ABS_Y: u8 = 0x59;
/// Exclusive OR Indexed Indirect
pub const EOR_IND_X: u8 = 0x41;
/// Exclusive OR Indirect Indexed
pub const EOR_IND_Y: u8 = 0x51;
/// Increment Zero Page memory
pub const INC_ZP: u8 = 0xE6;
/// Increment memory Zero Page X
pub const INC_ZP_X: u8 = 0xF6;
/// Increment memory Absolute
pub const INC_ABS: u8 = 0xEE;
/// Increment memory Absolute X
pub const INC_ABS_X: u8 = 0xFE;
/// Increment X Implied
pub const INX: u8 = 0xE8;
/// Increment Y Implied
pub const INY: u8 = 0xC8;
/// Jump Absolute
pub const JMP_ABS: u8 = 0x4C;
/// Jump Indirect
pub const JMP_IND: u8 = 0x6C;
/// Jump to Subroutine
pub const JSR: u8 = 0x20;
/// Right Shift Accumulator
pub const LSR_A: u8 = 0x4A;
/// Right Shift Zero PAge
pub const LSR_ZP: u8 = 0x46;
/// Right Shift Zero Page X
pub const LSR_ZP_X: u8 = 0x56;
/// Right Shift Absolute
pub const LSR_ABS: u8 = 0x4E;
/// Right Shift Absolute X:w
pub const LSR_ABS_X: u8 = 0x5E;
/// No Operation
pub const NOP: u8 = 0xEA;
/// Or A Immediate
pub const ORA_I: u8 = 0x09;
/// Or A Zero Page
pub const ORA_ZP: u8 = 0x05;
/// Or A Zero Page X
pub const ORA_ZP_X: u8 = 0x15;
/// Or A Absolute
pub const ORA_ABS: u8 = 0x0D;
/// Or A Absolute X
pub const ORA_ABS_X: u8 = 0x1D;
/// Or A Absolute Y
pub const ORA_ABS_Y: u8 = 0x19;
/// Or A Indexed Indirect
pub const ORA_IND_X: u8 = 0x01;
/// Or A Indirect Indexed
pub const ORA_IND_Y: u8 = 0x11;
/// Push A
pub const PHA: u8 = 0x48;
// Push Status Processer
pub const PHP: u8 = 0x08;
/// Pull to A
pub const PLA: u8 = 0x68;
/// Pull to Status Processor
pub const PLP: u8 = 0x28;
/// Rotate A left
pub const ROL_A: u8 = 0x2A;
/// Rotate Left Zero Page
pub const ROL_ZP: u8 = 0x26;
/// Rotate Left Zero Page X
pub const ROL_ZP_X: u8 = 0x36;
/// Rotate Left Absolute
pub const ROL_ABS: u8 = 0x2E;
/// Rotate Left Absolute X
pub const ROL_ABS_X: u8 = 0x3E;
/// Rotate A right
pub const ROR_A: u8 = 0x6A;
/// Rotate Right Zero Page
pub const ROR_ZP: u8 = 0x66;
/// Rotate Right Zero Page X
pub const ROR_ZP_X: u8 = 0x76;
/// Rotate Right Absolute
pub const ROR_ABS: u8 = 0x6E;
/// Rotate Right Absolute X
pub const ROR_ABS_X: u8 = 0x7E;
/// Return from interrupt
pub const RTI: u8 = 0x40;
/// Return from subroutine
pub const RTS: u8 = 0x60;
/// Subtract with Carry Immediate
pub const SBC_I: u8 = 0xE9;
/// Subtract with Carry Zero Page
pub const SBC_ZP: u8 = 0xE5;
/// Subtract with Carry Zero Page X
pub const SBC_ZP_X: u8 = 0xF5;
/// Subtract with Carry Absolute
pub const SBC_ABS: u8 = 0xED;
/// Subtract with Carry Absolute X
pub const SBC_ABS_X: u8 = 0xFD;
/// Subtract with Carry Absolute Y
pub const SBC_ABS_Y: u8 = 0xF9;
/// Subtract with Carry Indexed Indirect
pub const SBC_IND_X: u8 = 0xE1;
/// Subtract with Carry Indirect Indexed
pub const SBC_IND_Y: u8 = 0xF1;
/// Set C flag
pub const SEC: u8 = 0x38;
/// Set D flag
pub const SED: u8 = 0xF8;
/// Set I flag
pub const SEI: u8 = 0x78;
/// Store Accumulator Zero Page
pub const STA_ZP: u8 = 0x85;
/// Store Accumulator Zero Page X
pub const STA_ZP_X: u8 = 0x95;
/// Store Accumulator Absolute
pub const STA_ABS: u8 = 0x8D;
/// Store Accumulator Absolute X
pub const STA_ABS_X: u8 = 0x9D;
/// Store Acuumulator Absolute Y
pub const STA_ABS_Y: u8 = 0x99;
/// Store Accumulator Indexed Indirect
pub const STA_IND_X: u8 = 0x81;
/// Store Accumulator Indirect Indexed
pub const STA_IND_Y: u8 = 0x91;
pub const STX_ZP: u8 = 0x86;
pub const STX_ZP_Y: u8 = 0x96;
pub const STX_ABS: u8 = 0x8E;
pub const STY_ZP: u8 = 0x84;
pub const STY_ZP_X: u8 = 0x94;
pub const STY_ABS: u8 = 0x8C;
