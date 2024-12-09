use std::cmp::max;

use log::*;
use rand::seq::index::sample;

use super::Cartridge;

// Todo move
const CPU_CLOCK_SPEED_HZ: u32 = 1_789_000;

pub trait AudioRegister {
    /// Return whether the register has been muted
    fn muted(&self) -> bool;
    fn value(&self) -> u32 {
        0
    }
    fn set_enabled(&mut self, enabled: bool);
}

#[derive(Clone, Copy, Default, Debug)]
pub struct LengthCounter {
    pub halt: bool,
    pub load: usize,
}
impl LengthCounter {
    fn muted(&self) -> bool {
        self.load == 0
    }
    fn clock(&mut self) {
        if !self.halt && self.load > 0 {
            self.load -= 1;
        }
    }
}
#[derive(Clone, Copy, Default, Debug)]
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
impl Envelope {
    pub fn clock(&mut self, restart: bool) {
        // Clock volume divider
        if self.divider == 0 {
            self.divider = self.volume;
            // Clock volume decay
            if self.decay == 0 {
                // Reset if loop flag is set
                if restart {
                    self.decay = 0xF;
                }
            } else {
                self.decay -= 1;
            }
        } else {
            self.divider -= 1;
        }
    }
    pub fn value(&self) -> u32 {
        if self.constant {
            self.volume as u32
        } else {
            self.decay as u32
        }
    }
}
#[derive(Clone, Copy, Default, Debug)]
pub struct PulseRegister {
    /// The index of the duty to use
    pub duty: u32,
    /// The period of the pulse wave
    pub timer: usize,
    // The amount to reload the timer with when it hits 0
    pub timer_reload: usize,
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
    // Sequencer, i.e. the index of the pulse value currently being sent ot the mixer
    pub sequencer: usize,
}
const DUTY_CYCLES: [[u32; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 0, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1],
];
impl AudioRegister for PulseRegister {
    fn muted(&self) -> bool {
        // Conditions for register being disabled
        !self.enabled
            || self.sweep_target_period > 0x7FF
            || self.length_counter.muted()
            || self.timer_reload < 8
    }
    fn value(&self) -> u32 {
        if !self.enabled
            || self.sweep_target_period > 0x7FF
            || self.length_counter.muted()
            || self.timer_reload < 8
            || DUTY_CYCLES[self.duty as usize][self.sequencer] == 0
        {
            0
        } else {
            self.envelope.value()
        }
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !self.enabled {
            self.length_counter.load = 0;
        }
    }
}
const LENGTH_TABLE: [usize; 0x20] = [
    0x0A, 0xFE, 0x14, 0x02, 0x28, 0x04, 0x50, 0x06, 0xA0, 0x08, 0x3C, 0x0A, 0x0E, 0x0C, 0x1A, 0x0E,
    0x0C, 0x10, 0x18, 0x12, 0x30, 0x14, 0x60, 0x16, 0xC0, 0x18, 0x48, 0x1A, 0x10, 0x1C, 0x20, 0x1E,
];

#[derive(Clone, Copy, Default, Debug)]
pub struct TriangleRegister {
    pub length_counter: LengthCounter,
    pub linear_counter: usize,
    // Linear counter reload value
    pub linear_counter_reload: usize,
    pub reload_flag: bool,
    pub timer_reload: u32,
    pub enabled: bool,
    pub sequencer: u32,
    pub timer: u32,
}
impl AudioRegister for TriangleRegister {
    fn muted(&self) -> bool {
        !self.enabled
            || self.length_counter.muted()
            || self.timer_reload < 2
            || self.linear_counter == 0
    }
    fn value(&self) -> u32 {
        if self.muted() {
            0
        } else {
            if self.sequencer <= 15 {
                15 - self.sequencer
            } else {
                self.sequencer - 16
            }
        }
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !self.enabled {
            self.length_counter.load = 0;
        }
    }
}

