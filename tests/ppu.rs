use yane::core::{Cartridge, Nes};
mod common;

#[test]
fn test_palette_ram() {
    rom_test!("./test_roms/ppu_palette_ram.nes");
}
#[test]
fn test_oam_ram() {
    rom_test!("./test_roms/ppu_oam_ram.nes");
}
#[test]
fn test_vbl_timing() {
    rom_test!("./test_roms/ppu_vbl_clear_timing.nes");
}
#[test]
fn test_vram_access() {
    rom_test!("./test_roms/ppu_vram_access.nes", 400);
}
#[test]
fn test_open_bus() {
    rom_test!("./test_roms/ppu_open_bus.nes", 300);
}
#[test]
fn test_oam_read() {
    rom_test!("./test_roms/ppu_oam_read.nes");
}
#[test]
fn test_spr_overflow() {
    rom_test!("./test_roms/spr_overflow_basics.nes");
}
#[test]
fn test_spr_overflow_details() {
    rom_test!("./test_roms/spr_overflow_details.nes");
}
