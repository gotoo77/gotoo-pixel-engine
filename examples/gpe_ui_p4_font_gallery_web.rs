#[cfg(target_arch = "wasm32")]
mod gallery {
    include!("gpe_ui_p4_font_gallery.rs");

    pub(super) fn new_gallery() -> Result<impl gotoo_pixel_engine::Game, &'static str> {
        Gallery::new()
    }
}

#[cfg(target_arch = "wasm32")]
use gotoo_pixel_engine::{EngineConfig, run};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
const WIDTH: u32 = 1200;
#[cfg(target_arch = "wasm32")]
const HEIGHT: u32 = 800;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let gallery = gallery::new_gallery().map_err(JsValue::from_str)?;
    run(
        EngineConfig {
            title: "GPE.UI P4 / Font Finder A-Z / Web".into(),
            framebuffer_width: WIDTH,
            framebuffer_height: HEIGHT,
            window_width: WIDTH,
            window_height: HEIGHT,
        },
        gallery,
    )
    .map_err(|error| JsValue::from_str(&error.to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("gpe_ui_p4_font_gallery_web targets wasm32-unknown-unknown");
}
