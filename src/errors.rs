use std::fmt;

/// Error that is returned when the processor encounters an invalid opcode
#[derive(Debug, Clone)]
pub struct OpcodeError {
    /// The invalid opcode that was encountered
    opcode: u16,
    /// The program conuter at the moment the invalid opcode was encountered
    pc: u8,
}

impl OpcodeError {
    pub fn new(opcode: u16, pc: u8) -> OpcodeError {
        OpcodeError { opcode, pc }
    }
}

impl fmt::Display for OpcodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Invalid opcode {:x} encountered at PC = {:x}",
            self.opcode, self.pc
        )
    }
}
