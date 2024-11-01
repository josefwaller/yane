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

        let mut debug_window = DebugWindow::new(&nes, &video);
        let mut window = Window::new(&nes, &video, &sdl);

        let mut last_render = Instant::now();
        let mut s1 = Instant::now();
        let mut s2 = Instant::now();
        let mut delta = Instant::now();
        let wait_time_per_cycle_nanos = 1_000_000.0 / 1_789_000.0;
        loop {
            window.update(&mut nes);

            let mut cycles = 0;
            (0..50).for_each(|_| cycles += nes.step().unwrap());
            // These functions will hopefully eventually be called from nes.step
            if Instant::now().duration_since(s1) > Duration::from_millis(1000 / 240) {
                nes.apu.on_quater_frame();
                s1 = Instant::now();
            }
            if Instant::now().duration_since(s2) > Duration::from_millis(1000 / 120) {
                nes.apu.on_half_frame();
                s2 = Instant::now();
            }
            // println!(
            //     "{:X} {:X} {:X} status = {:X}",
            //     nes.read_byte(nes.cpu.p_c as usize),
            //     nes.read_byte(nes.cpu.p_c as usize + 1),
            //     nes.read_byte(nes.cpu.p_c as usize + 2),
            //     nes.ppu.status
            // );
            if Instant::now().duration_since(last_render) >= Duration::from_millis(1000 / 60) {
                last_render = Instant::now();

                if window.render(&mut nes) {
                    break;
                }
                debug_window.render(&nes);
                nes.ppu.on_vblank();
                if nes.ppu.get_nmi_enabled() {
                    nes.on_nmi();
                }
            }
            // println!(
            //     "Sleeping {}",
            //     1.0 * (c1 + c2) as f64 * wait_time_per_cycle_millis
            // );
            let new_delta = Instant::now();
            let emu_elapsed = (cycles as f64 * wait_time_per_cycle_nanos) as u64;
            let actual_elapsed = new_delta.duration_since(delta).as_nanos() as u64;
            // If we are going too fast, slow down
            if emu_elapsed > actual_elapsed {
                sleep(Duration::from_nanos(emu_elapsed - actual_elapsed));
            }
            delta = new_delta;
        }
    }
}
