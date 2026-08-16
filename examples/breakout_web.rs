#[allow(dead_code)]
#[path = "breakout/game.rs"]
mod breakout;

use breakout::{BreakoutGame, FRAMEBUFFER_HEIGHT, TOUCH_FRAMEBUFFER_WIDTH};
use gotoo_pixel_engine::{EngineConfig, run};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    run(
        EngineConfig {
            title: "Breakout".into(),
            framebuffer_width: TOUCH_FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width: TOUCH_FRAMEBUFFER_WIDTH * 3,
            window_height: FRAMEBUFFER_HEIGHT * 3,
        },
        BreakoutGame::new_touch(),
    )
    .map_err(|err| JsValue::from_str(&err.to_string()))
}
