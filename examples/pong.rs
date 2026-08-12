#[path = "pong/game.rs"]
mod game;

use game::{FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH, PongGame};
use gotoo_pixel_engine::{EngineConfig, EngineError, run};

fn main() -> Result<(), EngineError> {
    run(
        EngineConfig {
            title: "Pong - Gotoo Pixel Engine".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width: 960,
            window_height: 540,
        },
        PongGame::new(),
    )
}
