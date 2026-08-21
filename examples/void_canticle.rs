pub(crate) mod game {
    #[allow(dead_code, clippy::collapsible_if, clippy::too_many_arguments)]
    mod legacy {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/void_canticle/legacy.rs"
        ));
    }

    #[allow(dead_code, clippy::collapsible_if, clippy::too_many_arguments)]
    mod presentation {
        use super::legacy::*;
        use super::legacy::v07::*;
        use super::legacy::v07::v09::*;
        use super::legacy::v07::v09::v10::*;
        use super::legacy::v07::v09::v10::v11::*;
        use super::legacy::v07::v09::v10::v11::v12::*;
        use super::legacy::v07::v09::v10::v11::v12::v13::*;
        use super::legacy::v07::v09::v10::v11::v12::v13::v14::*;
        use super::legacy::v07::v09::v10::v11::v12::v13::v14::v15::*;
        use super::legacy::v07::v09::v10::v11::v12::v13::v14::v15::v16::*;
        use super::legacy::v07::v09::v10::v11::v12::v13::v14::v15::v16::v16b::*;
        use super::legacy::v07::v09::v10::v11::v12::v13::v14::v15::v16::v16b::v17::*;
        use super::legacy::v07::v09::v10::v11::v12::v13::v14::v15::v16::v16b::v17::v18::*;
        use super::legacy::v07::v09::v10::v11::v12::v13::v14::v15::v16::v16b::v17::v18::v19::*;
        use super::legacy::v07::v09::v10::v11::v12::v13::v14::v15::v16::v16b::v17::v18::v19::v20::*;
        use super::legacy::v07::v09::v10::v11::v12::v13::v14::v15::v16::v16b::v17::v18::v19::v20::v21::*;
        use super::legacy::v07::v09::v10::v11::v12::v13::v14::v15::v16::v16b::v17::v18::v19::v20::v21::v22::*;
        use super::legacy::v07::v09::v10::v11::v12::v13::v14::v15::v16::v16b::v17::v18::v19::v20::v21::v22::v23::*;

        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/void_canticle/presentation/mod.rs"
        ));
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/void_canticle/presentation/choice_runtime.rs"
        ));
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/void_canticle/presentation/frontend.rs"
        ));
    }

    pub(crate) use presentation::run_void_canticle_with_obs_mirror;
    #[cfg(target_arch = "wasm32")]
    pub(crate) use presentation::vc27_preload_choice_catalog_web as preload_choice_catalog_web;
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), gotoo_pixel_engine::EngineError> {
    game::run_void_canticle_with_obs_mirror()
}
