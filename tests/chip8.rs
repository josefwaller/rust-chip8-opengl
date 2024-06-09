#[cfg(test)]
extern crate assert_hex;

mod tests {
    use assert_hex::assert_eq_hex;
    use rand::Rng;
    use rust_chip8_opengl::chip8::{Chip8, SPRITES};

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
        return rand::thread_rng().gen_range(0..=max) as u16;
    }
    #[test]
    fn test_ld_vx_kk() {
        let mut emu = Chip8::new();
        let mut set_vals: [u8; 16] = [0; 16];
        for x in 0..15 {
            let val_x = rand_byte(0xFF) as u8;
            set_vals[x] = val_x as u8;
            emu.execute(build_inst(0x6, x as u8, val_x >> 4, val_x));
            for j in (0..x).rev() {
                assert_eq_hex!(emu.get_register_value(j as u8), set_vals[j]);
            }
        }
    }
    #[test]
    fn test_ld_vx_vy() {
        stress_test(|emu, x, y, _val_x, val_y| {
            emu.execute(build_inst(8, x, y, 0));
            assert_eq_hex!(emu.get_register_value(x), val_y);
            assert_eq_hex!(emu.get_register_value(y), val_y);
        })
    }
    #[test]
    fn test_or_vx_vy() {
        stress_test(|emu, x, y, val_x, val_y| {
            emu.execute(build_inst(8, x, y, 1));
            assert_eq_hex!(emu.get_register_value(x), val_x | val_y);
        })
    }
    #[test]
    fn test_or_vx_vy_same_register() {
        stress_test(|emu, x, _y, val_x, _val_y| {
            emu.execute(build_inst(8, x, x, 1));
            assert_eq_hex!(emu.get_register_value(x), val_x);
        })
    }
    #[test]
    fn test_and_vx_vy() {
        stress_test(|emu, x, y, val_x, val_y| {
            emu.execute(build_inst(0x8, x, y, 2));
            assert_eq_hex!(emu.get_register_value(x), val_x & val_y);
        })
    }
    #[test]
    fn test_and_vx_vy_same_register() {
        stress_test(|emu, x, _y, val_x, _val_y| {
            emu.execute(build_inst(8, x, x, 2));
            assert_eq_hex!(emu.get_register_value(x), val_x);
        })
    }
    #[test]
    fn test_xor_vx_vy() {
        stress_test(|emu, x, y, val_x, val_y| {
            emu.execute(build_inst(0x8, x, y, 3));
            assert_eq_hex!(emu.get_register_value(x), val_x ^ val_y);
        })
    }
    #[test]
    fn test_xor_vx_vy_same_register() {
        stress_test(|emu, x, _y, _val_x, _val_y| {
            emu.execute(build_inst(8, x, x, 3));
            assert_eq_hex!(emu.get_register_value(x), 0x00);
        })
    }
    #[test]
    fn test_add_vx_vy() {
        stress_test(|emu, vx, vy, val_x, val_y| {
            emu.execute(build_inst(0x8, vx, vy, 4));
            assert_eq_hex!(emu.get_register_value(vx), val_x.wrapping_add(val_y));
            assert_eq_hex!(
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
            emu.execute(build_inst(8, vx, vx, 4));
            assert_eq_hex!(emu.get_register_value(vx), (val_x.wrapping_add(val_x)));
            assert_eq_hex!(
                emu.get_register_value(0xF),
                if 2 * val_x as u16 > 0xFF { 1 } else { 0 }
            );
        })
    }
    #[test]
    fn test_sub_vx_vy() {
        stress_test(|emu, vx, vy, val_x, val_y| {
            emu.execute(build_inst(0x8, vx, vy, 5));
            assert_eq_hex!(emu.get_register_value(vx), val_x - val_y);
        })
    }
    #[test]
    fn test_sub_vx_vy_same_register() {
        stress_test(|emu, vx, _vy, _val_x, _val_y| {
            emu.execute(build_inst(8, vx, vx, 5));
            assert_eq_hex!(emu.get_register_value(vx), 0);
            assert_eq_hex!(emu.get_register_value(0xF), 0);
        });
    }
    #[test]
    fn test_sub_vx_vy_underflow() {
        stress_test(|emu, x, y, val_x, val_y| {
            emu.execute(build_inst(0x8, y, x, 5));
            assert_eq_hex!(emu.get_register_value(y), val_y.wrapping_sub(val_x));
            assert_eq_hex!(emu.get_register_value(0xF), 1);
        })
    }
    #[test]
    fn test_shr() {
        stress_test(|emu, vx, vy, val_x, val_y| {
            emu.execute(build_inst(8, vx, vy, 6));
            assert_eq_hex!(emu.get_register_value(vx), val_x >> 1);
            assert_eq_hex!(emu.get_register_value(0xF), val_x & 1);
            // Check vy is not changed
            assert_eq_hex!(emu.get_register_value(vy), val_y);
        });
    }
    #[test]
    fn test_subn() {
        stress_test(|emu, x, y, val_x, val_y| {
            emu.execute(build_inst(8, y, x, 7));
            assert_eq_hex!(emu.get_register_value(y), val_x - val_y);
            assert_eq_hex!(emu.get_register_value(0xF), 0x0);
        })
    }
    #[test]
    fn test_subn_same_register() {
        stress_test(|emu, x, _y, _val_x, _val_y| {
            emu.execute(build_inst(8, x, x, 7));
            assert_eq_hex!(emu.get_register_value(x), 0);
            assert_eq_hex!(emu.get_register_value(0xF), 0);
        });
    }
    #[test]
    fn test_subn_underflow() {
        stress_test(|emu, x, y, val_x, val_y| {
            emu.execute(build_inst(8, x, y, 7));
            assert_eq_hex!(emu.get_register_value(x), val_y.wrapping_sub(val_x));
            assert_eq_hex!(emu.get_register_value(0xF), 0x1);
        })
    }
    #[test]
    fn test_shl() {
        stress_test(|emu, x, y, val_x, _val_y| {
            emu.execute(build_inst(8, x, y, 0xE));
            assert_eq_hex!(emu.get_register_value(x), val_x << 1);
            assert_eq_hex!(
                emu.get_register_value(0xF),
                if val_x & 0x80 == 0x80 { 1 } else { 0 }
            );
        });
    }
    #[test]
    fn test_jmp() {
        let mut emu = Chip8::new();
        let v = rand_byte(0xFFF);
        emu.execute(0x1000 | v);
        assert_eq_hex!(emu.get_program_counter(), (v - 2) as usize);
    }
    #[test]
    fn test_call_ret() {
        let mut emu = Chip8::new();
        let addr = rand_byte(0x0FFF);
        emu.execute(addr | 0x2000);
        assert_eq_hex!(emu.get_program_counter(), addr as usize);
        emu.execute(0x00EE);
        assert_eq_hex!(emu.get_program_counter(), 0x200 as usize);
    }
    #[test]
    fn stress_test_call_ret() {
        let mut emu = Chip8::new();
        let mut addrs = [0 as usize; 16];
        for i in 0..16 {
            addrs[i] = rand_byte(0xFFF) as usize;
            emu.execute(addrs[i] as u16 | 0x2000);
            assert_eq_hex!(emu.get_program_counter(), addrs[i]);
        }
        for i in (0..16).rev() {
            assert_eq_hex!(emu.get_program_counter(), addrs[i]);
            emu.execute(0x00EE);
        }
        assert_eq_hex!(emu.get_program_counter(), 0x200);
    }
    #[test]
    fn test_se_const() {
        stress_test(|emu, x, y, val_x, val_y| {
            let pc = emu.get_program_counter();
            emu.execute(build_inst(3, x, val_x >> 4, val_x));
            assert_eq_hex!(emu.get_program_counter(), pc + 2);
            emu.execute(build_inst(3, x, val_y >> 4, val_y));
            assert_eq_hex!(emu.get_program_counter(), pc + 2);
            emu.execute(build_inst(3, y, val_y >> 4, val_y));
            assert_eq_hex!(emu.get_program_counter(), pc + 4);
            emu.execute(build_inst(3, y, val_x >> 4, val_x));
            assert_eq_hex!(emu.get_program_counter(), pc + 4);
        });
    }
    #[test]
    fn test_se_reg() {
        stress_test(|emu, x, y, _val_x, val_y| {
            let pc = emu.get_program_counter();
            emu.execute(build_inst(5, x, y, 0));
            assert_eq_hex!(emu.get_program_counter(), pc);
            emu.execute(build_inst(6, x, val_y >> 4, val_y));
            emu.execute(build_inst(5, x, y, 0));
            assert_eq_hex!(emu.get_program_counter(), pc + 2);
            emu.execute(build_inst(5, y, x, 0));
            assert_eq_hex!(emu.get_program_counter(), pc + 4);
        })
    }
    #[test]
    fn test_se_reg_same_reg() {
        stress_test(|emu, x, _y, _val_x, _val_y| {
            let pc = emu.get_program_counter();
            emu.execute(build_inst(5, x, x, 0));
            assert_eq_hex!(emu.get_program_counter(), pc + 2);
        })
    }
    #[test]
    fn test_sne() {
        stress_test(|emu, x, y, val_x, val_y| {
            let pc = emu.get_program_counter();
            emu.execute(build_inst(4, x, val_x >> 4, val_x));
            assert_eq_hex!(emu.get_program_counter(), pc);
            emu.execute(build_inst(4, x, val_y >> 4, val_y));
            assert_eq_hex!(emu.get_program_counter(), pc + 2);
            emu.execute(build_inst(4, y, val_y >> 4, val_y));
            assert_eq_hex!(emu.get_program_counter(), pc + 2);
            emu.execute(build_inst(4, y, val_x >> 4, val_x));
            assert_eq_hex!(emu.get_program_counter(), pc + 4);
        });
    }
    #[test]
    fn test_ld_i() {
        stress_test(|emu, _x, _y, _val_x, _val_y| {
            let v = rand_byte(0xFFF);
            emu.execute(build_inst(0xA, (v >> 8) as u8, (v >> 4) as u8, v as u8));
            assert_eq_hex!(emu.get_i(), v);
        })
    }
    #[test]
    fn test_ld_i_vx() {
        stress_test(|emu, x, _y, _val_x, _val_y| {
            // We are loading up to 16 values into memory so max is 16 bits less
            let addr = rand_byte(0xFF0);
            emu.execute(build_inst(
                0xA,
                (addr >> 8) as u8,
                (addr >> 4) as u8,
                addr as u8,
            ));
            let mut exp_mem: Vec<u8> = vec![];
            let i = emu.get_i();
            for j in 0..(x + 1) {
                let r = rand_byte(0xFF) as u8;
                exp_mem.push(r);
                emu.execute(build_inst(0x6, j, r >> 4, r));
            }
            emu.execute(build_inst(0xF, x, 5, 5));
            assert_eq_hex!(emu.get_i(), i);
            for j in 0..(x + 1) as usize {
                assert_eq_hex!(exp_mem[j], emu.get_mem_at(i as usize + j));
                assert_eq_hex!(exp_mem[j], emu.get_register_value(j as u8));
            }
        })
    }
    #[test]
    fn test_ld_vx_i() {
        stress_test(|emu, x, _y, _val_x, _val_y| {
            let addr = rand_byte(0xFF0);
            emu.execute(build_inst(
                0xA,
                (addr >> 8) as u8,
                (addr >> 4) as u8,
                addr as u8,
            ));
            let mut exp_mem: Vec<u8> = vec![];
            let i = emu.get_i();
            for j in 0..(x + 1) {
                let r = rand_byte(0xFF) as u8;
                exp_mem.push(r);
                // Manually save to memory
                let new_i = i + j as u16;
                emu.execute(build_inst(
                    0xA,
                    (new_i >> 8) as u8,
                    (new_i >> 4) as u8,
                    new_i as u8,
                ));
                emu.execute(build_inst(0x6, 0, r >> 4, r));
                emu.execute(build_inst(0xF, 0, 5, 5));
            }
            // Reset I
            emu.execute(build_inst(0xA, (i >> 8) as u8, (i >> 4) as u8, i as u8));
            emu.execute(build_inst(0xF, x, 6, 5));
            assert_eq_hex!(emu.get_i(), i);
            for j in 0..(x + 1) as usize {
                assert_eq_hex!(exp_mem[j], emu.get_mem_at(i as usize + j));
                assert_eq_hex!(exp_mem[j], emu.get_register_value(j as u8));
            }
        })
    }
    #[test]
    fn test_add_i_vx() {
        stress_test(|emu, x, _y, val_x, val_y| {
            emu.execute(0xA000 | val_y as u16);
            emu.execute(build_inst(0xF, x, 0x1, 0xE));
            assert_eq_hex!(emu.get_i(), val_x as u16 + val_y as u16);
        })
    }
    #[test]
    fn test_sne_vx_vy() {
        stress_test(|emu, x, y, _val_x, _val_y| {
            let pc = emu.get_program_counter();
            emu.execute(build_inst(0x9, x, y, 0));
            assert_eq_hex!(emu.get_program_counter(), pc + 2);
            emu.execute(build_inst(8, x, y, 0));
            emu.execute(build_inst(0x9, x, y, 0));
            assert_eq_hex!(emu.get_program_counter(), pc + 2);
        })
    }
    #[test]
    fn test_sne_vx_vy_same_register() {
        stress_test(|emu, x, _y, _val_x, _val_y| {
            let pc = emu.get_program_counter();
            emu.execute(build_inst(0x9, x, x, 0));
            assert_eq_hex!(emu.get_program_counter(), pc);
        })
    }
    #[test]
    fn test_jmp_v0() {
        let mut emu = Chip8::new();
        for _ in 0..16 {
            let addr = rand_byte(0xFFF);
            let v = rand_byte(0xFF) as u8;
            emu.execute(build_inst(6, 0, v >> 4, v));
            emu.execute(build_inst(
                0xB,
                (addr >> 8) as u8,
                (addr >> 4) as u8,
                addr as u8,
            ));
            assert_eq_hex!(emu.get_program_counter(), (addr + v as u16) as usize - 2)
        }
    }
    #[test]
    fn test_rand() {
        stress_test(|emu, x, _y, _val_x, _val_y| {
            emu.execute(build_inst(0xC, x, 0x0, 0xF));
            assert_eq_hex!(emu.get_register_value(x) & 0xF0, 0);
            emu.execute(build_inst(0xC, x, 0xF, 0x0));
            assert_eq_hex!(emu.get_register_value(x) & 0x0F, 0);
        })
    }
    #[test]
    fn test_drw() {
        stress_test(|emu, x, y, _val_x, _val_y| {
            // Choose a random amount to XOR
            let n = rand_byte(64 / 8) as u8;
            // Set mem[I] to something
            let mut spr = vec![];
            for i in 0..n {
                let v = rand_byte(0xFF) as u8;
                spr.push(v);
                emu.execute(build_inst(0x6, i as u8, v >> 4, v));
            }
            emu.execute(build_inst(0xF, n, 0x5, 0x5));
            // Choose a random location
            let px: u8 = rand_byte(64) as u8;
            let py = rand_byte(32) as u8;
            // Save what was in the screen before
            let prev_screen: Vec<bool> = (0..n as usize * 8)
                .map(|i| emu.get_pixel_at(px + (i % 8) as u8, py + (i / 8) as u8))
                .collect();
            // Load into VX VY
            emu.execute(build_inst(0x6, x, px >> 4, px));
            emu.execute(build_inst(0x6, y, py >> 4, py));
            // Draw
            emu.execute(build_inst(0xD, x, y, n));
            // Check memory was XORed
            (0..n as usize * 8).for_each(|i| {
                let v = (spr[i as usize / 8] << (i % 8) & 0x80) != 0;
                assert_eq_hex!(
                    emu.get_pixel_at(px + (i % 8) as u8, py + (i / 8) as u8),
                    prev_screen[i as usize] ^ v
                );
            })
        })
    }
    #[test]
    fn test_skp() {
        stress_test(|emu, x, _y, _val_x, _val_y| {
            let val_x = rand_byte(0xF) as u8;
            emu.execute(build_inst(6, x, 0, val_x));
            let pc = emu.get_program_counter();
            let mut inputs = [false; 0x10];
            // Inputs should be false by default
            emu.execute(build_inst(0xE, x, 0x9, 0xE));
            assert_eq_hex!(emu.get_program_counter(), pc);
            emu.update_inputs(inputs);
            emu.execute(build_inst(0xE, x, 0x9, 0xE));
            assert_eq_hex!(emu.get_program_counter(), pc);
            inputs[val_x as usize] = true;
            emu.update_inputs(inputs);
            emu.execute(build_inst(0xE, x, 0x9, 0xE));
            assert_eq_hex!(emu.get_program_counter(), pc + 2);
            inputs[val_x as usize] = false;
            emu.update_inputs(inputs);
            emu.execute(build_inst(0xE, x, 0x9, 0xE));
            assert_eq_hex!(emu.get_program_counter(), pc + 2);
        })
    }
    #[test]
    fn test_sknp() {
        stress_test(|emu, x, _y, _val_x, _val_y| {
            let val_x = rand_byte(0xF) as u8;
            emu.execute(build_inst(6, x, 0, val_x));
            let pc = emu.get_program_counter();
            let mut inputs = [false; 0x10];
            // Inputs should be false by default
            emu.execute(build_inst(0xE, x, 0xA, 0x1));
            assert_eq_hex!(emu.get_program_counter(), pc + 2);
            emu.update_inputs(inputs);
            emu.execute(build_inst(0xE, x, 0xA, 0x1));
            assert_eq_hex!(emu.get_program_counter(), pc + 4);
            inputs[val_x as usize] = true;
            emu.update_inputs(inputs);
            emu.execute(build_inst(0xE, x, 0xA, 0x1));
            assert_eq_hex!(emu.get_program_counter(), pc + 4);
            inputs[val_x as usize] = false;
            emu.update_inputs(inputs);
            emu.execute(build_inst(0xE, x, 0xA, 0x1));
            assert_eq_hex!(emu.get_program_counter(), pc + 6);
        })
    }
    #[test]
    fn test_ld_dt_vx() {
        stress_test(|emu, x, _y, val_x, _val_y| {
            emu.execute(build_inst(0xF, x, 0x1, 0x5));
            assert_eq_hex!(emu.get_dt(), val_x);
        })
    }
    #[test]
    fn test_ld_vx_dt() {
        stress_test(|emu, x, y, val_x, _val_y| {
            emu.execute(build_inst(0xF, x, 0x1, 0x5));
            assert_eq_hex!(emu.get_dt(), val_x);
            emu.execute(build_inst(0xF, y, 0x0, 0x07));
            assert_eq_hex!(emu.get_dt(), val_x);
            assert_eq_hex!(emu.get_register_value(y), val_x);
        })
    }
    #[test]
    fn test_ld_vx_kp() {
        stress_test(|emu, x, _y, val_x, _val_y| {
            let mut inputs = [false; 0x10];
            emu.update_inputs(inputs);
            emu.execute(build_inst(0xF, x, 0x0, 0xA));
            assert_eq_hex!(emu.get_register_value(x), val_x);
            let new_val = rand_byte(0xF);
            inputs[new_val as usize] = true;
            emu.update_inputs(inputs);
            emu.execute(build_inst(0xF, x, 0x0, 0xA));
            assert_eq_hex!(emu.get_register_value(x), new_val as u8);
        })
    }
    #[test]
    fn test_ld_st_vx() {
        stress_test(|emu, x, _y, val_x, _val_y| {
            emu.execute(build_inst(0xf, x, 0x1, 0x8));
            assert_eq_hex!(emu.get_st(), val_x);
        })
    }
    #[test]
    fn test_ld_vx_spr() {
        stress_test(|emu, x, y, val_x, _val_y| {
            emu.execute(build_inst(0xf, x, 0x2, 0x9));
            // If the implementation changes this test will have to change
            let i = emu.get_i();
            assert_eq_hex!(i, 5 * (val_x as u16 & 0xF));
            for j in 0..5 {
                println!("{}", j);
                assert_eq_hex!(
                    emu.get_mem_at(i as usize + j),
                    SPRITES[val_x as usize & 0xF][j]
                );
            }
        })
    }

    /*
     * Run a block of tests on two random registers with 2 random values assigned to them
     * Used for basic tests
     * Value of register X is guaranteed to be larger than value of register y
     */
    fn stress_test(f: fn(&mut Chip8, u8, u8, u8, u8)) {
        let mut emu: Chip8 = Chip8::new();
        for vx in 0..15 {
            let mut vy = rand_byte(0xE - 1) as u8;
            if vy >= vx {
                vy += 1;
            }
            let val_x = rand_byte(0xFE) as u8 + 1;
            let val_y = rand_byte(val_x as u16 - 1) as u8;
            emu.execute(build_inst(6, vx, val_x >> 4, val_x));
            emu.execute(build_inst(6, vy, val_y >> 4, val_y));
            println!("val_x = {}, val_y = {}", val_x, val_y);
            assert_eq_hex!(emu.get_register_value(vx), val_x);
            assert_eq_hex!(emu.get_register_value(vy), val_y);
            // Set VF to something other than 1 or 0
            emu.execute(build_inst(6, 0xF, 0, 2));
            f(&mut emu, vx, vy, val_x, val_y);
        }
    }
}
