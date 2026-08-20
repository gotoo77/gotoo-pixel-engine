#[path = "smart_boy_hero_iso/game.rs"]
mod game;

use game::{FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH, SmartBoyHeroIsoGame};
use gotoo_pixel_engine::{EngineConfig, EngineError, run};

fn window_size() -> (u32, u32) {
    if std::env::var_os("WSL_DISTRO_NAME").is_some() {
        (960, 612)
    } else {
        (FRAMEBUFFER_WIDTH * 2, FRAMEBUFFER_HEIGHT * 2)
    }
}

fn main() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: "Smart Boy Hero Iso Slice".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width,
            window_height,
        },
        SmartBoyHeroIsoGame::new(),
    )
}
