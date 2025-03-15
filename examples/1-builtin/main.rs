use sdl2::event::{Event, WindowEvent};
use std::{
    env,
    fs::File,
    io::Read,
    process::exit,
    thread::sleep,
    time::{Duration, Instant},
};
use yane::{
    app::{Audio, Config, Input, Window},
    core::{Cartridge, Nes, CPU_CLOCK_SPEED},
};

/// A minimal use of yane to create a fully-functional NES emulator
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
    // Create NES
    let mut nes = Nes::with_cartridge(Cartridge::from_ines(&ines_bytes, None));
    // Initialize SDL
    let sdl = sdl2::init().expect("Unable to initialize SDL");
    let mut sdl_video = sdl.video().expect("Unable to initailize SDL video");
    let sdl_audio = sdl.audio().expect("Unable to initialize SDL audio");
    let mut event_pump = sdl
        .event_pump()
        .expect("Unable to initialize SDL event pump");
    // Create window
    let mut window = Window::from_sdl_video(&mut sdl_video);
    // Get the underlying SDL window to change the title
    window
        .sdl_window()
        .set_title("Example 1 - Built In")
        .unwrap();
    // Create audio
    let mut audio = Audio::from_sdl_audio(&sdl_audio);
    // Create input
    let mut input = Input::new();
    // Just use default settings
    // Could be configured to use different key bindings, change the speed or volume, etc.
    let mut config = Config::default();
    // The amount of time the emulator has been running, in emulator time
    // Used to keep the emulator in sync with the real world
    let mut emu_duration = Duration::ZERO;
    // The time the emulation started
    let start_time = Instant::now();
    // Main loop
    loop {
        // Advance the NES by 1 frame
        let num_cpu_cycles_elapsed = nes
            .advance_frame(&config.emu_settings)
            .expect("Unable to advance NES");
        // Render the NES
        window.render(&nes, &config);
        // Output the NES's audio
        audio.update(&mut nes, &config);
        // Update the NES's inputs
        input.update(&mut nes, &event_pump, &mut config);
        // Check if the window has been closed
        if event_pump.poll_iter().any(|e| match e {
            Event::Window { win_event, .. } => win_event == WindowEvent::Close,
            _ => false,
        }) {
            // Exit the loop
            break;
        }
        // Wait for the appropriate amount of time
        let time_elapsed = Duration::from_nanos(
            1_000_000_000 * num_cpu_cycles_elapsed as u64 / CPU_CLOCK_SPEED as u64,
        );
        emu_duration += time_elapsed;
        let real_duration = Instant::now().duration_since(start_time);
        if emu_duration > real_duration {
            sleep(emu_duration - real_duration);
        }
    }
}
