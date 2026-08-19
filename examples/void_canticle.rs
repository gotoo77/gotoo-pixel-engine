mod game {
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
                                                                use super::*;
                                                                include!(
                                                                    "void_canticle/v20_game.rs"
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

    pub(crate) use legacy_base::v07::v09::v10::v11::v12::v13::v14::v15::v16::v16b::v17::v18::v19::v20::run_v20_with_obs_mirror;
}

fn main() -> Result<(), gotoo_pixel_engine::EngineError> {
    game::run_v20_with_obs_mirror()
}