const NOISE_TIMER_PERIODS: [u32; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];
#[derive(Clone, Copy, Debug)]
pub struct NoiseRegister {
    pub length_counter: LengthCounter,
    pub enabled: bool,
    pub timer: u32,
    pub timer_reload: u32,
    pub envelope: Envelope,
    // false = 0, true = 1
    pub mode: bool,
    // This is actually 15 bits wide
    pub shift: u16,
}

impl Default for NoiseRegister {
    fn default() -> Self {
        NoiseRegister {
            length_counter: LengthCounter::default(),
            enabled: false,
            timer: 0,
            timer_reload: 0,
            envelope: Envelope::default(),
            mode: false,
            shift: 1,
        }
    }
}

impl AudioRegister for NoiseRegister {
    fn muted(&self) -> bool {
        !self.enabled || self.length_counter.muted() || self.shift & 0x01 == 1
    }
    fn value(&self) -> u32 {
        if self.muted() {
            0
        } else {
            self.envelope.value()
        }
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !self.enabled {
            self.length_counter.load = 0;
        }
    }
}

const DMC_RATES: [u32; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
];

#[derive(Clone, Debug, Default)]
pub struct DmcRegister {
    enabled: bool,
    irq_enabled: bool,
    repeat: bool,
    rate: u32,
    timer: u32,
    time_reload: u32,
    // Address of the sample, in CPU memory space
    sample_addr: usize,
    // Length of hte sample in bytes
    sample_len: usize,
    // Number of bytes remaining in the sample
    bytes_remaining: usize,
    // Byte of the sample currently in buffer
    sample: u8,
    // todo maybe remove
    sample_full: Vec<u8>,
    bits_left: u32,
    output: u32,
}
impl AudioRegister for DmcRegister {
    fn muted(&self) -> bool {
        false
    }
    fn value(&self) -> u32 {
        self.output
    }
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

#[derive(Debug, Clone)]
pub struct Apu {
    pub pulse_registers: [PulseRegister; 2],
    pub triangle_register: TriangleRegister,
    pub noise_register: NoiseRegister,
    pub dmc_register: DmcRegister,
    irq_inhibit: bool,
    // false = 0, true = 1
    mode: bool,
    // Timer to get to next step
    cycles: u32,
    // Step in either a 4 or 5 step sequence, depending on mode
    step: u32,
}

impl Apu {
    pub fn new() -> Apu {
        Apu {
            pulse_registers: [PulseRegister::default(); 2],
            triangle_register: TriangleRegister::default(),
            noise_register: NoiseRegister::default(),
            dmc_register: DmcRegister::default(),
            irq_inhibit: true,
            mode: false,
            cycles: 0,
            step: 0,
        }
    }
    /// Write a byte of data to the APU given its address in CPU memory space
    //TODO : Remove cartridge from here and solve the audio cartridge issue
    pub fn write_byte(&mut self, addr: usize, value: u8, cartridge: &Cartridge) {
        match addr {
            0x4000..0x4004 => self.set_pulse_byte(0, addr, value),
            0x4004..0x4008 => self.set_pulse_byte(1, addr, value),
            0x4008 => {
                self.triangle_register.length_counter.halt = (value & 0x80) != 0;
                self.triangle_register.linear_counter_reload = (value & 0x7F) as usize;
            }
            0x4009 => {}
            0x400A => {
                self.triangle_register.timer_reload =
                    (self.triangle_register.timer_reload & 0x700) + value as u32;
            }
            0x400B => {
                self.triangle_register.timer_reload =
                    (self.triangle_register.timer_reload & 0x0FF) + ((value as u32 & 0x07) << 8);
                self.triangle_register.length_counter.load =
                    LENGTH_TABLE[(value as usize & 0xF8) >> 3];
                self.triangle_register.reload_flag = true;
            }
            0x400C => {
                self.noise_register.length_counter.halt = (value & 0x20) != 0;
                self.noise_register.envelope.constant = (value & 0x10) != 0;
                self.noise_register.envelope.volume = (value & 0x0F) as usize;
            }
            0x400E => {
                self.noise_register.mode = (value & 0x80) != 0;
                self.noise_register.timer_reload = NOISE_TIMER_PERIODS[(value & 0x0F) as usize];
            }
            0x400F => {
                if self.noise_register.enabled {
                    self.noise_register.length_counter.load =
                        LENGTH_TABLE[(value as usize & 0xF8) >> 3];
                }
                self.noise_register.envelope.decay = 0xF;
                self.noise_register.envelope.divider = self.noise_register.envelope.volume;
            }
            0x4010 => {
                self.dmc_register.irq_enabled = (value & 0x80) != 0;
                self.dmc_register.repeat = (value & 0x40) != 0;
                self.dmc_register.rate = DMC_RATES[(value & 0x0F) as usize];
                self.dmc_register.time_reload = self.dmc_register.rate;
            }
            0x4011 => self.dmc_register.output = (value & 0x7F) as u32,
            0x4012 => {
                let d = &mut self.dmc_register;
                d.sample_addr = (value as usize * 64) + 0xC000;
                d.sample_full = (0..d.sample_len)
                    .map(|i| cartridge.read_cpu(d.sample_addr + i))
                    .collect();
                d.sample = if d.sample_full.len() == 0 {
                    0
                } else {
                    d.sample_full[0]
                };
            }
            0x4013 => {
                let d = &mut self.dmc_register;
                d.sample_len = (value as usize) * 16 + 1;
                d.sample_full = (0..d.sample_len)
                    .map(|i| cartridge.read_cpu(d.sample_addr + i))
                    .collect();
                d.sample = if d.sample_full.len() == 0 {
                    0
                } else {
                    d.sample_full[0]
                };
                d.bytes_remaining = d.sample_len;
            }
            0x4015 => {
                self.pulse_registers[0].set_enabled((value & 0x01) != 0);
                self.pulse_registers[1].set_enabled((value & 0x02) != 0);
                self.triangle_register.set_enabled((value & 0x04) != 0);
                self.noise_register.set_enabled((value & 0x08) != 0);
                self.dmc_register.set_enabled((value & 0x10) != 0);
            }
            0x4017 => {
                self.mode = (value & 0x80) != 0;
                self.irq_inhibit = (value & 0x40) != 0;
            }
            _ => warn!("Trying to write {:X} to APU address {:X}", value, addr),
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
                reg.timer_reload = (reg.timer_reload & 0x0700) | value as usize;
            }
            3 => {
                reg.timer_reload = (reg.timer_reload & 0x00FF) | ((value as usize & 0x07) << 8);
                let length = (value & 0xF8) as usize >> 3;
                reg.length_counter.load = LENGTH_TABLE[length];
                // Mimic setting volume start flag
                reg.envelope.decay = 0xF;
                reg.envelope.divider = reg.envelope.volume;
            }
            _ => {} // _ => panic!("Invalid address given to APU"),
        }
    }
    pub fn advance_cycles(&mut self, apu_cycles: u32) {
        const FRAME_CYCLES: u32 = 3728;
        (0..apu_cycles).for_each(|_| {
            self.cycles = (self.cycles + 1) % FRAME_CYCLES;
            if self.cycles == 0 {
                if self.mode == false {
                    // 4 step sequence
                    self.step = (self.step + 1) % 4;
                    self.on_quater_frame();
                    if self.step % 2 == 0 {
                        self.on_half_frame();
                    }
                } else {
                    // 5 step sequence
                    self.step = (self.step + 1) % 5;
                    if self.step != 4 {
                        self.on_quater_frame();
                    }
                    if self.step == 2 || self.step == 0 {
                        self.on_half_frame();
                    }
                }
            }
            self.pulse_registers.iter_mut().for_each(|p| {
                if p.timer == 0 {
                    if p.sequencer == 0 {
                        p.sequencer = 7;
                    } else {
                        p.sequencer -= 1;
                    }
                    p.timer = p.timer_reload;
                } else {
                    p.timer -= 1;
                }
            });
            (0..2).for_each(|_| {
                self.triangle_register.timer = (self.triangle_register.timer + 1)
                    % max(self.triangle_register.timer_reload as u32, 1);
                if self.triangle_register.timer == 0 {
                    self.triangle_register.sequencer = (self.triangle_register.sequencer + 1) % 32;
                }
                let n = &mut self.noise_register;
                n.timer = (n.timer + 1) % max(n.timer_reload, 1);
                if n.timer == 0 {
                    // XOR bit 0 with bit 1 in mode 1 and with bit 6 in mode 0
                    let feedback = (n.shift ^ (n.shift >> if n.mode { 6 } else { 1 })) & 0x01;
                    n.shift = (n.shift >> 1) | (feedback << 14);
                }
                let d = &mut self.dmc_register;
                if d.enabled {
                    d.timer = (d.timer + 1) % max(d.time_reload, 1);
                    if d.timer == 0 {
                        if d.bits_left == 0 {
                            // Go to next byte
                            if d.bytes_remaining == 1 {
                                if d.repeat {
                                    d.bytes_remaining = d.sample_len - 1;
                                    d.sample = d.sample_full[0];
                                    d.bits_left = 8;
                                } else {
                                    d.enabled = false;
                                }
                            } else if d.bytes_remaining > 0 {
                                d.bytes_remaining -= 1;
                                d.bits_left = 8;
                                d.sample = d.sample_full[d.sample_len - 1 - d.bytes_remaining];
                            }
                        } else {
                            // Todo: Don't just clamp, check range
                            d.output = (d.output as i32
                                + if (d.sample & 0x01) == 1 { 2 } else { -2 })
                            .clamp(0, 127) as u32;
                            d.sample = d.sample >> 1;
                            d.bits_left -= 1;
                        }
                        d.timer = d.time_reload;
                    }
                }
            });
        });
    }
    pub fn on_quater_frame(&mut self) {
        self.pulse_registers.iter_mut().for_each(|reg| {
            reg.envelope.clock(reg.length_counter.halt);
        });
        self.noise_register
            .envelope
            .clock(self.noise_register.length_counter.halt);
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
            reg.length_counter.clock();
            // Clock sweep divider
            if reg.sweep_divider == 0 {
                // Compute sweep change
                let sweep_change = reg.timer_reload >> reg.sweep_shift;
                reg.sweep_target_period = max(
                    reg.timer_reload as i32
                        + if reg.sweep_negate {
                            -(sweep_change as i32) - 1
                        } else {
                            -(sweep_change as i32)
                        },
                    0,
                ) as usize;
                if reg.sweep_enabled
                    && reg.timer_reload >= 8
                    && reg.sweep_shift > 0
                    && reg.sweep_target_period < 0x7FF
                {
                    reg.timer_reload = reg.sweep_target_period;
                }
            } else {
                reg.sweep_divider -= 1;
            }
        });
        self.triangle_register.length_counter.clock();
        self.noise_register.length_counter.clock();
    }
    // The current output from the mixer
    pub fn mixer_output(&self) -> f32 {
        // Add up the pulse registers
        let pulse: u32 = self.pulse_registers.iter().map(|p| p.value()).sum();
        if pulse > 30 {
            warn!("Pulse is higher than 30 ({})", pulse);
        }
        let pulse_out = 95.88 / ((8128.9 / pulse as f32) + 100.0);
        let t = self.triangle_register.value();
        let n = self.noise_register.value();
        let d = self.dmc_register.value();
        let tnd_out = if t + n + d == 0 {
            0.0
        } else {
            159.79 / (1.0 / (t as f32 / 8227.0 + n as f32 / 12241.0 + d as f32 / 22638.0) + 100.0)
        };
        tnd_out + pulse_out
    }
}
