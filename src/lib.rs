//! CHIP-8 emulation library
//!
//! Simple library for emulating CHIP-8 programs, or running individual opcodes.
//! Also can be used as a proper emulator, either rendering the ROM using opengl or in the terminal.
//!
//! ```
//! extern crate rust_chip8_opengl;
//! use rust_chip8_opengl::processor::Processor;
//! fn main() {
//!   let mut p = Processor::new();
//!
//!   // Execute individual opcodes
//!   p.execute(0x60FF);
//!   p.execute(0x6118);
//!   assert_eq!(p.get_register_value(0x0), 0xFF);
//!   assert_eq!(p.get_register_value(0x1), 0x18);
//!
//!   // Load a program and execute it step by step
//!   let program = [0x6005, 0x6105, 0x8014];
//!   // load_program also available for u8
//!   p.load_program_u16(&program);
//!   p.step();
//!   p.step();
//!   p.step();
//!   assert_eq!(p.get_register_value(0x0), 0xA);
//! }
//! ```
mod errors;
#[doc(hidden)]
pub mod interfaces;
#[doc(hidden)]
pub mod processor;

pub use self::errors::OpcodeError;
pub use self::processor::Processor;
