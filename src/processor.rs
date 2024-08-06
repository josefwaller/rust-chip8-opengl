use crate::errors::OpcodeError;
use rand::Rng;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

/// The sprites for the digits 0-F, as bytes
pub const SPRITES: [[u8; 5]; 16] = [
    [0xF0, 0x90, 0x90, 0x90, 0xF0],
    [0x00, 0x60, 0x20, 0x20, 0x70],
    [0xF0, 0x10, 0xF0, 0x80, 0xF0],
    [0xF0, 0x10, 0xF0, 0x10, 0xF0],
    [0x90, 0x90, 0xF0, 0x10, 0x10],
    [0xF0, 0x80, 0xF0, 0x10, 0xF0],
    [0xF0, 0x80, 0xF0, 0x90, 0xF0],
    [0xF0, 0x10, 0x20, 0x40, 0x40],
    [0xF0, 0x90, 0xF0, 0x90, 0xF0],
    [0xF0, 0x90, 0xF0, 0x10, 0xF0],
    [0xF0, 0x90, 0xF0, 0x90, 0x90],
    [0xE0, 0x90, 0xE0, 0x90, 0xE0],
    [0xF0, 0x80, 0x80, 0x80, 0xF0],
    [0xE0, 0x90, 0x90, 0x90, 0xE0],
    [0xF0, 0x80, 0xF0, 0x80, 0xF0],
    [0xF0, 0x80, 0xF0, 0x80, 0x80],
];

/**
 * The actual CHIP-8 processor.
 * Decodes and runs any opcodes, stores memory, stores screen.
 * Needs to be paired with an interface to allow the user to actually interact with the program.
 */
pub struct Processor {
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
    // debug print mode used in tests
    debug_print: bool,
    // Key that was just released, should be set by the interface
    // Used for LD X KP and nothing else
    // Reset every execution
    last_key_released: Option<u8>,
}

impl Processor {
    pub fn new() -> Processor {
        let mut c = Processor {
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
            debug_print: false,
            last_key_released: None,
        };
        (0..0x10).for_each(|i| c.mem[(6 * i)..(6 * i + 5)].copy_from_slice(&SPRITES[i]));

        return c;
    }
    /**
     * Load a program into memory
     **/
    pub fn load_program(&mut self, program: &[u8]) {
        program
            .iter()
            .enumerate()
            .for_each(|(i, v)| self.mem[0x200 + i] = *v);
    }
    /**
     * Load a program as u16s instead of u8s.
     * Mostly just a convenience function.
     **/
    #[allow(dead_code)]
    pub fn load_program_u16(&mut self, program: &[u16]) {
        (0..program.len()).for_each(|i| {
            self.mem[0x200 + 2 * i] = (program[i] >> 8) as u8;
            self.mem[0x200 + 2 * i + 1] = program[i] as u8;
        });
    }
    /**
     * Perform the next step in whatever program has been loaded into memory.
     * Equivalent to just calling `execute` and incrementing `PC` by 2
     **/
    pub fn step(&mut self) -> Result<(), OpcodeError> {
        let r = self.execute(((self.mem[self.pc] as u16) << 8) | self.mem[self.pc + 1] as u16);
        self.pc += 2;
        r
    }
    /**
     * Function that decrements both timer registers.
     * Should be called at a rate of 60Hz.
     **/
    pub fn on_tick(&mut self) {
        self.dt = self.dt.saturating_sub(1);
        self.st = self.st.saturating_sub(1);
    }
    /**
     * Update the current input states to the inputs given.
     **/
    pub fn update_inputs(&mut self, inputs: [bool; 0x10]) {
        inputs.iter().enumerate().for_each(|(i, v)| {
            if !v && self.input_state[i] {
                self.last_key_released = Some(i as u8);
            }
        });
        self.input_state = inputs;
    }
    /**
     * Executes a single given instruction.
     * Does not increment PC or affect DT or ST.
     **/
    pub fn execute(&mut self, inst: u16) -> Result<(), OpcodeError> {
        if self.debug_print {
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
                _ => {}
            },
            0x1000 => self.jmp(inst & 0x0FFF),
            0x2000 => self.call(inst & 0xFFF),
            0x3000 => self.se_r_kk(reg_at(inst, 1), (inst & 0xFF) as u8),
            0x4000 => self.sne_r_kk(inst),
            0x5000 => {
                if inst & 0xF != 0 {
                    return Err(OpcodeError::new(inst, self.pc as u8));
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
                    6 => self.shr(rx, ry),
                    7 => self.subn(rx, ry),
                    0xE => self.shl(rx, ry),
                    _ => return Err(OpcodeError::new(inst, self.pc as u8)),
                }
            }
            0x9000 => {
                if inst & 0xF != 0 {
                    return Err(OpcodeError::new(inst, self.pc as u8));
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
                _ => return Err(OpcodeError::new(inst, self.pc as u8)),
            },
            0xF000 => match inst & 0x00FF {
                0x07 => self.ld_r_dt(reg_at(inst, 1)),
                0x0A => self.ld_r_kp(reg_at(inst, 1)),
                0x15 => self.ld_dt_r(reg_at(inst, 1)),
                0x18 => self.ld_st_r(reg_at(inst, 1)),
                0x1E => self.add_i_r(reg_at(inst, 1)),
                0x29 => self.ld_i_spr_x(reg_at(inst, 1)),
                0x33 => self.ld_bcd_r(reg_at(inst, 1)),
                0x55 => self.store_at_i(reg_at(inst, 1)),
                0x65 => self.load_from_i(reg_at(inst, 1)),
                _ => return Err(OpcodeError::new(inst, self.pc as u8)),
            },
            _ => return Err(OpcodeError::new(inst, self.pc as u8)),
        }
        if self.debug_print {
            println!("Post state:");
            self.dump_state();
        }
        self.last_key_released = None;

        return Ok(());
    }

