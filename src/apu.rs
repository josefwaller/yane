use std::time::Instant;

#[derive(Clone, Copy)]
pub struct PulseRegister {
    // The index of the duty to use
    pub duty: u32,
    // Constant volume flag
    pub constant_volume: bool,
    // Volume value (either the volume or the volume reload value)
    pub volume: u32,
    // The period of the pulse wave
    pub timer: usize,
    // The length limit on the beep's duration
    pub length: usize,
    // The length halt flag
    pub length_halt: bool,
    // Sweep enabled flag
    pub sweep_enabled: bool,
    // Sweep period
    pub sweep_period: usize,
    // Sweep divider
    pub sweep_divider: usize,
    pub sweep_negate: bool,
    pub sweep_shift: usize,
    // Whether the register is enabled
    pub enabled: bool,
    pub actual_volume: u32,
    pub volume_step: usize,
}
impl PulseRegister {
    pub fn new() -> PulseRegister {
        PulseRegister {
            duty: 0,
            length_halt: false,
            constant_volume: false,
            volume: 0,
            timer: 0,
            length: 0,
            sweep_enabled: false,
            sweep_period: 0,
            sweep_divider: 0,
            sweep_negate: false,
            sweep_shift: 0,
            enabled: false,
            actual_volume: 0,
            volume_step: 0,
        }
    }
}
const LENGTH_TABLE: [usize; 0x20] = [
    0x0A, 0xFE, 0x14, 0x02, 0x28, 0x04, 0x50, 0x06, 0xA0, 0x08, 0x3C, 0x0A, 0x0E, 0x0C, 0x1A, 0x0E,
    0x0C, 0x10, 0x18, 0x12, 0x30, 0x14, 0x60, 0x16, 0xC0, 0x18, 0x48, 0x1A, 0x10, 0x1C, 0x20, 0x1E,
];

#[derive(Clone, Copy)]
pub struct TriangleRegister {
    // Length halt flag
    // todo rename
    pub length_halt: bool,
    // Length counter value
    pub linear_counter: usize,
    // Linear counter reload value
    pub linear_counter_reload: usize,
    pub reload_flag: bool,
    pub length: usize,
    pub timer: usize,
    pub enabled: bool,
}
impl TriangleRegister {
    pub fn new() -> TriangleRegister {
        TriangleRegister {
            length_halt: false,
            linear_counter: 0,
            linear_counter_reload: 0,
            reload_flag: false,
            length: 0,
            timer: 0,
            enabled: false,
        }
    }
}

pub struct Apu {
    pub pulse_registers: [PulseRegister; 2],
    pub triangle_register: TriangleRegister,
    step: usize,
}

impl Apu {
    pub fn new() -> Apu {
        Apu {
            pulse_registers: [PulseRegister::new(); 2],
            triangle_register: TriangleRegister::new(),
            step: 0,
        }
    }
    /// Write a byte of data to the APU given its address in CPU memory space
    pub fn write_byte(&mut self, addr: usize, value: u8) {
        match addr {
            0x4000..0x4004 => self.set_pulse_byte(0, addr, value),
            0x4004..0x4008 => self.set_pulse_byte(1, addr, value),
            0x4008 => {
                self.triangle_register.length_halt = (value & 0x80) != 0;
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
                self.triangle_register.length = LENGTH_TABLE[(value as usize & 0xF8) >> 3];
                self.triangle_register.reload_flag = true;
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
                reg.length_halt = (value & 0x20) != 0;
                reg.constant_volume = (value & 0x10) != 0;
                reg.volume = (value & 0x0F) as u32;
                reg.actual_volume = 0xF;
                reg.volume_step = reg.volume as usize;
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
                reg.length = LENGTH_TABLE[length];
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
            // Decrease volume if not constant
            if reg.volume_step == 0 {
                reg.volume_step = reg.volume as usize;
                if reg.actual_volume > 0 {
                    reg.actual_volume -= 1;
                } else if reg.length_halt {
                    reg.actual_volume = 0xF;
                }
            } else {
                reg.volume_step -= 1;
            }
        });
        if self.triangle_register.reload_flag {
            self.triangle_register.linear_counter = self.triangle_register.linear_counter_reload;
        }
        if !self.triangle_register.length_halt {
            self.triangle_register.reload_flag = false;
            if self.triangle_register.linear_counter > 0 {
                self.triangle_register.linear_counter -= 1;
            }
        }
    }
    pub fn on_half_frame(&mut self) {
        self.pulse_registers.iter_mut().for_each(|reg| {
            // Update length halt
            if !reg.length_halt && reg.length > 0 {
                reg.length -= 1;
            }
            // Update pulse registers
            if reg.sweep_divider == 0 {
                // reg.sweep_divider = reg.sweep_period;
            } else {
                reg.sweep_divider -= 1;
            }
        });
        if !self.triangle_register.length_halt {
            if self.triangle_register.length > 0 {
                self.triangle_register.length -= 1;
            }
        }
    }
}
