mod demo;

use demo::DemoGame;
use gotoo_pixel_engine::{EngineConfig, EngineError, run};

const FRAMEBUFFER_WIDTH: u32 = 320;
const FRAMEBUFFER_HEIGHT: u32 = 180;

fn main() -> Result<(), EngineError> {
    run(
        EngineConfig {
            title: "gotoo-pixel-engine M1.3".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width: 960,
            window_height: 540,
        },
        DemoGame::new(
            FRAMEBUFFER_WIDTH as f32 / 2.0,
            FRAMEBUFFER_HEIGHT as f32 / 2.0,
        ),
    )
}
