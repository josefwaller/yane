use clap::{Parser, Subcommand};
use log::*;
use regex::Regex;
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
};
use serde::de::DeserializeOwned;
use simplelog::{ColorChoice, CombinedLogger, LevelFilter, TermLogger, TerminalMode, WriteLogger};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::thread::sleep;
use std::{fs::File, io::Write, path::PathBuf};
use std::{
    path::Path,
    time::{Duration, Instant},
};
use wavers::{write, Samples};
use yane::{
    app::{Audio, Config, DebugWindow, Input, KeyMap, Window},
    core::{Cartridge, Nes, CPU_CLOCK_SPEED},
};

const SETTINGS_FILENAME: &str = "settings.yaml";
const KEYMAP_FILENAME: &str = "key_map.yaml";

// Used for argument default values
fn get_file_in_config_dir(path: &str) -> PathBuf {
    let mut buf = get_config_dir_path().unwrap_or_default();
    buf.push(path);
    buf
}

fn get_cli_styles() -> clap::builder::Styles {
    use anstyle::{AnsiColor::*, Color::Ansi, Style};
    use clap::builder::Styles;
    Styles::styled()
        .header(Style::new().fg_color(Some(Ansi(Red))).underline())
        .usage(Style::new().fg_color(Some(Ansi(Red))).underline())
        .placeholder(Style::new().fg_color(Some(Ansi(BrightBlack))))
        .valid(Style::new().fg_color(Some(Ansi(BrightRed))))
        .literal(Style::new().fg_color(Some(Ansi(BrightWhite))).bold())
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
    #[arg(long, default_value = get_file_in_config_dir(SETTINGS_FILENAME).into_os_string(), value_name = "FILE")]
    config_file: PathBuf,
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
    #[arg(long, default_value = get_file_in_config_dir(KEYMAP_FILENAME).into_os_string(), value_name = "FILE")]
    keymap_file: PathBuf,
}
#[derive(Subcommand)]
#[command(styles=get_cli_styles())]
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
    /// Load and run a savestate (.bin) file. Savestate must have been created by Yane
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
    #[cfg(unix)]
    p.set_mode(0o644);
    #[cfg(windows)]
    #[allow(clippy::permissions_set_readonly_false)]
    p.set_readonly(false);
    f.set_permissions(p)?;
    // Write contents
    debug!("Writing contents to {:?}", file_name);
    f.write_all(contents.as_bytes())?;
    debug!("Wrote contents to {:?}", file_name);
    Ok(())
}

