// Run a test rom

#[macro_export]
macro_rules! rom_test {
    ($nes_file: literal, $num_frames: literal) => {
        let file_contents = include_bytes!($nes_file);
        let mut nes = Nes::from_cartridge(Cartridge::new(file_contents));

        const CPU_CYCLES_PER_VBLANK: i64 = 2273;
        const CPU_CYCLES_PER_OAM: i64 = 513;
        // Run the emulator a bit
        (0..($num_frames)).for_each(|_| {
            let mut cycles = 0;
            (0..240).for_each(|scanline| {
                while cycles < 112 {
                    cycles += nes.step().unwrap();
                    nes.check_oam_dma();
                }
                nes.ppu.on_scanline(&nes.cartridge, scanline);
            });
            cycles -= 112;
            nes.ppu.on_vblank();
            if nes.ppu.get_nmi_enabled() {
                nes.on_nmi();
            }
            while cycles < CPU_CYCLES_PER_VBLANK {
                cycles += nes.step().unwrap();
                // Check if DMA occurred
                // TODO: Decide whether this is the best way to do this
                if nes.check_oam_dma() {
                    cycles += CPU_CYCLES_PER_OAM;
                }
            }
        });
        // Compare background
        let screen: Vec<String> = nes
            .ppu
            .nametable_ram
            .chunks(32)
            .map(|row| {
                row.iter()
                    .map(|r| format!("{:2X?}", r))
                    .collect::<Vec<String>>()
                    .join(" ")
            })
            .collect();
        insta::assert_debug_snapshot!(screen.as_slice());
    };
}
