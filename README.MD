# Y.A.N.E. - Yet Another N.E.S. Emulator

[Nintendo Entertainment System](https://en.wikipedia.org/wiki/Nintendo_Entertainment_System) emulator and emulation library.

[![Crates.io](https://img.shields.io/crates/v/yane)](https://crates.io/crates/yane)
[![Rust](https://github.com/josefwaller/yane/actions/workflows/rust.yml/badge.svg)](https://github.com/josefwaller/yane/actions/workflows/rust.yml)
[![Docs.rs](https://docs.rs/yane/badge.svg)](https://docs.rs/yane/)

Can be used as either a standalone CLI emulator or as a ready-out-of-the-box rust crate for emulating an N.E.S.

## Usage as an emulator
Download the correct release for your operating system and add it to your `$PATH`.
If cargo's install directory is already in your `$PATH`, you can simply run `cargo install yane`.
```terminal, ignore
> cargo install yane
...
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
> yane ines path/to/my/rom.nes --muted --debug --keymap-file=my/custom/keymap.yaml
```

## Usage as a library

```rust, ignore
// Load an iNes (.ines) file
let ines = include_bytes!("./my_game.nes");
// Create a new N.E.S. console with the game inserted
let mut nes = Nes::with_cartridge(Cartridge::from_bytes(ines, None));
let config = Config::default();
// Advance the N.E.S. by 1 frame (i.e. until the VBlank interval)
nes.advance_frame(&config);
// Reset the N.E.S.
nes.reset();
```

See the [documentation on the library portion](https://docs.rs/yane/latest/yane/core/index.html) or the [examples](https://github.com/josefwaller/yane/tree/main/examples) folder for more.