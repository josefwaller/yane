use crate::{
    apu::{AudioRegister, NoiseRegister, PulseRegister, TriangleRegister},
    Nes, Settings,
};
use sdl2::{
    audio::{AudioCallback, AudioDevice, AudioSpecDesired},
    Sdl,
};

#[derive(Clone, Copy)]
struct PulseWave {
    // The pulse register this device is attached to
    register: PulseRegister,
    // How many sampels are taken every second, i.e. how many times callback is called per second
    samples_per_second: i32,
    // The phase of the wave, i.e. the progress through a single wave
    phase: f32,
    max_volume: f32,
}
impl AudioCallback for PulseWave {
    type Channel = f32;

    // Ouputs the amplitude
    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            // Output no sound if muted one way or another
            if self.register.muted() {
                *x = 0.0;
            } else {
                *x = self.max_volume * 0.25 * (1.0 - 2.0 * self.register.amp(self.phase));
            }
            // Advance phase
            let clock = 1_789_000.0;
            let freq = clock / (16.0 * (self.register.timer as f32 + 1.0));
            self.phase = (self.phase + (freq / self.samples_per_second as f32)) % 1.0;
        }
    }
}

#[derive(Clone, Copy)]
struct TriangleWave {
    register: TriangleRegister,
    phase: f32,
    sample_rate: i32,
    max_volume: f32,
}
impl AudioCallback for TriangleWave {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            let amp = self.register.amp(self.phase);
            if self.register.muted() {
                *x = 0.0;
            } else {
                *x = self.max_volume * 0.25 * (2.0 * amp - 1.0);
            }
            let freq = 1_789_000.0 / (32.0 * (self.register.timer + 1) as f32);
            self.phase = (self.phase + (freq / self.sample_rate as f32)) % 1.0;
        }
    }
}
struct NoiseWave {
    register: NoiseRegister,
    max_volume: f32,
}
impl AudioCallback for NoiseWave {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            if self.register.muted() {
                *x = 0.0;
            } else {
                // Generate random noise
                *x = self.max_volume * 0.25 * (1.0 - self.register.amp(0.0));
            }
        }
    }
}

pub struct Audio {
    pulse_devices: [AudioDevice<PulseWave>; 2],
    triangle_device: AudioDevice<TriangleWave>,
    noise_device: AudioDevice<NoiseWave>,
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
        let pulse_devices: [AudioDevice<PulseWave>; 2] = core::array::from_fn(|i| {
            let device = audio
                .open_playback(None, &spec, |spec| PulseWave {
                    register: PulseRegister::default(),
                    samples_per_second: spec.freq,
                    phase: 0.0,
                    max_volume: 1.0,
                })
                .unwrap();
            device.resume();
            device
        });
        let triangle_device = audio
            .open_playback(None, &spec, |spec| TriangleWave {
                register: TriangleRegister::default(),
                sample_rate: spec.freq,
                phase: 0.0,
                max_volume: 1.0,
            })
            .unwrap();
        triangle_device.resume();
        let noise_device = audio
            .open_playback(None, &spec, |spec| NoiseWave {
                register: NoiseRegister::default(),
                max_volume: 1.0,
            })
            .unwrap();
        noise_device.resume();
        Audio {
            pulse_devices,
            triangle_device,
            noise_device,
        }
    }
    pub fn update_audio(&mut self, nes: &Nes, settings: &Settings) {
        self.pulse_devices
            .iter_mut()
            .enumerate()
            .for_each(|(i, d)| {
                d.lock().register = nes.apu.pulse_registers[i];
                d.lock().max_volume = settings.volume;
            });
        self.triangle_device.lock().register = nes.apu.triangle_register;
        self.triangle_device.lock().max_volume = settings.volume;
        self.noise_device.lock().register = nes.apu.noise_register;
        self.noise_device.lock().max_volume = settings.volume;
    }
}
