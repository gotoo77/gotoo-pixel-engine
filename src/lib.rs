pub mod framebuffer;
mod input;
mod pixel;
mod platform;
mod renderer;

pub use framebuffer::Framebuffer;
pub use input::{ButtonState, Input, Key, MouseButton};
pub use pixel::Pixel;
pub use platform::{EngineConfig, EngineError, Frame, Game, GameResult, run};
