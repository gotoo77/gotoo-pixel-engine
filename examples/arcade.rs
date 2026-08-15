#[path = "arcade/game.rs"]
mod arcade;

use arcade::{ArcadeApp, ArcadeInteractionMode};
use gotoo_pixel_engine::{EngineConfig, run};

fn window_size(framebuffer_width: u32, framebuffer_height: u32) -> (u32, u32) {
    if std::env::var_os("WSL_DISTRO_NAME").is_some() {
        // WSLg/Weston is known to crash for some surface sizes. 960x612 is
        // documented as stable in docs/investigations/wslg-surface-present-stall.md.
        (960, 612)
    } else {
        (framebuffer_width * 3, framebuffer_height * 3)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let size = ArcadeInteractionMode::Native.framebuffer_size();
    let (window_width, window_height) = window_size(size.width, size.height);
    run(
        EngineConfig {
            title: "GPE Arcade".into(),
            framebuffer_width: size.width,
            framebuffer_height: size.height,
            window_width,
            window_height,
        },
        ArcadeApp::new(ArcadeInteractionMode::Native),
    )?;
    Ok(())
}
