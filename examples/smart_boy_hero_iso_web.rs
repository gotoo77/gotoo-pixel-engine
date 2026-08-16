#[allow(dead_code)]
#[path = "smart_boy_hero_iso/game.rs"]
mod game;

use game::{FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH, SmartBoyHeroIsoGame};
use gotoo_pixel_engine::{EngineConfig, run};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    run(
        EngineConfig {
            title: "Smart Boy Hero Iso Slice".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width: FRAMEBUFFER_WIDTH * 2,
            window_height: FRAMEBUFFER_HEIGHT * 2,
        },
        SmartBoyHeroIsoGame::new(),
    )
    .map_err(|err| JsValue::from_str(&err.to_string()))
}
