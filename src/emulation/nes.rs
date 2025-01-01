use std::{collections::VecDeque, fmt::Debug};

use log::*;

use crate::{
    opcodes::*, Apu, Cartridge, Controller, Cpu, Ppu, Screen, Settings, CPU_CYCLES_PER_OAM,
    CPU_CYCLES_PER_SCANLINE,
};
pub struct NesState {
    cpu: Cpu,
    opcode: u8,
    operands: Vec<u8>,
}

impl NesState {
    pub fn new(nes: &Nes, instruction: &[u8]) -> NesState {
        NesState {
            cpu: nes.cpu.clone(),
            opcode: instruction[0],
            operands: instruction[1..].to_vec(),
        }
    }
}
impl Debug for NesState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} NEXT INST={} (OPCODE={:X} OPERANDS={:X?})",
            self.cpu,
            format_opcode(self.opcode, self.operands.as_slice()),
            self.opcode,
            self.operands.as_slice()
        )
    }
}

const NUMBER_STORED_STATES: usize = 200;

/// The NES.
pub struct Nes {
    /// CPU of the NES
    pub cpu: Cpu,
    /// PPU of the NES
    pub ppu: Ppu,
    /// APU of the NES
    pub apu: Apu,
    /// Memory of the NES
    pub mem: [u8; 0x800],
    // Cartridge inserted in the NES
    pub cartridge: Cartridge,
    // Play 1 and 2 controller states
    pub controllers: [Controller; 2],
    // Cached controller states, the ROM will need to poll to keep these up to date
    cached_controllers: [Controller; 2],
    // Current bit being read from the controller
    controller_bits: [usize; 2],
    // Last 200 instructions executed, stored for debugging purposes
    pub previous_states: VecDeque<NesState>,
}

impl Nes {
    pub fn new() -> Nes {
        let c = [
            &vec!['N' as u8, 'E' as u8, 'S' as u8, 0x1A][..],
            &vec![0; 32 * 0x4000 + 16 * 0x2000][..],
        ]
        .concat();
        Nes {
            cpu: Cpu::new(),
            ppu: Ppu::new(),
            apu: Apu::new(),
            mem: [0x00; 0x800],
            cartridge: Cartridge::new(c.as_slice(), None),
            controllers: [Controller::new(); 2],
            cached_controllers: [Controller::new(); 2],
            controller_bits: [0; 2],
            previous_states: VecDeque::with_capacity(NUMBER_STORED_STATES),
        }
    }
    pub fn from_cartridge(cartridge: Cartridge) -> Nes {
        let mut nes = Nes {
            cpu: Cpu::new(),
            ppu: Ppu::new(),
            apu: Apu::new(),
            mem: [0x00; 0x800],
            cartridge,
            controllers: [Controller::new(); 2],
            cached_controllers: [Controller::new(); 2],
            controller_bits: [0; 2],
            previous_states: VecDeque::with_capacity(NUMBER_STORED_STATES),
        };
        nes.cpu.p_c = ((nes.cartridge.read_cpu(0xFFFD) as u16) << 8)
            + (nes.cartridge.read_cpu(0xFFFC) as u16);
        info!("Initialized PC to {:#X}", nes.cpu.p_c);
        nes
    }

    fn read_controller_bit(&mut self, num: usize) -> u8 {
        let pressed = match self.controller_bits[num] {
            0 => self.cached_controllers[num].a,
            1 => self.cached_controllers[num].b,
            2 => self.cached_controllers[num].select,
            3 => self.cached_controllers[num].start,
            4 => self.cached_controllers[num].up,
            5 => self.cached_controllers[num].down,
            6 => self.cached_controllers[num].left,
            7 => self.cached_controllers[num].right,
            _ => true,
        };
        self.controller_bits[num] += 1;
        return if pressed { 1 } else { 0 };
    }

    pub fn read_byte(&mut self, addr: usize) -> u8 {
        return match addr {
            0..0x2000 => self.mem[addr % 0x0800],
            0x2000..0x4000 => self.ppu.read_byte(addr, &self.cartridge),
            0x4016 => self.read_controller_bit(0),
            0x4017 => self.read_controller_bit(1),
            0x4000..0x4020 => self.apu.read_byte(addr),
            0x4020..0x10000 => self.cartridge.read_cpu(addr),
            _ => panic!("Invalid read address provided: {:#X}", addr),
        };
    }
    fn write_byte(&mut self, addr: usize, value: u8) {
        match addr {
            0..0x2000 => self.mem[addr % 0x0800] = value,
            0x2000..0x4000 => self.ppu.write_byte(addr, value, &mut self.cartridge),
            0x4014 => {
                // Set PPU DMA register
                self.ppu.oam_dma = Some(value);
            }
            // Input byte
            // Sets whether to poll or not
            0x4016 => {
                // TODO: Delay this until 0 is written
                self.cached_controllers = self.controllers;
                self.controller_bits[0] = 0;
                self.controller_bits[1] = 0;
            }
            // 0x4017 => self.controller_bit = 0,
            // APU Registers
            0x4000..0x4020 => self.apu.write_byte(addr, value),
            0x4020..0x10000 => self.cartridge.write_cpu(addr, value),
            _ => panic!("Invalid write address provided: {:#X}", addr),
        };
    }

    /// Update the internal controller state in thte NES.
    /// The ROM will still have to poll for the controller state.
    /// `num` should either be 0 or 1, depending on whose controller state is being updated
    pub fn set_input(&mut self, num: usize, state: Controller) {
        self.controllers[num] = state;
    }

