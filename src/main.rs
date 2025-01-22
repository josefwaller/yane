use log::*;
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
};
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::{
    ffi::OsStr,
    fs::OpenOptions,
    io::BufReader,
    path::Path,
    time::{Duration, Instant},
};
use std::{ffi::OsString, thread::sleep};
use std::{fs::File, path::PathBuf};
use wavers::{write, Samples};
use yane::{
    Cartridge, DebugWindow, Nes, Screen, Settings, Window, CPU_CYCLES_PER_OAM,
    CPU_CYCLES_PER_SCANLINE, CPU_CYCLES_PER_VBLANK,
};

fn main() {
    {
        // Initialize logger
        CombinedLogger::init(vec![
            TermLogger::new(
                LevelFilter::Debug,
                Config::default(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ),
            WriteLogger::new(
                LevelFilter::Debug,
                Config::default(),
                File::create("./yane.log").unwrap(),
            ),
        ])
        .expect("Unable to create logger");
        // Read file and init NES
        let args: Vec<String> = std::env::args().collect();
        let file_name = args[1].clone();
        // Read cartridge data
        let data = std::fs::read(file_name.clone()).expect("Please provide an iNES (.NES) file");
        // Read savedata, if there is any
        let mut savedata_path = PathBuf::from(file_name.clone());
        savedata_path.set_extension("srm");
        let savedata = match std::fs::read(savedata_path.clone()) {
            Ok(d) => Some(d),
            Err(_) => None,
        };
        let mut nes = Nes::from_cartridge(Cartridge::new(data.as_slice(), savedata));
        let mut settings = Settings::default();

        let sdl = sdl2::init().unwrap();
        // Setup video
        let video = sdl.video().unwrap();
        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(3, 3);
        // Setup input
        // The two windows need a shared event pump since SDL only allows one at a time
        let mut event_pump = sdl.event_pump().unwrap();

        let mut debug_window = DebugWindow::new(&nes, &video, &sdl);
        let mut window = Window::new(&video, &sdl);

        let mut last_debug_window_render = Instant::now();
        // Various constants for keeping emulator time in check with real time
        const DEBUG_WINDOW_REFRESH_RATE: Duration = Duration::from_millis(1000 / 60);
        const CPU_CYCLES_PER_FRAME: f32 = 262.0 * CPU_CYCLES_PER_SCANLINE;
        let wait_time_per_cycle =
            Duration::from_nanos(1_000_000_000 / 60 / CPU_CYCLES_PER_FRAME as u64);
        info!(
            "FPS = 60, cycles/scanline={CPU_CYCLES_PER_SCANLINE}, cycles/vblank={CPU_CYCLES_PER_VBLANK}, cycles/frame={CPU_CYCLES_PER_FRAME}, wait time={wait_time_per_cycle:?}",
        );
        let fps =
            1_000_000_000.0 / (CPU_CYCLES_PER_FRAME as f64 * wait_time_per_cycle.as_nanos() as f64);
        info!("Calculated FPS: {fps}");
        // Used for logging information every 100 frames
        let mut last_hundred_frames = Instant::now();
        let mut frame_cycles = 0;
        let mut frame_count = 0;
        let mut frame_wait_time = Duration::ZERO;
        let mut delta = Instant::now();
        loop {
            // Update IMGUI/Window input
            let mut should_exit = false;
            for event in event_pump.poll_iter() {
                match event {
                    Event::Window { win_event, .. } => match win_event {
                        WindowEvent::Close => should_exit = true,
                        _ => {}
                    },
                    _ => debug_window.handle_event(&event),
                }
            }
            if should_exit {
                break;
            }
            // Update game input
            let keys: Vec<Keycode> = event_pump
                .keyboard_state()
                .pressed_scancodes()
                .filter_map(Keycode::from_scancode)
                .collect();

            // Render debug window
            if Instant::now().duration_since(last_debug_window_render) >= DEBUG_WINDOW_REFRESH_RATE
            {
                last_debug_window_render += DEBUG_WINDOW_REFRESH_RATE;
                debug_window.render(&nes, &event_pump, &mut settings);
                window.screen().set_settings(settings.clone());
            }
            // Update window
            window.update(&mut nes, keys, &settings);
            // Update CPU
            if settings.paused {
                delta = Instant::now();
            } else {
                // Advance 1 frame
                window.make_gl_current();
                let cycles_to_wait = match nes.advance_frame(Some(settings)) {
                    Ok(c) => c,
                    Err(e) => {
                        error!("Error encountered while advancing emulator: {:X?}", e);
                        break;
                    }
                };
                frame_cycles += cycles_to_wait;
                // Debug log FPS info
                frame_count += 1;
                if frame_count == 600 {
                    frame_count = 0;
                    let now = Instant::now();
                    info!(
                        "Over last 600 frames: Avg FPS: {}, duration: {:?}, avg cycles: {}, avg wait time {:#?}",
                        600.0
                            / (now.duration_since(last_hundred_frames).as_millis() as f32 / 1000.0),
                        now.duration_since(last_hundred_frames),
                        frame_cycles as f64 / 600.0,
                        frame_wait_time.div_f64(600.0)
                    );
                    // Uncomment this to verify screenshot results
                    let screen: Vec<String> = nes
                        .ppu
                        .nametable_ram
                        .chunks(32)
                        .map(|row| {
                            row.iter()
                                .map(|r| format!("{:2X?}", r))
                                .collect::<Vec<String>>()
                                .join(" ")
                        })
                        .collect();
                    info!("{:?}", screen);

                    frame_cycles = 0;
                    frame_wait_time = Duration::ZERO;
                    last_hundred_frames = now;
                }
                // Render window
                window.render(&nes, &settings);
                // Calculate how much time has passed in the emulation
                let emu_elapsed = wait_time_per_cycle
                    .saturating_mul(cycles_to_wait as u32)
                    .div_f32(settings.speed);
                // Calculate how much time has actually passed
                let actual_elapsed = Instant::now().duration_since(delta);
                // Wait for the difference
                let wait_duration = emu_elapsed.saturating_sub(actual_elapsed);
                // If we are going too fast, slow down
                // Check if we want to slow down first, since sleep is costly even if wait_duration is 0
                if wait_duration != Duration::ZERO {
                    frame_wait_time += wait_duration;
                    sleep(wait_duration);
                } else if Instant::now().duration_since(delta) > Duration::from_millis(500) {
                    // If we have fallen way behind (by messing with the speed in settings)
                    delta = Instant::now();
                }
                // Advance real time by amount of emulator time that will have passed
                // Since sleep may overshoot, this will let us catch up next frame/scanline
                delta += emu_elapsed;
            }
        }
        // Save audio recording

        let data = window.audio.all_samples.into_boxed_slice();
        let samples = Samples::new(data);
        write(&Path::new("./sample.wav"), &samples, 1_789_000, 1).unwrap();
        // Save game if we want to
        if nes.cartridge.has_battery_backed_ram() {
            info!("Writing savedata to to {:#?}", savedata_path);
            std::fs::write(savedata_path, nes.cartridge.memory.prg_ram)
                .expect("Unable to save savefile");
        }
    }
}
