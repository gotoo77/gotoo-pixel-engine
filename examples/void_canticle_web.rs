#[path = "void_canticle.rs"]
mod void_canticle;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    wasm_bindgen_futures::spawn_local(async {
        let _ = void_canticle::game::vc27_preload_choice_catalog_web(
            "./assets/void_canticle/ui/choice",
        )
        .await;

        if let Err(error) =
            void_canticle::game::run_v27_showcase_presentation_with_obs_mirror()
        {
            panic!("Void Canticle Web failed to start: {error}");
        }
    });
    Ok(())
}
