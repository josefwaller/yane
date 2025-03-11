# Example 2 - Explicit

Compiles the emulator without using any of the built in components.
Rather, gets the library output directly and feeds that to an SDL canvas/queue/etc
in order to demonstrate how to interact with the library directly.

We disable the SDL components (`Window`, `Audio`, etc), by omitting the `sdl` feature,
which is enabled by default.

Run with:
```
cargo run --example 2-explicit --no-default-features -- path/to/rom.nes
```