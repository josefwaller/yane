use yane::{Cartridge, Nes};
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
    rom_test!("./test_roms/ppu_vbl_timing.nes");
}
#[test]
fn test_vram_access() {
    rom_test!("./test_roms/ppu_vram_access.nes", 400);
}
