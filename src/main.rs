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
        let wait_time_per_cycle_millis = 1000.0 / 1_789_000.0;
        loop {
            let c1 = nes.step().unwrap();
            let c2 = nes.step().unwrap();
            (0..((c1 + c2) / 2)).for_each(|_| nes.apu.step());
            gui.update_audio(&nes);
            // println!(
            //     "{:X} {:X} {:X} status = {:X}",
            //     nes.read_byte(nes.cpu.p_c as usize),
            //     nes.read_byte(nes.cpu.p_c as usize + 1),
            //     nes.read_byte(nes.cpu.p_c as usize + 2),
            //     nes.ppu.status
            // );
            if Instant::now().duration_since(last_render) >= Duration::from_millis(1000 / 60) {
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
            sleep(Duration::from_millis(
                (1.0 * (c1 + c2) as f64 * wait_time_per_cycle_millis) as u64,
            ));
        }
    }
}
