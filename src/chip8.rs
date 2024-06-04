pub struct Chip8 {
    // Buffer for the screen
    screen_buffer: [[bool; 32]; 64],
    // Registers (1 through F)
    registers: [u8; 16],
}

impl Chip8 {
    pub fn new() -> Chip8 {
        return Chip8 {
            screen_buffer: [[false; 32]; 64],
            registers: [0x00; 16],
        };
    }
    pub fn step(&mut self, inst: u16) {
        if cfg!(debug_assertions) {
            println!("Inst is {:X}", inst);
            println!("Initial state:");
            self.dump_state();
        }
        if inst & 0xF000 == 0x6000 {
            self.ld_vx_kk(inst);
        } else if inst & 0xF000 == 0x8000 {
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
                _ => panic!("Invalid opcode: '{:X}'", inst),
            }
        } else {
            panic!("Invalid opcode: '{:X}'", inst);
        }
        if cfg!(debug_assertions) {
            self.dump_state();
        }
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

    pub fn get_register_value(&mut self, register: u8) -> u8 {
        return self.registers[register as usize];
    }
}
