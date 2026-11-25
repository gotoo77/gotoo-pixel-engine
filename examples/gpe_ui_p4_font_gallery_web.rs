#[cfg(target_arch = "wasm32")]
mod gallery {
    include!("gpe_ui_p4_font_gallery.rs");
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    gallery::main().map_err(|error| JsValue::from_str(&error.to_string()))
}
