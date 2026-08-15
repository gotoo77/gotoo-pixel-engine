#[path = "arcade/game.rs"]
mod arcade;

use arcade::{ArcadeApp, ArcadeInteractionMode};
use gotoo_pixel_engine::{EngineConfig, run};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let size = ArcadeInteractionMode::Touch.framebuffer_size();
    run(
        EngineConfig {
            title: "GPE Arcade".into(),
            framebuffer_width: size.width,
            framebuffer_height: size.height,
            window_width: size.width * 2,
            window_height: size.height * 2,
        },
        ArcadeApp::new(ArcadeInteractionMode::Touch),
    )
    .map_err(|err| JsValue::from_str(&err.to_string()))
}