    pub fn step(&mut self) -> Result<u32, String> {
        let pc = self.cpu.p_c as usize;
        let mut inst: [u8; 3] = [0; 3];
        inst.copy_from_slice(&[
            self.read_byte(pc),
            self.read_byte(pc + 1),
            self.read_byte(pc + 2),
        ]);
        self.previous_states.push_back(NesState::new(&self, &inst));
        if self.previous_states.len() > NUMBER_STORED_STATES {
            self.previous_states.pop_front();
        }
        // Add instruction for debugging purposes
        match self.decode_and_execute(&inst) {
            Ok((bytes, cycles)) => {
                self.cpu.p_c = self.cpu.p_c.wrapping_add(bytes);
                return Ok(cycles as u32);
            }
            Err(s) => {
                error!("Encountered an error \"{}\" while processing {:X?}, printing last 200 states\n{:#X?}",
                    s,
                    inst,
                    self.previous_states
                );
                return Err(s);
            }
        }
    }

    pub fn on_nmi(&mut self) {
        self.push_to_stack_u16(self.cpu.p_c);
        self.push_to_stack(self.cpu.s_r.to_byte());
        // Go to NMI vector
        self.cpu.p_c = ((self.read_byte(0xFFFB) as u16) << 8) + self.read_byte(0xFFFA) as u16;
    }

