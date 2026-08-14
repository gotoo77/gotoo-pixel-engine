#[allow(dead_code)]
#[path = "tetris/game.rs"]
mod game;

use game::{FRAMEBUFFER_HEIGHT, TOUCH_FRAMEBUFFER_WIDTH, TetrisGame};
use gotoo_pixel_engine::{EngineConfig, run};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    run(
        EngineConfig {
            title: "Tetris".into(),
            framebuffer_width: TOUCH_FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width: TOUCH_FRAMEBUFFER_WIDTH * 3,
            window_height: FRAMEBUFFER_HEIGHT * 3,
        },
        TetrisGame::new_touch(),
    )
    .map_err(|err| JsValue::from_str(&err.to_string()))
}
