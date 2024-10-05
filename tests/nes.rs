extern crate yane;
use std::{
    cmp::min,
    fs::{read, File},
    io::{BufRead, BufReader},
    string,
};

use assert_hex::assert_eq_hex;
use yane::{opcodes::*, Cpu, Nes};

// Runs the NES test CPU file and checks the state of the NES after each execution
#[test]
fn test_nestest() {
    let rom: Vec<u8> = read("./tests/nestest.nes").unwrap();
    let mut nes = Nes::from_cartridge(&rom);
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
            "Executing {:#X} {:#X} {:#X} at PC = {:#X}",
            nes.read_byte(nes.cpu.p_c as usize),
            nes.read_byte(nes.cpu.p_c as usize + 1),
            nes.read_byte(nes.cpu.p_c as usize + 2),
            nes.cpu.p_c
        );
        match nes.step() {
            Ok(c) => cycles += c,
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
