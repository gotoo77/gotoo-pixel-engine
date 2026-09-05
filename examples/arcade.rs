#[path = "arcade/high_res.rs"]
mod high_res;

use gotoo_pixel_engine::{EngineConfig, run};
use high_res::{ArcadeHighResApp, HOST_SIZE};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run(
        EngineConfig {
            title: "GPE Arcade".into(),
            framebuffer_width: HOST_SIZE.width,
            framebuffer_height: HOST_SIZE.height,
            window_width: HOST_SIZE.width,
            window_height: HOST_SIZE.height,
        },
        ArcadeHighResApp::new(),
    )?;
    Ok(())
}
