mod common;
use log::*;
use std::{
    cmp::min,
    fs::{read, File},
    io::{BufRead, BufReader, Read},
    string,
};

use assert_hex::assert_eq_hex;
use yane::{Cartridge, Controller, Nes};

// Runs the NES test CPU file and checks the state of the NES after each execution
#[test]
fn test_nestest_log() {
    let rom = read("./tests/test_roms/cpu_nestest.nes").unwrap();
    let mut nes = Nes::from_cartridge(Cartridge::new(&rom, None));
    nes.cpu.p_c = 0xC000;
    let f = File::open("./tests/nestest.log").unwrap();
    let buf = BufReader::new(f);
    let mut cycles: i64 = 7;
    for line in buf.lines() {
        let l = line.unwrap();
        macro_rules! assert_eq_substr {
            ($left: expr, $start: expr, $end: expr, $name: expr) => {
                assert_eq_hex!(
                    $left as u16,
                    get_hex_u16(&l, $start, $end),
                    "Expected {} to be {:X}, but was {:X}",
                    $name,
                    get_hex_u16(&l, $start, $end),
                    $left,
                );
            };
        }
        println!("{}", l);
        assert_eq_substr!(nes.cpu.p_c, 0, 4, "PC");
        assert_eq_substr!(nes.cpu.a, 50, 2, "A");
        assert_eq_substr!(nes.cpu.x, 55, 2, "X");
        assert_eq_substr!(nes.cpu.y, 60, 2, "Y");
        assert_eq_substr!(nes.cpu.s_r.to_byte(), 65, 2, "SR");
        assert_eq_substr!(nes.cpu.s_p, 71, 2, "SP");
        let c = l[90..].parse::<i64>().unwrap();
        assert_eq!(cycles, c, "Cycles should be {}, is {}", c, cycles);
        println!(
            "Executing {:#X} {:#X} {:#X} at PC = {:#X} (A = {:X}, X = {:X}, Y = {:X}, SR = {:X} = {:b})",
            nes.read_byte(nes.cpu.p_c as usize),
            nes.read_byte(nes.cpu.p_c as usize + 1),
            nes.read_byte(nes.cpu.p_c as usize + 2),
            nes.cpu.p_c,
            nes.cpu.a,
            nes.cpu.x,
            nes.cpu.y,
            nes.cpu.s_r.to_byte(),
            nes.cpu.s_r.to_byte()
        );
        match nes.step() {
            Ok(c) => cycles += c as i64,
            Err(s) => panic!("{}", s),
        }
    }
}

fn get_hex(s: &String, start: usize, len: usize) -> u8 {
    get_hex_u16(s, start, len) as u8
}
fn get_hex_u16(s: &String, start: usize, len: usize) -> u16 {
    u16::from_str_radix(&s[start..min(start + len, s.len())], 16).unwrap()
}

// Check the background to verify the NES test results
#[test]
fn test_nestest_file() {
    let mut nes = nes_with_rom!("./test_roms/cpu_nestest.nes");
    advance_nes_frames!(nes, 100);
    press_button!(nes, 0, start);
    advance_nes_frames!(nes, 100);
    release_button!(nes, 0, start);
    assert_background_snapshot!("nestest_official", nes);
    press_button!(nes, 0, select);
    advance_nes_frames!(nes, 100);
    press_button!(nes, 0, start);
    advance_nes_frames!(nes, 100);
    assert_background_snapshot!("nestest_unofficial", nes);
}
#[test]
fn test_official() {
    rom_test!("./test_roms/cpu_official.nes", 8_000);
}

#[test]
fn test_branch() {
    rom_test!("./test_roms/cpu_branch.nes");
}