    /// Decode and then execute first byte of `opcode` as an NES opcode.
    /// Returns `(bytes, cycles`, where `bytes` is how much the program counter should be incremented by,
    /// i.e. how many bytes were used by the opcode, and `cycles` is how many cycles the operation needed.
    /// Does not change the program counter.
    ///
    /// # Examples
    /// ```
    /// use yane::Nes;
    /// let mut nes = Nes::new();
    /// // Load 0x18 into A
    /// nes.decode_and_execute(&[0xA9, 0x18]);
    /// // Load the memory at 0x0234 into A
    /// nes.decode_and_execute(&[0xAE, 0x34, 0x02]);
    /// // Perform a nop
    /// nes.decode_and_execute(&[0xEA]);
    /// ```
    pub fn decode_and_execute(&mut self, instruction: &[u8]) -> Result<(u16, i64), String> {
        let [opcode, operands @ ..] = instruction else {
            return Err(format!(
                "Invalid instruction provided: '{:#X?}'",
                instruction
            ));
        };
        /*
         * Simple macro to create a block that just calls a CPU function
         */
        macro_rules! cpu_func {
            ($func: ident, $read_addr: ident, $bytes: expr, $cycles: expr) => {{
                let v = self.$read_addr(operands);
                self.cpu.$func(v);
                Ok(($bytes, $cycles))
            }};
            ($func: ident, $read_addr: ident, $pc: ident, $bytes: expr, $cycles_no_pc: expr, $cycles_pc: expr) => {{
                let v = self.$read_addr(operands);
                self.cpu.$func(v);
                Ok((
                    $bytes,
                    if self.$pc(operands) {
                        $cycles_pc
                    } else {
                        $cycles_no_pc
                    },
                ))
            }};
        }
        /*
         * Simple macro to create a block that calls a CPU function and stores the result somewhere
         */
        macro_rules! cpu_write_func {
            ($func: ident, $read_addr: ident, $write_addr: ident, $bytes: expr, $cycles: expr) => {{
                let v = self.$read_addr(operands);
                self.$write_addr(operands, v);
                let value = self.cpu.$func(v);
                self.$write_addr(operands, value);
                Ok(($bytes, $cycles))
            }};
        }
        // Macro to set or unset a CPU flag
        macro_rules! flag_func {
            ($flag: ident, $val: expr) => {{
                self.cpu.s_r.$flag = $val;
                Ok((1, 2))
            }};
        }
        // Macro to write a CPU register to memory
        macro_rules! store_func {
            ($reg: ident, $write_addr: ident, $bytes: expr, $cycles: expr) => {{
                self.$write_addr(operands, self.cpu.$reg);
                Ok(($bytes, $cycles))
            }};
            ($value: expr, $write_addr: ident, $bytes: expr, $cycles: expr) => {{
                self.$write_addr(operands, $value);
                Ok(($bytes, $cycles))
            }};
        }
        macro_rules! transfer_func {
            ($from_reg: ident, $to_reg: ident) => {{
                self.cpu.$to_reg = self.cpu.$from_reg;
                self.cpu.s_r.z = self.cpu.$to_reg == 0;
                self.cpu.s_r.n = (self.cpu.$to_reg & 0x80) != 0;
                Ok((1, 2))
            }};
        }
        match *opcode {
            // LDA
            LDA_I => cpu_func!(lda, read_immediate, 2, 2),
            LDA_ZP => cpu_func!(lda, read_zp, 2, 3),
            LDA_ZP_X => cpu_func!(lda, read_zp_x, 2, 4),
            LDA_ABS => cpu_func!(lda, read_abs, 3, 4),
            LDA_ABS_X => cpu_func!(lda, read_abs_x, pc_x, 3, 4, 5),
            LDA_ABS_Y => cpu_func!(lda, read_abs_y, pc_y, 3, 4, 5),
            LDA_IND_X => cpu_func!(lda, read_indexed_indirect, 2, 6),
            LDA_IND_Y => cpu_func!(lda, read_indirect_indexed, pc_ind, 2, 5, 6),
            // LDX
            LDX_I => cpu_func!(ldx, read_immediate, 2, 2),
            LDX_ZP => cpu_func!(ldx, read_zp, 2, 3),
            LDX_ZP_Y => cpu_func!(ldx, read_zp_y, 2, 4),
            LDX_ABS => cpu_func!(ldx, read_abs, 3, 4),
            LDX_ABS_Y => cpu_func!(ldx, read_abs_y, pc_y, 3, 4, 5),
            // LDY
            LDY_I => cpu_func!(ldy, read_immediate, 2, 2),
            LDY_ZP => cpu_func!(ldy, read_zp, 2, 3),
            LDY_ZP_X => cpu_func!(ldy, read_zp_x, 2, 4),
            LDY_ABS => cpu_func!(ldy, read_abs, 3, 4),
            LDY_ABS_X => cpu_func!(ldy, read_abs_x, pc_x, 3, 4, 5),
            // ADC
            ADC_I => cpu_func!(adc, read_immediate, 2, 2),
            ADC_ZP => cpu_func!(adc, read_zp, 2, 3),
            ADC_ZP_X => cpu_func!(adc, read_zp_x, 2, 4),
            ADC_ABS => cpu_func!(adc, read_abs, 3, 4),
            ADC_ABS_X => cpu_func!(adc, read_abs_x, pc_x, 3, 4, 5),
            ADC_ABS_Y => cpu_func!(adc, read_abs_y, pc_y, 3, 4, 5),
            ADC_IND_X => cpu_func!(adc, read_indexed_indirect, 2, 6),
            ADC_IND_Y => cpu_func!(adc, read_indirect_indexed, pc_ind, 2, 5, 6),
            // AND
            AND_I => cpu_func!(and, read_immediate, 2, 2),
            AND_ZP => cpu_func!(and, read_zp, 2, 3),
            AND_ZP_X => cpu_func!(and, read_zp_x, 2, 4),
            AND_ABS => cpu_func!(and, read_abs, 3, 4),
            AND_ABS_X => cpu_func!(and, read_abs_x, pc_x, 3, 4, 5),
            AND_ABS_Y => cpu_func!(and, read_abs_y, pc_y, 3, 4, 5),
            AND_IND_X => cpu_func!(and, read_indexed_indirect, 2, 6),
            AND_IND_Y => cpu_func!(and, read_indirect_indexed, pc_ind, 2, 5, 6),
            // ASL
            ASL_A => cpu_write_func!(asl, read_a, write_a, 1, 2),
            ASL_ZP => cpu_write_func!(asl, read_zp, write_zp, 2, 5),
            ASL_ZP_X => cpu_write_func!(asl, read_zp_x, write_zp_x, 2, 6),
            ASL_ABS => cpu_write_func!(asl, read_abs, write_abs, 3, 6),
            ASL_ABS_X => cpu_write_func!(asl, read_abs_x, write_abs_x, 3, 7),
            // Various branching functions
            BCS => Ok((2, self.cpu.branch_if(self.cpu.s_r.c, operands[0]))),
            BCC => Ok((2, self.cpu.branch_if(!self.cpu.s_r.c, operands[0]))),
            BEQ => Ok((2, self.cpu.branch_if(self.cpu.s_r.z, operands[0]))),
            BNE => Ok((2, self.cpu.branch_if(!self.cpu.s_r.z, operands[0]))),
            BMI => Ok((2, self.cpu.branch_if(self.cpu.s_r.n, operands[0]))),
            BPL => Ok((2, self.cpu.branch_if(!self.cpu.s_r.n, operands[0]))),
            BVS => Ok((2, self.cpu.branch_if(self.cpu.s_r.v, operands[0]))),
            BVC => Ok((2, self.cpu.branch_if(!self.cpu.s_r.v, operands[0]))),
            // BIT
            BIT_ZP => cpu_func!(bit, read_zp, 2, 3),
            BIT_ABS => cpu_func!(bit, read_abs, 3, 4),
            // BRK
            BRK => {
                // Copy into stack
                self.push_to_stack_u16(self.cpu.p_c.wrapping_add(2));
                self.push_to_stack(self.cpu.s_r.to_byte());
                self.cpu.s_r.i = true;
                self.cpu.p_c = (((self.read_byte(0xFFFE) as u16) << 8)
                    + self.read_byte(0xFFFF) as u16)
                    .wrapping_sub(1);
                Ok((1, 7))
            }
            // Various flag clearing functions
            CLC => flag_func!(c, false),
            CLD => flag_func!(d, false),
            CLI => flag_func!(i, false),
            CLV => flag_func!(v, false),
            // CMP
            CMP_I => cpu_func!(cmp, read_immediate, 2, 2),
            CMP_ZP => cpu_func!(cmp, read_zp, 2, 3),
            CMP_ZP_X => cpu_func!(cmp, read_zp_x, 2, 4),
            CMP_ABS => cpu_func!(cmp, read_abs, 3, 4),
            CMP_ABS_X => cpu_func!(cmp, read_abs_x, pc_x, 3, 4, 5),
            CMP_ABS_Y => cpu_func!(cmp, read_abs_y, pc_y, 3, 4, 5),
            CMP_IND_X => cpu_func!(cmp, read_indexed_indirect, 2, 6),
            CMP_IND_Y => cpu_func!(cmp, read_indirect_indexed, pc_ind, 2, 5, 6),
            // CPX
            CPX_I => cpu_func!(cpx, read_immediate, 2, 2),
            CPX_ZP => cpu_func!(cpx, read_zp, 2, 3),
            CPX_ABS => cpu_func!(cpx, read_abs, 3, 4),
            // CPX
            CPY_I => cpu_func!(cpy, read_immediate, 2, 2),
            CPY_ZP => cpu_func!(cpy, read_zp, 2, 3),
            CPY_ABS => cpu_func!(cpy, read_abs, 3, 4),
            // DEC
            DEC_ZP => cpu_write_func!(dec, read_zp, write_zp, 2, 5),
            DEC_ZP_X => cpu_write_func!(dec, read_zp_x, write_zp_x, 2, 6),
            DEC_ABS => cpu_write_func!(dec, read_abs, write_abs, 3, 6),
            DEC_ABS_X => cpu_write_func!(dec, read_abs_x, write_abs_x, 3, 7),
            DEX => {
                self.cpu.x = self.cpu.dec(self.cpu.x);
                Ok((1, 2))
            }
            DEY => {
                self.cpu.y = self.cpu.dec(self.cpu.y);
                Ok((1, 2))
            }
            // EOR
            EOR_I => cpu_func!(eor, read_immediate, 2, 2),
            EOR_ZP => cpu_func!(eor, read_zp, 2, 3),
            EOR_ZP_X => cpu_func!(eor, read_zp_x, 2, 4),
            EOR_ABS => cpu_func!(eor, read_abs, 3, 4),
            EOR_ABS_X => cpu_func!(eor, read_abs_x, pc_x, 3, 4, 5),
            EOR_ABS_Y => cpu_func!(eor, read_abs_y, pc_y, 3, 4, 5),
            EOR_IND_X => cpu_func!(eor, read_indexed_indirect, 2, 6),
            EOR_IND_Y => cpu_func!(eor, read_indirect_indexed, pc_ind, 2, 5, 6),
            // INC
            INC_ZP => cpu_write_func!(inc, read_zp, write_zp, 2, 5),
            INC_ZP_X => cpu_write_func!(inc, read_zp_x, write_zp_x, 2, 6),
            INC_ABS => cpu_write_func!(inc, read_abs, write_abs, 3, 6),
            INC_ABS_X => cpu_write_func!(inc, read_abs_x, write_abs_x, 3, 7),
            INX => {
                self.cpu.x = self.cpu.inc(self.cpu.x);
                Ok((1, 2))
            }
            INY => {
                self.cpu.y = self.cpu.inc(self.cpu.y);
                Ok((1, 2))
            }
            JMP_ABS => {
                self.cpu.p_c = (Nes::get_absolute_addr(operands) as u16).wrapping_sub(3);
                Ok((3, 3))
            }
            JMP_IND => {
                self.cpu.p_c = (Nes::get_absolute_addr(&[
                    self.read_abs(operands),
                    // Wrapping add here due to a bug with the NES where reading addresses wraps around the page boundary
                    self.read_abs(&[operands[0].wrapping_add(1), operands[1]]),
                ]) as u16)
                    .wrapping_sub(3);
                Ok((3, 5))
            }
            JSR => {
                // Push PC to stack
                self.push_to_stack_u16(self.cpu.p_c.wrapping_add(2));
                // Set new PC from instruction
                self.cpu.p_c = (Nes::get_absolute_addr(operands) as u16).wrapping_sub(3);
                Ok((3, 6))
            }
            // LSR
            LSR_A => cpu_write_func!(lsr, read_a, write_a, 1, 2),
            LSR_ZP => cpu_write_func!(lsr, read_zp, write_zp, 2, 5),
            LSR_ZP_X => cpu_write_func!(lsr, read_zp_x, write_zp_x, 2, 6),
            LSR_ABS => cpu_write_func!(lsr, read_abs, write_abs, 3, 6),
            LSR_ABS_X => cpu_write_func!(lsr, read_abs_x, write_abs_x, 3, 7),
            NOP => Ok((1, 2)),
            // ORA
            ORA_I => cpu_func!(ora, read_immediate, 2, 2),
            ORA_ZP => cpu_func!(ora, read_zp, 2, 3),
            ORA_ZP_X => cpu_func!(ora, read_zp_x, 2, 4),
            ORA_ABS => cpu_func!(ora, read_abs, 3, 4),
            ORA_ABS_X => cpu_func!(ora, read_abs_x, pc_x, 3, 4, 5),
            ORA_ABS_Y => cpu_func!(ora, read_abs_y, pc_y, 3, 4, 5),
            ORA_IND_X => cpu_func!(ora, read_indexed_indirect, 2, 6),
            ORA_IND_Y => cpu_func!(ora, read_indirect_indexed, pc_ind, 2, 5, 6),
            // Pushing to stack
            PHA => {
                self.push_to_stack(self.cpu.a);
                Ok((1, 3))
            }
            PHP => {
                // B should be set when manually pushing to stack
                self.push_to_stack(self.cpu.s_r.to_byte() | 0x10);
                Ok((1, 3))
            }
            // Pulling from stack
            PLA => {
                self.cpu.a = self.pull_from_stack();
                self.cpu.s_r.z = self.cpu.a == 0;
                self.cpu.s_r.n = (self.cpu.a & 0x80) != 0;
                Ok((1, 4))
            }
            PLP => {
                let v = self.pull_from_stack();
                self.cpu.s_r.from_byte(v);
                Ok((1, 4))
            }
            // ROL
            ROL_A => cpu_write_func!(rol, read_a, write_a, 1, 2),
            ROL_ZP => cpu_write_func!(rol, read_zp, write_zp, 2, 5),
            ROL_ZP_X => cpu_write_func!(rol, read_zp_x, write_zp_x, 2, 6),
            ROL_ABS => cpu_write_func!(rol, read_abs, write_abs, 3, 6),
            ROL_ABS_X => cpu_write_func!(rol, read_abs_x, write_abs_x, 3, 7),
            ROR_A => cpu_write_func!(ror, read_a, write_a, 1, 2),
            ROR_ZP => cpu_write_func!(ror, read_zp, write_zp, 2, 5),
            ROR_ZP_X => cpu_write_func!(ror, read_zp_x, write_zp_x, 2, 6),
            ROR_ABS => cpu_write_func!(ror, read_abs, write_abs, 3, 6),
            ROR_ABS_X => cpu_write_func!(ror, read_abs_x, write_abs_x, 3, 7),
            RTI => {
                let v = self.pull_from_stack();
                self.cpu.s_r.from_byte(v);
                // Subtract one for the byte that will be added
                self.cpu.p_c = self.pull_from_stack_u16() - 1;
                Ok((1, 6))
            }
            RTS => {
                // We want to add one byte here, but that is done for us by the one byte we are returning
                self.cpu.p_c = self.pull_from_stack_u16();
                Ok((1, 6))
            }
            SBC_I => cpu_func!(sbc, read_immediate, 2, 2),
            SBC_ZP => cpu_func!(sbc, read_zp, 2, 3),
            SBC_ZP_X => cpu_func!(sbc, read_zp_x, 2, 4),
            SBC_ABS => cpu_func!(sbc, read_abs, 3, 4),
            SBC_ABS_X => cpu_func!(sbc, read_abs_x, pc_x, 3, 4, 5),
            SBC_ABS_Y => cpu_func!(sbc, read_abs_y, pc_y, 3, 4, 5),
            SBC_IND_X => cpu_func!(sbc, read_indexed_indirect, 2, 6),
            SBC_IND_Y => cpu_func!(sbc, read_indirect_indexed, pc_ind, 2, 5, 6),
            SEC => flag_func!(c, true),
            SEI => flag_func!(i, true),
            SED => flag_func!(d, true),
            STA_ZP => store_func!(a, write_zp, 2, 3),
            STA_ZP_X => store_func!(a, write_zp_x, 2, 4),
            STA_ABS => store_func!(a, write_abs, 3, 4),
            STA_ABS_X => store_func!(a, write_abs_x, 3, 5),
            STA_ABS_Y => store_func!(a, write_abs_y, 3, 5),
            STA_IND_X => store_func!(a, write_indexed_indirect, 2, 6),
            STA_IND_Y => store_func!(a, write_indirect_indexed, 2, 6),
            STX_ZP => store_func!(x, write_zp, 2, 3),
            STX_ZP_Y => store_func!(x, write_zp_y, 2, 4),
            STX_ABS => store_func!(x, write_abs, 3, 4),
            STY_ZP => store_func!(y, write_zp, 2, 3),
            STY_ZP_X => store_func!(y, write_zp_x, 2, 4),
            STY_ABS => store_func!(y, write_abs, 3, 4),
            TAX => transfer_func!(a, x),
            TAY => transfer_func!(a, y),
            TSX => transfer_func!(s_p, x),
            TXA => transfer_func!(x, a),
            // This one does not affect flags for some reason
            TXS => {
                self.cpu.s_p = self.cpu.x;
                Ok((1, 2))
            }
            TYA => transfer_func!(y, a),
            // Unofficial opcodes
            unofficial::ALR_I => {
                self.cpu.and(operands[0]);
                self.cpu.a = self.cpu.lsr(self.cpu.a);
                Ok((2, 2))
            }
            _ if unofficial::ANC_I.contains(opcode) => {
                self.cpu.and(operands[0]);
                self.cpu.s_r.c = self.cpu.s_r.n;
                Ok((2, 2))
            }
            unofficial::ARR_I => {
                self.cpu.and(operands[0]);
                self.cpu.a = self.cpu.ror(self.cpu.a);
                self.cpu.s_r.c = (self.cpu.a & 0x40) != 0;
                self.cpu.s_r.n = ((self.cpu.a & 0x40) != 0) ^ ((self.cpu.a & 0x20) != 0);
                Ok((2, 2))
            }
            unofficial::AXS_I => {
                let v = self.cpu.a & self.cpu.x;
                self.cpu.x = (self.cpu.a & self.cpu.x).wrapping_sub(operands[0]);
                self.cpu.s_r.z = self.cpu.x == 0;
                self.cpu.s_r.n = (self.cpu.x & 0x80) != 0;
                self.cpu.s_r.c = v > operands[0];
                Ok((2, 2))
            }
            unofficial::LAX_ZP => cpu_func!(lax, read_zp, 2, 3),
            unofficial::LAX_ZP_Y => cpu_func!(lax, read_zp_y, 2, 4),
            unofficial::LAX_ABS => cpu_func!(lax, read_abs, 3, 4),
            unofficial::LAX_ABS_Y => cpu_func!(lax, read_abs_y, 3, 4),
            unofficial::LAX_IND_X => cpu_func!(lax, read_indexed_indirect, 2, 6),
            unofficial::LAX_IND_Y => cpu_func!(lax, read_indirect_indexed, pc_ind, 2, 5, 6),
            unofficial::SAX_ZP => store_func!(self.cpu.a & self.cpu.x, write_zp, 2, 3),
            unofficial::SAX_ZP_Y => store_func!(self.cpu.a & self.cpu.x, write_zp_y, 2, 4),
            unofficial::SAX_ABS => store_func!(self.cpu.a & self.cpu.x, write_abs, 3, 4),
            unofficial::SAX_IND_X => {
                store_func!(self.cpu.a & self.cpu.x, write_indexed_indirect, 2, 6)
            }
            unofficial::DCP_ZP => cpu_write_func!(dcp, read_zp, write_zp, 2, 5),
            unofficial::DCP_ZP_X => cpu_write_func!(dcp, read_zp_x, write_zp_x, 2, 6),
            unofficial::DCP_ABS => cpu_write_func!(dcp, read_abs, write_abs, 3, 6),
            unofficial::DCP_ABS_X => cpu_write_func!(dcp, read_abs_x, write_abs_x, 3, 7),
            unofficial::DCP_ABS_Y => cpu_write_func!(dcp, read_abs_y, write_abs_y, 3, 7),
            unofficial::DCP_IND_X => {
                cpu_write_func!(dcp, read_indexed_indirect, write_indexed_indirect, 2, 8)
            }
            unofficial::DCP_IND_Y => {
                cpu_write_func!(dcp, read_indirect_indexed, write_indirect_indexed, 2, 8)
            }
            unofficial::ISC_ZP => cpu_write_func!(isc, read_zp, write_zp, 2, 5),
            unofficial::ISC_ZP_X => cpu_write_func!(isc, read_zp_x, write_zp_x, 2, 6),
            unofficial::ISC_ABS => cpu_write_func!(isc, read_abs, write_abs, 3, 6),
            unofficial::ISC_ABS_X => cpu_write_func!(isc, read_abs_x, write_abs_x, 3, 7),
            unofficial::ISC_ABS_Y => cpu_write_func!(isc, read_abs_y, write_abs_y, 3, 7),
            unofficial::ISC_IND_X => {
                cpu_write_func!(isc, read_indexed_indirect, write_indexed_indirect, 2, 8)
            }
            unofficial::ISC_IND_Y => {
                cpu_write_func!(isc, read_indirect_indexed, write_indirect_indexed, 2, 8)
            }
            unofficial::RLA_ZP => cpu_write_func!(rla, read_zp, write_zp, 2, 5),
            unofficial::RLA_ZP_X => cpu_write_func!(rla, read_zp_x, write_zp_x, 2, 6),
            unofficial::RLA_ABS => cpu_write_func!(rla, read_abs, write_abs, 3, 6),
            unofficial::RLA_ABS_X => cpu_write_func!(rla, read_abs_x, write_abs_x, 3, 7),
            unofficial::RLA_ABS_Y => cpu_write_func!(rla, read_abs_y, write_abs_y, 3, 7),
            unofficial::RLA_IND_X => {
                cpu_write_func!(rla, read_indexed_indirect, write_indexed_indirect, 2, 8)
            }
            unofficial::RLA_IND_Y => {
                cpu_write_func!(rla, read_indirect_indexed, write_indirect_indexed, 2, 8)
            }
            unofficial::RRA_ZP => cpu_write_func!(rra, read_zp, write_zp, 2, 5),
            unofficial::RRA_ZP_X => cpu_write_func!(rra, read_zp_x, write_zp_x, 2, 6),
            unofficial::RRA_ABS => cpu_write_func!(rra, read_abs, write_abs, 3, 6),
            unofficial::RRA_ABS_X => cpu_write_func!(rra, read_abs_x, write_abs_x, 3, 7),
            unofficial::RRA_ABS_Y => cpu_write_func!(rra, read_abs_y, write_abs_y, 3, 7),
            unofficial::RRA_IND_X => {
                cpu_write_func!(rra, read_indexed_indirect, write_indexed_indirect, 2, 8)
            }
            unofficial::RRA_IND_Y => {
                cpu_write_func!(rra, read_indirect_indexed, write_indirect_indexed, 2, 8)
            }
            unofficial::SLO_ZP => cpu_write_func!(slo, read_zp, write_zp, 2, 5),
            unofficial::SLO_ZP_X => cpu_write_func!(slo, read_zp_x, write_zp_x, 2, 6),
            unofficial::SLO_ABS => cpu_write_func!(slo, read_abs, write_abs, 3, 6),
            unofficial::SLO_ABS_X => cpu_write_func!(slo, read_abs_x, write_abs_x, 3, 7),
            unofficial::SLO_ABS_Y => cpu_write_func!(slo, read_abs_y, write_abs_y, 3, 7),
            unofficial::SLO_IND_X => {
                cpu_write_func!(slo, read_indexed_indirect, write_indexed_indirect, 2, 8)
            }
            unofficial::SLO_IND_Y => {
                cpu_write_func!(slo, read_indirect_indexed, write_indirect_indexed, 2, 8)
            }
            unofficial::SRE_ZP => cpu_write_func!(sre, read_zp, write_zp, 2, 5),
            unofficial::SRE_ZP_X => cpu_write_func!(sre, read_zp_x, write_zp_x, 2, 6),
            unofficial::SRE_ABS => cpu_write_func!(sre, read_abs, write_abs, 3, 6),
            unofficial::SRE_ABS_X => cpu_write_func!(sre, read_abs_x, write_abs_x, 3, 7),
            unofficial::SRE_ABS_Y => cpu_write_func!(sre, read_abs_y, write_abs_y, 3, 7),
            unofficial::SRE_IND_X => {
                cpu_write_func!(sre, read_indexed_indirect, write_indexed_indirect, 2, 8)
            }
            unofficial::SRE_IND_Y => {
                cpu_write_func!(sre, read_indirect_indexed, write_indirect_indexed, 2, 8)
            }
            unofficial::SBC => cpu_func!(sbc, read_immediate, 2, 2),
            _ if unofficial::NOPS.contains(opcode) => Ok((1, 2)),
            _ if unofficial::SKBS.contains(opcode) => Ok((2, 2)),
            _ if unofficial::IGN_ZP.contains(opcode) => Ok((2, 3)),
            _ if unofficial::IGN_ZP_X.contains(opcode) => Ok((2, 4)),
            unofficial::IGN_ABS => {
                self.read_abs(&operands);
                Ok((3, 4))
            }
            _ if unofficial::IGN_ABS_X.contains(opcode) => {
                if self.pc_x(operands) {
                    return Ok((3, 5));
                }
                Ok((3, 4))
            }
            _ => {
                return Err(format!(
                    "Unknown opcode '{:#04X}' at location '{:#04X}'",
                    opcode, self.cpu.p_c
                ))
            }
        }
    }
    /// Advance the NES by 1 frame, approx 29780 cycles.
    /// Just finishes rendering the frame if the NES is halfway done rendering one.
    /// Returns the total number of cycles ran.
    pub fn advance_frame(&mut self, settings: Option<Settings>) -> Result<u32, String> {
        let mut cycles = 0;
        loop {
            let scanline = self.ppu.scanline();
            let mut c = match self.step() {
                Ok(x) => x as u32,
                Err(e) => return Err(e),
            };
            // If we are in VBlank
            if self.ppu.in_vblank() {
                if self.check_oam_dma() {
                    c += CPU_CYCLES_PER_OAM as u32;
                }
            }
            self.apu.advance_cpu_cycles(c, &mut self.cartridge);
            self.cartridge.advance_cpu_cycles(c);
            cycles += c;
            if self
                .ppu
                .advance_dots(3 * c as u32, &self.cartridge, settings)
                && self.ppu.get_nmi_enabled()
            {
                self.on_nmi();
                cycles += 7;
                self.apu.advance_cpu_cycles(7, &mut self.cartridge);
                self.cartridge.advance_cpu_cycles(7);
                self.ppu.advance_dots(21, &self.cartridge, settings);
            }
            // If we have finished VBlank and are rendering the next frame
            if scanline != 0 && self.ppu.scanline() == 0 {
                break;
            }
        }
        Ok(cycles)
    }

