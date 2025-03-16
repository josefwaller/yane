# Y.A.N.E. - Yet Another N.E.S. Emulator

[Nintendo Entertainment System](https://en.wikipedia.org/wiki/Nintendo_Entertainment_System) emulator and emulation library.

[![Crates.io](https://img.shields.io/crates/v/yane)](https://crates.io/crates/yane)
[![Rust](https://github.com/josefwaller/yane/actions/workflows/rust.yml/badge.svg)](https://github.com/josefwaller/yane/actions/workflows/rust.yml)
[![Docs.rs](https://docs.rs/yane/badge.svg)](https://docs.rs/yane/)

Can be used as either a standalone CLI emulator or as a ready-out-of-the-box rust crate for emulating an N.E.S.

## Usage as an emulator

### Install via `cargo`
```terminal, ignore
> cargo install yane
...
> yane [COMMAND] [OPTIONS]
...
```
Make sure your cargo install directory (default `~/.cargo/bin`) is in your `$PATH`.

**WINDOWS USERS**: Make sure the appropriate `SDL2.lib` and `SDL2.dll` are present in your `$PATH`.
See [the official releases](https://github.com/libsdl-org/SDL/releases).

### Download prebuilt binaries
Download the appropriate release from [the releases page](https://github.com/josefwaller/yane/releases),
then extract the entire folder contents to somewhere in your `$PATH`.

### Build from scratch
**WINDOWS USERS**: Make sure the appropriate `SDL2.lib` and `SDL2.dll` are present in your `$PATH`.
See [the official releases](https://github.com/libsdl-org/SDL/releases).

```terminal, ignore
> git clone https://github.com/josefwaller/yane.git
...
> cd yane
> cargo build -r
...
> ./target/release/yane [COMMAND] [ARGS]
```

## Run the emulator
Once installed, run `yane ines path/to/my/rom.nes`.
Yane uses a simple command line interface, use `yane -h` and `yane [COMMAND] -h` to see the available options.

```terminal, ignore
> yane -h
An N.E.S. emulator.

Usage: yane [COMMAND]

Commands:
  setup      Initialize the configuration files at $HOME/.yane/
  ines       Load and run an iNES (.nes) file
  savestate  Load and run a savestate (.yane.bin) file.
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

> yane ines -h
Usage: yane ines [OPTIONS] <NES_FILE>

Arguments:
  <NES_FILE>  The iNes file to run

Options:
  -s, --savedata-file <SAVEDATA_FILE>  The savedata file (.SRM). Only used if the NES game supports battery backed static ram. Will default to using the NES file name and location
      --config-file <FILE>             The configuration file for the emulator's settings [default: ~/.yane/settings.yaml]
  -d, --debug                          Start in debug mode
  -p, --paused                         Start paused, after advancing one frame in order to render one frame
  -m, --muted                          Start the emulator muted
      --keymap-file <FILE>             The .YAML file defining the key mappings for the NES [default: ~/.yane/key_map.yaml]
      --tail                           Tail the logs in terminal as well as the logging file
      --log-dir <DIRECTORY>            Directory to save logs to [default: ~/.yane/logs]
  -h, --help                           Print help

> yane setup
Successfully created configuration files

> yane ines path/to/my/rom.nes --muted --debug --keymap-file=my/custom/keymap.yaml
```
See [KeyMap](https://docs.rs/yane/latest/yane/app/struct.KeyMap.html) for the default key bindings.

Running `yane setup` will create a bunch of configuration files at `$HOME/.yane`.
These can then be edited to change the keymappings, settings, logging directory, etc.
Yane will check for these files before running every time and fall back on the defaults if they don't exist,
so you can run the emulator without running `yane setup` if you want.


## Usage as a library

All of Yane's functionality is available as a standalone, pure rust crate.

```rust
use yane::core::{Nes, Settings, Cartridge, Controller};
// Create a new N.E.S. console with an empty cartridge inserted
// You could use Cartridge::from_bytes(include_bytes!("path/to/my/game.nes", None)) to load from an iNes file instead.
let mut nes = Nes::new();
// Use the default settings for the emulator
let settings = Settings::default();
// Advance the N.E.S. by 1 frame (i.e. until the VBlank interval)
nes.advance_frame(&settings);
// Advance the N.E.S. by one single instruction
nes.advance_instruction(&settings);
// Press the A button on player 1's controller
nes.set_controller_state(0, Controller {
  a: true,
  ..Controller::default()
});
// Reset the N.E.S.
nes.reset();
```

See the [documentation on the library portion](https://docs.rs/yane/latest/yane/core/index.html) or the [examples](https://github.com/josefwaller/yane/tree/main/examples) for more.

## Feature Flags
Yane includes one feature flag, `sdl`, which is enabled by default.
It includes the SDL interface that comes with Yane (i.e. everything in [yane::app](https://docs.rs/yane/latest/yane/app/index.html)).
If you want to use yane as a pure rust library, you can omit this flag.

# Credits
All test roms are from [the nes-test-rom](https://github.com/christopherpow/nes-test-roms) repository,
without which this project would probably not be possible.
Some examples use models or NES games not made by me, there are `credits.txt` files where appropriate.