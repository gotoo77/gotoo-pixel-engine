#[path = "arcade/game.rs"]
mod arcade;

use arcade::{ArcadeApp, ArcadeInteractionMode};
use gotoo_pixel_engine::{EngineConfig, run};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let size = ArcadeInteractionMode::Native.framebuffer_size();
    run(
        EngineConfig {
            title: "GPE Arcade".into(),
            framebuffer_width: size.width,
            framebuffer_height: size.height,
            window_width: size.width * 3,
            window_height: size.height * 3,
        },
        ArcadeApp::new(ArcadeInteractionMode::Native),
    )?;
    Ok(())
}
