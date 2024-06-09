#[cfg(test)]
extern crate assert_hex;

mod tests {
    use assert_hex::assert_eq_hex;
    use rust_chip8_opengl::chip8::{Chip8, SPRITES};

    #[test]
    fn test_fibinnaci() {
        let mut emu = Chip8::new();
        // Simple program to compute the first 100 fibinnaci sequence
        const PROGRAM: [u16; 14] = [
            0x6001, // LD V0 0x01
            0x6101, // LD V1 0x01
            0x6201, // LD V2, 0x01
            0x6301, // LD V3, 0x01
            0xA400, // LD I 0x400
            0xF055, // Save V0 in memory
            0xF21E, // ADD I V2 (Inc I)
            0x8400, // LD V4 V0
            0x8014, // ADD V0 V1
            0x8140, // LD V1, V2
            0x7301, // ADD V3 0x01
            0x3365, // SE V3, 101
            0x120A, // JMP start of loop
            0x1002, // JMP 0 (Ends program)
        ];
        // run program
        emu.load_program(&PROGRAM);
        while emu.get_program_counter() != 0x002 {
            emu.step();
        }
        // Check the memory
        let mut a = 1;
        let mut b = 1;
        for i in 0..100 {
            assert_eq_hex!(emu.get_mem_at(0x400 + i), a);
            let temp = a;
            a = a.wrapping_add(b);
            b = temp;
        }
    }
    #[test]
    fn test_draw_smiley() {
        let mut emu = Chip8::new();
        const PROGRAM: [u16; 9] = [
            0x6042, 0x6100, 0x6242, 0x633C, 0xA400, 0xF355, 0xA400, 0xD454, 0x1002,
        ];
        emu.load_program(&PROGRAM);
        while emu.get_program_counter() != 0x002 {
            emu.step();
        }
        const SMILEY: [[bool; 8]; 4] = [
            [false, true, false, false, false, false, true, false],
            [false; 8],
            [false, true, false, false, false, false, true, false],
            [false, false, true, true, true, true, false, false],
        ];
        SMILEY.iter().enumerate().for_each(|(y, r)| {
            r.iter()
                .enumerate()
                .for_each(|(x, v)| assert_eq_hex!(emu.get_pixel_at(x as u8, y as u8), v.to_owned()))
        });
    }
    #[test]
    fn test_drawing_values_to_screen() {
        let mut emu = Chip8::new();
        const PROGRAM: [u16; 7] = [0x6000, 0x6101, 0x00E0, 0xF029, 0xD005, 0x8014, 0x1204];
        emu.load_program(&PROGRAM);
        // 2 instruction setup
        emu.step();
        emu.step();
        // Check for each digit
        (0x0..=0xF).for_each(|i| {
            // Step 5 times
            (0..5).for_each(|_| emu.step());
            // Check the digit was drawn at (i, i)
            (0..5).for_each(|y| {
                (0..8).for_each(|x| {
                    let pixel = (SPRITES[i][y] >> (7 - x) & 0x1) == 0x1;
                    assert_eq!(emu.get_pixel_at((i + x) as u8, (i + y) as u8), pixel);
                });
            })
        })
    }
}
