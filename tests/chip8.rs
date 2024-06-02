#[cfg(test)]
mod tests {
    use rand::Rng;
    use rust_chip8_opengl::chip8::Chip8;

    // Build an instruction from 4 4bit values
    // Returns 0x[a][b][c][d]
    // All of a, b, c, d should be at most 4 bits long
    fn build_inst(a: u8, b: u8, c: u8, d: u8) -> u16 {
        return ((a as u16) << 12) | ((b as u16) << 8) | ((c as u16) << 4) | d as u16;
    }

    #[test]
    fn test_ld_vx_kk() {
        let mut emu = Chip8::new();
        let mut set_vals = [0; 16];
        for i in 0..16 {
            let v = rand::thread_rng().gen_range(0x00..0xFF);
            set_vals[i] = v;
            let inst: u16 = build_inst(0x6, i as u8, v >> 4, v as u8);
            emu.step(inst);
            for j in (0..i).rev() {
                assert_eq!(emu.get_register_value(j as u8), set_vals[j]);
            }
        }
    }

    #[test]
    fn test_ld_vx_vy() {
        let mut emu = Chip8::new();
        for i in 0..15 {
            let v = rand::thread_rng().gen_range(0x00..0xFF);
            let target = rand::thread_rng().gen_range(0x0..0xF);
            let set_inst = build_inst(0x6, i as u8, v >> 4, v as u8);
            emu.step(set_inst);
            let mov_inst = build_inst(8, target, i, 0);
            emu.step(mov_inst);
            assert_eq!(emu.get_register_value(target as u8), v);
        }
    }
}
