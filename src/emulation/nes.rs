use crate::{opcodes::*, Apu, Cartridge, Controller, Cpu, Ppu};

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
    controllers: [Controller; 2],
    // Cached controller states, the ROM will need to poll to keep these up to date
    cached_controllers: [Controller; 2],
    // Current bit being read from the controller
    controller_bits: [usize; 2],
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
            cartridge: Cartridge::new(c.as_slice()),
            controllers: [Controller::new(); 2],
            cached_controllers: [Controller::new(); 2],
            controller_bits: [0; 2],
        }
    }
    pub fn from_cartridge(bytes: &[u8]) -> Nes {
        let mut nes = Nes {
            cpu: Cpu::new(),
            ppu: Ppu::new(),
            apu: Apu::new(),
            mem: [0x00; 0x800],
            cartridge: Cartridge::new(bytes),
            controllers: [Controller::new(); 2],
            cached_controllers: [Controller::new(); 2],
            controller_bits: [0; 2],
        };
        nes.cpu.p_c = ((nes.cartridge.read_byte(0xFFFD) as u16) << 8)
            + (nes.cartridge.read_byte(0xFFFC) as u16);
        println!("Initialized PC to {:#X}", nes.cpu.p_c);
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
            0x4017 => return self.read_controller_bit(1),
            // TBA
            0x4000..0x4020 => 0,
            0x4020..0x10000 => self.cartridge.read_byte(addr),
            _ => panic!("Invalid read address provided: {:#X}", addr),
        };
    }
    fn write_byte(&mut self, addr: usize, value: u8) {
        match addr {
            0..0x2000 => self.mem[addr % 0x0800] = value,
            0x2000..0x4000 => self.ppu.write_byte(addr, value, &mut self.cartridge),
            0x4014 => {
                // Perform DMA
                // TODO: Make this better
                let addr = (value as usize) << 8;
                for i in 0..0x100 {
                    self.ppu.oam[i] = self.read_byte(addr + i);
                }
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
            0x4020..0x10000 => self.cartridge.write_byte(addr, value),
            _ => panic!("Invalid write address provided: {:#X}", addr),
        };
    }

    /// Update the internal controller state in thte NES.
    /// The ROM will still have to poll for the controller state.
    /// `num` should either be 0 or 1, depending on whose controller state is being updated
    pub fn set_input(&mut self, num: usize, state: Controller) {
        self.controllers[num] = state;
    }

    pub fn step(&mut self) -> Result<i64, String> {
        let pc = self.cpu.p_c as usize;
        let mut inst: [u8; 3] = [0; 3];
        inst.copy_from_slice(&[
            self.read_byte(pc),
            self.read_byte(pc + 1),
            self.read_byte(pc + 2),
        ]);
        match self.decode_and_execute(&inst) {
            Ok((bytes, cycles)) => {
                self.cpu.p_c += bytes;
                return Ok(cycles);
            }
            Err(s) => return Err(s),
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
                self.push_to_stack_u16(self.cpu.p_c);
                self.push_to_stack(self.cpu.s_r.to_byte());
                // self.mem[(0x100 + self.cpu.s_p as usize - 2)..(0x100 + self.cpu.s_p as usize + 1)]
                //     .copy_from_slice(&to_push);
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
                self.cpu.p_c = self.pull_from_stack_u16() - 1;
                Ok((1, 6))
            }
            RTS => {
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
            unofficial::ANC_I => {
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
            unofficial::IGN_ABS => Ok((3, 4)),
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
    /// Read a single byte from memory using indirect indexed addressing.
    /// A 2 byte value is read from the zero page address `addr`.
    /// The Y value is then added to this value, and the result is used as the little endian address of the actual value.
    /// ```
    /// let mut nes = yane::Nes::new();
    /// nes.read_indirect_indexed(&[0x18]);
    /// ```
    pub fn read_indirect_indexed(&mut self, addr: &[u8]) -> u8 {
        let first_addr = addr[0];
        let second_addr = (self.read_byte(first_addr as usize) as u16
            + ((self.read_byte(first_addr.wrapping_add(1) as usize) as u16) << 8))
            .wrapping_add(self.cpu.y as u16);
        return self.read_byte(second_addr as usize);
    }
    /// Write a single byte to memory using indirect indexed addressing.
    /// ```
    /// let mut nes = yane::Nes::new();
    /// nes.write_indirect_indexed(&[0x12], 0x34);
    /// assert_eq!(nes.read_indirect_indexed(&[0x12]), 0x34);
    /// ```
    pub fn write_indirect_indexed(&mut self, addr: &[u8], value: u8) {
        let first_addr = addr[0] as usize;
        let second_addr = (self.read_byte(first_addr) as u16
            + ((self.read_byte(first_addr.wrapping_add(1)) as u16) << 8))
            .wrapping_add(self.cpu.y as u16)
            .to_owned();
        self.write_byte(second_addr as usize, value)
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
    fn to_bytes(v: u16) -> [u8; 2] {
        [(v & 0xFF) as u8, (v >> 8) as u8]
    }
}

#[cfg(test)]
mod tests {
    use rand::random;
    use std::cmp::{max, min};

    use super::Nes;
    use crate::{opcodes::*, StatusRegister};
    use assert_hex::assert_eq_hex;

    #[test]
    fn test_init() {
        // Should not throw
        Nes::new();
    }
    // Macros used to generate basic test cases
    macro_rules! test_immediate {
        ($opcode: ident) => {
            #[test]
            fn test_immediate() {
                run_test(|nes, v| {
                    assert_eq!(nes.decode_and_execute(&[$opcode, v]), Ok((2, 2)));
                })
            }
        };
    }
    macro_rules! test_zp {
        ($opcode: ident) => {
            #[test]
            fn test_zp() {
                run_test(|nes, v| {
                    let addr = set_addr_zp(nes, v);
                    assert_eq!(nes.decode_and_execute(&[$opcode, addr[0]]), Ok((2, 3)));
                })
            }
        };
    }
    macro_rules! test_zp_offset {
        ($opcode: ident, $off_reg: ident) => {
            run_test(|nes, v| {
                let addr = random::<u8>();
                nes.cpu.$off_reg = random::<u8>();
                nes.mem[addr.wrapping_add(nes.cpu.$off_reg) as usize] = v;
                assert_eq!(nes.decode_and_execute(&[$opcode, addr]), Ok((2, 4)));
            })
        };
    }
    macro_rules! test_zp_x {
        ($opcode: ident) => {
            #[test]
            fn test_zp_x() {
                test_zp_offset!($opcode, x);
            }
        };
    }
    macro_rules! test_zp_y {
        ($opcode: ident) => {
            #[test]
            fn test_zp_y() {
                test_zp_offset!($opcode, y);
            }
        };
    }
    macro_rules! test_absolute {
        ($opcode: ident) => {
            #[test]
            fn test_absolute() {
                run_test(|nes, v| {
                    let addr = set_addr_abs(nes, v);
                    assert_eq!(
                        nes.decode_and_execute(&[$opcode, addr[0], addr[1]]),
                        Ok((3, 4))
                    );
                })
            }
        };
    }
    macro_rules! test_absolute_offset {
        ($opcode: ident, $off_reg: ident) => {
            run_test(|nes, v| {
                let addr = random::<u16>() & 0xFF;
                nes.cpu.$off_reg = random::<u8>();
                nes.mem[addr.wrapping_add(nes.cpu.$off_reg as u16) as usize] = v;
                nes.decode_and_execute(&[$opcode, first_byte(addr), second_byte(addr)])
                    .unwrap();
            })
        };
    }
    macro_rules! test_absolute_x {
        ($opcode: ident) => {
            #[test]
            fn test_absolute_x() {
                test_absolute_offset!($opcode, x);
            }
        };
    }
    macro_rules! test_absolute_y {
        ($opcode: ident) => {
            #[test]
            fn test_absolute_y() {
                test_absolute_offset!($opcode, y);
            }
        };
    }
    macro_rules! test_indexed_indirect {
        ($opcode: ident) => {
            #[test]
            fn test_indexed_indirect() {
                run_test(|nes, v| {
                    let addr = random::<u16>() & 0xFF;
                    nes.mem[addr as usize] = v;
                    let mut operand = random::<u8>();
                    nes.cpu.x = random::<u8>();
                    let mut second_addr = operand.wrapping_add(nes.cpu.x);
                    // Avoid collisions
                    if second_addr as u16 == addr || second_addr as u16 == addr.wrapping_sub(1) {
                        second_addr = second_addr.wrapping_add(2);
                        operand = operand.wrapping_add(2);
                    }
                    nes.mem[second_addr as usize] = first_byte(addr);
                    nes.mem[second_addr as usize + 1] = second_byte(addr);
                    assert_eq!(nes.decode_and_execute(&[$opcode, operand]), Ok((2, 6)));
                });
            }
        };
    }
    macro_rules! test_indirect_indexed {
        ($opcode: ident) => {
            #[test]
            fn test_indirect_indexed() {
                run_test(|nes, v| {
                    let addr = random::<u16>() & 0xFF;
                    nes.cpu.y = random::<u8>();
                    nes.mem[addr.wrapping_add(nes.cpu.y as u16) as usize] = v;
                    let mut operand = random::<u8>();
                    if operand as u16 == addr || operand as u16 == addr.wrapping_sub(1) {
                        operand = operand.wrapping_add(2);
                    }
                    nes.mem[operand as usize] = first_byte(addr);
                    nes.mem[operand as usize + 1] = second_byte(addr);
                    nes.decode_and_execute(&[$opcode, operand]).unwrap();
                })
            }
        };
    }
    mod lda {
        use super::*;
        fn run_test<F: Fn(&mut Nes, u8)>(test: F) {
            let mut nes = Nes::new();
            let v = random::<u8>();
            test(&mut nes, v);
            assert_eq_hex!(nes.cpu.a, v);
        }
        test_immediate!(LDA_I);
        test_zp!(LDA_ZP);
        test_zp_x!(LDA_ZP_X);
        test_absolute!(LDA_ABS);
        test_absolute_x!(LDA_ABS_X);
        test_absolute_y!(LDA_ABS_Y);
        test_indexed_indirect!(LDA_IND_X);
        test_indirect_indexed!(LDA_IND_Y);
    }
    mod ldx {
        use super::*;
        fn run_test<F: Fn(&mut Nes, u8)>(test: F) {
            let mut nes = Nes::new();
            let v = random::<u8>();
            test(&mut nes, v);
            assert_eq_hex!(nes.cpu.x, v);
        }
        test_immediate!(LDX_I);
        test_zp!(LDX_ZP);
        test_zp_y!(LDX_ZP_Y);
        test_absolute!(LDX_ABS);
        test_absolute_y!(LDX_ABS_Y);
    }
    mod ldy {
        use super::*;
        fn run_test<F: Fn(&mut Nes, u8)>(test: F) {
            let mut nes = Nes::new();
            let v = random::<u8>();
            test(&mut nes, v);
            assert_eq_hex!(nes.cpu.y, v);
        }
        test_immediate!(LDY_I);
        test_zp!(LDY_ZP);
        test_zp_x!(LDY_ZP_X);
        test_absolute!(LDY_ABS);
        test_absolute_x!(LDY_ABS_X);
    }
    mod adc {
        use super::*;
        fn run_test<F: Fn(&mut Nes, u8)>(test: F) {
            let mut nes = Nes::new();
            nes.cpu.a = random::<u8>();
            let v = random::<u8>();
            let exp = nes.cpu.a.wrapping_add(v);
            test(&mut nes, v);
            assert_eq_hex!(nes.cpu.a, exp);
        }
        test_immediate!(ADC_I);
        test_zp!(ADC_ZP);
        test_zp_x!(ADC_ZP_X);
        test_absolute!(ADC_ABS);
        test_absolute_x!(ADC_ABS_X);
        test_absolute_y!(ADC_ABS_Y);
        test_indexed_indirect!(ADC_IND_X);
        test_indirect_indexed!(ADC_IND_Y);
    }
    mod and {
        use super::*;
        fn run_test<F: Fn(&mut Nes, u8)>(f: F) {
            let mut nes = Nes::new();
            let v_one = random::<u8>();
            nes.cpu.a = v_one;
            let v_two = random::<u8>();
            f(&mut nes, v_two);
            assert_eq_hex!(nes.cpu.a, v_one & v_two);
        }
        test_immediate!(AND_I);
        test_zp!(AND_ZP);
        test_zp_x!(AND_ZP_X);
        test_absolute!(AND_ABS);
        test_absolute_x!(AND_ABS_X);
        test_absolute_y!(AND_ABS_Y);
        test_indexed_indirect!(AND_IND_X);
        test_indirect_indexed!(AND_IND_Y);
    }
    mod asl {
        use super::*;
        use test_case::test_case;

        macro_rules! check_flags {
            ($nes: ident, $zero: expr, $negative: expr, $carry: expr) => {
                assert_eq!(
                    $nes.cpu.s_r.z,
                    $zero,
                    "zero should be {}",
                    if $zero { "set" } else { "unset" }
                );
                assert_eq!(
                    $nes.cpu.s_r.n,
                    $negative,
                    "negative should be {}",
                    if $negative { "set " } else { "unset" }
                );
                assert_eq!(
                    $nes.cpu.s_r.c,
                    $carry,
                    "carry should be {}",
                    if $carry { "set " } else { "unset" }
                );
            };
        }

        #[test_case(0x18, 0x30, false, false, false ; "happy case")]
        #[test_case(0x45, 0x8A, false, true, false ; "negative is set")]
        #[test_case(0x88, 0x10, false, false, true ; "carry is set")]
        #[test_case(0x80, 0x00, true, false, true; "zero is set")]
        fn test_accumulator(value: u8, shifted: u8, zero: bool, negative: bool, carry: bool) {
            let mut nes = Nes::new();
            nes.cpu.a = value;
            assert_eq!(nes.decode_and_execute(&[ASL_A]), Ok((1, 2)));
            assert_eq_hex!(nes.cpu.a, shifted, "shifted is correct");
            check_flags!(nes, zero, negative, carry);
        }
        #[test_case(0x01, 0x02, false, false, false ; "happy case")]
        #[test_case(0x44, 0x88, false, true, false ; "negative is set")]
        #[test_case(0x00, 0x00, true, false, false ; "zero is set")]
        #[test_case(0x8A, 0x14, false, false, true ; "carry is set")]
        fn test_zp(value: u8, shifted: u8, zero: bool, negative: bool, carry: bool) {
            let mut nes = Nes::new();
            let addr = set_addr_zp(&mut nes, value);
            assert_eq!(nes.decode_and_execute(&[ASL_ZP, addr[0]]), Ok((2, 5)));
            assert_eq_hex!(nes.mem[addr[0] as usize], shifted);
            check_flags!(nes, zero, negative, carry);
        }
        #[test_case(0x33, 0x66, false, false, false ; "happy case")]
        #[test_case(0x45, 0x8A, false, true, false ; "negative set")]
        #[test_case(0x8F, 0x1E, false, false, true ; "carry set")]
        #[test_case(0x00, 0x00, true, false, false ; "zero set")]
        fn test_zp_x(value: u8, shifted: u8, zero: bool, negative: bool, carry: bool) {
            let mut nes = Nes::new();
            let x_value = nes.cpu.x;
            nes.cpu.x = x_value;
            let addr = set_addr_zp_offset(&mut nes, value, x_value);
            assert_eq!(nes.decode_and_execute(&[ASL_ZP_X, addr[0]]), Ok((2, 6)));
            assert_eq_hex!(nes.mem[addr[0] as usize], shifted);
            check_flags!(nes, zero, negative, carry);
        }
        #[test_case(0x08, 0x10, false, false, false ; "happy case")]
        #[test_case(0x48, 0x90, false, true, false ; "negative set")]
        #[test_case(0x88, 0x10, false, false, true ; "carry set")]
        #[test_case(0x00, 0x00, true, false, false ; "zero set")]
        fn test_abs(value: u8, shifted: u8, zero: bool, negative: bool, carry: bool) {
            let mut nes = Nes::new();
            let addr = set_addr_abs(&mut nes, value);
            assert_eq!(
                nes.decode_and_execute(&[ASL_ABS, addr[0], addr[1]]),
                Ok((3, 6))
            );
            assert_eq_hex!(nes.mem[addr_from_bytes(addr)], shifted);
            check_flags!(nes, zero, negative, carry);
        }
        #[test_case(0x07, 0x0E, false, false, false ; "happy case")]
        #[test_case(0x00, 0x00, true, false, false ; "zero set")]
        #[test_case(0x45, 0x8A, false, true, false ; "negative set")]
        #[test_case(0x86, 0x0C, false, false, true ; "carry set")]
        fn test_abs_x(value: u8, shifted: u8, zero: bool, negative: bool, carry: bool) {
            let mut nes = Nes::new();
            let addr = set_addr_abs_x(&mut nes, value);
            assert_eq!(
                nes.decode_and_execute(&[ASL_ABS_X, addr[0], addr[1]]),
                Ok((3, 7))
            );
            assert_eq_hex!(
                nes.mem[Nes::get_absolute_addr_offset(&addr, nes.cpu.x)],
                shifted
            );
            check_flags!(nes, zero, negative, carry);
        }
    }
    macro_rules! branch_tests {
        ($name: ident, $opcode: ident, $flag: ident, $value: expr) => {
            mod $name {
                use super::*;
                use test_case::test_case;
                #[test_case(true, 0x12, 0x34, 0x46, 3 ; "branched")]
                #[test_case(false, 0x12, 0x34, 0x12, 2 ; "doesn't branch")]
                #[test_case(true, 0x18, 0x00, 0x18, 3 ; "branches to same location")]
                #[test_case(true, 0x00ff, 0x05, 0x0104, 3 ; "branches to a different page")]
                fn test_implied(
                    should_branch: bool,
                    pc: u16,
                    operand: u8,
                    new_pc: u16,
                    cycles: i64,
                ) {
                    let mut nes = Nes::new();
                    nes.cpu.p_c = pc;
                    nes.cpu.s_r.$flag = if should_branch { $value } else { !$value };
                    assert_eq!(nes.decode_and_execute(&[$opcode, operand]), Ok((2, cycles)));
                    assert_eq_hex!(nes.cpu.p_c, new_pc);
                }
            }
        };
    }
    branch_tests!(bcs, BCS, c, true);
    branch_tests!(bcc, BCC, c, false);
    branch_tests!(beq, BEQ, z, true);
    branch_tests!(bne, BNE, z, false);
    branch_tests!(bmi, BMI, n, true);
    branch_tests!(bpl, BPL, n, false);
    branch_tests!(bvs, BVS, v, true);
    branch_tests!(bvc, BVC, v, false);
    mod bit {
        use super::*;
        use test_case::test_case;
        macro_rules! bit_test {
            ($name: ident, $opcode: ident, $addr_func: ident, $result: expr) => {
                #[test_case(0x18, 0x27, true, false, false; "should set the zero flag")]
                #[test_case(0x18, 0x1F, false, false, false; "should clear the zero flag")]
                #[test_case(0x12, 0x74, false, true, false; "should set V")]
                #[test_case(0x11, 0x80, true, false, true; "should set N")]
                #[test_case(0x18, 0xFF, false, true, true ; "should set Z and N flag")]
                fn $name(a: u8, value: u8, z: bool, v: bool, n: bool) {
                    let mut nes = Nes::new();
                    nes.cpu.a = a;
                    let addr = $addr_func(&mut nes, value);
                    assert_eq!(
                        nes.decode_and_execute(&prepend_with_opcode($opcode, &addr)),
                        Ok($result)
                    );
                    assert_eq_hex!(nes.cpu.a, a, "A is changed");
                    assert_eq!(nes.cpu.s_r.z, z, "Z is wrong");
                    assert_eq!(nes.cpu.s_r.v, v, "V is wrong");
                    assert_eq!(nes.cpu.s_r.n, n, "N is wrong");
                }
            };
        }
        bit_test!(test_zero_page, BIT_ZP, set_addr_zp, (2, 3));
        bit_test!(test_absolute, BIT_ABS, set_addr_abs, (3, 4));
    }
    // mod brk {
    //     use super::*;
    //     use test_case::test_case;
    //     #[test_case(
    //         0x1234, 0x4567, true, false, true, false, true, false, true, 0b10110101 ; "happy case"
    //     )]
    //     #[test_case(0xFFFF, 0x0000, true, true, true, true, true, true, true, 0b11111111 ; "all flags true")]
    //     #[test_case(0xAABB, 0xBDF1, false, false, false, false, false, false, false, 0b00100000 ; "all flags false")]
    //     #[test_case(0x6789, 0x6789, false, false, true, false, true, true, true, 0b00110111 ; "no change in PC")]
    //     fn test_implied(
    //         init_pc: u16,
    //         final_pc: u16,
    //         n: bool,
    //         v: bool,
    //         b: bool,
    //         d: bool,
    //         i: bool,
    //         z: bool,
    //         c: bool,
    //         sr: u8,
    //     ) {
    //         let mut nes = Nes::new();
    //         nes.cpu.s_r.n = n;
    //         nes.cpu.s_r.v = v;
    //         nes.cpu.s_r.b = b;
    //         nes.cpu.s_r.d = d;
    //         nes.cpu.s_r.i = i;
    //         nes.cpu.s_r.z = z;
    //         nes.cpu.s_r.c = c;
    //         nes.cpu.p_c = init_pc;
    //         // Set memeory to be read into PC
    //         nes.mem[0xFFFE] = first_byte(final_pc);
    //         nes.mem[0xFFFF] = second_byte(final_pc);
    //         assert_eq!(nes.decode_and_execute(&[BRK]), Ok((1, 7)));
    //         // Check flag is set
    //         assert_eq!(nes.cpu.s_r.i, true);
    //         // Check PC was set
    //         assert_eq_hex!(nes.cpu.p_c, final_pc);
    //         // Check stuff was pushed onto stack
    //         assert_eq_hex!(nes.mem[0x1FD], first_byte(init_pc));
    //         assert_eq_hex!(nes.mem[0x1FE], second_byte(init_pc));
    //         assert_eq_hex!(nes.mem[0x1FF], sr);
    //     }
    // }
    macro_rules! test_clear {
        ($name: ident, $opcode: ident, $flag: ident) => {
            #[test]
            fn $name() {
                let mut nes = Nes::new();
                nes.cpu.s_r.$flag = true;
                assert_eq!(nes.decode_and_execute(&[$opcode]), Ok((1, 2)));
                assert_eq!(nes.cpu.s_r.c, false);
            }
        };
    }
    test_clear!(test_clc, CLC, c);
    test_clear!(test_cli, CLI, i);
    test_clear!(test_clv, CLV, v);
    test_clear!(test_cld, CLD, d);
    macro_rules! compare_test {
        ($reg: ident, $opcode: ident, $addr_func: ident, $cycles: expr, $bytes: expr, $test_name: ident) => {
            #[test_case(0x12, 0x11, true, false, false ; "Should set the carry bit")]
            #[test_case(0x45, 0x45, true, true, false ; "Should set the zero flag")]
            #[test_case(0x00, 0x01, false, false, true ; "Should set the negative flag")]
            #[test_case(0x80, 0xFF, false, false, true ; "Should set N two")]
            #[test_case(0x7F, 0x00, true, false, false ; "Should set C two")]
            #[test_case(0x8F, 0x00, true, false, true; "Should set C and N")]
            fn $test_name(reg_value: u8, comp_value: u8, c: bool, z: bool, n: bool) {
                let mut nes = Nes::new();
                nes.cpu.$reg = reg_value;
                let addr = $addr_func(&mut nes, comp_value);
                assert_eq!(
                    nes.decode_and_execute(&prepend_with_opcode($opcode, &addr)),
                    Ok(($cycles, $bytes))
                );
                assert_eq!(
                    nes.cpu.s_r.c,
                    c,
                    "C should be {}",
                    if c { "set" } else { "unset" }
                );
                assert_eq!(
                    nes.cpu.s_r.z,
                    z,
                    "Z should be {}",
                    if z { "set" } else { "unset" }
                );
                assert_eq!(
                    nes.cpu.s_r.n,
                    n,
                    "N should be {}",
                    if n { "set" } else { "unset" }
                );
            }
        };
    }
    mod cpm {
        use super::*;
        use test_case::test_case;
        compare_test!(a, CMP_I, set_addr_i, 2, 2, test_immediate);
        compare_test!(a, CMP_ZP, set_addr_zp, 2, 3, test_zp);
        compare_test!(a, CMP_ZP_X, set_addr_zp_x, 2, 4, test_zp_x);
        compare_test!(a, CMP_ABS, set_addr_abs, 3, 4, test_abs);
        compare_test!(a, CMP_ABS_X, set_addr_abs_x, 3, 4, test_abs_x);
        compare_test!(a, CMP_ABS_X, set_addr_abs_x_pc, 3, 5, test_abs_x_pc);
        compare_test!(a, CMP_ABS_Y, set_addr_abs_y, 3, 4, test_abs_y);
        compare_test!(a, CMP_ABS_Y, set_addr_abs_y_pc, 3, 5, test_abs_y_pc);
        compare_test!(a, CMP_IND_X, set_addr_ind_x, 2, 6, test_ind_x);
        compare_test!(a, CMP_IND_Y, set_addr_ind_y, 2, 5, test_ind_y);
    }
    mod cpx {
        use super::*;
        use test_case::test_case;
        compare_test!(x, CPX_I, set_addr_i, 2, 2, test_immediate);
        compare_test!(x, CPX_ZP, set_addr_zp, 2, 3, test_zp);
        compare_test!(x, CPX_ABS, set_addr_abs, 3, 4, test_abs);
    }
    mod cpy {
        use super::*;
        use test_case::test_case;
        compare_test!(y, CPY_I, set_addr_i, 2, 2, test_immediate);
        compare_test!(y, CPY_ZP, set_addr_zp, 2, 3, test_zp);
        compare_test!(y, CPY_ABS, set_addr_abs, 3, 4, test_abs);
    }
    macro_rules! dec_test {
        ($opcode: ident, $get_addr: ident, $set_addr: ident, $cycles: expr, $bytes: expr, $test_name: ident) => {
            #[test_case(0x12, 0x11, false, false; "happy case")]
            #[test_case(0x01, 0x00, true, false ; "should set z")]
            #[test_case(0x00, 0xFF, false, true ; "should wrap")]
            #[test_case(0x81, 0x80, false, true ; "should set n")]
            #[test_case(0x80, 0x7F, false, false ; "should set neither")]
            fn $test_name(pre_val: u8, post_val: u8, z: bool, n: bool) {
                let mut nes = Nes::new();
                let addr = $set_addr(&mut nes, pre_val);
                assert_eq!(
                    nes.decode_and_execute(&prepend_with_opcode($opcode, &addr)),
                    Ok(($cycles, $bytes)),
                );
                assert_eq_hex!($get_addr(&nes, &addr), post_val);
                assert_z(&nes, z);
                assert_n(&nes, n);
            }
        };
    }
    mod dec {
        use super::*;
        use test_case::test_case;
        dec_test!(DEC_ZP, get_addr_zp, set_addr_zp, 2, 5, test_zp);
        dec_test!(DEC_ZP_X, get_addr_zp_x, set_addr_zp_x, 2, 6, test_zp_x);
        dec_test!(DEC_ABS, get_addr_abs, set_addr_abs, 3, 6, test_abs);
        dec_test!(DEC_ABS_X, get_addr_abs_x, set_addr_abs_x, 3, 7, test_abs_x);
    }
    mod dex {
        use super::*;
        use test_case::test_case;
        dec_test!(DEX, get_x, set_x, 1, 2, test_implied);
    }
    mod dey {
        use super::*;
        use test_case::test_case;
        dec_test!(DEY, get_y, set_y, 1, 2, test_implied);
    }
    mod eor {
        use super::*;
        use test_case::test_case;
        macro_rules! eor_test {
            ($name: ident, $opcode: ident, $addr_func: ident, $bytes: expr, $cycles: expr) => {
                #[test_case(0xAB, 0xCD, 0xAB ^ 0xCD, false, false ; "happy case")]
                #[test_case(0xFF, 0xFF, 0x00, true, false ; "should be zero 1")]
                #[test_case(0x18, 0x18, 0x00, true, false ; "should be zero 2")]
                #[test_case(0x18, 0x80, 0x98, false, true ; "should be negatvie")]
                fn $name(a: u8, val: u8, a_post: u8, z: bool, n: bool) {
                    let mut nes = Nes::new();
                    nes.cpu.a = a;
                    let addr = $addr_func(&mut nes, val);
                    assert_eq!(
                        nes.decode_and_execute(&prepend_with_opcode($opcode, &addr)),
                        Ok(($bytes, $cycles))
                    );
                    assert_eq_hex!(nes.cpu.a, a_post);
                    assert_z(&nes, z);
                    assert_n(&nes, n);
                }
            };
        }
        eor_test!(test_immediate, EOR_I, set_addr_i, 2, 2);
        eor_test!(test_zp, EOR_ZP, set_addr_zp, 2, 3);
        eor_test!(test_zp_x, EOR_ZP_X, set_addr_zp_x, 2, 4);
        eor_test!(test_abs, EOR_ABS, set_addr_abs, 3, 4);
        eor_test!(test_abs_x, EOR_ABS_X, set_addr_abs_x, 3, 4);
        eor_test!(test_abs_x_pc, EOR_ABS_X, set_addr_abs_x_pc, 3, 5);
        eor_test!(test_abs_y, EOR_ABS_Y, set_addr_abs_y, 3, 4);
        eor_test!(test_abs_y_pc, EOR_ABS_Y, set_addr_abs_y_pc, 3, 5);
        eor_test!(test_ind_x, EOR_IND_X, set_addr_ind_x, 2, 6);
        eor_test!(test_ind_y, EOR_IND_Y, set_addr_ind_y, 2, 5);
        eor_test!(test_ind_y_pc, EOR_IND_Y, set_addr_ind_y_pc, 2, 6);
    }
    macro_rules! inc_test {
        ($name: ident, $opcode: ident, $set_addr: ident, $get_addr: ident, $bytes: expr, $cycles: expr) => {
            #[test_case(0x00, 0x01, false, false; "happy case")]
            #[test_case(0xFF, 0x00, true, false ; "should wrap")]
            #[test_case(0x80, 0x81, false, true ; "should be negative")]
            #[test_case(0x18, 0x19, false, false ; "happy case 2")]
            fn $name(pre_val: u8, post_val: u8, z: bool, n: bool) {
                let mut nes = Nes::new();
                let addr = $set_addr(&mut nes, pre_val);
                assert_eq!(
                    nes.decode_and_execute(&prepend_with_opcode($opcode, &addr)),
                    Ok(($bytes, $cycles))
                );
                assert_eq_hex!($get_addr(&nes, &addr), post_val);
                assert_n(&nes, n);
                assert_z(&nes, z);
            }
        };
    }
    mod inc {
        use super::*;
        use test_case::test_case;

        inc_test!(test_zp, INC_ZP, set_addr_zp, get_addr_zp, 2, 5);
        inc_test!(test_zp_x, INC_ZP_X, set_addr_zp_x, get_addr_zp_x, 2, 6);
        inc_test!(test_abs, INC_ABS, set_addr_abs, get_addr_abs, 3, 6);
        inc_test!(test_abs_x, INC_ABS_X, set_addr_abs_x, get_addr_abs_x, 3, 7);
    }
    mod inx {
        use super::*;
        use test_case::test_case;
        inc_test!(test_implied, INX, set_x, get_x, 1, 2);
    }
    mod iny {
        use super::*;
        use test_case::test_case;
        inc_test!(test_implied, INY, set_y, get_y, 1, 2);
    }
    mod jmp {
        use super::*;
        use test_case::test_case;

        #[test_case(0xABCD ; "happy case")]
        #[test_case(0x0000 ; "should be zero")]
        #[test_case(0xFFFF ; "should be max")]
        fn test_abs(addr: u16) {
            let mut nes = Nes::new();
            assert_eq!(
                nes.decode_and_execute(&[JMP_ABS, first_byte(addr), second_byte(addr)]),
                Ok((3, 3))
            );
            assert_eq_hex!(nes.cpu.p_c, addr.wrapping_sub(3));
        }
        #[test_case(0x0120, 0xFC, 0xBA, 0xBAFC; "happy case")]
        #[test_case(0x0000, 0xFF, 0xFF, 0xFFFF ; "jump to end")]
        #[test_case(0x02FF, 0x00, 0xA9, 0x0000 ; "should wrap around page boundary")]
        #[test_case(0x02FE, 0xAB, 0xCD, 0xCDAB ; "should not wrap around page boundary")]
        fn test_indirect(addr: usize, v1: u8, v2: u8, p_c: u16) {
            let mut nes = Nes::new();
            nes.mem[addr] = v1;
            nes.mem[addr + 1] = v2;
            assert_eq!(
                nes.decode_and_execute(&[
                    JMP_IND,
                    first_byte(addr as u16),
                    second_byte(addr as u16)
                ]),
                Ok((3, 5))
            );
            assert_eq_hex!(nes.cpu.p_c, p_c.wrapping_sub(3));
        }
    }
    mod jsr {
        use super::*;
        use test_case::test_case;
        #[test_case(0x1234, 0x67, 0x45, 0x4564, 0x12, 0x36; "happy case")]
        #[test_case(0xFFFF, 0x00, 0x00, 0xFFFD, 0x00, 0x01; "should wrap")]
        fn test_absolute(pc: u16, op_one: u8, op_two: u8, post_pc: u16, mem_one: u8, mem_two: u8) {
            let mut nes = Nes::new();
            nes.cpu.p_c = pc;
            assert_eq!(nes.decode_and_execute(&[JSR, op_one, op_two]), Ok((3, 6)));
            assert_eq_hex!(nes.cpu.p_c, post_pc);
            assert_eq_hex!(nes.mem[0x1FD], mem_one);
            assert_eq_hex!(nes.mem[0x1FC], mem_two);
        }
    }
    mod lsr {
        use super::*;
        use test_case::test_case;

        macro_rules! lsr_test {
            ($name: ident, $opcode: ident, $set_addr: ident, $get_addr: ident, $bytes: expr, $cycles: expr) => {
                #[test_case(0x18, 0x0C, false, false ; "happy case")]
                fn $name(pre_val: u8, post_val: u8, c: bool, z: bool) {
                    let mut nes = Nes::new();
                    let addr = $set_addr(&mut nes, pre_val);
                    assert_eq!(
                        nes.decode_and_execute(&prepend_with_opcode($opcode, &addr)),
                        Ok(($bytes, $cycles))
                    );
                    assert_eq_hex!($get_addr(&nes, &addr), post_val);
                    assert_c(&nes, c);
                    assert_z(&nes, z);
                }
            };
        }
        lsr_test!(test_acc, LSR_A, set_a, get_a, 1, 2);
        lsr_test!(test_zp, LSR_ZP, set_addr_zp, get_addr_zp, 2, 5);
        lsr_test!(test_zp_x, LSR_ZP_X, set_addr_zp_x, get_addr_zp_x, 2, 6);
        lsr_test!(test_abs, LSR_ABS, set_addr_abs, get_addr_abs, 3, 6);
        lsr_test!(test_abs_x, LSR_ABS_X, set_addr_abs_x, get_addr_abs_x, 3, 7);
    }
    mod ora {
        use super::*;
        use test_case::test_case;
        macro_rules! ora_test {
            ($name: ident, $opcode: ident, $set_addr: ident, $bytes: expr, $cycles: expr) => {
                #[test_case(0x00, 0xAB, 0xAB, false, true; "happy case")]
                fn $name(a: u8, pre_val: u8, post_val: u8, z: bool, n: bool) {
                    let mut nes = Nes::new();
                    nes.cpu.a = a;
                    let addr = $set_addr(&mut nes, pre_val);
                    assert_eq!(
                        nes.decode_and_execute(&prepend_with_opcode($opcode, &addr)),
                        Ok(($bytes, $cycles))
                    );
                    assert_eq!(nes.cpu.a, post_val);
                    assert_n(&nes, n);
                    assert_z(&nes, z);
                }
            };
        }
        ora_test!(test_immediate, ORA_I, set_addr_i, 2, 2);
        ora_test!(test_zp, ORA_ZP, set_addr_zp, 2, 3);
        ora_test!(test_zp_x, ORA_ZP_X, set_addr_zp_x, 2, 4);
        ora_test!(test_abs, ORA_ABS, set_addr_abs, 3, 4);
        ora_test!(test_abs_x, ORA_ABS_X, set_addr_abs_x, 3, 4);
        ora_test!(test_abs_x_pc, ORA_ABS_X, set_addr_abs_x_pc, 3, 5);
        ora_test!(test_abs_y, ORA_ABS_Y, set_addr_abs_y, 3, 4);
        ora_test!(test_abs_y_pc, ORA_ABS_Y, set_addr_abs_y_pc, 3, 5);
        ora_test!(test_ind_x, ORA_IND_X, set_addr_ind_x, 2, 6);
        ora_test!(test_ind_y, ORA_IND_Y, set_addr_ind_y, 2, 5);
        ora_test!(test_ind_y_pc, ORA_IND_Y, set_addr_ind_y_pc, 2, 6);
    }
    #[test]
    fn test_pha() {
        let mut nes = Nes::new();
        nes.cpu.a = 0x18;
        assert_eq!(nes.decode_and_execute(&[PHA]), Ok((1, 3)));
        assert_eq_hex!(nes.mem[0x1FD], 0x18);
        assert_eq!(nes.decode_and_execute(&[PHA]), Ok((1, 3)));
        assert_eq_hex!(nes.mem[0x1FC], 0x18);
    }
    #[test]
    fn test_php() {
        let mut nes = Nes::new();
        nes.cpu.s_r.c = true;
        nes.cpu.s_r.i = false;
        nes.cpu.s_r.n = true;
        assert_eq!(nes.decode_and_execute(&[PHP]), Ok((1, 3)));
        assert_eq_hex!(nes.mem[0x1FD], 0xB1);
        assert_eq!(nes.decode_and_execute(&[PHP]), Ok((1, 3)));
        assert_eq_hex!(nes.mem[0x1FC], 0xB1);
    }
    #[test]
    fn test_pla() {
        let mut nes = Nes::new();
        nes.mem[0x1FF] = 0x12;
        nes.mem[0x1FE] = 0x34;
        nes.cpu.s_p = 0xFD;
        assert_eq!(nes.decode_and_execute(&[PLA]), Ok((1, 4)));
        assert_eq_hex!(nes.cpu.a, 0x34);
        assert_eq!(nes.decode_and_execute(&[PLA]), Ok((1, 4)));
        assert_eq_hex!(nes.cpu.a, 0x12);
    }
    #[test]
    fn test_plp() {
        let mut nes = Nes::new();
        nes.mem[0x1FF] = 0x18;
        nes.mem[0x1FE] = 0x00;
        nes.mem[0x1FD] = 0xFF;
        nes.cpu.s_p = 0xFC;
        assert_eq!(nes.decode_and_execute(&[PLP]), Ok((1, 4)));
        assert_c(&nes, true);
        assert_z(&nes, true);
        assert_d(&nes, true);
        assert_v(&nes, true);
        assert_n(&nes, true);
        assert_eq!(nes.decode_and_execute(&[PLP]), Ok((1, 4)));
        assert_c(&nes, false);
        assert_z(&nes, false);
        assert_d(&nes, false);
        assert_v(&nes, false);
        assert_n(&nes, false);
        assert_eq!(nes.decode_and_execute(&[PLP]), Ok((1, 4)));
        assert_c(&nes, false);
        assert_z(&nes, false);
        assert_d(&nes, true);
        assert_v(&nes, false);
        assert_n(&nes, false);
    }
    mod rol {
        use super::*;
        use test_case::test_case;
        macro_rules! rol_test {
            ($name: ident, $opcode: ident, $get_addr: ident, $set_addr: ident, $bytes: expr, $cycles: expr) => {
                #[test_case(0x18, false, 0x30, false, false, false; "happy case")]
                #[test_case(0x8F, false, 0x1E, true, false, false; "should set C")]
                #[test_case(0x40, false, 0x80, false, true, false ; "should set N")]
                #[test_case(0x80, false, 0x00, true, false, true ; "should set Z")]
                #[test_case(0xFF, true, 0xFF, true, true, false ; "full rotation")]
                #[test_case(0x00, false, 0x00, false, false, true ; "fully zero")]
                #[test_case(0x00, true, 0x01, false, false, false ; "only carry set")]
                fn $name(value: u8, c_before: bool, rotated: u8, c: bool, n: bool, z: bool) {
                    let mut nes = Nes::new();
                    let addr = $set_addr(&mut nes, value);
                    nes.cpu.s_r.c = c_before;
                    assert_eq!(
                        nes.decode_and_execute(&prepend_with_opcode($opcode, &addr)),
                        Ok(($bytes, $cycles))
                    );
                    assert_eq_hex!($get_addr(&nes, &addr), rotated);
                    assert_c(&nes, c);
                    assert_n(&nes, n);
                    assert_z(&nes, z);
                }
            };
        }
        rol_test!(test_a, ROL_A, get_a, set_a, 1, 2);
        rol_test!(test_zp, ROL_ZP, get_addr_zp, set_addr_zp, 2, 5);
        rol_test!(test_zp_x, ROL_ZP_X, get_addr_zp_x, set_addr_zp_x, 2, 6);
        rol_test!(test_abs, ROL_ABS, get_addr_abs, set_addr_abs, 3, 6);
        rol_test!(test_abs_x, ROL_ABS_X, get_addr_abs_x, set_addr_abs_x, 3, 7);
    }
    mod ror {
        use super::*;
        use test_case::test_case;
        macro_rules! ror_test {
            ($name: ident, $opcode: ident, $get_addr: ident, $set_addr: ident, $bytes: expr, $cycles: expr) => {
                #[test_case(0x18, false, 0x0C, false, false, false; "happy case")]
                #[test_case(0x81, false, 0x40, true, false, false; "should set C")]
                #[test_case(0x00, true, 0x80, false, true, false ; "should set N")]
                #[test_case(0x01, false, 0x00, true, false, true ; "should set Z")]
                #[test_case(0xFF, true, 0xFF, true, true, false ; "full rotation")]
                #[test_case(0x00, false, 0x00, false, false, true ; "fully zero")]
                #[test_case(0x00, true, 0x80, false, true, false; "only carry set")]
                fn $name(value: u8, c_before: bool, rotated: u8, c: bool, n: bool, z: bool) {
                    let mut nes = Nes::new();
                    let addr = $set_addr(&mut nes, value);
                    nes.cpu.s_r.c = c_before;
                    assert_eq!(
                        nes.decode_and_execute(&prepend_with_opcode($opcode, &addr)),
                        Ok(($bytes, $cycles))
                    );
                    assert_eq_hex!($get_addr(&nes, &addr), rotated);
                    assert_c(&nes, c);
                    assert_n(&nes, n);
                    assert_z(&nes, z);
                }
            };
        }

        ror_test!(test_a, ROR_A, get_a, set_a, 1, 2);
        ror_test!(test_zp, ROR_ZP, get_addr_zp, set_addr_zp, 2, 5);
        ror_test!(test_zp_x, ROR_ZP_X, get_addr_zp_x, set_addr_zp_x, 2, 6);
        ror_test!(test_abs, ROR_ABS, get_addr_abs, set_addr_abs, 3, 6);
        ror_test!(test_abs_x, ROR_ABS_X, get_addr_abs_x, set_addr_abs_x, 3, 7);
    }
    // #[test_case::test_case(0x0018, true, true, true, true, true, true, true ; "happy case")]
    // #[test_case::test_case(0x00FF, false, false, false, false, false, false, false ; "happy case 2")]
    // #[test_case::test_case(0x0000, true, false, true, false, true, false, true ; "happy case 3")]
    // fn test_rti(p_c: u16, c: bool, z: bool, i: bool, d: bool, b: bool, v: bool, n: bool) {
    //     let mut nes = Nes::new();
    //     nes.cpu.p_c = p_c;
    //     nes.cpu.s_r = StatusRegister {
    //         c,
    //         z,
    //         i,
    //         d,
    //         b,
    //         v,
    //         n,
    //     };
    //     assert_eq!(nes.decode_and_execute(&[BRK]), Ok((1, 7)));
    //     nes.cpu.p_c = random::<u16>();
    //     nes.cpu.s_r = StatusRegister {
    //         c: !c,
    //         z: !z,
    //         i: !i,
    //         d: !d,
    //         b: !b,
    //         v: !v,
    //         n: !n,
    //     };
    //     assert_eq!(nes.decode_and_execute(&[RTI]), Ok((1, 6)));
    //     assert_eq_hex!(nes.cpu.p_c, p_c);
    //     assert_c(&nes, c);
    //     assert_z(&nes, z);
    //     assert_i(&nes, i);
    //     assert_d(&nes, d);
    //     assert_b(&nes, b);
    //     assert_v(&nes, v);
    //     assert_n(&nes, n);
    // }
    #[test_case::test_case(0x1234, 0x5678 ; "happy case")]
    fn test_rts(p_c: u16, addr: u16) {
        let mut nes = Nes::new();
        nes.cpu.p_c = p_c;
        assert_eq!(
            nes.decode_and_execute(&prepend_with_opcode(
                JSR,
                &[first_byte(addr), second_byte(addr)]
            )),
            Ok((3, 6))
        );
        assert_eq_hex!(nes.cpu.p_c, addr.wrapping_sub(3));
        assert_eq!(nes.decode_and_execute(&[RTS]), Ok((1, 6)));
        // Wrapping add here since we should save 1 byte before the next address
        // And since the JSR command is 3 bytes long, we want to end up 2 bytes ahead of where we left
        assert_eq_hex!(nes.cpu.p_c, p_c.wrapping_add(2));
    }
    mod sbc {
        use super::*;
        use test_case::test_case;

        macro_rules! sbc_test {
            ($name: ident, $opcode: ident, $get_addr: ident, $bytes: expr, $cycles: expr) => {
                #[test_case(0x0C, 0x01, 0x0B, true, true, false, false, false ; "simple happy case")]
                #[test_case(0xFF, 0xFF, 0x00, true, true, true, false, false ; "subtracts to 0")]
                #[test_case(0xAB, 0x00, 0xAB, true, true, false, false, true ; "is unchanged")]
                #[test_case(0x05, 0x0A, 0xFB, true, false, false, false, true ; "is negative")]
                #[test_case(0x00, 0x7F, 0x81, true, false, false, false, true ; "is negative 127")]
                #[test_case(0x80, 0x00, 0x7F, false, true, false, true, false ; "happy case 2")]
                #[test_case(0xC0, 0x40, 0x7F, false, true, false, true, false; "test")]
                fn $name(
                    pre_a: u8,
                    value: u8,
                    post_a: u8,
                    pre_c: bool,
                    c: bool,
                    z: bool,
                    v: bool,
                    n: bool,
                ) {
                    let mut nes = Nes::new();
                    nes.cpu.a = pre_a;
                    nes.cpu.s_r.c = pre_c;
                    let addr = $get_addr(&mut nes, value);
                    assert_eq!(
                        nes.decode_and_execute(&prepend_with_opcode($opcode, &addr)),
                        Ok(($bytes, $cycles))
                    );
                    assert_eq_hex!(nes.cpu.a, post_a);
                    assert_c(&nes, c);
                    assert_z(&nes, z);
                    assert_v(&nes, v);
                    assert_n(&nes, n);
                }
            };
        }

        sbc_test!(test_i, SBC_I, set_addr_i, 2, 2);
        sbc_test!(test_zp, SBC_ZP, set_addr_zp, 2, 3);
        sbc_test!(test_zp_x, SBC_ZP_X, set_addr_zp_x, 2, 4);
        sbc_test!(test_zp_abs, SBC_ABS, set_addr_abs, 3, 4);
        sbc_test!(test_zp_abs_x, SBC_ABS_X, set_addr_abs_x, 3, 4);
        sbc_test!(test_zp_abs_x_pc, SBC_ABS_X, set_addr_abs_x_pc, 3, 5);
        sbc_test!(test_zp_abs_y, SBC_ABS_Y, set_addr_abs_y, 3, 4);
        sbc_test!(test_zp_abs_y_pc, SBC_ABS_Y, set_addr_abs_y_pc, 3, 5);
        sbc_test!(test_ind_x, SBC_IND_X, set_addr_ind_x, 2, 6);
        sbc_test!(test_ind_y, SBC_IND_Y, set_addr_ind_y, 2, 5);
        sbc_test!(test_ind_y_pc, SBC_IND_Y, set_addr_ind_y_pc, 2, 6);
    }
    macro_rules! test_set {
        ($name: ident, $opcode: ident, $flag: ident) => {
            #[test]
            fn $name() {
                let mut nes = Nes::new();
                assert_eq!(nes.decode_and_execute(&[$opcode]), Ok((1, 2)));
                assert_eq!(nes.cpu.s_r.$flag, true);
                // Test it stays true
                assert_eq!(nes.decode_and_execute(&[$opcode]), Ok((1, 2)));
                assert_eq!(nes.cpu.s_r.$flag, true);
            }
        };
    }
    test_set!(test_sec, SEC, c);
    test_set!(test_sed, SED, d);
    test_set!(test_sei, SEI, i);
    macro_rules! test_store {
        ($name: ident, $opcode: ident, $reg: ident, $get_addr: ident, $set_addr: ident, $bytes: expr, $cycles: expr) => {
            #[test_case(0x12 ; "happy case")]
            fn $name(v: u8) {
                let mut nes = Nes::new();
                nes.cpu.$reg = v;
                // The value we set here doesn't matter, we just want to get the address (and potentially set up X/Y)
                let addr = $set_addr(&mut nes, 0x00);
                assert_eq!(
                    nes.decode_and_execute(&prepend_with_opcode($opcode, &addr)),
                    Ok(($bytes, $cycles))
                );
                assert_eq_hex!($get_addr(&mut nes, &addr), v);
            }
        };
    }
    mod sta {
        use super::*;
        use test_case::test_case;
        test_store!(test_zp, STA_ZP, a, get_addr_zp, set_addr_zp, 2, 3);
        test_store!(test_zp_x, STA_ZP_X, a, get_addr_zp_x, set_addr_zp_x, 2, 4);
        test_store!(test_abs, STA_ABS, a, get_addr_abs, set_addr_abs, 3, 4);
        test_store!(
            test_abs_x,
            STA_ABS_X,
            a,
            get_addr_abs_x,
            set_addr_abs_x,
            3,
            5
        );
        test_store!(
            test_abs_y,
            STA_ABS_Y,
            a,
            get_addr_abs_y,
            set_addr_abs_y,
            3,
            5
        );
    }
    mod stx {
        use super::*;
        use test_case::test_case;
        test_store!(test_zp, STX_ZP, x, get_addr_zp, set_addr_zp, 2, 3);
        test_store!(test_zp_y, STX_ZP_Y, x, get_addr_zp_y, set_addr_zp_y, 2, 4);
        test_store!(test_abs, STX_ABS, x, get_addr_abs, set_addr_abs, 3, 4);
    }
    mod sty {
        use super::*;
        use test_case::test_case;
        test_store!(test_zp, STY_ZP, y, get_addr_zp, set_addr_zp, 2, 3);
        test_store!(test_zp_x, STY_ZP_X, y, get_addr_zp_x, set_addr_zp_x, 2, 4);
        test_store!(test_abs, STY_ABS, y, get_addr_abs, set_addr_abs, 3, 4);
    }
    mod tranfer {
        use super::*;
        use test_case::test_case;
        macro_rules! test_transfer {
            ($name: ident, $opcode: ident, $from_reg: ident, $to_reg: ident) => {
                #[test_case(0x18, false, false ; "happy case")]
                #[test_case(0x00, true, false ; "sets Z")]
                #[test_case(0x80, false, true ; "sets N")]
                fn $name(v: u8, z: bool, n: bool) {
                    let mut nes = Nes::new();
                    nes.cpu.$from_reg = v;
                    assert_eq!(nes.decode_and_execute(&[$opcode]), Ok((1, 2)));
                    assert_eq_hex!(nes.cpu.$to_reg, v);
                    assert_eq_hex!(nes.cpu.$from_reg, v);
                    assert_z(&nes, z);
                    assert_n(&nes, n);
                }
            };
        }
        test_transfer!(test_tax, TAX, a, x);
        test_transfer!(test_tay, TAY, a, y);
        test_transfer!(test_tsx, TSX, s_p, x);
        test_transfer!(test_txa, TXA, x, a);
        test_transfer!(test_tya, TYA, y, a);
    }
    // Utility functions to get and setsome addresses in memory set to the value given
    fn get_addr_zp(nes: &Nes, addr: &[u8]) -> u8 {
        nes.mem[addr[0] as usize]
    }
    fn set_addr_zp(nes: &mut Nes, value: u8) -> [u8; 1] {
        set_addr_zp_offset(nes, value, 0)
    }
    fn set_addr_zp_offset(nes: &mut Nes, value: u8, offset: u8) -> [u8; 1] {
        let addr = random::<u8>();
        nes.mem[addr.wrapping_add(offset) as usize] = value;
        return [addr];
    }
    fn set_addr_zp_x(nes: &mut Nes, value: u8) -> [u8; 1] {
        nes.cpu.x = random::<u8>();
        set_addr_zp_offset(nes, value, nes.cpu.x)
    }
    fn set_addr_zp_y(nes: &mut Nes, value: u8) -> [u8; 1] {
        nes.cpu.y = random::<u8>();
        set_addr_zp_offset(nes, value, nes.cpu.y)
    }
    fn get_addr_zp_x(nes: &Nes, value: &[u8]) -> u8 {
        nes.mem[value[0].wrapping_add(nes.cpu.x) as usize]
    }
    fn get_addr_zp_y(nes: &Nes, value: &[u8]) -> u8 {
        nes.mem[value[0].wrapping_add(nes.cpu.y) as usize]
    }
    fn set_addr_abs_offset_no_pc(nes: &mut Nes, value: u8, offset: u8) -> [u8; 2] {
        // Make sure we don't cross a page
        let m = (255 - offset) as u16;
        // TODO: 0 + ... <- this zero should be replaced later once cartridge RAM/ROM is implemented
        let addr = 0 + if m != 0 { random::<u16>() % m } else { 0x00 };
        nes.mem[(addr + offset as u16) as usize] = value;
        println!(
            "Setting {:#X?} + {:#X?} (= {:#X?}) to {:#X} (should not be a page cross)",
            addr,
            offset,
            addr + offset as u16,
            value
        );
        return [first_byte(addr), second_byte(addr)];
    }
    fn set_addr_abs(nes: &mut Nes, value: u8) -> [u8; 2] {
        set_addr_abs_offset_no_pc(nes, value, 0)
    }
    fn get_addr_abs(nes: &Nes, addr: &[u8]) -> u8 {
        nes.mem[((addr[1] as usize) << 8) + addr[0] as usize]
    }
    fn set_addr_abs_x(nes: &mut Nes, value: u8) -> [u8; 2] {
        nes.cpu.x = random::<u8>();
        set_addr_abs_offset_no_pc(nes, value, nes.cpu.x)
    }
    fn get_addr_abs_x(nes: &Nes, addr: &[u8]) -> u8 {
        let a = ((addr[1] as u16) << 8) + addr[0] as u16;
        nes.mem[a.wrapping_add(nes.cpu.x as u16) as usize]
    }
    fn get_addr_abs_y(nes: &Nes, addr: &[u8]) -> u8 {
        let a = ((addr[1] as u16) << 8) + addr[0] as u16;
        nes.mem[a.wrapping_add(nes.cpu.y as u16) as usize]
    }
    fn set_addr_abs_y(nes: &mut Nes, value: u8) -> [u8; 2] {
        nes.cpu.y = random::<u8>();
        set_addr_abs_offset_no_pc(nes, value, nes.cpu.y)
    }
    fn set_addr_abs_offset_pc(nes: &mut Nes, value: u8, offset: u8) -> [u8; 2] {
        // See comment above in set_addr_abs_offset_no_pc
        let addr = 0 + (0xFF - (random::<u16>() % offset as u16));
        nes.mem[(addr + offset as u16) as usize] = value;
        println!(
            "Setting {:#X?} + {:#X?} (= {:#X?}) to {:#X} (should be a page cross)",
            addr,
            offset,
            addr + offset as u16,
            value
        );

        [first_byte(addr), second_byte(addr)]
    }
    fn set_addr_abs_x_pc(nes: &mut Nes, value: u8) -> [u8; 2] {
        nes.cpu.x = max(random::<u8>(), 1);
        set_addr_abs_offset_pc(nes, value, nes.cpu.x)
    }
    fn set_addr_abs_y_pc(nes: &mut Nes, value: u8) -> [u8; 2] {
        nes.cpu.y = max(random::<u8>(), 1);
        set_addr_abs_offset_pc(nes, value, nes.cpu.y)
    }
    // These are just so that we can use these function in macros instead of using set_addr_zp or set_addr_abs
    fn set_addr_i(_nes: &mut Nes, value: u8) -> [u8; 1] {
        [value]
    }
    fn get_addr_i(_name: &mut Nes, addr: u8) -> [u8; 1] {
        [addr]
    }
    fn set_a(nes: &mut Nes, value: u8) -> [u8; 0] {
        nes.cpu.a = value;
        []
    }
    fn get_a(nes: &Nes, _addr: &[u8]) -> u8 {
        nes.cpu.a
    }
    fn set_x(nes: &mut Nes, value: u8) -> [u8; 0] {
        nes.cpu.x = value;
        []
    }
    fn get_x(nes: &Nes, _addr: &[u8]) -> u8 {
        nes.cpu.x
    }
    fn set_y(nes: &mut Nes, value: u8) -> [u8; 0] {
        nes.cpu.y = value;
        []
    }
    fn get_y(nes: &Nes, _addr: &[u8]) -> u8 {
        nes.cpu.y
    }
    fn set_addr_ind_y(nes: &mut Nes, value: u8) -> [u8; 1] {
        nes.cpu.y = random::<u8>();
        let addr = set_addr_abs_offset_no_pc(nes, value, nes.cpu.y);
        // Now store addr in ZP
        let mut addr_two = random::<u8>();
        if addr_two == (addr[0].wrapping_add(nes.cpu.y))
            || addr_two == (addr[0].wrapping_sub(1).wrapping_add(nes.cpu.y)) && addr[1] == 0
        {
            addr_two = addr_two.wrapping_add(2);
        }
        if addr_two == 0xFF {
            addr_two = 0x00;
        }
        println!(
            "Storing memory address [{:#X}, {:#X}] at {:#X}",
            addr[0], addr[1], addr_two
        );
        nes.mem[addr_two as usize] = addr[0];
        nes.mem[addr_two.wrapping_add(1) as usize] = addr[1];
        [addr_two]
    }
    fn set_addr_ind_y_pc(nes: &mut Nes, value: u8) -> [u8; 1] {
        nes.cpu.y = max(random::<u8>(), 1);
        let addr = set_addr_abs_offset_pc(nes, value, nes.cpu.y);
        let mut addr_two = random::<u8>();
        if (addr_two == addr[0] || addr_two == addr[0].wrapping_sub(1)) && addr[1] == 0 {
            addr_two = addr_two.wrapping_add(2);
        }
        nes.mem[addr_two as usize] = addr[0];
        nes.mem[addr_two.wrapping_add(1) as usize] = addr[1];
        [addr_two]
    }
    fn set_addr_ind_x(nes: &mut Nes, value: u8) -> [u8; 1] {
        nes.cpu.x = random::<u8>();
        let addr = set_addr_abs(nes, value);
        let mut addr_two = random::<u8>();
        let actual_addr = addr_two.wrapping_add(nes.cpu.x);
        if (actual_addr == addr[0] || actual_addr == addr[0].wrapping_sub(1)) && addr[1] == 0 {
            addr_two = addr_two.wrapping_add(2);
        }
        let final_addr = addr_two.wrapping_add(nes.cpu.x) as u16;
        println!(
            "Setting indirect address {:#X} to {:#X} and {:#X}",
            final_addr, addr[0], addr[1]
        );
        nes.mem[final_addr as usize] = addr[0];
        nes.mem[final_addr.wrapping_add(1) as usize] = addr[1];
        [addr_two]
    }
    fn first_byte(addr: u16) -> u8 {
        (addr & 0xFF) as u8
    }
    fn second_byte(addr: u16) -> u8 {
        (addr >> 8) as u8
    }
    fn addr_from_bytes(addr: [u8; 2]) -> usize {
        ((addr[1] as usize) << 8) + (addr[0] as usize)
    }
    // ten here since all instructions are way less than 10 bytes and extra ones can ust be ignored
    fn prepend_with_opcode(opcode: u8, arr: &[u8]) -> [u8; 10] {
        let mut a: [u8; 10] = [0; 10];
        a[0] = opcode;
        a[1..(arr.len() + 1)].copy_from_slice(arr);
        a
    }
    // Flag assertion functions
    macro_rules! create_flag_assert_func {
        ($flag: ident, $str: literal, $name: ident) => {
            fn $name(nes: &Nes, $flag: bool) {
                assert_eq!(
                    nes.cpu.s_r.$flag,
                    $flag,
                    "{} should be {}",
                    $str,
                    if $flag { "set" } else { "unset" }
                );
            }
        };
    }
    create_flag_assert_func!(c, "C", assert_c);
    create_flag_assert_func!(z, "Z", assert_z);
    create_flag_assert_func!(n, "N", assert_n);
    create_flag_assert_func!(i, "I", assert_i);
    create_flag_assert_func!(b, "B", assert_b);
    create_flag_assert_func!(d, "D", assert_d);
    create_flag_assert_func!(v, "V", assert_v);
}
