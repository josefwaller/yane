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
        loop {
            nes.step().unwrap();
            // println!(
            //     "{:X} {:X} {:X} status = {:X}",
            //     nes.read_byte(nes.cpu.p_c as usize),
            //     nes.read_byte(nes.cpu.p_c as usize + 1),
            //     nes.read_byte(nes.cpu.p_c as usize + 2),
            //     nes.ppu.status
            // );
            gui.set_input(&mut nes);
            if Instant::now().duration_since(last_render) >= Duration::from_millis(1000 / 60) {
                nes.ppu.on_vblank();
                if nes.ppu.get_nmi_enabled() {
                    if gui.render(&mut nes) {
                        break;
                    }
                    last_render = Instant::now();
                    nes.on_nmi();
                }
            }
        }
    }
}
