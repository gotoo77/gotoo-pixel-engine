#[allow(dead_code)]
#[path = "pong.rs"]
mod pong;

use gotoo_pixel_engine::{EngineConfig, run};
use pong::{FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH, PongApp};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    run(
        EngineConfig {
            title: "Pong".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width: FRAMEBUFFER_WIDTH * 3,
            window_height: FRAMEBUFFER_HEIGHT * 3,
        },
        PongApp::new_touch(),
    )
    .map_err(|err| JsValue::from_str(&err.to_string()))
}
