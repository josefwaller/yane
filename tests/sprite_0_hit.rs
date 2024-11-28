mod common;
use yane::{Cartridge, Nes};

#[test]
fn test_basic() {
    rom_test!("./test_roms/spr0_basic.nes", 300);
}
#[test]
fn test_alignment() {
    rom_test!("./test_roms/spr0_alignment.nes", 300);
}
