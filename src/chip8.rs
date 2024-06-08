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
        // Match first digit of opcode
        match inst & 0xF000 {
            0x0000 => match inst & 0x0FFF {
                0x0E0 => self.clr(),
                0x0EE => self.ret(),
                _ => self.unknown_opcode_panic(inst),
            },
            0x1000 => self.jmp(inst),
            0x2000 => self.call(inst),
            0x3000 => self.se_const(inst),
            0x4000 => self.sne_vx_kk(inst),
            0x5000 => self.se_reg(inst),
            0x6000 => self.ld_vx_kk(inst),
            0x7000 => self.add_vx_kk(inst),
            0x8000 => {
                let vx = self.get_reg_idx(inst, 1);
                let vy = self.get_reg_idx(inst, 2);
                match inst & 0x000F {
                    0 => self.ld_vx_vy(vx, vy),
                    1 => self.or_vx_vy(vx, vy),
                    2 => self.and_vx_vy(vx, vy),
                    3 => self.xor_vx_vy(vx, vy),
                    4 => self.add_vx_vy(vx, vy),
                    5 => self.sub_vx_vy(vx, vy),
                    6 => self.shr(vx),
                    7 => self.subn(vx, vy),
                    0xE => self.shl(vx),
                    _ => self.unknown_opcode_panic(inst),
                }
            }
            0x9000 => self.sne_vx_vy(inst),
            0xA000 => self.ld_i(inst),
            0xB000 => self.jmp_v0(inst),
            0xC000 => self.rand(inst),
            0xD000 => self.draw(inst),
            0xE000 => match inst & 0x00FF {
                0x9E => self.skp_vx(self.get_reg_idx(inst, 1)),
                0xA1 => self.sknp_vx(self.get_reg_idx(inst, 1)),
                _ => self.unknown_opcode_panic(inst),
            },
            0xF000 => match inst & 0x00FF {
                0x07 => self.ld_vx_dt(self.get_reg_idx(inst, 1)),
                0x15 => self.ld_dt_vx(self.get_reg_idx(inst, 1)),
                0x55 => self.store_at_i(inst),
                0x65 => self.load_from_i(inst),
                0x1E => self.add_i_vx(inst),
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

    // Get index of the register given the instruction
    // and the position of the byte from the left in the instruction
    // i.e. inst = 0xABCD, pos = 3, res = 0x000C
    fn get_reg_idx(&self, inst: u16, pos: u8) -> usize {
        return ((inst >> (12 - pos * 4)) & 0x000F) as usize;
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

    fn ld_vx_kk(&mut self, inst: u16) {
        let r = self.get_reg_idx(inst, 1);
        let kk = (inst & 0x00FF) as u8;
        self.registers[r] = kk;
    }

    fn ld_vx_vy(&mut self, v_x: usize, v_y: usize) {
        self.registers[v_x] = self.registers[v_y];
    }

    fn or_vx_vy(&mut self, v_x: usize, v_y: usize) {
        self.registers[v_x] = self.registers[v_x] | self.registers[v_y];
    }

    fn and_vx_vy(&mut self, v_x: usize, v_y: usize) {
        self.registers[v_x] = self.registers[v_x] & self.registers[v_y];
    }

    fn xor_vx_vy(&mut self, v_x: usize, v_y: usize) {
        self.registers[v_x] = self.registers[v_x] ^ self.registers[v_y];
    }
    fn add_vx_vy(&mut self, vx: usize, vy: usize) {
        let v = self.registers[vx] as u16 + self.registers[vy] as u16;
        self.registers[vx] = v as u8;
        self.registers[0xF] = if v > 0xFF { 1 } else { 0 };
    }
    fn sub_vx_vy(&mut self, vx: usize, vy: usize) {
        self.registers[0xF] = if self.registers[vy] > self.registers[vx] {
            1
        } else {
            0
        };
        self.registers[vx] = self.registers[vx].wrapping_sub(self.registers[vy]);
    }
    fn shr(&mut self, v: usize) {
        self.registers[0xF] = if self.registers[v] & 0x01 == 1 { 1 } else { 0 };
        self.registers[v] = self.registers[v] >> 1;
    }
    fn subn(&mut self, vx: usize, vy: usize) {
        self.registers[0xF] = if self.registers[vx] > self.registers[vy] {
            1
        } else {
            0
        };
        self.registers[vx] = self.registers[vy].wrapping_sub(self.registers[vx]);
    }
    fn shl(&mut self, v: usize) {
        self.registers[0xF] = if self.registers[v] & 0x80 == 0x80 {
            1
        } else {
            0
        };
        self.registers[v] = self.registers[v] << 1;
    }
    fn clr(&mut self) {
        self.screen_buffer = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
    }
    fn ret(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp] as usize;
    }
    fn jmp(&mut self, inst: u16) {
        self.pc = ((inst & 0x0FFF) - 2) as usize;
    }
    fn jmp_v0(&mut self, inst: u16) {
        self.pc = ((inst & 0x0FFF) + self.registers[0] as u16) as usize - 2;
    }
    fn call(&mut self, inst: u16) {
        self.stack[self.sp] = self.pc as u16;
        self.sp += 1;
        self.pc = (inst & 0x0FFF) as usize;
    }
    fn se_const(&mut self, inst: u16) {
        if self.registers[self.get_reg_idx(inst, 1)] as u16 == inst & 0xFF {
            self.pc += 2;
        }
    }
    fn se_reg(&mut self, inst: u16) {
        if self.registers[self.get_reg_idx(inst, 1)] == self.registers[self.get_reg_idx(inst, 2)] {
            self.pc += 2;
        }
    }
    fn sne_vx_kk(&mut self, inst: u16) {
        if self.registers[self.get_reg_idx(inst, 1)] as u16 != inst & 0xFF {
            self.pc += 2;
        }
    }
    fn sne_vx_vy(&mut self, inst: u16) {
        let x = self.registers[self.get_reg_idx(inst, 1)];
        let y = self.registers[self.get_reg_idx(inst, 2)];
        if x != y {
            self.pc += 2;
        }
    }
    fn skp_vx(&mut self, x: usize) {
        if self.input_state[self.registers[x] as usize] {
            self.pc += 2;
        }
    }
    fn sknp_vx(&mut self, x: usize) {
        if !self.input_state[self.registers[x] as usize] {
            self.pc += 2;
        }
    }
    fn ld_i(&mut self, inst: u16) {
        self.i = (inst & 0x0FFF) as u16;
    }
    fn load_from_i(&mut self, inst: u16) {
        let x = ((inst >> 8) & 0xF) + 1;
        for j in 0..x {
            self.registers[j as usize] = self.mem[(self.i + j) as usize];
        }
    }
    fn store_at_i(&mut self, inst: u16) {
        let x = ((inst >> 8) & 0xF) + 1;
        for j in 0..x {
            self.mem[(self.i + j) as usize] = self.registers[j as usize];
        }
    }
    fn add_i_vx(&mut self, inst: u16) {
        let x = self.get_reg_idx(inst, 1);
        self.i = self.i.wrapping_add(self.registers[x] as u16);
    }
    fn add_vx_kk(&mut self, inst: u16) {
        let x = self.get_reg_idx(inst, 1);
        self.registers[x] = self.registers[x].wrapping_add((inst & 0xFF) as u8);
    }
    fn rand(&mut self, inst: u16) {
        let r = rand::thread_rng().gen_range(0..0xFF) & (inst & 0xFF);
        self.registers[self.get_reg_idx(inst, 1)] = r as u8;
    }
    fn ld_dt_vx(&mut self, x: usize) {
        self.dt = self.registers[x];
    }
    fn ld_vx_dt(&mut self, x: usize) {
        self.registers[x] = self.dt;
    }
    fn draw(&mut self, inst: u16) {
        let x = self.registers[self.get_reg_idx(inst, 1)];
        let y = self.registers[self.get_reg_idx(inst, 2)];
        let n = self.get_reg_idx(inst, 3);
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
}
