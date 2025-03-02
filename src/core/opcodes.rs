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
pub const AND_IND_X: u8 = 0x21;
/// And Indirect Indexed
pub const AND_IND_Y: u8 = 0x31;
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
/// Store X Zero Page
pub const STX_ZP: u8 = 0x86;
/// Store X Zero Page Y
pub const STX_ZP_Y: u8 = 0x96;
/// Store X Absolute
pub const STX_ABS: u8 = 0x8E;
/// Store Y Zero Page
pub const STY_ZP: u8 = 0x84;
/// Store Y Zero Page X
pub const STY_ZP_X: u8 = 0x94;
/// Store Y Absolute
pub const STY_ABS: u8 = 0x8C;
/// Transfer A to X
pub const TAX: u8 = 0xAA;
/// Transfer A to Y
pub const TAY: u8 = 0xA8;
/// Transfer Stack Pointer to X
pub const TSX: u8 = 0xBA;
/// Transfer X to A
pub const TXA: u8 = 0x8A;
/// Transfer X to Stack Pointer
pub const TXS: u8 = 0x9A;
/// Transfer Y to A
pub const TYA: u8 = 0x98;
/// Unofficial opcodes
pub mod unofficial {
    /// AND and then LSR Immediate
    pub const ALR_I: u8 = 0x4B;
    /// AND and then copy N into C
    pub const ANC_I: [u8; 2] = [0x0B, 0x2B];
    // AND and then ROR, with slightly differnent flags set
    pub const ARR_I: u8 = 0x6B;
    /// Sets X to (A AND X) - value
    pub const AXS_I: u8 = 0xCB;
    /// Load into A And X Zero Page
    pub const LAX_ZP: u8 = 0xA7;
    /// Load into A and X Zero Page Y
    pub const LAX_ZP_Y: u8 = 0xB7;
    /// Load into A and X Absolute
    pub const LAX_ABS: u8 = 0xAF;
    /// Load into A and X Absolute Y
    pub const LAX_ABS_Y: u8 = 0xBF;
    /// Load into A and X Indexed Indirect
    pub const LAX_IND_X: u8 = 0xA3;
    /// Load into A and X Indirect Indexed
    pub const LAX_IND_Y: u8 = 0xB3;
    /// Store (A AND X) Zero Page
    pub const SAX_ZP: u8 = 0x87;
    /// Store (A AND X) Zero Page Y
    pub const SAX_ZP_Y: u8 = 0x97;
    /// Store (A AND X) Absolute
    pub const SAX_ABS: u8 = 0x8F;
    /// Store (A AND X) Indexed Indirect
    pub const SAX_IND_X: u8 = 0x83;
    /// Decrement then compare Zero Page
    pub const DCP_ZP: u8 = 0xC7;
    /// Decrement then compare Zero Page X
    pub const DCP_ZP_X: u8 = 0xD7;
    /// Decrement then compare Absolute
    pub const DCP_ABS: u8 = 0xCF;
    /// Decrement then compare Absolute X
    pub const DCP_ABS_X: u8 = 0xDF;
    /// Decrement then compare Absolute Y
    pub const DCP_ABS_Y: u8 = 0xDB;
    /// Decrement then compare Indexed Indirect
    pub const DCP_IND_X: u8 = 0xC3;
    /// Decrement then compare Indirect Indexed
    pub const DCP_IND_Y: u8 = 0xD3;
    /// Increment then subtract with carry Zero Page
    pub const ISC_ZP: u8 = 0xE7;
    /// Increment then subtract with carry Zero Page X
    pub const ISC_ZP_X: u8 = 0xF7;
    /// Increment then subtract with carry Absolute
    pub const ISC_ABS: u8 = 0xEF;
    /// Increment then subtract with carry Absolute X
    pub const ISC_ABS_X: u8 = 0xFF;
    /// Increment then subtract with carry Absolute Y
    pub const ISC_ABS_Y: u8 = 0xFB;
    /// Increment then subtract with carry Indexed Indirect
    pub const ISC_IND_X: u8 = 0xE3;
    /// Increment then subtract with carry Indirect Indexed
    pub const ISC_IND_Y: u8 = 0xF3;
    /// Rotate Left then AND Zero Page
    pub const RLA_ZP: u8 = 0x27;
    /// Rotate Left then AND Zero Page X
    pub const RLA_ZP_X: u8 = 0x37;
    /// Rotate Left then AND Absolute
    pub const RLA_ABS: u8 = 0x2F;
    /// Rotate Left then AND Absolute X
    pub const RLA_ABS_X: u8 = 0x3F;
    /// Rotate Left then AND Absolute Y
    pub const RLA_ABS_Y: u8 = 0x3B;
    /// Rotate Left then AND Indexed Indirect
    pub const RLA_IND_X: u8 = 0x23;
    /// Rotate Left then AND Indirect Indexed
    pub const RLA_IND_Y: u8 = 0x33;
    /// Rotate Right then add with carry Zero Page
    pub const RRA_ZP: u8 = 0x67;
    /// Rotate Right then add with carry Zero Page X
    pub const RRA_ZP_X: u8 = 0x77;
    /// Rotate Right then add with carry Absolute
    pub const RRA_ABS: u8 = 0x6F;
    /// Rotate Right then add with carry Absolute X
    pub const RRA_ABS_X: u8 = 0x7F;
    /// Rotate Right then add with carry Absolute Y
    pub const RRA_ABS_Y: u8 = 0x7B;
    /// Rotate Right then add with carry Indexed Indirect
    pub const RRA_IND_X: u8 = 0x63;
    /// Rotate Right then add with carry Indirect Indexed
    pub const RRA_IND_Y: u8 = 0x73;
    /// Shift left then OR with A Zero Page
    pub const SLO_ZP: u8 = 0x07;
    /// Shift left then OR with A Zero Page X
    pub const SLO_ZP_X: u8 = 0x17;
    /// Shift left then OR with A Absolute
    pub const SLO_ABS: u8 = 0x0F;
    /// Shift left then OR with A Absolute X
    pub const SLO_ABS_X: u8 = 0x1F;
    /// Shift left then OR with A Absolute Y
    pub const SLO_ABS_Y: u8 = 0x1B;
    /// Shift left then OR with A Indexed Indirect
    pub const SLO_IND_X: u8 = 0x03;
    /// Shift left then OR with A Indirect Indexed
    pub const SLO_IND_Y: u8 = 0x13;
    /// Shift right then EOR with A Zero Page
    pub const SRE_ZP: u8 = 0x47;
    /// Shift right then EOR with A Zero Page X
    pub const SRE_ZP_X: u8 = 0x57;
    /// Shift right then EOR with A Absolute
    pub const SRE_ABS: u8 = 0x4F;
    /// Shift right then EOR with A Absolute X
    pub const SRE_ABS_X: u8 = 0x5F;
    /// Shift right then EOR with A Absolute Y
    pub const SRE_ABS_Y: u8 = 0x5B;
    /// Shift right then EOR with A Indexed Indirect
    pub const SRE_IND_X: u8 = 0x43;
    /// Shift right then EOR with A Indirect Indexed
    pub const SRE_IND_Y: u8 = 0x53;
    /// Unofficial clone of SBC (E9), behaves the same
    pub const SBC: u8 = 0xEB;
    /// Unofficial NOPs
    pub const NOPS: [u8; 6] = [0x1A, 0x3A, 0x5A, 0x7A, 0xDA, 0xFA];
    /// Read a byte and skip it (essentially a 2-byte NOP)
    pub const SKBS: [u8; 5] = [0x80, 0x82, 0x89, 0xC2, 0xE2];
    /// Ignore byte from memory Zero Page
    pub const IGN_ZP: [u8; 3] = [0x04, 0x44, 0x64];
    /// Ignore byte from memory Zero Page X
    pub const IGN_ZP_X: [u8; 6] = [0x14, 0x34, 0x54, 0x74, 0xD4, 0xF4];
    // TODO: Figure out what this means: The absolute version can be used to increment PPUADDR or reset the PPUSTATUS latch as an alternative to BIT.
    /// Ignore byte from memory Absolute
    pub const IGN_ABS: u8 = 0x0C;
    /// Ignore byte from memory Absolute X
    pub const IGN_ABS_X: [u8; 6] = [0x1C, 0x3C, 0x5C, 0x7C, 0xDC, 0xFC];
}

