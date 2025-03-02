mod common;
use yane::core::{Cartridge, Nes};

#[test]
fn test_basic() {
    rom_test!("./test_roms/spr0_basic.nes");
}
#[test]
fn test_alignment() {
    rom_test!("./test_roms/spr0_alignment.nes");
}
#[test]
fn test_corners() {
    rom_test!("./test_roms/spr0_corners.nes");
}
#[test]
fn test_flip() {
    rom_test!("./test_roms/spr0_flip.nes");
}
#[test]
fn test_clip() {
    rom_test!("./test_roms/spr0_clip.nes");
}
#[test]
fn test_right_edge() {
    rom_test!("./test_roms/spr0_right_edge.nes");
}
#[test]
fn test_bottom() {
    rom_test!("./test_roms/spr0_bottom.nes");
}
#[test]
fn test_8x16() {
    rom_test!("./test_roms/spr0_8x16.nes");
}
