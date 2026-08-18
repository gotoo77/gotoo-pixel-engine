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
                            }
                        }
                    }
                }
            }
        }
    }

    pub(crate) use legacy_base::v07::v09::v10::v11::v12::v13::run_v13_with_obs_mirror;
}

fn main() -> Result<(), gotoo_pixel_engine::EngineError> {
    game::run_v13_with_obs_mirror()
}
