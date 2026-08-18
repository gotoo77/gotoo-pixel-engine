#[path = "smart_boy_hero/game.rs"]
mod game;

use std::{fs, io};

use game::{FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH, SmartBoyHeroGame};
use gotoo_pixel_engine::{EngineConfig, run};

fn window_size() -> (u32, u32) {
    if std::env::var_os("WSL_DISTRO_NAME").is_some() {
        // WSLg/Weston is known to crash for some surface sizes. Reuse the
        // stable native size already used by Arcade and Space Invaders.
        (960, 612)
    } else {
        (FRAMEBUFFER_WIDTH * 3, FRAMEBUFFER_HEIGHT * 3)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (window_width, window_height) = window_size();
    let game = match std::env::args_os().nth(1) {
        Some(path) => {
            let json = fs::read_to_string(&path)?;
            SmartBoyHeroGame::from_level_json(&json)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?
        }
        None => SmartBoyHeroGame::new(),
    };

    run(
        EngineConfig {
            title: "Smart Boy Hero - Gotoo Pixel Engine".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width,
            window_height,
        },
        game,
    )?;
    Ok(())
}
