use rand::Rng;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

pub struct Chip8 {
    // Buffer for the screen
    screen_buffer: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    // Registers (1 through F)
    registers: [u8; 0x10],
    // Program Counter
    pc: usize,
    // Stack and stack pointer
    stack: [u16; 0x10],
    sp: usize,
    // Memory
    mem: [u8; 0x1000],
    // I (index register)
    i: u16,
    // Delay timer
    dt: u8,
    // Sound timer
    st: u8,
    // Keys that are currently pressed
    input_state: [bool; 0x10],
}

impl Chip8 {
    pub fn new() -> Chip8 {
        return Chip8 {
            screen_buffer: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            registers: [0x00; 0x10],
            pc: 0x200,
            stack: [0x00; 0x10],
            sp: 0,
            mem: [0x00; 0x1000],
            i: 0,
            dt: 0,
            st: 0,
            input_state: [false; 0x10],
        };
    }
    /*
     * Load a program into memory
     */
    pub fn load_program(&mut self, program: &[u16]) {
        (0..program.len()).for_each(|i| {
            self.mem[0x200 + 2 * i] = (program[i] >> 8) as u8;
            self.mem[0x200 + 2 * i + 1] = program[i] as u8;
        });
    }
    /*
     * Perform the next step in whatever program has been loaded into memory
     */
    pub fn step(&mut self) {
        self.execute(((self.mem[self.pc] as u16) << 8) | self.mem[self.pc + 1] as u16);
        self.pc += 2;
    }
    /*
     * Update the DT and ST registers given the amount of time that has elapsed in nanoseconds
     */
    pub fn update_timers(&mut self, dt_nanos: u64) {
        let change: u8 = (dt_nanos * 60 / 1000000) as u8;
        self.dt = self.dt.saturating_sub(change);
        self.st = self.st.saturating_sub(change);
    }
    /*
     * Update the current input states
     * i.e. Set a button as pressed or not
     */
    pub fn update_inputs(&mut self, inputs: [bool; 0x10]) {
        self.input_state = inputs;
    }
    /*
     * Executes a single instruction
     * Does not increment PC or affect DT or ST
     */
    pub fn execute(&mut self, inst: u16) {
        if cfg!(debug_assertions) {
            println!("Inst is {:X}", inst);
            println!("Initial state:");
            self.dump_state();
        }
        /*
         * Match opcode
         * Naming convention for all these functions
         * FUNC_[ARGS...]
         * FUNC: Description of what the function does (i.e. ld for load, add for adding)
         * ARGS: The arguments
         *       The first arg is the one being saved to, so ld_rx_ry will load Ry into Rx
         *       r[x]: Register index
         *       kk: Constant
         *       addr: Address
         *       kp: Key press
         */
        match inst & 0xF000 {
            0x0000 => match inst & 0x0FFF {
                0x0E0 => self.clr(),
                0x0EE => self.ret(),
                _ => self.unknown_opcode_panic(inst),
            },
            0x1000 => self.jmp(inst & 0x0FFF),
            0x2000 => self.call(inst & 0xFFF),
            0x3000 => self.se_r_kk(reg_at(inst, 1), (inst & 0xFF) as u8),
            0x4000 => self.sne_r_kk(inst),
            0x5000 => {
                if inst & 0xF != 0 {
                    self.unknown_opcode_panic(inst);
                }
                self.se_rx_ry(reg_at(inst, 1), reg_at(inst, 2));
            }
            0x6000 => self.ld_r_kk(reg_at(inst, 1), (inst & 0xFF) as u8),
            0x7000 => self.add_r_kk(reg_at(inst, 1), (inst & 0xFF) as u8),
            0x8000 => {
                let rx = reg_at(inst, 1);
                let ry = reg_at(inst, 2);
                match inst & 0x000F {
                    0 => self.ld_rx_ry(rx, ry),
                    1 => self.or_rx_ry(rx, ry),
                    2 => self.and_rx_ry(rx, ry),
                    3 => self.xor_rx_ry(rx, ry),
                    4 => self.add_rx_ry(rx, ry),
                    5 => self.sub_rx_ry(rx, ry),
                    6 => self.shr(rx),
                    7 => self.subn(rx, ry),
                    0xE => self.shl(rx),
                    _ => self.unknown_opcode_panic(inst),
                }
            }
            0x9000 => {
                if inst & 0xF != 0 {
                    self.unknown_opcode_panic(inst);
                }
                self.sne_rx_ry(reg_at(inst, 1), reg_at(inst, 2));
            }
            0xA000 => self.ld_i(inst & 0x0FFF),
            0xB000 => self.jmp_r0(inst & 0x0FFF),
            0xC000 => self.rand(reg_at(inst, 1), inst & 0xFF),
            0xD000 => self.draw(reg_at(inst, 1), reg_at(inst, 2), reg_at(inst, 3)),
            0xE000 => match inst & 0x00FF {
                0x9E => self.skp_r(reg_at(inst, 1)),
                0xA1 => self.sknp_r(reg_at(inst, 1)),
                _ => self.unknown_opcode_panic(inst),
            },
            0xF000 => match inst & 0x00FF {
                0x07 => self.ld_r_dt(reg_at(inst, 1)),
                0x0A => self.ld_r_kp(reg_at(inst, 1)),
                0x15 => self.ld_dt_r(reg_at(inst, 1)),
                0x18 => self.ld_st_r(reg_at(inst, 1)),
                0x55 => self.store_at_i(reg_at(inst, 1)),
                0x65 => self.load_from_i(reg_at(inst, 1)),
                0x1E => self.add_i_r(reg_at(inst, 1)),
                _ => self.unknown_opcode_panic(inst),
            },
            _ => self.unknown_opcode_panic(inst),
        }
        if cfg!(debug_assertions) {
            println!("Post state:");
            self.dump_state();
        }
    }

