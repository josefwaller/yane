use crate::{app::Config, core::Nes, core::CPU_CLOCK_SPEED};
use log::*;
use rubato::{Resampler, SincFixedIn, SincInterpolationParameters};
use sdl2::{
    audio::{AudioQueue, AudioSpecDesired},
    Sdl,
};

pub struct Audio {
    queue: AudioQueue<f32>,
    resampler: SincFixedIn<f32>,
    data_queue: Vec<f32>,
    last_speed: f32,
    pub all_samples: Vec<f32>,
}

impl Audio {
    pub fn new(sdl: &Sdl) -> Audio {
        // Setup audio
        let audio = sdl.audio().unwrap();
        let spec = AudioSpecDesired {
            freq: Some(44_800),
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

        // Fixed input, since the input is always the number of samples every frame
        // The output varies with the speed of the emulator
        let resampler = SincFixedIn::<f32>::new(
            queue.spec().freq as f64 / CPU_CLOCK_SPEED as f64,
            10.0,
            SincInterpolationParameters {
                sinc_len: 256,
                f_cutoff: 0.9,
                oversampling_factor: 128,
                interpolation: rubato::SincInterpolationType::Nearest,
                window: rubato::WindowFunction::BlackmanHarris2,
            },
            CPU_CLOCK_SPEED as usize / 60,
            1,
        )
        .expect("Unable to create resampler");
        Audio {
            queue,
            resampler,
            data_queue: Vec::<f32>::new(),
            last_speed: 1.0,
            all_samples: Vec::new(),
        }
    }
    pub fn update_audio(&mut self, nes: &mut Nes, config: &Config) {
        // Clear queue if it's too big
        // Should only happen on startup when SDL is booting up
        if self.queue.size() > CPU_CLOCK_SPEED / 60 {
            debug!("Queue is too big, clearing (was {})", self.queue.size());
            self.queue.clear();
        } else if self.queue.size() == 0 && !config.paused {
            warn!("Queue is empty!");
        }
        // Get and transform data
        let data = nes
            .apu
            .sample_queue()
            .iter()
            .map(|x| config.volume * x)
            .collect::<Vec<f32>>();
        self.data_queue.extend_from_slice(&data);
        if config.record_audio {
            self.all_samples.extend_from_slice(&data);
        }
        // Downsample to audio output rate
        let input_size = self.resampler.input_frames_next();
        let mut out = vec![vec![0.0; self.resampler.output_frames_max()]; 1];
        if config.speed != self.last_speed {
            self.last_speed = config.speed;
            let ratio = (self.queue.spec().freq as f64 / CPU_CLOCK_SPEED as f64)
                / config.speed.min(9.9999) as f64;
            self.resampler.reset();
            self.resampler
                .set_resample_ratio(ratio, false)
                .expect("Unable to change ratio");
            self.queue.clear();
        }
        while self.data_queue.len() >= input_size {
            let input: Vec<f32> = self.data_queue.drain(0..input_size).collect();
            let (_nbr_in, nbr_out) = self
                .resampler
                .process_into_buffer(&vec![input], &mut out, None)
                .expect("Unable to resample audio");
            // Add to queue
            self.queue
                .queue_audio(&out[0][..nbr_out])
                .expect("Unable to queue audio");
        }
    }
}
