#[allow(dead_code)]
#[path = "pong/game.rs"]
mod pong;

use gotoo_pixel_engine::{
    EngineConfig, Rect, Size, run,
    ui::{PauseConfig, PauseGame},
};
use pong::{FRAMEBUFFER_WIDTH, PongGame, TOUCH_FRAMEBUFFER_HEIGHT};
use wasm_bindgen::prelude::*;

const PAUSE_BUTTON: Rect = Rect {
    x: 204,
    y: 188,
    width: 48,
    height: 66,
};

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let size = Size {
        width: FRAMEBUFFER_WIDTH,
        height: TOUCH_FRAMEBUFFER_HEIGHT,
    };
    let game = PauseGame::new(
        PongGame::new_touch(),
        PauseConfig::new(size).with_touch_button(PAUSE_BUTTON),
    );

    run(
        EngineConfig {
            title: "Pong".into(),
            framebuffer_width: size.width,
            framebuffer_height: size.height,
            window_width: size.width * 3,
            window_height: size.height * 3,
        },
        game,
    )
    .map_err(|err| JsValue::from_str(&err.to_string()))
}