    fn unknown_opcode_panic(&self, opcode: u16) {
        panic!("Unknown opcode '{:04X}' provided!", opcode);
    }

    fn dump_state(&self) {
        for i in 0..16 {
            print!("V{:X} = {:#2X}, ", i, self.registers[i])
        }
        println!("");
        println!("Stack: {:X?}", self.stack);
        println!(
            "PC = {:X?}, SP = {:X?}, I = {:X?}, DT = {:X?}, ST = {:X?}",
            self.pc, self.sp, self.i, self.dt, self.st
        );
        print!("Memory:");
        self.mem.iter().enumerate().for_each(|(i, v)| {
            if i % 0x40 == 0 {
                print!("\n{:03X}: ", i);
            }
            print!("{:02X?}", v);
        });
        println!("");
        print!("Screen:");
        self.screen_buffer.iter().enumerate().for_each(|(i, s)| {
            if i % 64 == 0 {
                println!("");
            }
            print!("{}", if *s { 1 } else { 0 });
        });
        println!("");
    }

    fn ld_r_kk(&mut self, r: usize, kk: u8) {
        self.registers[r] = kk;
    }

    fn ld_rx_ry(&mut self, rx: usize, ry: usize) {
        self.registers[rx] = self.registers[ry];
    }

    fn or_rx_ry(&mut self, rx: usize, ry: usize) {
        self.registers[rx] = self.registers[rx] | self.registers[ry];
    }

    fn and_rx_ry(&mut self, rx: usize, ry: usize) {
        self.registers[rx] = self.registers[rx] & self.registers[ry];
    }

