struct VoidCanticleV21Runtime {
    game: VoidCanticleV21,
}

impl VoidCanticleV21Runtime {
    fn new() -> Self {
        Self {
            game: VoidCanticleV21::new(),
        }
    }

    fn cancel_pending_choices(&mut self) {
        let v14 = &mut self
            .game
            .combat
            .game
            .ui
            .game
            .combat
            .combat
            .combat
            .combat;
        v14.progression.level_choice = None;
        v14.mutation_choice = None;
    }

    fn sync_fatal_hull_to_legacy_game_over(&mut self) -> bool {
        if self.game.player_hull > 0.0 {
            return false;
        }

        // VC2.1 decides fatality after the legacy progression layers have
        // updated. A kill/XP pickup on the same frame can therefore have
        // opened a level-up or mutation panel before Hull reaches zero.
        // Death is terminal for the combat slice and must always preempt
        // those transient progression screens.
        self.cancel_pending_choices();

        let base = self.game.base_mut();
        base.lives = 0;
        base.game_over = true;
        true
    }

    fn render_fatal_state(&self, framebuffer: &mut Framebuffer) {
        self.game.base().render(framebuffer, false);
        self.game.render_version_overlay(framebuffer);
        self.game.render_player_survival(framebuffer);
    }
}

impl Game for VoidCanticleV21Runtime {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let result = self.game.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        if self.sync_fatal_hull_to_legacy_game_over() {
            // The inner progression layer may already have rendered a choice
            // during the fatal frame. Repaint immediately so the player never
            // sees LEVEL UP/MUTATION instead of the death state.
            self.render_fatal_state(frame.framebuffer);
        }
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

    #[test]
    fn fatal_hull_cancels_level_and_mutation_choices() {
        let mut runtime = VoidCanticleV21Runtime::new();
        runtime.game.player_hull = 0.0;

        {
            let v14 = &mut runtime
                .game
                .combat
                .game
                .ui
                .game
                .combat
                .combat
                .combat
                .combat;
            v14.progression.level_choice = Some(LevelChoice {
                offers: [
                    UpgradeKind::RapidFire,
                    UpgradeKind::MagnetField,
                    UpgradeKind::CoreSurge,
                ],
                menu: gotoo_pixel_engine::ui::MenuState::new(3),
            });
            v14.mutation_choice = Some(MutationChoice {
                offers: [
                    MutationKind::PiercingLance,
                    MutationKind::SplitVolley,
                    MutationKind::DeathNova,
                ],
                menu: gotoo_pixel_engine::ui::MenuState::new(3),
            });
        }

        assert!(runtime.sync_fatal_hull_to_legacy_game_over());

        let v14 = &runtime
            .game
            .combat
            .game
            .ui
            .game
            .combat
            .combat
            .combat
            .combat;
        assert!(v14.progression.level_choice.is_none());
        assert!(v14.mutation_choice.is_none());
        assert!(runtime.game.base().game_over);
        assert_eq!(runtime.game.base().lives, 0);
    }
}
