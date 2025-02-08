use clap::{Parser, Subcommand};
use log::*;
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
};
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::thread::sleep;
use std::{fs::File, io::Write, path::PathBuf};
use std::{
    path::Path,
    time::{Duration, Instant},
};
use wavers::{write, Samples};
use yane::{
    Cartridge, DebugWindow, KeyMap, Nes, Settings, Window, CPU_CYCLES_PER_SCANLINE,
    CPU_CYCLES_PER_VBLANK,
};

fn get_in_config_dir(path: &str) -> PathBuf {
    let mut buf = dirs::home_dir().unwrap_or(PathBuf::new());
    buf.push(".yane");
    buf.push(path);
    buf
}

#[derive(Parser)]
#[command(name = "Yane", version = "0.9", about = "An N.E.S. emulator.")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}
#[derive(Parser)]
struct CommonArgs {
    /// The configuration file for the emulator's settings
    #[arg(short, long, value_name = "~/.yane/settings.yml")]
    config_file: Option<String>,
    /// Start in debug mode
    #[arg(short, long)]
    debug: bool,
    /// Start paused, after advancing one frame in order to render one frame
    #[arg(short, long)]
    paused: bool,
    /// Start the emulator muted
    #[arg(short, long)]
    muted: bool,
    /// The .YAML file defining the key mappings for the NES
    #[arg(long, default_value = get_in_config_dir("key_map.yaml").into_os_string(), value_name = "FILE")]
    keymap_file: PathBuf,
}
#[derive(Subcommand)]
enum Command {
    /// Initialize the configuration files at $HOME/.yane/
    Setup,
    /// Load and run an iNES (.nes) file
    Ines {
        /// The iNes file to run
        nes_file: String,
        /// The savedata file (.SRM).
        /// Only used if the NES game supports battery backed static ram.
        /// Will default to using the NES file name and location.
        #[arg(short, long)]
        savedata_file: Option<String>,
        #[command(flatten)]
        args: CommonArgs,
    },
    /// Load and run a savestate
    Savestate {
        /// The binary savestate to load
        savestate_file: String,
        #[command(flatten)]
        args: CommonArgs,
    },
}

fn get_config_dir_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    match dirs::home_dir() {
        None => Err(Box::from("Unable to get home directory!")),
        Some(mut path) => {
            path.push(".yane");
            Ok(path)
        }
    }
}

fn add_config_file(file_name: &str, contents: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = get_config_dir_path()?;
    buf.push(file_name);
    debug!("Creating file {:?}", buf);
    let mut f = std::fs::File::create(&buf)?;
    debug!("Created file {}", file_name);
    let mut p = f.metadata()?.permissions();
    p.set_readonly(false);
    f.set_permissions(p)?;
    // Write contents
    f.write_all(contents.as_bytes())?;
    debug!("Wrote contents to {:?}", file_name);
    Ok(())
}

