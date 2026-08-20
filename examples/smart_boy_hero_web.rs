#[allow(dead_code)]
#[path = "smart_boy_hero/game.rs"]
mod game;

use game::{SmartBoyHeroGame, SmartBoyHeroMode};
use gotoo_pixel_engine::{EngineConfig, run};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let size = SmartBoyHeroMode::Touch.framebuffer_size();

    run(
        EngineConfig {
            title: "Smart Boy Hero".into(),
            framebuffer_width: size.width,
            framebuffer_height: size.height,
            window_width: size.width * 3,
            window_height: size.height * 3,
        },
        SmartBoyHeroGame::new_touch(),
    )
    .map_err(|err| JsValue::from_str(&err.to_string()))
}
