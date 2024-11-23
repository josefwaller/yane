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
        gl_attr.set_context_version(3, 3);
        // Setup input
        // The two windows need a shared event pump since SDL only allows one at a time
        let mut event_pump = sdl.event_pump().unwrap();

        let mut debug_window = DebugWindow::new(&nes, &video, &sdl);
        let mut window = Window::new(&nes, &video, &sdl);

        let mut s1 = Instant::now();
        let mut s2 = Instant::now();
        let mut delta = Instant::now();
        let mut last_debug_window_render = Instant::now();
        // Various constants for keeping emulator time in check with real time
        const DEBUG_WINDOW_REFRESH_RATE: Duration = Duration::from_millis(1000 / 60);
        const CPU_CYCLES_PER_SCANLINE: i64 = 113;
        const CPU_CYCLES_PER_VBLANK: i64 = 2273;
        const CPU_CYCLES_PER_OAM: i64 = 513;
        const CPU_CYCLES_PER_FRAME: i64 = 240 * 113 + 2273;
        let wait_time_per_cycle =
            Duration::from_nanos(1_000_000_000 / 60 / CPU_CYCLES_PER_FRAME as u64);
        info!(
            "FPS = 60, cycles/scanline={CPU_CYCLES_PER_SCANLINE}, cycles/vblank={CPU_CYCLES_PER_VBLANK}, cycles/frame={CPU_CYCLES_PER_FRAME}, wait time={wait_time_per_cycle:?}",
        );
        let fps =
            1_000_000_000.0 / (CPU_CYCLES_PER_FRAME as f64 * wait_time_per_cycle.as_nanos() as f64);
        info!("Calculated FPS: {fps}");
        let mut scanline = 0;
        let mut last_hundred_frames = Instant::now();
        let mut frame_count = 0;
        // Current cycle count
        let mut cycles = 0;
        let mut frame_cycles = 0;
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
            } else {
                // Reset scanline
                scanline = 0;
                // Debug log FPS info
                frame_count += 1;
                if frame_count == 100 {
                    frame_count = 0;
                    let now = Instant::now();
                    debug!(
                        "Over last 100 frames: Avg FPS: {}, avg cycles: {}",
                        100.0
                            / (now.duration_since(last_hundred_frames).as_millis() as f32 / 1000.0),
                        frame_cycles as f64 / 100.0
                    );
                    frame_cycles = 0;
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
                        // Check if DMA occurred
                        // TODO: Decide whether this is the best way to do this
                        if nes.check_oam_dma() {
                            cycles += CPU_CYCLES_PER_OAM;
                        }
                    }
                    cycles -= CPU_CYCLES_PER_VBLANK;
                    cycles_to_wait += CPU_CYCLES_PER_VBLANK;
                }
            }
            scanline += 1;
            // Calculate how much time has passed in the emulation
            let emu_elapsed = wait_time_per_cycle.saturating_mul(cycles_to_wait as u32);
            // Calculate how much time has actually passed
            let actual_elapsed = Instant::now().duration_since(delta);
            // Wait for the difference
            let wait_duration = emu_elapsed.saturating_sub(actual_elapsed);
            // If we are going too fast, slow down
            frame_cycles += cycles_to_wait;
            // Check if we want to slow down first, since sleep is costly even if wait_duration is 0
            if wait_duration != Duration::ZERO {
                sleep(wait_duration);
            }
            // Advance real time by amount of emulator time that will have passed
            // Since sleep may overshoot, this will let us catch up next frame/scanline
            delta += emu_elapsed;
        }
    }
}
