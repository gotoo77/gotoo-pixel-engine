#[path = "space_invaders/enhanced.rs"]
mod game;

use game::{EnhancedSpaceInvadersGame, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};
use gotoo_pixel_engine::{run, EngineConfig, EngineError};

fn main() -> Result<(), EngineError> {
    run(
        EngineConfig {
            title: "Space Invaders - Gotoo Pixel Engine".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width: 768,
            window_height: 672,
        },
        EnhancedSpaceInvadersGame::new(),
    )
}
