use log::*;
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
};
use std::thread::sleep;
use std::time::{Duration, Instant};
use yane::{DebugWindow, Nes, Window};

fn main() {
    {
        // Read file and init NES
        let args: Vec<String> = std::env::args().collect();
        let data = std::fs::read(args[1].clone()).unwrap();
        let mut nes = Nes::from_cartridge(data.as_slice());

        let sdl = sdl2::init().unwrap();
        // Setup video
        let video = sdl.video().unwrap();
        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(4, 0);
        // Setup input
        // The two windows need a shared event pump since SDL only allows one at a time
        let mut event_pump = sdl.event_pump().unwrap();

        let mut debug_window = DebugWindow::new(&nes, &video, &sdl);
        let mut window = Window::new(&nes, &video, &sdl);

        let mut s1 = Instant::now();
        let mut s2 = Instant::now();
        let mut delta = Instant::now();
        let mut last_debug_window_render = Instant::now();
        const DEBUG_WINDOW_REFRESH_RATE: Duration = Duration::from_millis(1000 / 60);
        // 113 cycles per actual rendering scanline + 27 per hblank
        const CPU_CYCLES_PER_SCANLINE: i64 = 140;
        const CPU_CYCLES_PER_VBLANK: i64 = 2273;
        let wait_time_per_cycle_nanos = 1_000_000.0 / 1_789_000.0;
        let mut scanline = 0;
        let mut last_hundred_frames = Instant::now();
        let mut frame_count = 0;
        // Current cycle count
        let mut cycles = 0;
        loop {
            // How many cycles to wait, in this loop
            let mut cycles_to_wait = 0;
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
            window.update(&mut nes, keys, debug_window.volume());
            // Render debug window
            if Instant::now().duration_since(last_debug_window_render) >= DEBUG_WINDOW_REFRESH_RATE
            {
                last_debug_window_render += DEBUG_WINDOW_REFRESH_RATE;
                debug_window.render(&nes, &event_pump);
            }
            // Update CPU
            if !debug_window.paused() {
                while cycles < CPU_CYCLES_PER_SCANLINE {
                    cycles += nes.step().unwrap();
                }
                cycles -= CPU_CYCLES_PER_SCANLINE;
                cycles_to_wait += CPU_CYCLES_PER_SCANLINE;
                // These functions will hopefully eventually be called from nes.step
                if Instant::now().duration_since(s1) > Duration::from_millis(1000 / 240) {
                    nes.apu.on_quater_frame();
                    s1 = Instant::now();
                }
                if Instant::now().duration_since(s2) > Duration::from_millis(1000 / 120) {
                    nes.apu.on_half_frame();
                    s2 = Instant::now();
                }
            }
            if scanline < 240 {
                window.render_scanline(&nes, scanline);
            } else if scanline == 256 {
                // Reset scanline
                scanline = 0;
                // Debug log FPS info
                frame_count += 1;
                if frame_count == 100 {
                    frame_count = 0;
                    let now = Instant::now();
                    debug!(
                        "Approx FPS: {}",
                        100.0
                            / (now.duration_since(last_hundred_frames).as_millis() as f32 / 1000.0)
                    );
                    last_hundred_frames = now;
                }
                // Render window
                window.render(&mut nes, debug_window.debug_oam());
                // Do VBlank
                if !debug_window.paused() {
                    nes.ppu.on_vblank();
                    if nes.ppu.get_nmi_enabled() {
                        nes.on_nmi();
                    }
                    // Advance cycles
                    while cycles < CPU_CYCLES_PER_VBLANK {
                        cycles += nes.step().unwrap();
                    }
                    cycles -= CPU_CYCLES_PER_VBLANK;
                    cycles_to_wait += CPU_CYCLES_PER_VBLANK;
                }
            }
            scanline += 1;
            let new_delta = Instant::now();
            let emu_elapsed = (cycles_to_wait as f64 * wait_time_per_cycle_nanos) as u64;
            let actual_elapsed = new_delta.duration_since(delta).as_nanos() as u64;
            // If we are going too fast, slow down
            let wait_duration = Duration::from_nanos(if emu_elapsed > actual_elapsed {
                emu_elapsed - actual_elapsed
            } else {
                0
            });
            sleep(wait_duration);
            delta = new_delta;
        }
    }
}
