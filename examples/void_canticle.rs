mod game {
    include!("void_canticle/game.rs");
    include!("void_canticle/v07_visuals.rs");
    include!("void_canticle/v07_game.rs");
    include!("void_canticle/v07_weapon.rs");
    include!("void_canticle/v09_game.rs");
    include!("void_canticle/v10_game.rs");
}

fn main() -> Result<(), gotoo_pixel_engine::EngineError> {
    game::run_v10()
}
