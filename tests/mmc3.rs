mod common;
use yane::*;

#[test]
fn test_clocking() {
    rom_test!("./test_roms/mmc3_clocking.nes");
}
