extern crate gl;
extern crate glfw;

use crate::processor::Processor;

pub trait Interface {
    // Update the inputs in the processor and the sound
    // Return true if the program should exit, false otherwise
    fn update(&mut self, p: &mut Processor) -> bool;
    // Render the screen
    fn render(&mut self, p: &Processor);
    // Exit (do any cleanup necessarry)
    fn exit(&mut self);
}
