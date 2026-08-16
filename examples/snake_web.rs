#[path = "snake/game.rs"]
mod game;

use game::{SnakeGame, SnakeInteractionMode};
use gotoo_pixel_engine::{
    EngineConfig, Rect, run,
    ui::{PauseConfig, PauseGame},
};
use wasm_bindgen::prelude::*;

const PAUSE_BUTTON: Rect = Rect {
    x: 400,
    y: 2,
    width: 72,
    height: 20,
};

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let interaction_mode = SnakeInteractionMode::Touch;
    let framebuffer_size = interaction_mode.framebuffer_size();
    let game = PauseGame::new(
        SnakeGame::new(interaction_mode),
        PauseConfig::new(framebuffer_size).with_touch_button(PAUSE_BUTTON),
    );

    run(
        EngineConfig {
            title: "Snake".into(),
            framebuffer_width: framebuffer_size.width,
            framebuffer_height: framebuffer_size.height,
            window_width: framebuffer_size.width * 3,
            window_height: framebuffer_size.height * 3,
        },
        game,
    )
    .map_err(|err| JsValue::from_str(&err.to_string()))
}
