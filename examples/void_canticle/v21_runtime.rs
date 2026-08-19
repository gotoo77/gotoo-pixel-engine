struct VoidCanticleV21Runtime {
    game: VoidCanticleV21,
}

impl VoidCanticleV21Runtime {
    fn new() -> Self {
        Self {
            game: VoidCanticleV21::new(),
        }
    }

    fn sync_fatal_hull_to_legacy_game_over(&mut self) {
        if self.game.player_hull > 0.0 {
            return;
        }

        let base = self.game.base_mut();
        base.lives = 0;
        base.game_over = true;
    }
}

impl Game for VoidCanticleV21Runtime {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let result = self.game.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        self.sync_fatal_hull_to_legacy_game_over();
        GameResult::Continue
    }
}

pub fn run_v21_runtime_with_obs_mirror() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: format!("Void Canticle {VC21_VERSION} - Gotoo Pixel Engine"),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width,
            window_height,
        },
        gotoo_pixel_engine::ObsMirrorGame::from_env(
            VoidCanticleV21Runtime::new(),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v21_runtime_tests {
    use super::*;

    #[test]
    fn fatal_hull_is_zero_not_an_extra_life() {
        let mut shield = 0.0;
        let mut hull = 20.0;
        let (_, hull_damage) = vc21_apply_damage_to_layers(&mut shield, &mut hull, 35.0);
        assert_eq!(hull_damage, 20.0);
        assert_eq!(hull, 0.0);
    }
}
