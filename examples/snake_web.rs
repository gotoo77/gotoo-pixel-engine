#[path = "snake/game.rs"]
mod game;

use game::{SnakeGame, SnakeInteractionMode};
use gotoo_pixel_engine::{EngineConfig, run};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let interaction_mode = SnakeInteractionMode::Touch;
    let framebuffer_size = interaction_mode.framebuffer_size();

    run(
        EngineConfig {
            title: "Snake".into(),
            framebuffer_width: framebuffer_size.width,
            framebuffer_height: framebuffer_size.height,
            window_width: framebuffer_size.width * 3,
            window_height: framebuffer_size.height * 3,
        },
        SnakeGame::new(interaction_mode),
    )
    .map_err(|err| JsValue::from_str(&err.to_string()))
}
