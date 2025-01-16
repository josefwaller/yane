use crate::{Nes, Settings};
use log::*;
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use sdl2::{
    audio::{AudioQueue, AudioSpecDesired},
    Sdl,
};
use wavers::Samples;

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

        let params = SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.9,
            oversampling_factor: 128,
            interpolation: SincInterpolationType::Nearest,
            window: WindowFunction::BlackmanHarris2,
        };
        let resampler = SincFixedIn::<f32>::new(
            spec.freq.unwrap() as f64 / 1_789_000 as f64,
            10.0,
            params,
            1_789_000 / 60,
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
    pub fn update_audio(&mut self, nes: &mut Nes, settings: &Settings) {
        // Clear queue if it's too big
        if self.queue.size() > 16 * 2000 {
            info!("Queue is too big, clearing (was {})", self.queue.size());
            self.queue.clear();
        } else if self.queue.size() == 0 && !settings.paused {
            warn!("Queue is empty!");
        }
        // Get and transform data
        let data = nes
            .apu
            .sample_queue()
            .iter()
            .map(|x| settings.volume * x)
            .collect::<Vec<f32>>();
        self.data_queue.extend_from_slice(&data);
        if settings.record_audio {
            self.all_samples.extend_from_slice(&data);
        }
        // Downsample to audio output rate
        let input_size = self.resampler.input_frames_next();
        let mut out = vec![vec![0.0; self.resampler.output_frames_max()]; 1];
        if settings.speed != self.last_speed {
            self.last_speed = settings.speed;
            let ratio = (self.queue.spec().freq as f64 / 1_789_000 as f64)
                / settings.speed.min(9.9999) as f64;
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
