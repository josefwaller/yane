#[derive(Clone, Copy)]
pub struct PulseRegister {
    pub duty: u32,
    length_halt: bool,
    pub constant_volume: bool,
    pub volume: u32,
    // The timer as set by the CPU
    pub timer: usize,
    // The internal timer that counts down and is reset to timer
    pub internal_timer: usize,
    length: usize,
    sweep_enabled: bool,
    sweep_period: u32,
    sweep_negate: bool,
    sweep_shift: u32,
}

impl PulseRegister {
    pub fn new() -> PulseRegister {
        PulseRegister {
            duty: 0,
            length_halt: false,
            constant_volume: false,
            volume: 0,
            timer: 0,
            internal_timer: 0,
            length: 0,
            sweep_enabled: false,
            sweep_period: 0,
            sweep_negate: false,
            sweep_shift: 0,
        }
    }
}

pub struct Apu {
    pub pulse_registers: [PulseRegister; 2],
}

impl Apu {
    pub fn new() -> Apu {
        Apu {
            pulse_registers: [PulseRegister::new(); 2],
        }
    }
    /// Write a byte of data to the APU given its address in CPU memory space
    pub fn write_byte(&mut self, addr: usize, value: u8) {
        match addr {
            // 0x4000 => {
            //     self.pulse_registers[0].duty = ((value & 0xC0) >> 6) as u32;
            //     self.pulse_registers[0].length_halt = (value & 0x20) != 0;
            //     self.pulse_registers[0].constant_volume = (value & 0x10) != 0;
            //     self.pulse_registers[0].volume = (value & 0x0F) as u32;
            // }
            // 0x4002 => {
            //     self.pulse_registers[0].timer =
            //         (self.pulse_registers[0].timer & 0x0300) | value as usize;
            // }
            // 0x4003 => {
            //     self.pulse_registers[0].timer =
            //         (self.pulse_registers[0].timer & 0x00FF) | ((value as usize & 0x03) << 8);
            // }
            0x4000..0x4004 => self.set_pulse_byte(0, addr, value),
            0x4004..0x4008 => self.set_pulse_byte(1, addr, value),
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
            }
            2 => {
                reg.timer = (reg.timer & 0x0300) | value as usize;
            }
            3 => {
                reg.timer = (reg.timer & 0x00FF) | ((value as usize & 0x03) << 8);
            }
            _ => {} // _ => panic!("Invalid address given to APU"),
        }
    }
    pub fn step(&mut self) {
        self.pulse_registers[0].internal_timer = if self.pulse_registers[0].internal_timer > 0 {
            self.pulse_registers[0].internal_timer - 1
        } else {
            self.pulse_registers[0].timer
        };
    }
}
