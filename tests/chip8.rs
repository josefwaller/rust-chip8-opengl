#[cfg(test)]
mod tests {
    use rand::Rng;
    use rust_chip8_opengl::chip8::Chip8;

    // Build an instruction from 4 4bit values
    // Returns 0x[a][b][c][d]
    // All of a, b, c, d should be at most 4 bits long
    fn build_inst(a: u16, b: u16, c: u16, d: u16) -> u16 {
        return ((a & 0x0F) << 12) | ((b & 0x0F) << 8) | ((c & 0x0F) << 4) | (d & 0x0F) as u16;
    }
    fn rand_byte(max: u8) -> u16 {
        return rand::thread_rng().gen_range(0..max) as u16;
    }
    #[test]
    fn test_ld_vx_kk() {
        let mut emu = Chip8::new();
        let mut set_vals: [u8; 16] = [0; 16];
        for i in 0..16 {
            let v = rand_byte(0xFF);
            set_vals[i] = v as u8;
            emu.step(build_inst(0x6, i as u16, v >> 4, v));
            for j in (0..i).rev() {
                assert_eq!(emu.get_register_value(j as u8), set_vals[j]);
            }
        }
    }
    #[test]
    fn test_ld_vx_vy() {
        let mut emu = Chip8::new();
        for i in 0..15 {
            let v = rand_byte(0xFF);
            let target = rand_byte(0xF);
            let set_inst = build_inst(0x6, i, v as u16 >> 4, v as u16);
            emu.step(set_inst);
            let mov_inst = build_inst(8, target as u16, i, 0);
            emu.step(mov_inst);
            assert_eq!(emu.get_register_value(target as u8), v as u8);
        }
    }
    #[test]
    fn test_or_vx_vy() {
        let mut emu = Chip8::new();
        for i in 0..15 {
            let v = rand_byte(0xFF) as u16;
            let v_target = rand_byte(0xFF) as u16;
            let mut target = rand_byte(0xF) as u16;
            if target >= i {
                target += 1;
            }
            assert_ne!(i, target);
            let set_inst = build_inst(0x6, i, v as u16 >> 4, v);
            emu.step(set_inst);
            let set_inst = build_inst(0x6, target, v_target >> 4, v_target);
            emu.step(set_inst);
            let or_inst = build_inst(8, target, i, 1);
            emu.step(or_inst);
        }
    }
    #[test]
    fn test_or_vx_vy_same_register() {
        let mut emu = Chip8::new();
        for i in 0..15 {
            emu.step(build_inst(0x6, i, i >> 4, i));
            emu.step(build_inst(8, i >> 4, i & 0x0F, 1));
            assert_eq!(emu.get_register_value(i as u8), i as u8);
        }
    }
    #[test]
    fn test_and_vx_vy() {
        let mut emu = Chip8::new();
        for i in 0..15 {
            let v = rand_byte(0xFF);
            let mut target = rand_byte(0xE);
            if target >= i {
                target += 1;
            }
            let v_target = rand_byte(0xFF);
            emu.step(build_inst(0x6, i, v >> 4, v));
            emu.step(build_inst(0x6, target, v_target >> 4, v_target));
            emu.step(build_inst(0x8, target, i, 2));
            assert_eq!(emu.get_register_value(target as u8), (v_target & v) as u8);
        }
    }
    #[test]
    fn test_and_vx_vy_same_register() {
        let mut emu = Chip8::new();
        for i in 0..15 {
            emu.step(build_inst(0x6, i, i >> 4, i));
            emu.step(build_inst(8, i, i & 0x0F, 2));
            assert_eq!(emu.get_register_value(i as u8), i as u8);
        }
    }
    #[test]
    fn test_xor_vx_vy() {
        let mut emu = Chip8::new();
        for i in 0..15 {
            let v = rand_byte(0xFF);
            let mut target = rand_byte(0xE);
            if target >= i {
                target += 1;
            }
            let v_target = rand_byte(0xFF);
            emu.step(build_inst(0x6, i, v >> 4, v));
            emu.step(build_inst(0x6, target, v_target >> 4, v_target));
            emu.step(build_inst(0x8, target, i, 3));
            assert_eq!(emu.get_register_value(target as u8), (v_target ^ v) as u8);
        }
    }
    #[test]
    fn test_xor_vx_vy_same_register() {
        let mut emu = Chip8::new();
        for i in 0..15 {
            emu.step(build_inst(0x6, i, i >> 4, i));
            emu.step(build_inst(8, i, i & 0x0F, 3));
            assert_eq!(emu.get_register_value(i as u8), 0x00);
        }
    }
}
