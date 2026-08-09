#[path = "../src/demo.rs"]
mod demo;

use demo::DemoGame;
use gotoo_pixel_engine::{EngineConfig, run};
use wasm_bindgen::prelude::*;

const FRAMEBUFFER_WIDTH: u32 = 320;
const FRAMEBUFFER_HEIGHT: u32 = 180;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    run(
        EngineConfig {
            title: "gotoo-pixel-engine M1.5 web spike".into(),
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
    .map_err(|err| JsValue::from_str(&err.to_string()))
}
