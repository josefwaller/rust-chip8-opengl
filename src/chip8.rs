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
                _ => panic!("Invalid opcode: '{:X}'", inst),
            }
        } else {
            panic!("Invalid opcode: '{:X}'", inst);
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
        if cfg!(debug_assertions) {
            self.dump_state();
        }
        self.registers[r] = kk;
    }

    fn ld_vx_vy(&mut self, v_x: usize, v_y: usize) {
        self.registers[v_x] = self.registers[v_y];
        if cfg!(debug_assertions) {
            self.dump_state();
        }
    }

    fn or_vx_vy(&mut self, v_x: usize, v_y: usize) {
        self.registers[v_x] = self.registers[v_x] | self.registers[v_y];
        if cfg!(debug_assertions) {
            self.dump_state();
        }
    }

    fn and_vx_vy(&mut self, v_x: usize, v_y: usize) {
        self.registers[v_x] = self.registers[v_x] & self.registers[v_y];
        if cfg!(debug_assertions) {
            self.dump_state();
        }
    }

    fn xor_vx_vy(&mut self, v_x: usize, v_y: usize) {
        self.registers[v_x] = self.registers[v_x] ^ self.registers[v_y];
        if cfg!(debug_assertions) {
            self.dump_state();
        }
    }

    pub fn get_register_value(&mut self, register: u8) -> u8 {
        return self.registers[register as usize];
    }
}
