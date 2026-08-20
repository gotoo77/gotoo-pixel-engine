pub(crate) mod game {
    // Historical implementation is isolated behind one semantic boundary.
    // Current callers never depend on the old vXX module graph directly.
    #[allow(dead_code, clippy::collapsible_if, clippy::too_many_arguments)]
    mod legacy {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/void_canticle/legacy.rs"
        ));
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) use legacy::preload_choice_catalog_web;
    pub(crate) use legacy::run_void_canticle_with_obs_mirror;
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), gotoo_pixel_engine::EngineError> {
    game::run_void_canticle_with_obs_mirror()
}
