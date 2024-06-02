pub struct Chip8 {
    // Buffer for the screen
    screen_buffer: [[bool; 32]; 64],
    // Registers (1 through F)
    registers: [u16; 16],
}

impl Chip8 {
    pub fn new() -> Chip8 {
        return Chip8 {
            screen_buffer: [[false; 32]; 64],
            registers: [0x00; 16],
        };
    }
    pub fn step(&mut self, instruction: u16) {}

    pub fn get_register_value(&mut self, register: u8) -> u16 {
        return self.registers[register as usize];
    }
}
