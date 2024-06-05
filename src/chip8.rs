pub struct Chip8 {
    // Buffer for the screen
    screen_buffer: [[bool; 32]; 64],
    // Registers (1 through F)
    registers: [u8; 16],
    // Program Counter
    pc: usize,
    // Stack and stack pointer
    stack: [u16; 16],
    sp: usize,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        return Chip8 {
            screen_buffer: [[false; 32]; 64],
            registers: [0x00; 16],
            pc: 0,
            stack: [0x00; 16],
            sp: 0,
        };
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
            0x4000 => self.sne(inst),
            0x5000 => self.se_reg(inst),
            0x6000 => self.ld_vx_kk(inst),
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
            _ => self.unknown_opcode_panic(inst),
        }
        if cfg!(debug_assertions) {
            println!("Post state:");
            self.dump_state();
        }
    }

    fn unknown_opcode_panic(&self, opcode: u16) {
        panic!("Unknown opcode '{:X}' provided!", opcode);
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
        println!("PC = {:X?}, SP = {:X?}", self.pc, self.sp);
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
        self.screen_buffer = [[false; 32]; 64];
    }
    fn ret(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp] as usize;
    }
    fn jmp(&mut self, inst: u16) {
        self.pc = (inst & 0x0FFF) as usize;
    }
    fn call(&mut self, inst: u16) {
        self.stack[self.sp] = self.pc as u16;
        self.sp += 1;
        // Maybe minus one here
        self.pc = (inst & 0x0FFF) as usize;
    }
    fn se_const(&mut self, inst: u16) {
        if self.registers[self.get_reg_idx(inst, 1)] as u16 == inst & 0xFF {
            self.pc += 1;
        }
    }
    fn se_reg(&mut self, inst: u16) {
        println!("{}", self.registers[self.get_reg_idx(inst, 1)]);
        println!("{}", self.registers[self.get_reg_idx(inst, 2)]);
        if self.registers[self.get_reg_idx(inst, 1)] == self.registers[self.get_reg_idx(inst, 2)] {
            self.pc += 1;
        }
    }
    fn sne(&mut self, inst: u16) {
        if self.registers[self.get_reg_idx(inst, 1)] as u16 != inst & 0xFF {
            self.pc += 1;
        }
    }

    pub fn get_register_value(&mut self, register: u8) -> u8 {
        return self.registers[register as usize];
    }
    pub fn get_program_counter(&mut self) -> usize {
        return self.pc;
    }
}
