extern crate gl;
extern crate glfw;

use crate::processor::Processor;

/**
 * An interface between the user and the CHIP-8 Processor.
 * Can be implemented for different interfaces (i.e. terminal, open-gl) and used
 * by the same main program.
 */
pub trait Interface {
    /**
     * Update the inputs in the processor and the sound.
     * Return `true` if the program should exit, `false` otherwise.
     */
    fn update(&mut self, p: &mut Processor) -> bool;
    /**
     * Render the screen.
     */
    fn render(&mut self, p: &Processor);
    /**
     * Cleanup function that should be called on exit before the program quits.
     */
    fn exit(&mut self);
}
