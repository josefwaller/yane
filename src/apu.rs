use std::cmp::max;

#[derive(Clone, Copy, Default)]
pub struct LengthCounter {
    pub halt: bool,
    pub load: usize,
}
#[derive(Clone, Copy, Default)]
pub struct Envelope {
    /// Constant volume flag
    pub constant: bool,
    /// Volume value (either the volume or the volume reload value)
    pub volume: usize,
    /// Current value of the volume divider
    pub divider: usize,
    /// Current value of the volume decay
    pub decay: usize,
}
#[derive(Clone, Copy, Default)]
pub struct PulseRegister {
    /// The index of the duty to use
    pub duty: u32,
    /// The period of the pulse wave
    pub timer: usize,
    /// The envelope
    pub envelope: Envelope,
    pub length_counter: LengthCounter,
    // Sweep enabled flag
    pub sweep_enabled: bool,
    /// Sweep period
    pub sweep_period: usize,
    pub sweep_target_period: usize,
    /// Sweep divider
    pub sweep_divider: usize,
    /// Sweep negate flag
    pub sweep_negate: bool,
    /// Sweep shift amount
    pub sweep_shift: usize,
    // Whether the register is enabled
    pub enabled: bool,
}
const LENGTH_TABLE: [usize; 0x20] = [
    0x0A, 0xFE, 0x14, 0x02, 0x28, 0x04, 0x50, 0x06, 0xA0, 0x08, 0x3C, 0x0A, 0x0E, 0x0C, 0x1A, 0x0E,
    0x0C, 0x10, 0x18, 0x12, 0x30, 0x14, 0x60, 0x16, 0xC0, 0x18, 0x48, 0x1A, 0x10, 0x1C, 0x20, 0x1E,
];

#[derive(Clone, Copy, Default)]
pub struct TriangleRegister {
    pub length_counter: LengthCounter,
    pub linear_counter: usize,
    // Linear counter reload value
    pub linear_counter_reload: usize,
    pub reload_flag: bool,
    pub timer: usize,
    pub enabled: bool,
}
#[derive(Clone, Copy, Default)]
pub struct NoiseRegister {
    pub lenth_counter: LengthCounter,
}

pub struct Apu {
    pub pulse_registers: [PulseRegister; 2],
    pub triangle_register: TriangleRegister,
    pub noise_register: NoiseRegister,
    step: usize,
}

