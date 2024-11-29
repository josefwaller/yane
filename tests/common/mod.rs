// Create an NES from a test rom
#[macro_export]
macro_rules! nes_with_rom {
    ($rom_path: literal) => {{
        let file_contents = include_bytes!($rom_path);
        let nes = Nes::from_cartridge(Cartridge::new(file_contents.as_slice()));
        nes
    }};
}

// Advance the NES a certain number of frames
#[macro_export]
macro_rules! advance_nes_frames {
    ($nes: ident, $frames: literal) => {{
        use yane::{CPU_CYCLES_PER_OAM, CPU_CYCLES_PER_SCANLINE, CPU_CYCLES_PER_VBLANK};

        // Run the emulator a bit
        (0..($frames)).for_each(|_| {
            let mut cycles = 0;
            (0..240).for_each(|scanline| {
                while cycles < CPU_CYCLES_PER_SCANLINE {
                    cycles += $nes.step().unwrap();
                    $nes.check_oam_dma();
                }
                $nes.ppu.on_scanline(&$nes.cartridge, scanline);
            });
            cycles -= CPU_CYCLES_PER_SCANLINE;
            $nes.ppu.on_vblank();
            if $nes.ppu.get_nmi_enabled() {
                $nes.on_nmi();
            }
            while cycles < CPU_CYCLES_PER_VBLANK {
                cycles += $nes.step().unwrap();
                // Check if DMA occurred
                // TODO: Decide whether this is the best way to do this
                if $nes.check_oam_dma() {
                    cycles += CPU_CYCLES_PER_OAM;
                }
            }
        });
    }};
}

// Run a test rom
#[macro_export]
macro_rules! rom_test {
    ($nes_file: literal, $num_frames: literal) => {
        let file_contents = include_bytes!($nes_file);
        let mut nes = Nes::from_cartridge(Cartridge::new(file_contents));

        advance_nes_frames!(nes, $num_frames);

        assert_background_snapshot!(nes);
    };
}

#[macro_export]
macro_rules! set_button {
    ($nes: ident, $player_number: literal, $key: ident, $value: literal) => {{
        let mut controller = $nes.controllers[$player_number];
        controller.$key = $value;
        $nes.set_input($player_number, controller);
        println!("{:?}", controller);
    }};
}
#[macro_export]
macro_rules! press_button {
    ($nes: ident, $player_number: literal, $key: ident) => {
        set_button!($nes, $player_number, $key, true);
    };
}
#[macro_export]
macro_rules! release_button {
    ($nes: ident, $player_number: literal, $key: ident) => {
        set_button!($nes, $player_number, $key, false);
    };
}

// This is just used for both background snapshot macros
#[macro_export]
macro_rules! get_screen_str_vec {
    ($nes: ident) => {
        $nes.ppu
            .nametable_ram
            .chunks(32)
            .map(|row| {
                row.iter()
                    .map(|r| format!("{:2X?}", r))
                    .collect::<Vec<String>>()
                    .join(" ")
            })
            .collect::<Vec<String>>()
    };
}

#[macro_export]
macro_rules! assert_background_snapshot {
    ($nes: ident) => {
        // Compare background
        let screen = get_screen_str_vec!($nes);
        insta::assert_debug_snapshot!(screen.as_slice());
    };
    ($name: literal, $nes: ident) => {
        let screen = get_screen_str_vec!($nes);
        insta::assert_debug_snapshot!($name, screen.as_slice());
    };
}