    fn xor_rx_ry(&mut self, rx: usize, ry: usize) {
        self.registers[rx] = self.registers[rx] ^ self.registers[ry];
    }
    fn add_rx_ry(&mut self, rx: usize, ry: usize) {
        let v = self.registers[rx] as u16 + self.registers[ry] as u16;
        self.registers[rx] = v as u8;
        self.registers[0xF] = if v > 0xFF { 1 } else { 0 };
    }
    fn sub_rx_ry(&mut self, rx: usize, ry: usize) {
        self.registers[0xF] = if self.registers[ry] > self.registers[rx] {
            1
        } else {
            0
        };
        self.registers[rx] = self.registers[rx].wrapping_sub(self.registers[ry]);
    }
    fn shr(&mut self, r: usize) {
        self.registers[0xF] = if self.registers[r] & 0x01 == 1 { 1 } else { 0 };
        self.registers[r] = self.registers[r] >> 1;
    }
    fn subn(&mut self, rx: usize, ry: usize) {
        self.registers[0xF] = if self.registers[rx] > self.registers[ry] {
            1
        } else {
            0
        };
        self.registers[rx] = self.registers[ry].wrapping_sub(self.registers[rx]);
    }
    fn shl(&mut self, r: usize) {
        self.registers[0xF] = if self.registers[r] & 0x80 == 0x80 {
            1
        } else {
            0
        };
        self.registers[r] = self.registers[r] << 1;
    }
    fn clr(&mut self) {
        self.screen_buffer = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
    }
    fn ret(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp] as usize;
    }
    fn jmp(&mut self, addr: u16) {
        // addr - 2 since we are about to add 2
        self.pc = (addr - 2) as usize;
    }
    fn jmp_r0(&mut self, addr: u16) {
        self.pc = (addr + self.registers[0] as u16) as usize - 2;
    }
    fn call(&mut self, addr: u16) {
        self.stack[self.sp] = self.pc as u16;
        self.sp += 1;
        self.pc = addr as usize;
    }
    fn se_r_kk(&mut self, r: usize, kk: u8) {
        if self.registers[r] == kk {
            self.pc += 2;
        }
    }
    fn se_rx_ry(&mut self, rx: usize, ry: usize) {
        if self.registers[rx] == self.registers[ry] {
            self.pc += 2;
        }
    }
    fn sne_r_kk(&mut self, inst: u16) {
        if self.registers[reg_at(inst, 1)] as u16 != inst & 0xFF {
            self.pc += 2;
        }
    }
    fn sne_rx_ry(&mut self, x: usize, y: usize) {
        if self.registers[x] != self.registers[y] {
            self.pc += 2;
        }
    }
    fn skp_r(&mut self, x: usize) {
        if self.input_state[self.registers[x] as usize] {
            self.pc += 2;
        }
    }
    fn sknp_r(&mut self, x: usize) {
        if !self.input_state[self.registers[x] as usize] {
            self.pc += 2;
        }
    }
    fn ld_r_kp(&mut self, r: usize) {
        match self.input_state.iter().enumerate().find(|(_i, v)| **v) {
            Some((i, _v)) => self.registers[r] = i as u8,
            // Sneaky hack - in order to "wait" we just decrement PC so that we reach this addr again
            // In retrospect this probably isn't that sneaky
            None => self.pc -= 2,
        }
    }
    fn ld_i(&mut self, addr: u16) {
        self.i = addr;
    }
    fn load_from_i(&mut self, n: usize) {
        for j in 0..(n + 1) {
            self.registers[j as usize] = self.mem[(self.i + j as u16) as usize];
        }
    }
    fn store_at_i(&mut self, n: usize) {
        for j in 0..(n + 1) {
            self.mem[(self.i + j as u16) as usize] = self.registers[j as usize];
        }
    }
    fn add_i_r(&mut self, r: usize) {
        self.i = self.i.wrapping_add(self.registers[r] as u16);
    }
    fn add_r_kk(&mut self, r: usize, kk: u8) {
        self.registers[r] = self.registers[r].wrapping_add(kk);
    }
    fn rand(&mut self, r: usize, mask: u16) {
        self.registers[r] = (rand::thread_rng().gen_range(0..0xFF) & mask) as u8;
    }
    fn ld_st_r(&mut self, r: usize) {
        self.st = self.registers[r];
    }
    fn ld_dt_r(&mut self, x: usize) {
        self.dt = self.registers[x];
    }
    fn ld_r_dt(&mut self, x: usize) {
        self.registers[x] = self.dt;
    }
    fn draw(&mut self, rx: usize, ry: usize, n: usize) {
        let x = self.registers[rx];
        let y = self.registers[ry];
        // XOR data onto screen
        for j in 0..(n + 1) {
            let val = self.mem[self.i as usize + j];
            for k in 0..8 {
                let coord: usize = ((y as usize + j) % SCREEN_HEIGHT) * SCREEN_WIDTH
                    + (x as usize + k) % SCREEN_WIDTH;
                let p = (val << k) & 0x80 != 0; // Get MSB
                if p && self.screen_buffer[coord] {
                    self.registers[0xF] = 1;
                }
                self.screen_buffer[coord] = p ^ self.screen_buffer[coord];
            }
        }
    }

    pub fn get_register_value(&mut self, register: u8) -> u8 {
        return self.registers[register as usize];
    }
    pub fn get_program_counter(&mut self) -> usize {
        return self.pc;
    }
    pub fn get_i(&mut self) -> u16 {
        return self.i;
    }
    pub fn get_mem_at(&mut self, addr: usize) -> u8 {
        return self.mem[addr];
    }
    pub fn get_pixel_at(&self, x: u8, y: u8) -> bool {
        return self.screen_buffer
            [((x as usize % 64) + y as usize * 64) % self.screen_buffer.len()];
    }
    pub fn get_dt(&self) -> u8 {
        return self.dt;
    }
    pub fn get_st(&self) -> u8 {
        return self.st;
    }
}

// Get index of the register given the instruction
// and the position of the byte from the left in the instruction
// i.e. inst = 0xABCD, pos = 3, res = 0x000C
fn reg_at(inst: u16, pos: u8) -> usize {
    return ((inst >> (12 - pos * 4)) & 0x000F) as usize;
}