fn combine_le_bytes(bytes: &[u8]) -> u16 {
    ((bytes[1] as u16) << 8) + bytes[0] as u16
}
fn format_implied(opcode: &str) -> String {
    opcode.to_string()
}
fn format_immediate(opcode: &str, operands: &[u8]) -> String {
    format!("{} #{:X}", opcode, combine_le_bytes(operands))
}
fn format_a(opcode: &str) -> String {
    format!("{}, A", opcode)
}
fn format_zp(opcode: &str, operands: &[u8]) -> String {
    format!("{} ${:2X}", opcode, operands[0])
}
fn format_zp_x(opcode: &str, operands: &[u8]) -> String {
    format!("{}, X", format_zp(opcode, operands))
}
fn format_zp_y(opcode: &str, operands: &[u8]) -> String {
    format!("{}, X", format_zp(opcode, operands))
}
fn format_relative(opcode: &str, operands: &[u8]) -> String {
    format!(
        "{}, *{:+X} ({:+})",
        opcode, operands[0] as i8, operands[0] as i8
    )
}
fn format_absolute(opcode: &str, operands: &[u8]) -> String {
    format!("{}, ${:4X}", opcode, combine_le_bytes(operands))
}
fn format_absolute_x(opcode: &str, operands: &[u8]) -> String {
    format!("{}, X", format_absolute(opcode, operands))
}
fn format_absolute_y(opcode: &str, operands: &[u8]) -> String {
    format!("{}, Y", format_absolute(opcode, operands))
}
fn format_indirect(opcode: &str, operands: &[u8]) -> String {
    format!("{} ({:4X})", opcode, combine_le_bytes(operands))
}
fn format_indexed_indirect(opcode: &str, operands: &[u8]) -> String {
    format!("{} (${:2X}, X)", opcode, operands[0])
}
fn format_indirect_indexed(opcode: &str, operands: &[u8]) -> String {
    format!("{} (${:2X}), Y", opcode, operands[0])
}

