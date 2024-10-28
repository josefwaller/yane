use std::thread::sleep;
use std::time::{Duration, Instant};
#[cfg(feature = "gui")]
use yane::{Gui, Nes};

fn main() {
    #[cfg(not(feature = "gui"))]
    panic!("Can only use yane as a program if the gui feature is enabled, reinstall with --features \"gui\"");
    #[cfg(feature = "gui")]
    {
        let args: Vec<String> = std::env::args().collect();
        let data = std::fs::read(args[1].clone()).unwrap();
        let mut nes = Nes::from_cartridge(data.as_slice());
        let mut gui = Gui::new(&nes);
        let mut last_render = Instant::now();
        let mut s1 = Instant::now();
        let mut s2 = Instant::now();
        let mut delta = Instant::now();
        let wait_time_per_cycle_nanos = 1_000_000.0 / 1_789_000.0;
        loop {
            let mut cycles = 0;
            (0..100).for_each(|_| cycles += nes.step().unwrap());
            // (0..(cycles / 2)).for_each(|_| nes.apu.step());
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
                gui.update_audio(&nes);
                gui.set_input(&mut nes);
                nes.ppu.on_vblank();
                last_render = Instant::now();
                if gui.render(&mut nes) {
                    break;
                }
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
