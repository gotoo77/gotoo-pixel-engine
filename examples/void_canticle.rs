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

                    pub(crate) fn run_v10_with_obs_mirror() -> Result<(), EngineError> {
                        let (window_width, window_height) = window_size();
                        run(
                            EngineConfig {
                                title: "Void Canticle - Gotoo Pixel Engine".to_string(),
                                framebuffer_width: FRAMEBUFFER_WIDTH,
                                framebuffer_height: FRAMEBUFFER_HEIGHT,
                                window_width,
                                window_height,
                            },
                            gotoo_pixel_engine::ObsMirrorGame::from_env(
                                VoidCanticlePause::new(VoidCanticleV10::new()),
                                FRAMEBUFFER_WIDTH,
                                FRAMEBUFFER_HEIGHT,
                            ),
                        )
                    }
                }
            }
        }
    }

    pub(crate) use legacy_base::v07::v09::v10::run_v10_with_obs_mirror;
}

fn main() -> Result<(), gotoo_pixel_engine::EngineError> {
    game::run_v10_with_obs_mirror()
}
