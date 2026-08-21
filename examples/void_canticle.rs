pub(crate) mod game {
    #[allow(dead_code, clippy::collapsible_if, clippy::too_many_arguments)]
    mod legacy {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/void_canticle/legacy.rs"
        ));
    }

    // Single semantic facade over the quarantined historical implementation.
    // New callers never need to know or propagate the old v07 -> ... -> v23 chain.
    mod current {
        pub(crate) use super::legacy::v07::v09::v10::v11::v12::v13::v14::v15::v16::v16b::v17::v18::v19::v20::v21::v22::v23::presentation::run_void_canticle_with_obs_mirror;
        #[cfg(target_arch = "wasm32")]
        pub(crate) use super::legacy::v07::v09::v10::v11::v12::v13::v14::v15::v16::v16b::v17::v18::v19::v20::v21::v22::v23::presentation::vc27_preload_choice_catalog_web as preload_choice_catalog_web;
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) use current::preload_choice_catalog_web;
    pub(crate) use current::run_void_canticle_with_obs_mirror;
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), gotoo_pixel_engine::EngineError> {
    game::run_void_canticle_with_obs_mirror()
}
