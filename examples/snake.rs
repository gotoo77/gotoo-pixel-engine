#[path = "snake/game.rs"]
mod game;

use game::{FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH, SnakeGame};
use gotoo_pixel_engine::{EngineConfig, EngineError, run};

fn main() -> Result<(), EngineError> {
    run(
        EngineConfig {
            title: "Snake".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width: FRAMEBUFFER_WIDTH * 3,
            window_height: FRAMEBUFFER_HEIGHT * 3,
        },
        SnakeGame::new(),
    )
}
