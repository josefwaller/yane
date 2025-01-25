mod common;
use yane::*;

#[test]
fn test_clocking() {
    rom_test!("./test_roms/mmc3_clocking.nes");
}
#[test]
fn test_details() {
    rom_test!("./test_roms/mmc3_details.nes");
}
#[test]
fn test_a12_clocking() {
    rom_test!("./test_roms/mmc3_a12_clocking.nes");
}
