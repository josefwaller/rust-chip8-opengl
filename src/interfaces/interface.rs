extern crate gl;
extern crate glfw;

use crate::processor::Processor;

pub trait Interface {
    // Update the inputs in the processor
    // Return true if the program should exit, false otherwise
    fn update_inputs(&mut self, p: &mut Processor) -> bool;
    // Render the screen
    fn render(&mut self, p: &Processor);
    // Exit (do any cleanup necessarry)
    fn exit(&mut self);
}