fn setup_config_directory() -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = get_config_dir_path()?;
    debug!("Creating directory at {:?}", &config_dir);
    let _ = std::fs::create_dir(&config_dir);
    debug!("Created directory at {:?}", &config_dir);
    let k = KeyMap::default();
    let contents = serde_yaml::to_string(&k)?;
    add_config_file("key_map.yaml", &contents)?;
    Ok(())
}
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
        let sdl = sdl2::init().expect("Unable to initialize SDL");
        // Setup video
        let video = sdl.video().expect("Unable to initialize SDL video");
        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(3, 3);
        // Setup input
        // The two windows need a shared event pump since SDL only allows one at a time
        let mut event_pump = sdl
            .event_pump()
            .expect("Unable to initialize SDL event pump");

        // Read file and init NES
        let cli = Cli::parse();
        let (mut nes, savedata_path, args) = match &cli.command {
            Some(Command::Setup) => match setup_config_directory() {
                Ok(()) => {
                    info!("Successfully created configuration files");
                    std::process::exit(0)
                }
                Err(e) => {
                    error!("Unable to create configuration files: {}", e);
                    std::process::exit(1)
                }
            },
            Some(Command::Ines {
                nes_file,
                savedata_file,
                args,
            }) => {
                // Read cartridge data
                let data: Vec<u8> = match std::fs::read(nes_file.clone()) {
                    Ok(data) => data,
                    Err(_) => {
                        println!("Unable to read the file '{}'", nes_file);
                        std::process::exit(1);
                    }
                };
                // Read savedata, if there is any
                let (savedata_path, savedata) = match savedata_file {
                    Some(f) => match std::fs::read(f) {
                        Ok(data) => (f.into(), Some(data)),
                        Err(_) => {
                            println!("Unable to read the .SRM file at '{}'", f);
                            std::process::exit(1);
                        }
                    },
                    None => {
                        let mut savedata_path = PathBuf::from(nes_file.clone());
                        savedata_path.set_extension("srm");
                        let savedata = match std::fs::read(savedata_path.clone()) {
                            Ok(d) => Some(d),
                            Err(_) => None,
                        };
                        (
                            savedata_path.into_os_string().into_string().unwrap(),
                            savedata,
                        )
                    }
                };
                let nes = Nes::from_cartridge(Cartridge::new(data.as_slice(), savedata));
                (nes, savedata_path, args)
            }
            Some(Command::Savestate {
                savestate_file,
                args,
            }) => match std::fs::read(savestate_file) {
                Err(e) => {
                    error!("Unable to read {}: {}", savestate_file, e);
                    std::process::exit(1);
                }
                Ok(data) => match Nes::from_savestate(data) {
                    Err(e) => {
                        error!("Unable to deserialize NES: {}", e);
                        std::process::exit(1);
                    }
                    Ok(nes) => (nes, "".to_string(), args),
                },
            },
            None => {
                todo!()
            }
        };
        let mut settings = load_settings(args.config_file.clone());
        let path = &args.keymap_file;
        debug!("Loading key map from {:?}", &path);
        let key_map = match std::fs::read_to_string(path) {
            Err(e) => {
                info!("No custom key mapping file found, using defaults");
                KeyMap::default()
            }
            Ok(contents) => match serde_yaml::from_str(contents.as_str()) {
                Ok(km) => {
                    debug!("Successfully read keymappings");
                    km
                }
                Err(e) => {
                    error!(
                        "Unable to parse the file {:?}: {}. Using default key mappings",
                        path, e
                    );
                    KeyMap::default()
                }
            },
        };
        settings.key_map = key_map;

        let mut debug_window = if args.debug {
            Some(DebugWindow::new(&nes, &video, &sdl))
        } else {
            None
        };
        let mut window = Window::new(&video, &sdl);
        if args.paused {
            settings.paused = true;
            match nes.advance_frame(&settings) {
                Err(e) => error!("Error when advancing NES first frame: {}", e),
                Ok(_) => {}
            }
        }
        if args.muted {
            settings.volume = 0.0;
        }

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
                    _ => match debug_window.as_mut() {
                        Some(d) => d.handle_event(&event),
                        None => {}
                    },
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
                match debug_window.as_mut() {
                    Some(d) => {
                        if d.render(&mut nes, &event_pump, &mut settings) {
                            nes.reset();
                        }
                    }
                    None => {}
                }
                window.screen().set_settings(settings.clone());
            }
            // Update window
            window.update(&mut nes, &keys, &mut settings);
            // Render window
            window.render(&nes, &settings);
            // Update CPU
            if settings.paused {
                delta = Instant::now();
            } else {
                // Advance 1 frame
                window.make_gl_current();
                let cycles_to_wait = match nes.advance_frame(&settings) {
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

                    frame_cycles = 0;
                    frame_wait_time = Duration::ZERO;
                    last_hundred_frames = now;
                }
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
        write(
            &Path::new(format!("./{}.wav", settings.record_audio_filename).as_str()),
            &samples,
            1_789_000,
            1,
        )
        .unwrap();
        // Save game if we want to
        if nes.cartridge.has_battery_backed_ram() {
            info!("Writing savedata to to {:#?}", savedata_path);
            std::fs::write(&savedata_path, nes.cartridge.memory.prg_ram)
                .expect("Unable to save savefile");
        }
    }
}

fn load_settings(settings_path: Option<String>) -> Settings {
    // TODO: Read settings
    Settings::default()
}
