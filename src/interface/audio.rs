use std::time::{Duration, Instant};

use crate::{
    apu::{AudioRegister, NoiseRegister, PulseRegister, TriangleRegister},
    Apu, Cartridge, Nes, Settings,
};
use log::*;
use sdl2::{
    audio::{AudioCallback, AudioDevice, AudioSpecDesired},
    Sdl,
};

struct ApuChannel {
    apu: Apu,
    last_cycle: Duration,
    sample_rate: u32,
    last_output: f32,
}
impl AudioCallback for ApuChannel {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        let apu_cycle_duration = Duration::from_secs(1) / (1_789_773 / 2);
        for x in out.iter_mut() {
            let v = self.apu.mixer_output();
            if v > 1.0 || v < -1.0 {
                warn!("V is wrong {}", v);
                *x = 0.0;
            } else {
                // Set new value
                let val = (v + 1.0) / 2.0;
                *x = val;
                self.last_output = val;
            }
            self.last_cycle += Duration::from_secs(1) / self.sample_rate;
            while self.last_cycle >= apu_cycle_duration {
                self.apu.advance_cycles(1);
                self.last_cycle -= apu_cycle_duration;
            }
        }
    }
}

pub struct Audio {
    device: AudioDevice<ApuChannel>,
}

impl Audio {
    pub fn new(sdl: &Sdl) -> Audio {
        // Setup audio
        let audio = sdl.audio().unwrap();
        let spec = AudioSpecDesired {
            freq: Some(44_100),
            channels: Some(1),
            samples: None,
        };
        let device = audio
            .open_playback(None, &spec, |spec| ApuChannel {
                apu: Apu::new(),
                last_cycle: Duration::ZERO,
                sample_rate: spec.freq as u32,
                last_output: 0.0,
            })
            .unwrap();
        device.resume();
        info!(
            "Device samples={}, freq={}",
            device.spec().samples,
            device.spec().freq
        );
        Audio { device }
    }
    pub fn update_audio(&mut self, nes: &Nes, settings: &Settings) {
        // This is farily messy right now, but basically to avoid skipping in the sound
        // We want to preserve the timer and sequencer
        // So if the wave doesn't change the sound doesn't change
        let v = self
            .device
            .lock()
            .apu
            .pulse_registers
            .map(|p| (p.sequencer, p.timer));
        let t = self.device.lock().apu.triangle_register.timer;
        let s = self.device.lock().apu.triangle_register.sequencer;
        self.device.lock().apu = nes.apu.clone();
        v.iter().enumerate().for_each(|(i, (s, t))| {
            self.device.lock().apu.pulse_registers[i].sequencer = *s;
            self.device.lock().apu.pulse_registers[i].timer = *t;
        });
        self.device.lock().apu.triangle_register.timer = t;
        self.device.lock().apu.triangle_register.sequencer = s;
    }
}
