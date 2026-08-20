pub(crate) mod game {
    mod legacy_base {
        #![allow(dead_code, clippy::collapsible_if, clippy::too_many_arguments)]

        include!("void_canticle/game.rs");

        pub(crate) mod v07 {
            #![allow(dead_code, clippy::collapsible_if)]

            use super::*;
            include!("void_canticle/v07_visuals.rs");
            include!("void_canticle/v07_game.rs");
            include!("void_canticle/v07_weapon.rs");

            pub(crate) mod v09 {
                #![allow(dead_code, clippy::approx_constant, clippy::collapsible_if)]

                use super::*;
                include!("void_canticle/v09_game.rs");

                pub(crate) mod v10 {
                    use super::*;
                    include!("void_canticle/v10_game.rs");

                    pub(crate) mod v11 {
                        #![allow(clippy::assign_op_pattern)]

                        use super::*;
                        include!("void_canticle/v11_game.rs");

                        pub(crate) mod v12 {
                            use super::*;
                            include!("void_canticle/v12_game.rs");

                            pub(crate) mod v13 {
                                use super::*;
                                include!("void_canticle/v13_game.rs");

                                pub(crate) mod v14 {
                                    use super::*;
                                    include!("void_canticle/v14_game.rs");

                                    pub(crate) mod v15 {
                                        use super::*;
                                        include!("void_canticle/v15_game.rs");

                                        pub(crate) mod v16 {
                                            use super::*;
                                            include!("void_canticle/v16_game.rs");

                                            pub(crate) mod v16b {
                                                use super::*;
                                                include!("void_canticle/v16b_game.rs");

                                                pub(crate) mod v17 {
                                                    use super::*;
                                                    include!("void_canticle/v17_game.rs");

                                                    pub(crate) mod v18 {
                                                        use super::*;
                                                        include!("void_canticle/v18_game.rs");

                                                        pub(crate) mod v19 {
                                                            use super::*;
                                                            include!("void_canticle/v19_game.rs");

                                                            pub(crate) mod v20 {
                                                                #![allow(unused_assignments)]

                                                                use super::*;
                                                                include!(
                                                                    "void_canticle/v20_game.rs"
                                                                );

                                                                pub(crate) mod v21 {
                                                                    use super::*;
                                                                    include!(
                                                                        "void_canticle/v21_game.rs"
                                                                    );
                                                                    include!(
                                                                        "void_canticle/v21_runtime.rs"
                                                                    );
                                                                    include!(
                                                                        "void_canticle/v21_tuning.rs"
                                                                    );
                                                                    include!(
                                                                        "void_canticle/v21_stabilization.rs"
                                                                    );
                                                                    include!(
                                                                        "void_canticle/v21_survival_cleanup.rs"
                                                                    );

                                                                    pub(crate) mod v22 {
                                                                        use super::*;
                                                                        include!(
                                                                            "void_canticle/v22_game.rs"
                                                                        );
                                                                        include!(
                                                                            "void_canticle/v22_movement.rs"
                                                                        );
                                                                        include!(
                                                                            "void_canticle/v22_passives.rs"
                                                                        );

                                                                        pub(crate) mod v23 {
                                                                            use super::*;
                                                                            include!(
                                                                                "void_canticle/v23_game.rs"
                                                                            );
                                                                            include!(
                                                                                "void_canticle/v23_sustain.rs"
                                                                            );
                                                                            include!(
                                                                                "void_canticle/v23_visual_foundation.rs"
                                                                            );

                                                                            pub(crate) mod v27 {
                                                                                use super::*;
                                                                                include!(
                                                                                    "void_canticle/v27.rs"
                                                                                );
                                                                                include!(
                                                                                    "void_canticle/v27/choice_runtime.rs"
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
    }

    pub(crate) use legacy_base::v07::v09::v10::v11::v12::v13::v14::v15::v16::v16b::v17::v18::v19::v20::v21::v22::v23::v27::run_v27_showcase_presentation_with_obs_mirror;
    #[cfg(target_arch = "wasm32")]
    pub(crate) use legacy_base::v07::v09::v10::v11::v12::v13::v14::v15::v16::v16b::v17::v18::v19::v20::v21::v22::v23::v27::vc27_preload_choice_catalog_web;
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), gotoo_pixel_engine::EngineError> {
    game::run_v27_showcase_presentation_with_obs_mirror()
}
