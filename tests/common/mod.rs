// Create an NES from a test rom
#[macro_export]
macro_rules! nes_with_rom {
    ($rom_path: literal) => {
        Nes::with_cartridge(Cartridge::from_ines(include_bytes!($rom_path), None).unwrap())
    };
}

// Advance the NES a certain number of frames
#[macro_export]
macro_rules! advance_nes_frames {
    ($nes: ident, $frames: literal) => {{
        use yane::core::Settings;
        let s = Settings::default();
        // Run the emulator a bit
        (0..($frames)).for_each(|_| {
            $nes.advance_frame(&s)
                .expect("Error when advancing NES by a frame");
        });
    }};
}

// Run a test rom
#[macro_export]
macro_rules! rom_test {
    ($nes_file: literal, $num_frames: literal) => {
        let file_contents = include_bytes!($nes_file);
        let mut nes = Nes::with_cartridge(Cartridge::from_ines(file_contents, None).unwrap());

        advance_nes_frames!(nes, $num_frames);

        assert_background_snapshot!(nes);
    };
    // Default to 300 frames
    ($nes_file: literal) => {
        rom_test!($nes_file, 300);
    };
}

#[macro_export]
macro_rules! set_button {
    ($nes: ident, $player_number: literal, $key: ident, $value: literal) => {{
        let mut controller = $nes.controllers[$player_number];
        controller.$key = $value;
        $nes.set_controller_state($player_number, controller);
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

#[macro_export]
macro_rules! assert_background_snapshot {
    ($nes: ident) => {
        // Compare background
        insta::assert_debug_snapshot!($nes.ppu.nametable_ram.as_slice());
    };
    ($name: literal, $nes: ident) => {
        insta::assert_debug_snapshot!($name, $nes.ppu.nametable_ram.as_slice());
    };
}
