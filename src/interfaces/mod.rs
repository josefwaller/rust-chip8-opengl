mod interface;
#[cfg(feature = "open-gl")]
mod opengl_interface;
#[cfg(feature = "terminal")]
mod terminal_interface;

pub use interface::Interface;
#[cfg(feature = "open-gl")]
pub use opengl_interface::OpenGlInterface;
#[cfg(feature = "terminal")]
pub use terminal_interface::TerminalInterface;
