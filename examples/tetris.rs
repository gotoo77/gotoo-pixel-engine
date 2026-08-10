#[path = "tetris/game.rs"]
mod game;

use game::{FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH, TetrisGame};
use gotoo_pixel_engine::{EngineConfig, EngineError, run};

fn main() -> Result<(), EngineError> {
    run(
        EngineConfig {
            title: "Tetris - Gotoo Pixel Engine".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT, 
            // Temporary WSLg workaround: this surface size is known to be stable.
            // Some larger/taller sizes currently stall during presentation.
            window_width: 960,
            window_height: 612,
        },
        TetrisGame::new(),
    )
}
