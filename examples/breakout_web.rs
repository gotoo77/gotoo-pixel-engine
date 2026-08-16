#[allow(dead_code)]
#[path = "breakout/game.rs"]
mod breakout;

use breakout::{BreakoutGame, FRAMEBUFFER_HEIGHT, TOUCH_FRAMEBUFFER_WIDTH};
use gotoo_pixel_engine::{
    EngineConfig, Rect, Size, run,
    ui::{PauseConfig, PauseGame},
};
use wasm_bindgen::prelude::*;

const PAUSE_BUTTON: Rect = Rect {
    x: 342,
    y: 8,
    width: 116,
    height: 28,
};

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let size = Size {
        width: TOUCH_FRAMEBUFFER_WIDTH,
        height: FRAMEBUFFER_HEIGHT,
    };
    let game = PauseGame::new(
        BreakoutGame::new_touch(),
        PauseConfig::new(size).with_touch_button(PAUSE_BUTTON),
    );

    run(
        EngineConfig {
            title: "Breakout".into(),
            framebuffer_width: size.width,
            framebuffer_height: size.height,
            window_width: size.width * 3,
            window_height: size.height * 3,
        },
        game,
    )
    .map_err(|err| JsValue::from_str(&err.to_string()))
}
