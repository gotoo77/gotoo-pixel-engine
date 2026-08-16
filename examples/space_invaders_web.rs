#[allow(dead_code)]
#[path = "space_invaders/enhanced.rs"]
mod game;

use game::{EnhancedSpaceInvadersGame, FRAMEBUFFER_HEIGHT, TOUCH_FRAMEBUFFER_WIDTH};
use gotoo_pixel_engine::{
    EngineConfig, Rect, Size, run,
    ui::{PauseConfig, PauseGame},
};
use wasm_bindgen::prelude::*;

const PAUSE_BUTTON: Rect = Rect {
    x: 270,
    y: 4,
    width: 92,
    height: 24,
};

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let size = Size {
        width: TOUCH_FRAMEBUFFER_WIDTH,
        height: FRAMEBUFFER_HEIGHT,
    };
    let game = PauseGame::new(
        EnhancedSpaceInvadersGame::new_touch(),
        PauseConfig::new(size).with_touch_button(PAUSE_BUTTON),
    );

    run(
        EngineConfig {
            title: "Space Invaders".into(),
            framebuffer_width: size.width,
            framebuffer_height: size.height,
            window_width: size.width * 3,
            window_height: size.height * 3,
        },
        game,
    )
    .map_err(|err| JsValue::from_str(&err.to_string()))
}
