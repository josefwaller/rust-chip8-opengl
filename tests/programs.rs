#[cfg(test)]
extern crate assert_hex;

mod tests {
    use assert_hex::assert_eq_hex;
    use rust_chip8_opengl::chip8::Chip8;

    #[test]
    fn test_fibinnaci() {
        let mut emu = Chip8::new();
        // Simple program to compute the first 100 fibinnaci sequence
        let program: [u8; 28] = [
            0x60, 0x01, // LD V0 0x01
            0x61, 0x01, // LD V1 0x01
            0x62, 0x01, // LD V2, 0x01
            0x63, 0x01, // LD V3, 0x01
            0xA4, 0x00, // LD I 0x400
            0xF0, 0x55, // Save V0 in memory
            0xF2, 0x1E, // ADD I V2 (Inc I)
            0x84, 0x00, // LD V4 V0
            0x80, 0x14, // ADD V0 V1
            0x81, 0x40, // LD V1, V2
            0x73, 0x01, // ADD V3 0x01
            0x33, 0x65, // SE V3, 101
            0x12, 0x0A, // JMP start of loop
            0x10, 0x02, // JMP 0 (Ends program)
        ];
        emu.run_program(&program);
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
}
