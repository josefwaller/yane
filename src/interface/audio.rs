use std::time::{Duration, Instant};

use crate::{
    apu::{AudioRegister, NoiseRegister, PulseRegister, TriangleRegister},
    Apu, Cartridge, Nes, Settings,
};
use log::*;
use sdl2::{
    audio::{AudioCallback, AudioDevice, AudioQueue, AudioSpecDesired, AudioStatus},
    Sdl,
};

pub struct Audio {
    // device: AudioDevice<ApuChannel>,
    queue: AudioQueue<f32>,
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
        let queue = audio.open_queue(None, &spec).unwrap();
        queue.resume();
        info!(
            "Created queue, samples={}, freq={}",
            queue.spec().samples,
            queue.spec().freq
        );
        Audio { queue }
    }
    pub fn update_audio(&mut self, nes: &mut Nes, settings: &Settings) {
        // Clear queue if it's too big
        if self.queue.size() > 8 * 2000 {
            debug!("Clear queue");
            self.queue.clear();
        }
        // Get and transform data
        let data = nes.apu.sample_queue();
        let sample = data
            .chunks(1_789_000 / 2 / 44_100)
            .map(|x| 0.1 * (-1.0 + 2.0 * x.iter().fold(0.0, |a, e| a + *e)))
            .into_iter()
            .collect::<Vec<f32>>();
        // Add to queue
        self.queue
            .queue_audio(
                sample.as_slice(), // data.as_slice()
            )
            .unwrap();
    }
}
