#[path = "smart_boy_hero/game.rs"]
mod game;

use game::{FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH, SmartBoyHeroGame};
use gotoo_pixel_engine::{EngineConfig, EngineError, run};

fn window_size() -> (u32, u32) {
    if std::env::var_os("WSL_DISTRO_NAME").is_some() {
        // WSLg/Weston is known to crash for some surface sizes. Reuse the
        // stable native size already used by Arcade and Space Invaders.
        (960, 612)
    } else {
        (FRAMEBUFFER_WIDTH * 3, FRAMEBUFFER_HEIGHT * 3)
    }
}

fn main() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: "Smart Boy Hero - Gotoo Pixel Engine".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width,
            window_height,
        },
        SmartBoyHeroGame::new(),
    )
}