fn setup_config_directory() -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = get_config_dir_path()?;
    debug!("Creating directory at {:?}", &config_dir);
    let _ = std::fs::create_dir(&config_dir);
    debug!("Created directory at {:?}", &config_dir);
    let contents = serde_yaml::to_string(&KeyMap::default())?;
    add_config_file(KEYMAP_FILENAME, &contents)?;
    let contents = serde_yaml::to_string(&Config::default())?;
    add_config_file(SETTINGS_FILENAME, &contents)?;
    Ok(())
}
// Try to create a directory, falling back on the current directory (".") if it fails
// Retuns a PathBuf to the directory created
fn try_create_dir(path: &PathBuf) -> PathBuf {
    if Path::exists(path) {
        path.clone()
    } else {
        match std::fs::create_dir_all(path) {
            Ok(()) => {
                debug!("Created {:?}", path);
                path.clone()
            }
            Err(e) => {
                error!("Unable to create {:?}: {}", path, e);
                PathBuf::from(".")
            }
        }
    }
}
fn read_config_file<T>(path: &PathBuf, fallback: T) -> T
where
    T: DeserializeOwned + Clone,
{
    debug!("Reading config file {:?}", path);

    match std::fs::read_to_string(path) {
        Err(e) => {
            debug!(
                "Unable to read file {:?}: {}, using fallback value",
                path, e
            );
            fallback
        }
        Ok(contents) => match serde_yaml::from_str(&contents) {
            Ok(value) => {
                debug!("Successfully deserialized file {:?}", path);
                value
            }
            Err(e) => {
                error!(
                    "Unable to parse the file {:?}: {}, using fallback value",
                    path, e
                );
                fallback
            }
        },
    }
}
fn get_filename(path: &str) -> Result<String, String> {
    Ok(PathBuf::from(path)
        .with_extension("")
        .file_name()
        .ok_or("Cannot get filename")?
        .to_str()
        .ok_or("Cannot convert to str")?
        .to_string())
}
fn game_name_from_savestate(savestate_path: &str) -> Result<String, String> {
    let game_name_regex = Regex::new(
        "savestate_(.+)__[0-9]{4}_[0-9]{2}_[0-9]{2}__[0-9]{2}_[0-9]{2}_[0-9]{2}.yane.bin",
    )
    .map_err(|e| e.to_string())?;
    Ok(game_name_regex
        .captures(savestate_path)
        .ok_or("Does not match Regex")?
        .get(1)
        .ok_or("No matches")?
        .as_str()
        .to_string())
}
fn savedata_path_and_data(savedata_path: &str) -> (Option<String>, Option<Vec<u8>>) {
    // If the file exists
    if std::fs::exists(savedata_path).is_ok_and(|v| v) {
        match std::fs::read(savedata_path) {
            // Happy path, savedata arg is present and the file can be read
            Ok(data) => (Some(savedata_path.to_string()), Some(data)),
            // Can't read from file and file already exists, log error and save to nothing
            // Don't want to overwrite save data
            Err(e) => {
                error!("Unable to read the .SRM file at '{}': {}", savedata_path, e);
                (None, None)
            }
        }
    } else {
        // File doesn't exist and arg was provided
        // Save to savedata file
        (Some(savedata_path.to_string()), None)
    }
}
fn main() {
    {
        // Initialize logger
        CombinedLogger::init(vec![
            TermLogger::new(
                LevelFilter::Debug,
                simplelog::Config::default(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ),
            WriteLogger::new(
                LevelFilter::Debug,
                simplelog::Config::default(),
                File::create("./yane.log").unwrap(),
            ),
        ])
        .expect("Unable to create logger");
        // Initialise SDL
        let sdl = sdl2::init().expect("Unable to initialise SDL");
        let mut sdl_video = sdl.video().expect("Unable to initialise VideoSubsystem");
        let mut window = Window::from_sdl_video(&mut sdl_video);
        let sdl_audio = sdl.audio().expect("Unable to initialise AudioSubsystem");
        let mut audio = Audio::from_sdl_audio(&sdl_audio);
        let mut input = Input::new();

        let mut event_pump = sdl
            .event_pump()
            .expect("Unable to initialize SDL event pump");

        // Read file and init NES
        let cli = Cli::parse();
        let (mut nes, savedata_path, game_name, args) = match &cli.command {
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
                let game_name = match get_filename(nes_file) {
                    Ok(s) => {
                        debug!("Parsed game name as {}", &s);
                        Some(s)
                    }
                    Err(e) => {
                        error!("Unable to read file name from {}: {}", nes_file, e);
                        None
                    }
                };
                // Read savedata, if there is any
                let (savedata_path, savedata) = match savedata_file {
                    // Savedata file arg present, try to read file
                    Some(f) => savedata_path_and_data(f),
                    None => {
                        // Try to read savedata from right beside the ines file
                        let mut savedata_path = PathBuf::from(nes_file.clone());
                        savedata_path.set_extension("srm");
                        let s = savedata_path.into_os_string().into_string().unwrap();
                        savedata_path_and_data(&s)
                    }
                };
                let nes = Nes::from_cartridge(Cartridge::from_ines(data.as_slice(), savedata));
                (nes, savedata_path, game_name, args)
            }
            Some(Command::Savestate {
                savestate_file,
                args,
            }) => {
                let nes = match std::fs::read(savestate_file) {
                    Err(e) => {
                        error!("Unable to read {}: {}", savestate_file, e);
                        std::process::exit(1);
                    }
                    Ok(data) => match Nes::from_savestate(data) {
                        Err(e) => {
                            error!("Unable to deserialize NES: {}", e);
                            std::process::exit(1);
                        }
                        Ok(nes) => nes,
                    },
                };
                let game_name = match game_name_from_savestate(savestate_file) {
                    Ok(s) => {
                        debug!("Parsed game name as {}", &s);
                        Some(s)
                    }
                    Err(e) => {
                        error!("Unable to parse game name out of {}: {}", savestate_file, e);
                        None
                    }
                };
                (nes, None, game_name, args)
            }
            None => {
                todo!()
            }
        };
        match savedata_path.as_ref() {
            Some(s) => info!("Savedata will be saved at {:?}", s),
            None => debug!("No savedata"),
        }
        // Load config
        let mut config = read_config_file(&args.config_file, Config::default());
        config.game_name = game_name;
        // Load key map
        let key_map = read_config_file(&args.keymap_file, KeyMap::default());
        config.key_map = key_map;
        // Create debug window if debug argument was passed
        let mut debug_window = if args.debug {
            Some(DebugWindow::new(&nes, &sdl_video))
        } else {
            None
        };
        // Set argument settings
        if args.paused {
            config.paused = true;
            if let Err(e) = nes.advance_frame(&config.emu_settings) {
                error!("Error when advancing NES first frame: {}", e)
            }
        }
        if args.muted {
            config.volume = 0.0;
        }
        // Setup savestate and savedata repositories
        config.savestate_dir = try_create_dir(&config.savestate_dir);
        debug!("Savestates will be saved in {:?}", config.savestate_dir);

        let mut last_debug_window_render = Instant::now();
        // Various constants for keeping emulator time in check with real time
        const DEBUG_WINDOW_REFRESH_RATE: Duration = Duration::from_millis(1000 / 60);
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
                    Event::Window { win_event, .. } => {
                        if win_event == WindowEvent::Close {
                            should_exit = true
                        }
                    }
                    _ => {
                        if let Some(d) = debug_window.as_mut() {
                            d.handle_event(&event)
                        }
                    }
                }
            }
            if should_exit {
                break;
            }

            // Render debug window
            if Instant::now().duration_since(last_debug_window_render) >= DEBUG_WINDOW_REFRESH_RATE
            {
                last_debug_window_render += DEBUG_WINDOW_REFRESH_RATE;
                if let Some(d) = debug_window.as_mut() {
                    d.render(&mut nes, &event_pump, &mut config)
                }
            }
            // Update window
            input.update(&mut nes, &event_pump, &mut config);
            // Update audio
            audio.update(&mut nes, &config);
            // Render window
            window.render(&nes, &config);
            // Update CPU
            if config.paused {
                delta = Instant::now();
            } else {
                // Advance 1 frame
                window.make_gl_current();
                let cycles_to_wait = match nes.advance_frame(&config.emu_settings) {
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
                    debug!(
                        "Over last 600 frames: Avg FPS: {}, duration: {:?}, total cycles: {}, avg wait time {:#?}",
                        600.0
                            / (now.duration_since(last_hundred_frames).as_millis() as f32 / 1000.0),
                        now.duration_since(last_hundred_frames),
                        frame_cycles,
                        frame_wait_time.div_f64(600.0)
                    );

                    frame_cycles = 0;
                    frame_wait_time = Duration::ZERO;
                    last_hundred_frames = now;
                }
                // Calculate how much time has passed in the emulation
                let emu_elapsed = Duration::from_nanos(
                    cycles_to_wait as u64 * 1_000_000_000 / CPU_CLOCK_SPEED as u64,
                )
                .div_f64(config.speed as f64);
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
        let data = audio.all_samples.into_boxed_slice();
        let samples = Samples::new(data);
        write(
            Path::new(format!("./{}.wav", config.record_audio_filename).as_str()),
            &samples,
            1_789_000,
            1,
        )
        .unwrap();
        // Save game if we want to
        if nes.cartridge.has_battery_backed_ram() {
            match savedata_path {
                Some(p) => {
                    info!("Writing savedata to to {:#?}", &p);
                    std::fs::write(&p, nes.cartridge.memory.prg_ram)
                        .expect("Unable to save savefile");
                }
                None => error!("Cartridge has savedata but no savedata path is present"),
            }
        }
    }
}
