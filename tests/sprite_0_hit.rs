mod common;
use yane::{Cartridge, Nes};

#[test]
fn test_basic() {
    rom_test!("./test_roms/spr0_basic.nes", 1000);
}
