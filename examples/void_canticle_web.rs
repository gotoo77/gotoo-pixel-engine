#[cfg(target_arch = "wasm32")]
#[path = "void_canticle.rs"]
mod void_canticle;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    wasm_bindgen_futures::spawn_local(async {
        let _ = void_canticle::game::preload_choice_catalog_web(
            "./assets/void_canticle/ui/choice",
        )
        .await;

        if let Err(error) = void_canticle::game::run_void_canticle_with_obs_mirror() {
            panic!("Void Canticle Web failed to start: {error}");
        }
    });
    Ok(())
}
