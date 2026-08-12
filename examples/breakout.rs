#[path = "breakout/game.rs"]
mod game;

use game::{BreakoutGame, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};
use gotoo_pixel_engine::{run, EngineConfig, EngineError};

fn main() -> Result<(), EngineError> {
    run(
        EngineConfig {
            title: "Breakout - Gotoo Pixel Engine".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width: 960,
            window_height: 540,
        },
        BreakoutGame::new(),
    )
}