impl Apu {
    pub fn new() -> Apu {
        Apu {
            pulse_registers: [PulseRegister::default(); 2],
            triangle_register: TriangleRegister::default(),
            noise_register: NoiseRegister::default(),
            step: 0,
        }
    }
    /// Write a byte of data to the APU given its address in CPU memory space
    pub fn write_byte(&mut self, addr: usize, value: u8) {
        match addr {
            0x4000..0x4004 => self.set_pulse_byte(0, addr, value),
            0x4004..0x4008 => self.set_pulse_byte(1, addr, value),
            0x4008 => {
                self.triangle_register.length_counter.halt = (value & 0x80) != 0;
                self.triangle_register.linear_counter_reload = (value & 0x7F) as usize;
            }
            0x4009 => {}
            0x400A => {
                self.triangle_register.timer =
                    (self.triangle_register.timer & 0x700) + value as usize;
            }
            0x400B => {
                self.triangle_register.timer =
                    (self.triangle_register.timer & 0x0FF) + ((value as usize & 0x07) << 8);
                self.triangle_register.length_counter.load =
                    LENGTH_TABLE[(value as usize & 0xF8) >> 3];
                self.triangle_register.reload_flag = true;
            }
            0x400C => {
                self.noise_register.lenth_counter.halt = (value & 0x20) != 0;
            }
            0x400F => {
                println!(
                    "Set noise length to {}",
                    self.noise_register.lenth_counter.load
                );
                self.noise_register.lenth_counter.load = (value as usize & 0xF8) >> 3;
            }
            0x4015 => {
                self.pulse_registers[0].enabled = (value & 0x01) != 0;
                self.pulse_registers[1].enabled = (value & 0x02) != 0;
                self.triangle_register.enabled = (value & 0x04) != 0;
            }
            _ => {} // _ => panic!("Invalid address given to APU"),
        }
    }
    fn set_pulse_byte(&mut self, pulse_index: usize, addr: usize, value: u8) {
        let reg: &mut PulseRegister = &mut self.pulse_registers[pulse_index];
        match addr % 4 {
            0 => {
                reg.duty = ((value & 0xC0) >> 6) as u32;
                reg.length_counter.halt = (value & 0x20) != 0;
                reg.envelope.constant = (value & 0x10) != 0;
                reg.envelope.volume = value as usize & 0x0F;
            }
            1 => {
                reg.sweep_enabled = (value & 0x80) != 0;
                reg.sweep_period = (value as usize & 0x70) >> 4;
                reg.sweep_negate = (value & 0x08) != 0;
                reg.sweep_shift = (value & 0x07) as usize;
            }
            2 => {
                reg.timer = (reg.timer & 0x0700) | value as usize;
            }
            3 => {
                reg.timer = (reg.timer & 0x00FF) | ((value as usize & 0x07) << 8);
                let length = (value & 0xF8) as usize >> 3;
                reg.length_counter.load = LENGTH_TABLE[length];
                // Mimic setting volume start flag
                reg.envelope.decay = 0xF;
                reg.envelope.divider = reg.envelope.volume;
            }
            _ => {} // _ => panic!("Invalid address given to APU"),
        }
    }
    pub fn step(&mut self) {
        if self.step % 3728 == 0 {
            self.on_quater_frame();
            if self.step % (2 * 3728) == 0 {
                self.on_half_frame();
            }
        }
        self.step = (self.step + 1) % 141915;
    }
    pub fn on_quater_frame(&mut self) {
        self.pulse_registers.iter_mut().for_each(|reg| {
            // Clock volume divider
            if reg.envelope.divider == 0 {
                reg.envelope.divider = reg.envelope.volume;
                // Clock volume decay
                if reg.envelope.decay == 0 {
                    // Reset if loop flag is set
                    if reg.length_counter.halt {
                        reg.envelope.decay = 0xF;
                    }
                } else {
                    reg.envelope.decay -= 1;
                }
            } else {
                reg.envelope.divider -= 1;
            }
        });
        if self.triangle_register.reload_flag {
            self.triangle_register.linear_counter = self.triangle_register.linear_counter_reload;
        }
        if !self.triangle_register.length_counter.halt {
            self.triangle_register.reload_flag = false;
            if self.triangle_register.linear_counter > 0 {
                self.triangle_register.linear_counter -= 1;
            }
        }
    }
    pub fn on_half_frame(&mut self) {
        self.pulse_registers.iter_mut().for_each(|reg| {
            // Clock length halt counter
            if !reg.length_counter.halt && reg.length_counter.load > 0 {
                reg.length_counter.load -= 1;
            }
            // Clock sweep divider
            if reg.sweep_divider == 0 {
                // Compute sweep change
                // TODO: Slight difference between pulse 1 and 2
                let sweep_change = reg.timer >> reg.sweep_shift;
                reg.sweep_target_period = max(
                    reg.timer as i32 + if reg.sweep_negate { -1 } else { 1 } * sweep_change as i32,
                    0,
                ) as usize;
                if reg.sweep_enabled
                    && reg.timer >= 8
                    && reg.sweep_shift > 0
                    && reg.sweep_target_period < 0x7FF
                {
                    reg.timer = reg.sweep_target_period;
                }
            } else {
                reg.sweep_divider -= 1;
            }
        });
        if !self.triangle_register.length_counter.halt {
            if self.triangle_register.length_counter.load > 0 {
                self.triangle_register.length_counter.load -= 1;
            }
        }
    }
}
