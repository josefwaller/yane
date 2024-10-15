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
        let mut gui = Gui::new();
        let mut last_render = Instant::now();
        loop {
            // println!(
            //     "{:#X}: {:#X}, {:2X}{:2X}, A: {:#X}, X: {:#X}, Y: {:#X}",
            //     nes.cpu.p_c,
            //     nes.read_byte(nes.cpu.p_c as usize),
            //     nes.read_byte(nes.cpu.p_c as usize + 2),
            //     nes.read_byte(nes.cpu.p_c as usize + 1),
            //     nes.cpu.a,
            //     nes.cpu.x,
            //     nes.cpu.y
            // );
            nes.step().unwrap();
            if Instant::now().duration_since(last_render) >= Duration::from_millis(1000 / 60) {
                if gui.render(&mut nes) {
                    break;
                }
                last_render = Instant::now();
                nes.on_nmi();
            }
        }
    }
}
