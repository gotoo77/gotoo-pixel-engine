mod game {
    #[allow(dead_code, clippy::collapsible_if, clippy::too_many_arguments)]
    include!("void_canticle/game.rs");
    include!("void_canticle/v07_visuals.rs");
    #[allow(clippy::collapsible_if)]
    include!("void_canticle/v07_game.rs");
    include!("void_canticle/v07_weapon.rs");
    #[allow(dead_code, clippy::approx_constant, clippy::collapsible_if)]
    include!("void_canticle/v09_game.rs");
    include!("void_canticle/v10_game.rs");
}

fn main() -> Result<(), gotoo_pixel_engine::EngineError> {
    game::run_v10()
}