    /**
     * Print the state of the machine (all registers, stack, PC, etc) in the console.
     */
    pub fn dump_state(&self) {
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
        self.registers[0xF] = 0x0;
    }

    fn and_rx_ry(&mut self, rx: usize, ry: usize) {
        self.registers[rx] = self.registers[rx] & self.registers[ry];
        self.registers[0xF] = 0x0;
    }

    fn xor_rx_ry(&mut self, rx: usize, ry: usize) {
        self.registers[rx] = self.registers[rx] ^ self.registers[ry];
        self.registers[0xF] = 0x0;
    }

    fn add_rx_ry(&mut self, rx: usize, ry: usize) {
        let v = self.registers[rx] as u16 + self.registers[ry] as u16;
        self.registers[rx] = v as u8;
        self.registers[0xF] = if v > 0xFF { 1 } else { 0 };
    }
    fn sub_rx_ry(&mut self, rx: usize, ry: usize) {
        let vf = if self.registers[ry] > self.registers[rx] {
            0
        } else {
            1
        };
        self.registers[rx] = self.registers[rx].wrapping_sub(self.registers[ry]);
        self.registers[0xF] = vf;
    }
    fn shr(&mut self, rx: usize, ry: usize) {
        let vf = if self.registers[ry] & 0x01 == 1 { 1 } else { 0 };
        self.registers[rx] = self.registers[ry] >> 1;
        self.registers[0xF] = vf;
    }
    fn subn(&mut self, rx: usize, ry: usize) {
        let vf = if self.registers[rx] > self.registers[ry] {
            0
        } else {
            1
        };
        self.registers[rx] = self.registers[ry].wrapping_sub(self.registers[rx]);
        self.registers[0xF] = vf;
    }
    fn shl(&mut self, rx: usize, ry: usize) {
        let vf = if self.registers[ry] & 0x80 == 0x80 {
            1
        } else {
            0
        };
        self.registers[rx] = self.registers[ry] << 1;
        self.registers[0xF] = vf;
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
        self.pc = addr as usize - 2;
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
        match self.last_key_released {
            Some(i) => self.registers[r] = i as u8,
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
        self.i += (n + 1) as u16;
    }
    fn store_at_i(&mut self, n: usize) {
        for j in 0..(n + 1) {
            self.mem[(self.i + j as u16) as usize] = self.registers[j as usize];
        }
        self.i += (n + 1) as u16;
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
        self.registers[0xF] = 0;
        let x = self.registers[rx] % SCREEN_WIDTH as u8;
        let y = self.registers[ry] % SCREEN_HEIGHT as u8;
        // XOR data onto screen
        for j in 0..n {
            let mut val = self.mem[self.i as usize + j];
            for k in 0..8 {
                if x as usize + k >= SCREEN_WIDTH || y as usize + j >= SCREEN_HEIGHT {
                    continue;
                }
                let coord: usize = (y as usize + j) * SCREEN_WIDTH + x as usize + k;
                let p = (val & 0x80) != 0; // Get MSB
                if p && self.screen_buffer[coord] {
                    self.registers[0xF] = 1;
                }
                self.screen_buffer[coord] = p ^ self.screen_buffer[coord];
                val = val << 1;
            }
        }
    }
    // Only loads the sprite for the LSByte of Vr
    fn ld_i_spr_x(&mut self, r: usize) {
        // 6 because that's the length of each sprite
        // 5 + 1 buffer row
        self.i = (self.registers[r] as u16 & 0xF) * 0x6;
    }
    fn ld_bcd_r(&mut self, r: usize) {
        self.mem[self.i as usize] = self.registers[r] / 100;
        self.mem[self.i as usize + 1] = (self.registers[r] / 10) % 10;
        self.mem[self.i as usize + 2] = self.registers[r] % 10;
    }

    /// Get the value of an R register
    pub fn get_register_value(&self, register: u8) -> u8 {
        return self.registers[register as usize];
    }
    /// Get the value of the program counter
    pub fn get_program_counter(&self) -> usize {
        return self.pc;
    }
    /// Get the value of the I register
    pub fn get_i(&self) -> u16 {
        return self.i;
    }
    /// Get a single byte of memory at the address given
    pub fn get_mem_at(&self, addr: usize) -> u8 {
        return self.mem[addr];
    }
    /**
     * Get a pixel at the given `x, y` position on the screen.
     * Accounts for screen wrapping.
     */
    pub fn get_pixel_at(&self, x: u8, y: u8) -> bool {
        return self.screen_buffer
            [((x as usize % 64) + y as usize * 64) % self.screen_buffer.len()];
    }
    /**
     * Return whether the processor has the `i` key currently being pressed.
     * This should be set by calls to `update_inputs`.
     */
    pub fn get_input_state(&self, i: usize) -> bool {
        return self.input_state[i];
    }
    /// Get the D timer register.
    pub fn get_dt(&self) -> u8 {
        return self.dt;
    }
    /// Get the S (sound) time register.
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
