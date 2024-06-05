#[cfg(test)]
mod tests {
    use rand::Rng;
    use rust_chip8_opengl::chip8::Chip8;

    // Build an instruction from 4 4bit values
    // Returns 0x[a][b][c][d]
    // All of a, b, c, d should be at most 4 bits long
    fn build_inst(a: u8, b: u8, c: u8, d: u8) -> u16 {
        return ((a as u16 & 0x0F) << 12)
            | ((b as u16 & 0x0F) << 8)
            | ((c as u16 & 0x0F) << 4)
            | (d as u16 & 0x0F) as u16;
    }
    fn rand_byte(max: u16) -> u16 {
        return rand::thread_rng().gen_range(0..max) as u16;
    }
    #[test]
    fn test_ld_vx_kk() {
        let mut emu = Chip8::new();
        let mut set_vals: [u8; 16] = [0; 16];
        for x in 0..15 {
            let val_x = rand_byte(0xFF) as u8;
            set_vals[x] = val_x as u8;
            emu.step(build_inst(0x6, x as u8, val_x >> 4, val_x));
            for j in (0..x).rev() {
                assert_eq!(emu.get_register_value(j as u8), set_vals[j]);
            }
        }
    }
    #[test]
    fn test_ld_vx_vy() {
        stress_test(|emu, x, y, _val_x, val_y| {
            emu.step(build_inst(8, x, y, 0));
            assert_eq!(emu.get_register_value(x), val_y);
            assert_eq!(emu.get_register_value(y), val_y);
        })
    }
    #[test]
    fn test_or_vx_vy() {
        stress_test(|emu, x, y, val_x, val_y| {
            emu.step(build_inst(8, x, y, 1));
            assert_eq!(emu.get_register_value(x), val_x | val_y);
        })
    }
    #[test]
    fn test_or_vx_vy_same_register() {
        stress_test(|emu, x, _y, val_x, _val_y| {
            emu.step(build_inst(8, x, x, 1));
            assert_eq!(emu.get_register_value(x), val_x);
        })
    }
    #[test]
    fn test_and_vx_vy() {
        stress_test(|emu, x, y, val_x, val_y| {
            emu.step(build_inst(0x8, x, y, 2));
            assert_eq!(emu.get_register_value(x), val_x & val_y);
        })
    }
    #[test]
    fn test_and_vx_vy_same_register() {
        stress_test(|emu, x, _y, val_x, _val_y| {
            emu.step(build_inst(8, x, x, 2));
            assert_eq!(emu.get_register_value(x), val_x);
        })
    }
    #[test]
    fn test_xor_vx_vy() {
        stress_test(|emu, x, y, val_x, val_y| {
            emu.step(build_inst(0x8, x, y, 3));
            assert_eq!(emu.get_register_value(x), val_x ^ val_y);
        })
    }
    #[test]
    fn test_xor_vx_vy_same_register() {
        stress_test(|emu, x, _y, _val_x, _val_y| {
            emu.step(build_inst(8, x, x, 3));
            assert_eq!(emu.get_register_value(x), 0x00);
        })
    }
    #[test]
    fn test_add_vx_vy() {
        stress_test(|emu, vx, vy, val_x, val_y| {
            emu.step(build_inst(0x8, vx, vy, 4));
            assert_eq!(emu.get_register_value(vx), val_x.wrapping_add(val_y));
            assert_eq!(
                emu.get_register_value(0xF),
                if (val_x as u16 + val_y as u16) > 0xFF {
                    1
                } else {
                    0
                }
            );
        });
    }
    #[test]
    fn test_add_vx_vy_same_register() {
        stress_test(|emu, vx, _vy, val_x, _val_y| {
            emu.step(build_inst(8, vx, vx, 4));
            assert_eq!(emu.get_register_value(vx), (val_x.wrapping_add(val_x)));
            assert_eq!(
                emu.get_register_value(0xF),
                if 2 * val_x as u16 > 0xFF { 1 } else { 0 }
            );
        })
    }
    #[test]
    fn test_sub_vx_vy() {
        stress_test(|emu, vx, vy, val_x, val_y| {
            emu.step(build_inst(0x8, vx, vy, 5));
            assert_eq!(emu.get_register_value(vx), val_x - val_y);
        })
    }
    #[test]
    fn test_sub_vx_vy_same_register() {
        stress_test(|emu, vx, _vy, _val_x, _val_y| {
            emu.step(build_inst(8, vx, vx, 5));
            assert_eq!(emu.get_register_value(vx), 0);
            assert_eq!(emu.get_register_value(0xF), 0);
        });
    }
    #[test]
    fn test_sub_vx_vy_underflow() {
        stress_test(|emu, x, y, val_x, val_y| {
            emu.step(build_inst(0x8, y, x, 5));
            assert_eq!(emu.get_register_value(y), val_y.wrapping_sub(val_x));
            assert_eq!(emu.get_register_value(0xF), 1);
        })
    }
    #[test]
    fn test_shr() {
        stress_test(|emu, vx, vy, val_x, val_y| {
            emu.step(build_inst(8, vx, vy, 6));
            assert_eq!(emu.get_register_value(vx), val_x >> 1);
            assert_eq!(emu.get_register_value(0xF), val_x & 1);
            // Check vy is not changed
            assert_eq!(emu.get_register_value(vy), val_y);
        });
    }
    #[test]
    fn test_subn() {
        stress_test(|emu, x, y, val_x, val_y| {
            emu.step(build_inst(8, y, x, 7));
            assert_eq!(emu.get_register_value(y), val_x - val_y);
            assert_eq!(emu.get_register_value(0xF), 0x0);
        })
    }
    #[test]
    fn test_subn_same_register() {
        stress_test(|emu, x, _y, _val_x, _val_y| {
            emu.step(build_inst(8, x, x, 7));
            assert_eq!(emu.get_register_value(x), 0);
            assert_eq!(emu.get_register_value(0xF), 0);
        });
    }
    #[test]
    fn test_subn_underflow() {
        stress_test(|emu, x, y, val_x, val_y| {
            emu.step(build_inst(8, x, y, 7));
            assert_eq!(emu.get_register_value(x), val_y.wrapping_sub(val_x));
            assert_eq!(emu.get_register_value(0xF), 0x1);
        })
    }
    #[test]
    fn test_shl() {
        stress_test(|emu, x, y, val_x, _val_y| {
            emu.step(build_inst(8, x, y, 0xE));
            assert_eq!(emu.get_register_value(x), val_x << 1);
            assert_eq!(
                emu.get_register_value(0xF),
                if val_x & 0x80 == 0x80 { 1 } else { 0 }
            );
        });
    }
    #[test]
    fn test_jmp() {
        let mut emu = Chip8::new();
        let v = rand_byte(0xFFF);
        emu.step(0x1000 | v);
        assert_eq!(emu.get_program_counter(), v as usize);
    }
    #[test]
    fn test_call_ret() {
        let mut emu = Chip8::new();
        let addr = rand_byte(0x0FFF);
        emu.step(addr | 0x2000);
        assert_eq!(emu.get_program_counter(), addr as usize);
        emu.step(0x00EE);
        assert_eq!(emu.get_program_counter(), 0 as usize);
    }
    #[test]
    fn stress_test_call_ret() {
        let mut emu = Chip8::new();
        let mut addrs = [0 as usize; 16];
        for i in 0..16 {
            addrs[i] = rand_byte(0xFFF) as usize;
            emu.step(addrs[i] as u16 | 0x2000);
            assert_eq!(emu.get_program_counter(), addrs[i]);
        }
        for i in (0..16).rev() {
            assert_eq!(emu.get_program_counter(), addrs[i]);
            emu.step(0x00EE);
        }
        assert_eq!(emu.get_program_counter(), 0);
    }

    /*
     * Run a block of tests on two random registers with 2 random values assigned to them
     * Used for basic tests
     * Value of register X is guaranteed to be larger than value of register y
     */
    fn stress_test(f: fn(&mut Chip8, u8, u8, u8, u8)) {
        let mut emu: Chip8 = Chip8::new();
        for vx in 0..15 {
            let mut vy = rand_byte(15 - 1) as u8;
            if vy >= vx {
                vy += 1;
            }
            let val_x = rand_byte(0xFE) as u8 + 1;
            let val_y = rand_byte(val_x as u16) as u8;
            emu.step(build_inst(6, vx, val_x >> 4, val_x));
            emu.step(build_inst(6, vy, val_y >> 4, val_y));
            // Set VF to something other than 1 or 0
            emu.step(build_inst(6, 0xF, 0, 2));
            f(&mut emu, vx, vy, val_x, val_y);
        }
    }
}
