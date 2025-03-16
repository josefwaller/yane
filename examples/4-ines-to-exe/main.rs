use sdl2::event::{Event, WindowEvent};
use std::{
    thread::sleep,
    time::{Duration, Instant},
};
use yane::{
    app::{Audio, Config, Input, Window},
    core::{Cartridge, Nes, CPU_CLOCK_SPEED},
};

/// A minimal use of yane to create a fully-functional NES emulator
fn main() {
    // Change this path to use your own NES game
    let ines_bytes = include_bytes!("./thwaite.nes");
    // Create NES
    let mut nes = Nes::with_cartridge(Cartridge::from_ines(ines_bytes, None));
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
    // The time the emulation started
    let start_time = Instant::now();
    let mut total_cycles = 0;
    // Main loop
    loop {
        // Advance the NES by 1 frame
        total_cycles += nes
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
        let emu_duration =
            Duration::from_nanos(1_000_000_000 * total_cycles as u64 / CPU_CLOCK_SPEED as u64);
        let real_duration = Instant::now().duration_since(start_time);
        if emu_duration > real_duration {
            sleep(emu_duration - real_duration);
        }
    }
}