pub fn format_opcode(opcode: u8, operands: &[u8]) -> String {
    use unofficial::*;
    match opcode {
        LDA_I => format_immediate("LDA", operands),
        LDA_ZP => format_zp("LDA", operands),
        LDA_ZP_X => format_zp_x("LDA", operands),
        LDA_ABS => format_absolute("LDA", operands),
        LDA_ABS_X => format_absolute_x("LDA", operands),
        LDA_ABS_Y => format_absolute_y("LDA", operands),
        LDA_IND_X => format_indexed_indirect("LDA", operands),
        LDA_IND_Y => format_indirect_indexed("LDA", operands),
        LDX_I => format_immediate("LDX", operands),
        LDX_ZP => format_zp("LDX", operands),
        LDX_ZP_Y => format_zp_y("LDX", operands),
        LDX_ABS => format_absolute("LDX", operands),
        LDX_ABS_Y => format_absolute_y("LDX", operands),
        LDY_I => format_immediate("LDY", operands),
        LDY_ZP => format_zp("LDY", operands),
        LDY_ZP_X => format_zp_x("LDY", operands),
        LDY_ABS => format_absolute("LDY", operands),
        LDY_ABS_X => format_absolute_x("LDY", operands),
        ADC_I => format_immediate("ADC", operands),
        ADC_ZP => format_zp("ADC", operands),
        ADC_ZP_X => format_zp_x("ADC", operands),
        ADC_ABS => format_absolute("ADC", operands),
        ADC_ABS_X => format_absolute_x("ADC", operands),
        ADC_ABS_Y => format_absolute_y("ADC", operands),
        ADC_IND_X => format_indexed_indirect("ADC", operands),
        ADC_IND_Y => format_indirect_indexed("ADC", operands),
        AND_I => format_immediate("AND", operands),
        AND_ZP => format_zp("AND", operands),
        AND_ZP_X => format_zp_x("AND", operands),
        AND_ABS => format_absolute("AND", operands),
        AND_ABS_X => format_absolute_x("AND", operands),
        AND_ABS_Y => format_absolute_y("AND", operands),
        AND_IND_X => format_indexed_indirect("AND", operands),
        AND_IND_Y => format_indirect_indexed("AND", operands),
        ASL_A => format_a("ASL"),
        ASL_ZP => format_zp("ASL", operands),
        ASL_ZP_X => format_zp_x("ASL", operands),
        ASL_ABS => format_absolute("ASL", operands),
        ASL_ABS_X => format_absolute_x("ASL", operands),
        BCC => format_relative("BCC", operands),
        BCS => format_relative("BCS", operands),
        BEQ => format_relative("BEQ", operands),
        BNE => format_relative("BNE", operands),
        BMI => format_relative("BMI", operands),
        BPL => format_relative("BPL", operands),
        BVC => format_relative("BVC", operands),
        BVS => format_relative("BVS", operands),
        BIT_ZP => format_zp("BIT", operands),
        BIT_ABS => format_absolute("BIT", operands),
        BRK => format_implied("BRK"),
        CLC => format_implied("CLC"),
        CLD => format_implied("CLD"),
        CLI => format_implied("CLI"),
        CLV => format_implied("CLV"),
        CMP_I => format_immediate("CMP", operands),
        CMP_ZP => format_zp("CMP", operands),
        CMP_ZP_X => format_zp_x("CMP", operands),
        CMP_ABS => format_absolute("CMP", operands),
        CMP_ABS_X => format_absolute_x("CMP", operands),
        CMP_ABS_Y => format_absolute_y("CMP", operands),
        CMP_IND_X => format_indexed_indirect("CMP", operands),
        CMP_IND_Y => format_indirect_indexed("CMP", operands),
        CPX_I => format_immediate("CPX", operands),
        CPX_ZP => format_zp("CPX", operands),
        CPX_ABS => format_absolute("CPX", operands),
        CPY_I => format_immediate("CPY", operands),
        CPY_ZP => format_zp("CPY", operands),
        CPY_ABS => format_absolute("CPY", operands),
        DEC_ZP => format_zp("DEC", operands),
        DEC_ZP_X => format_zp_x("DEC", operands),
        DEC_ABS => format_absolute("DEC", operands),
        DEC_ABS_X => format_absolute_x("DEC", operands),
        DEX => format_implied("DEX"),
        DEY => format_implied("DEY"),
        EOR_I => format_immediate("EOR", operands),
        EOR_ZP => format_zp("EOR", operands),
        EOR_ZP_X => format_zp_x("EOR", operands),
        EOR_ABS => format_absolute("EOR", operands),
        EOR_ABS_X => format_absolute_x("EOR", operands),
        EOR_ABS_Y => format_absolute_y("EOR", operands),
        EOR_IND_X => format_indexed_indirect("EOR", operands),
        EOR_IND_Y => format_indirect_indexed("EOR", operands),
        INC_ZP => format_zp("INC", operands),
        INC_ZP_X => format_zp_x("INC", operands),
        INC_ABS => format_absolute("INC", operands),
        INC_ABS_X => format_absolute_x("INC", operands),
        INX => format_implied("INX"),
        INY => format_implied("INY"),
        JMP_ABS => format_absolute("JMP", operands),
        JMP_IND => format_indirect("JMP", operands),
        JSR => format_absolute("JSR", operands),
        LSR_A => format_a("LSR"),
        LSR_ZP => format_zp("LSR", operands),
        LSR_ZP_X => format_zp_x("LSR", operands),
        LSR_ABS => format_absolute("LSR", operands),
        LSR_ABS_X => format_absolute_x("LSR", operands),
        NOP => format_implied("NOP"),
        ORA_I => format_immediate("ORA", operands),
        ORA_ZP => format_zp("ORA", operands),
        ORA_ZP_X => format_zp_x("ORA", operands),
        ORA_ABS => format_absolute("ORA", operands),
        ORA_ABS_X => format_absolute_x("ORA", operands),
        ORA_ABS_Y => format_absolute_y("ORA", operands),
        ORA_IND_X => format_indexed_indirect("ORA", operands),
        ORA_IND_Y => format_indirect_indexed("ORA", operands),
        PHA => format_implied("PHA"),
        PHP => format_implied("PHP"),
        PLA => format_implied("PLA"),
        PLP => format_implied("PLP"),
        ROL_A => format_a("ROL"),
        ROL_ZP => format_zp("ROL", operands),
        ROL_ZP_X => format_zp_x("ROL", operands),
        ROL_ABS => format_absolute("ROL", operands),
        ROL_ABS_X => format_absolute_x("ROL", operands),
        ROR_A => format_a("ROR"),
        ROR_ZP => format_zp("ROR", operands),
        ROR_ZP_X => format_zp_x("ROR", operands),
        ROR_ABS => format_absolute("ROR", operands),
        ROR_ABS_X => format_absolute_x("ROR", operands),
        RTI => format_implied("RTI"),
        RTS => format_implied("RTS"),
        SBC_I => format_immediate("SBC", operands),
        SBC_ZP => format_zp("SBC", operands),
        SBC_ZP_X => format_zp_x("SBC", operands),
        SBC_ABS => format_absolute("SBC", operands),
        SBC_ABS_X => format_absolute_x("SBC", operands),
        SBC_ABS_Y => format_absolute_y("SBC", operands),
        SBC_IND_X => format_indexed_indirect("SBC", operands),
        SBC_IND_Y => format_indirect_indexed("SBC", operands),
        SEC => format_implied("SEC"),
        SED => format_implied("SED"),
        SEI => format_implied("SEI"),
        STA_ZP => format_zp("STA", operands),
        STA_ZP_X => format_zp_x("STA", operands),
        STA_ABS => format_absolute("STA", operands),
        STA_ABS_X => format_absolute_x("STA", operands),
        STA_ABS_Y => format_absolute_y("STA", operands),
        STA_IND_X => format_indexed_indirect("STA", operands),
        STA_IND_Y => format_indirect_indexed("STA", operands),
        STX_ZP => format_zp("STX", operands),
        STX_ZP_Y => format_zp_y("STX", operands),
        STX_ABS => format_absolute("STX", operands),
        STY_ZP => format_zp("STY", operands),
        STY_ZP_X => format_zp_x("STY", operands),
        STY_ABS => format_absolute("STY", operands),
        TAX => format_implied("TAX"),
        TAY => format_implied("TAY"),
        TSX => format_implied("TSX"),
        TXA => format_implied("TXA"),
        TXS => format_implied("TXS"),
        TYA => format_implied("TYA"),
        ALR_I => format_immediate("ALR", operands),
        _ if ANC_I.contains(&opcode) => format_immediate("ANC", operands),
        ARR_I => format_immediate("ARR", operands),
        AXS_I => format_immediate("AXS", operands),
        LAX_ZP => format_zp("LAX", operands),
        LAX_ZP_Y => format_zp_y("LAX", operands),
        LAX_ABS => format_absolute("LAX", operands),
        LAX_ABS_Y => format_absolute_y("LAX", operands),
        LAX_IND_X => format_indexed_indirect("LAX", operands),
        LAX_IND_Y => format_indirect_indexed("LAX", operands),
        SAX_ZP => format_zp("SAX", operands),
        SAX_ZP_Y => format_zp_y("SAX", operands),
        SAX_ABS => format_absolute("SAX", operands),
        SAX_IND_X => format_indexed_indirect("SAX", operands),
        DCP_ZP => format_zp("DCP", operands),
        DCP_ZP_X => format_zp_x("DCP", operands),
        DCP_ABS => format_absolute("DCP", operands),
        DCP_ABS_X => format_absolute_x("DCP", operands),
        DCP_ABS_Y => format_absolute_y("DCP", operands),
        DCP_IND_X => format_indexed_indirect("DCP", operands),
        DCP_IND_Y => format_indirect_indexed("DCP", operands),
        ISC_ZP => format_zp("ISC", operands),
        ISC_ZP_X => format_zp_x("ISC", operands),
        ISC_ABS => format_absolute("ISC", operands),
        ISC_ABS_X => format_absolute_x("ISC", operands),
        ISC_ABS_Y => format_absolute_y("ISC", operands),
        ISC_IND_X => format_indexed_indirect("ISC", operands),
        ISC_IND_Y => format_indirect_indexed("ISC", operands),
        RLA_ZP => format_zp("RLA", operands),
        RLA_ZP_X => format_zp_x("RLA", operands),
        RLA_ABS => format_absolute("RLA", operands),
        RLA_ABS_X => format_absolute_x("RLA", operands),
        RLA_ABS_Y => format_absolute_y("RLA", operands),
        RLA_IND_X => format_indexed_indirect("RLA", operands),
        RLA_IND_Y => format_indirect_indexed("RLA", operands),
        RRA_ZP => format_zp("RRA", operands),
        RRA_ZP_X => format_zp_x("RRA", operands),
        RRA_ABS => format_absolute("RRA", operands),
        RRA_ABS_X => format_absolute_x("RRA", operands),
        RRA_ABS_Y => format_absolute_y("RRA", operands),
        RRA_IND_X => format_indexed_indirect("RRA", operands),
        RRA_IND_Y => format_indirect_indexed("RRA", operands),
        SLO_ZP => format_zp("SLO", operands),
        SLO_ZP_X => format_zp_x("SLO", operands),
        SLO_ABS => format_absolute("SLO", operands),
        SLO_ABS_X => format_absolute_x("SLO", operands),
        SLO_ABS_Y => format_absolute_y("SLO", operands),
        SLO_IND_X => format_indexed_indirect("SLO", operands),
        SLO_IND_Y => format_indirect_indexed("SLO", operands),
        SRE_ZP => format_zp("SRE", operands),
        SRE_ZP_X => format_zp_x("SRE", operands),
        SRE_ABS => format_absolute("SRE", operands),
        SRE_ABS_X => format_absolute_x("SRE", operands),
        SRE_ABS_Y => format_absolute_y("SRE", operands),
        SRE_IND_X => format_indexed_indirect("SRE", operands),
        SRE_IND_Y => format_indirect_indexed("SRE", operands),
        SBC => format_implied("SBC"),
        _ if NOPS.contains(&opcode) => format_implied("NOPS"),
        _ if SKBS.contains(&opcode) => format_implied("SKBS"),
        _ if IGN_ZP.contains(&opcode) => format_zp("IGN", operands),
        _ if IGN_ZP_X.contains(&opcode) => format_zp_x("IGN", operands),
        IGN_ABS => format_absolute("IGN", operands),
        _ if IGN_ABS_X.contains(&opcode) => format_absolute_x("IGN", operands),
        _ => panic!(
            "Invalid opcode provided to formatted: {:X} (operands: {:X?})",
            opcode, operands
        ),
    }
}