    /// Check if the PPU's OAM DMA register has been set.
    /// If it has been, execute the DMA and reset the regsiter to None.
    /// Return `true` if the DMA is executed, and `false` otherwise.
    pub fn check_oam_dma(&mut self) -> bool {
        if let Some(dma_reg) = self.ppu.oam_dma {
            let addr = (dma_reg as usize) << 8;
            (0..0x100).for_each(|i| {
                let value = self.read_byte(addr + i);
                self.ppu.write_oam(0, value);
            });
            self.ppu.oam_dma = None;
            return true;
        }
        false
    }

    #[inline]
    fn read_immediate(&self, addr: &[u8]) -> u8 {
        addr[0]
    }
    #[inline]
    fn read_a(&self, _addr: &[u8]) -> u8 {
        self.cpu.a
    }
    fn write_a(&mut self, _addr: &[u8], value: u8) {
        self.cpu.a = value;
    }
    /// Read a single byte from a zero page address.
    /// ```
    /// let mut nes = yane::Nes::new();
    /// nes.read_zp(&[0x18]);
    /// ```
    pub fn read_zp(&mut self, addr: &[u8]) -> u8 {
        self.read_byte(addr[0] as usize)
    }
    /// Write a single byte to memory using zero page addressing.
    /// ```
    /// let mut nes = yane::Nes::new();
    /// nes.write_zp(&[0x18], 0x29);
    /// assert_eq!(nes.read_zp(&[0x18]), 0x29);
    /// ```
    pub fn write_zp(&mut self, addr: &[u8], val: u8) {
        self.write_byte(addr[0] as usize, val);
    }
    /// Read a single byte using zero page addressing with X register offset.
    /// ```
    /// let mut nes = yane::Nes::new();
    /// nes.write_zp(&[0x18], 0x45);
    /// nes.cpu.ldx(0x08);
    /// assert_eq!(nes.read_zp_x(&[0x10]), 0x45);
    /// ```
    pub fn read_zp_x(&mut self, addr: &[u8]) -> u8 {
        self.read_zp_offset(addr[0], self.cpu.x)
    }
    /// Read a single byte using zero page addressing with Y register offset.
    /// ```
    /// let mut nes = yane::Nes::new();
    /// nes.write_zp(&[0x18], 0x45);
    /// nes.cpu.ldy(0x08);
    /// assert_eq!(nes.read_zp_y(&[0x10]), 0x45);
    /// ```
    pub fn read_zp_y(&mut self, addr: &[u8]) -> u8 {
        self.read_zp_offset(addr[0], self.cpu.y)
    }
    // Read a single byte using zero page offset addressing
    fn read_zp_offset(&mut self, addr: u8, offset: u8) -> u8 {
        self.read_byte(addr.wrapping_add(offset) as usize)
    }
    /// Write a single byte using zero page addressing with X register offset
    /// ```
    /// let mut nes = yane::Nes::new();
    /// nes.cpu.x = 0x10;
    /// nes.write_zp_x(&[0x18], 0x05);
    /// assert_eq!(nes.read_zp(&[0x28]), 0x05);
    /// ```
    pub fn write_zp_x(&mut self, addr: &[u8], value: u8) {
        self.write_zp_offset(addr[0], self.cpu.x, value)
    }
    /// Write a single byte using zero page addressing with Y register offset
    pub fn write_zp_y(&mut self, addr: &[u8], value: u8) {
        self.write_zp_offset(addr[0], self.cpu.y, value)
    }
    // Write a single byte using zero page offset addressing
    fn write_zp_offset(&mut self, addr: u8, offset: u8, value: u8) {
        self.write_byte(addr.wrapping_add(offset) as usize, value)
    }
    // Absolute addressing
    fn get_absolute_addr_offset(addr: &[u8], offset: u8) -> usize {
        (addr[0] as u16 + ((addr[1] as u16) << 8)).wrapping_add(offset as u16) as usize
    }
    fn get_absolute_addr(addr: &[u8]) -> usize {
        Nes::get_absolute_addr_offset(addr, 0)
    }
    /// Read a single byte from memory using absolute addressing.
    /// Note that absolute addressing uses a little endian system.
    /// ```
    /// let mut nes = yane::Nes::new();
    /// nes.mem[0x0034] = 0x56;
    /// assert_eq!(nes.read_abs(&[0x34, 0x00]), 0x56);
    /// ```
    pub fn read_abs(&mut self, addr: &[u8]) -> u8 {
        self.read_byte(Nes::get_absolute_addr(addr))
    }
    /// Write a single byte to memory using absolute addressing
    /// Note that absolute addressing uses a little endian system.
    /// ```
    /// let mut nes = yane::Nes::new();
    /// nes.write_abs(&[0x12, 0x00], 0x56);
    /// assert_eq!(nes.mem[0x0012], 0x56);
    /// ```
    pub fn write_abs(&mut self, addr: &[u8], value: u8) {
        self.write_byte(Nes::get_absolute_addr(addr), value)
    }
    // Read using absolute addressing with an offset
    fn read_abs_offset(&mut self, addr: &[u8], offset: u8) -> u8 {
        self.read_byte(Nes::get_absolute_addr_offset(addr, offset))
    }
    fn write_abs_offset(&mut self, addr: &[u8], offset: u8, value: u8) {
        self.write_byte(Nes::get_absolute_addr_offset(addr, offset), value)
    }
    /// Read a byte from memory using absolute addressing with X register offset.
    /// ```
    /// let mut nes = yane::Nes::new();
    /// nes.read_abs_x(&[0x12, 0x00]);
    /// ```
    pub fn read_abs_x(&mut self, addr: &[u8]) -> u8 {
        self.read_abs_offset(addr, self.cpu.x)
    }
    /// Read a byte from memory using absolute addressing with Y register offset.
    /// ```
    /// let mut nes = yane::Nes::new();
    /// nes.read_abs_y(&[0x12, 0x00]);
    /// ```
    pub fn read_abs_y(&mut self, addr: &[u8]) -> u8 {
        self.read_abs_offset(addr, self.cpu.y)
    }
    /// Write a byte to memory using absolute addressing with X register offset.
    /// ```
    /// let mut nes = yane::Nes::new();
    /// nes.write_abs_x(&[0x12, 0x34], 0x56);
    /// ```
    pub fn write_abs_x(&mut self, addr: &[u8], value: u8) {
        self.write_abs_offset(addr, self.cpu.x, value)
    }
    /// Write a byte to memory using absolute addressing with Y register offset.
    /// ```
    /// let mut nes = yane::Nes::new();
    /// nes.write_abs_y(&[0x12, 0x34], 0x56);
    /// ```
    pub fn write_abs_y(&mut self, addr: &[u8], value: u8) {
        self.write_abs_offset(addr, self.cpu.y, value)
    }
    /// Read a single byte from memory using indexed indirect addressing.
    /// A 2 byte value is read from the zero page address `addr`, which the X register is added to.
    /// This value is then used as a little endian address of the actual value.
    /// ```
    /// let mut nes = yane::Nes::new();
    /// nes.read_indexed_indirect(&[0x12]);
    /// ```
    pub fn read_indexed_indirect(&mut self, addr: &[u8]) -> u8 {
        let first_addr = addr[0].wrapping_add(self.cpu.x);
        let second_addr = [
            self.read_byte(first_addr as usize),
            self.read_byte(first_addr.wrapping_add(1) as usize),
        ];
        return self.read_abs(&second_addr);
    }
    /// Write a single byte using indexed indirect addressing
    pub fn write_indexed_indirect(&mut self, addr: &[u8], value: u8) {
        let first_addr = addr[0].wrapping_add(self.cpu.x);
        let second_addr = [
            self.read_byte(first_addr as usize),
            self.read_byte(first_addr.wrapping_add(1) as usize),
        ];
        self.write_abs(&second_addr, value);
    }
    fn indirect_indexed_addr(&mut self, addr: &[u8]) -> usize {
        let first_addr = addr[0];
        (self.read_byte(first_addr as usize) as u16
            + ((self.read_byte(first_addr.wrapping_add(1) as usize) as u16) << 8))
            .wrapping_add(self.cpu.y as u16) as usize
    }
    /// Read a single byte from memory using indirect indexed addressing.
    /// A 2 byte value is read from the zero page address `addr`.
    /// The Y value is then added to this value, and the result is used as the little endian address of the actual value.
    /// ```
    /// let mut nes = yane::Nes::new();
    /// nes.read_indirect_indexed(&[0x18]);
    /// ```
    pub fn read_indirect_indexed(&mut self, addr: &[u8]) -> u8 {
        let addr = self.indirect_indexed_addr(addr);
        return self.read_byte(addr as usize);
    }
    /// Write a single byte to memory using indirect indexed addressing.
    /// ```
    /// let mut nes = yane::Nes::new();
    /// nes.write_indirect_indexed(&[0x12], 0x34);
    /// assert_eq!(nes.read_indirect_indexed(&[0x12]), 0x34);
    /// ```
    pub fn write_indirect_indexed(&mut self, addr: &[u8], value: u8) {
        let addr = self.indirect_indexed_addr(addr);
        self.write_byte(addr as usize, value)
    }
    // Return true if a page is crossed by an operation using the absolute address and offset given
    // addr is in little endian form
    fn page_crossed_abs(addr: &[u8], offset: u8) -> bool {
        255 - addr[0] < offset
    }
    // Returns true if a page cross occurs when reading the absolute address given with the X register offset
    fn pc_x(&self, addr: &[u8]) -> bool {
        Nes::page_crossed_abs(addr, self.cpu.x)
    }
    // Returns true if a page cross occurs when reading the absolute address given with the Y register offset
    fn pc_y(&self, addr: &[u8]) -> bool {
        Nes::page_crossed_abs(addr, self.cpu.y)
    }
    // Return true if a page is crossed by the indirect indexed address and offset given
    fn page_crossed_ind_idx(&mut self, addr: &[u8], offset: u8) -> bool {
        255 - self.read_zp(addr) < offset
    }
    // Returns true if a page cross occurs when reading the indirect indexed address given with the Y register offset
    fn pc_ind(&mut self, addr: &[u8]) -> bool {
        self.page_crossed_ind_idx(addr, self.cpu.y)
    }
    fn push_to_stack(&mut self, v: u8) {
        self.write_byte(0x100 + self.cpu.s_p as usize, v);
        self.cpu.s_p = self.cpu.s_p.wrapping_sub(1);
    }
    fn pull_from_stack(&mut self) -> u8 {
        self.cpu.s_p = self.cpu.s_p.wrapping_add(1);
        self.mem[0x100 + self.cpu.s_p as usize]
    }
    fn push_to_stack_u16(&mut self, v: u16) {
        self.push_to_stack((v >> 8) as u8);
        self.push_to_stack((v & 0xFF) as u8);
    }
    fn pull_from_stack_u16(&mut self) -> u16 {
        (self.pull_from_stack() as u16) + ((self.pull_from_stack() as u16) << 8)
    }
}
