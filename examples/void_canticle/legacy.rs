macro_rules! legacy_include {
    ($path:literal) => {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/void_canticle/",
            $path
        ));
    };
}

legacy_include!("legacy/game.rs");

pub(crate) mod v07 {
    #![allow(dead_code, clippy::collapsible_if)]

    use super::*;
    legacy_include!("legacy/v07_visuals.rs");
    legacy_include!("legacy/v07_game.rs");
    legacy_include!("legacy/v07_weapon.rs");

    pub(crate) mod v09 {
        #![allow(dead_code, clippy::approx_constant, clippy::collapsible_if)]

        use super::*;
        legacy_include!("legacy/v09_game.rs");

        pub(crate) mod v10 {
            use super::*;
            legacy_include!("legacy/v10_game.rs");

            pub(crate) mod v11 {
                #![allow(clippy::assign_op_pattern)]

                use super::*;
                legacy_include!("legacy/v11_game.rs");

                pub(crate) mod v12 {
                    use super::*;
                    legacy_include!("legacy/v12_game.rs");

                    pub(crate) mod v13 {
                        use super::*;
                        legacy_include!("legacy/v13_game.rs");

                        pub(crate) mod v14 {
                            use super::*;
                            legacy_include!("legacy/v14_game.rs");

                            pub(crate) mod v15 {
                                use super::*;
                                legacy_include!("legacy/v15_game.rs");

                                pub(crate) mod v16 {
                                    use super::*;
                                    legacy_include!("legacy/v16_game.rs");

                                    pub(crate) mod v16b {
                                        use super::*;
                                        legacy_include!("legacy/v16b_game.rs");

                                        pub(crate) mod v17 {
                                            use super::*;
                                            legacy_include!("legacy/v17_game.rs");

                                            pub(crate) mod v18 {
                                                use super::*;
                                                legacy_include!("legacy/v18_game.rs");

                                                pub(crate) mod v19 {
                                                    use super::*;
                                                    legacy_include!("legacy/v19_game.rs");

                                                    pub(crate) mod v20 {
                                                        #![allow(unused_assignments)]

                                                        use super::*;
                                                        legacy_include!("legacy/v20_game.rs");

                                                        pub(crate) mod v21 {
                                                            use super::*;
                                                            legacy_include!("legacy/v21_game.rs");
                                                            legacy_include!("legacy/v21_runtime.rs");
                                                            legacy_include!("legacy/v21_tuning.rs");
                                                            legacy_include!("legacy/v21_stabilization.rs");
                                                            legacy_include!("legacy/v21_survival_cleanup.rs");

                                                            pub(crate) mod v22 {
                                                                use super::*;
                                                                legacy_include!("legacy/v22_game.rs");
                                                                legacy_include!("legacy/v22_movement.rs");
                                                                legacy_include!("legacy/v22_passives.rs");

                                                                pub(crate) mod v23 {
                                                                    use super::*;
                                                                    legacy_include!("legacy/v23_game.rs");
                                                                    legacy_include!("legacy/v23_sustain.rs");
                                                                    legacy_include!(
                                                                        "legacy/v23_visual_foundation.rs"
                                                                    );

                                                                    pub(crate) mod presentation {
                                                                        use super::*;
                                                                        legacy_include!(
                                                                            "presentation/mod.rs"
                                                                        );
                                                                        legacy_include!(
                                                                            "presentation/choice_runtime.rs"
                                                                        );
                                                                        legacy_include!(
                                                                            "presentation/frontend.rs"
                                                                        );
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// This is the only boundary through which current code may reach the historical
// implementation. The version-numbered module graph stays quarantined here.
pub(crate) use v07::v09::v10::v11::v12::v13::v14::v15::v16::v16b::v17::v18::v19::v20::v21::v22::v23::presentation::run_void_canticle_with_obs_mirror;

#[cfg(target_arch = "wasm32")]
pub(crate) use v07::v09::v10::v11::v12::v13::v14::v15::v16::v16b::v17::v18::v19::v20::v21::v22::v23::presentation::vc27_preload_choice_catalog_web as preload_choice_catalog_web;
