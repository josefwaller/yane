use sdl2::{
    audio::{AudioQueue, AudioSpecDesired},
    event::{Event, WindowEvent},
    keyboard::Scancode,
    rect::Point,
};
use std::{
    env,
    fs::File,
    io::Read,
    process::exit,
    thread::sleep,
    time::{Duration, Instant},
};
use yane::core::{Cartridge, Controller, Nes, Settings, CPU_CLOCK_SPEED, HV_TO_RGB};

fn main() {
    // Read first argument as the path to an iNes cartridge
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Please provide an iNes file to run");
        exit(0);
    }
    let ines_file = &args[1];
    println!("Running {}", ines_file);
    let ines_bytes = {
        let mut buf = Vec::new();
        File::open(ines_file)
            .expect("Unable to open iNes file")
            .read_to_end(&mut buf)
            .expect("Unable to read iNes file");
        buf
    };
    // Create the NES
    let mut nes = Nes::with_cartridge(Cartridge::from_ines(&ines_bytes, None));
    let settings = Settings::default();
    // Initialize SDL2
    let sdl = sdl2::init().unwrap();
    // Setup video
    let video = sdl.video().unwrap();
    let window = video
        .window("Example 2 - explicit", 256, 240)
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    // Setup audio
    let audio = sdl.audio().unwrap();
    let queue: AudioQueue<f32> = audio
        .open_queue::<f32, _>(
            None,
            &AudioSpecDesired {
                freq: Some(44_800),
                channels: Some(1),
                samples: None,
            },
        )
        .unwrap();
    let mut sample_queue = Vec::new();
    // Setup input
    let mut event_pump = sdl.event_pump().unwrap();

    let start_time = Instant::now();
    let mut emu_duration = Duration::ZERO;

    queue.clear();
    // While the window is open
    while !event_pump.poll_iter().any(|e| match e {
        Event::Window { win_event, .. } => win_event == WindowEvent::Close,
        _ => false,
    }) {
        // Advance the nes
        let cpu_cycles = nes.advance_frame(&settings).unwrap();

        // Render the NES's video output
        //
        // Get the NES's video output
        // This output will be a single byte per pixel on a 256x240 display,
        // with each byte representing a hue-value color
        let video_output: &Box<[[usize; 256]; 240]> = &nes.ppu.output;
        // Convert it to RGB colors
        let rgb_output: [[[u8; 3]; 256]; 240] =
            core::array::from_fn(|y| core::array::from_fn(|x| HV_TO_RGB[video_output[y][x]]));
        // Render it to the SDL canvas
        for y in 0..240 {
            for x in 0..256 {
                let pixel = rgb_output[y][x];
                canvas.set_draw_color((pixel[0], pixel[1], pixel[2]));
                canvas.draw_point(Point::new(x as i32, y as i32)).unwrap();
            }
        }
        canvas.present();

        // Add the NES's audio output to the SDL audio queue
        //
        // The NES will generate one new same every CPU clock (~1,789,000 Hz)
        // Our sample queue will consume samples at 44_800 Hz
        // So we average the NES sound output
        sample_queue.extend_from_slice(&nes.apu.sample_queue());
        let chunks = sample_queue
            .chunks_exact((CPU_CLOCK_SPEED as f64 / queue.spec().freq as f64).ceil() as usize);
        let remainder = chunks.remainder();
        let resampled_output: Vec<f32> = chunks
            .map(|c| c.iter().sum::<f32>() / c.len() as f32)
            .collect();
        // If we are over 1/2 a second ahead, clear the queue
        // This can sometimes happen because SDL takes a second to start up
        if queue.size() > queue.spec().freq as u32 {
            println!("Clearing queue");
            queue.clear();
        }
        queue.queue_audio(&resampled_output).unwrap();
        sample_queue = remainder.to_vec();
        queue.resume();

        // Update the NES's controllers
        //
        // Change the controller inputs for P1 only using the key map below
        // Up - W
        // Left - A
        // Down - S
        // Right - D
        // A - N
        // B - M
        // Start - R
        // Select - T
        let keys: Vec<Scancode> = event_pump.keyboard_state().pressed_scancodes().collect();
        let controller_state = Controller {
            up: keys.contains(&Scancode::W),
            left: keys.contains(&Scancode::A),
            down: keys.contains(&Scancode::S),
            right: keys.contains(&Scancode::D),
            a: keys.contains(&Scancode::N),
            b: keys.contains(&Scancode::M),
            start: keys.contains(&Scancode::R),
            select: keys.contains(&Scancode::T),
        };
        nes.set_input(0, controller_state);

        // Sync the NES with reality
        emu_duration +=
            Duration::from_nanos(1_000_000_000 * cpu_cycles as u64 / CPU_CLOCK_SPEED as u64);
        let real_duration = Instant::now().duration_since(start_time);
        if emu_duration > real_duration {
            sleep(emu_duration - real_duration);
        }
    }
}
