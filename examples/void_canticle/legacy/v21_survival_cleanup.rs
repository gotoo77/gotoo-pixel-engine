const VC21_VITAL_SPARK_HULL_BONUS: f32 = 15.0;

struct VoidCanticleV21SurvivalCleanup {
    game: VoidCanticleV21Stabilized,
    base_hull_cap: f32,
}

impl VoidCanticleV21SurvivalCleanup {
    fn new() -> Self {
        let game = VoidCanticleV21Stabilized::new();
        let base_hull_cap = game.tuning.player_hull;
        Self {
            game,
            base_hull_cap,
        }
    }

    fn v14(&self) -> &VoidCanticleV14 {
        &self
            .game
            .game
            .game
            .combat
            .game
            .ui
            .game
            .combat
            .combat
            .combat
            .combat
    }

    fn vital_spark_stacks(&self) -> u32 {
        self.v14().progression.build.vital_spark
    }

    fn hull_cap_for_stacks(&self, stacks: u32) -> f32 {
        self.base_hull_cap + VC21_VITAL_SPARK_HULL_BONUS * stacks as f32
    }

    fn prepare_frame(&mut self) -> u32 {
        let stacks = self.vital_spark_stacks();
        let hull_cap = self.hull_cap_for_stacks(stacks);
        self.game.tuning.player_hull = hull_cap;

        if self.game.game.game.player_hull > 0.0 {
            self.game.game.game.base_mut().lives = 3;
        }
        stacks
    }

    fn reconcile_vital_spark(&mut self, stacks_before: u32) {
        let stacks_after = self.vital_spark_stacks();
        let hull_cap = self.hull_cap_for_stacks(stacks_after);
        self.game.tuning.player_hull = hull_cap;

        let gained_stacks = stacks_after.saturating_sub(stacks_before);
        if gained_stacks > 0 {
            let repair = VC21_VITAL_SPARK_HULL_BONUS * gained_stacks as f32;
            self.game.game.game.player_hull =
                (self.game.game.game.player_hull + repair).min(hull_cap);
        } else {
            self.game.game.game.player_hull = self.game.game.game.player_hull.min(hull_cap);
        }

        if self.game.game.game.player_hull > 0.0 {
            self.game.game.game.base_mut().lives = 3;
        }
    }

    fn render_level_choice_overrides(&self, framebuffer: &mut Framebuffer) {
        let Some(choice) = &self.v14().progression.level_choice else {
            return;
        };

        for (index, upgrade) in choice.offers.iter().copied().enumerate() {
            if upgrade != UpgradeKind::VitalSpark {
                continue;
            }
            let y = 98 + index as i32 * 47;
            framebuffer.fill_rect(24, y + 20, 136, 11, Pixel::rgb(7, 10, 19));
            framebuffer.draw_text(
                28,
                y + 23,
                &format!("MAX HULL +{}", VC21_VITAL_SPARK_HULL_BONUS as u32),
                WRECK_LIGHT,
            );
        }
    }
}

impl Game for VoidCanticleV21SurvivalCleanup {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let vital_spark_before = self.prepare_frame();
        let result = self.game.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        self.reconcile_vital_spark(vital_spark_before);
        self.render_level_choice_overrides(frame.framebuffer);
        GameResult::Continue
    }
}

pub fn run_v21_survival_cleanup_with_obs_mirror() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: format!("Void Canticle {VC21_STABLE_VERSION} - Gotoo Pixel Engine"),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width,
            window_height,
        },
        gotoo_pixel_engine::ObsMirrorGame::from_env(
            VoidCanticleV21SurvivalCleanup::new(),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v21_survival_cleanup_tests {
    use super::*;

    #[test]
    fn vital_spark_increases_hull_budget_instead_of_lives() {
        let game = VoidCanticleV21SurvivalCleanup::new();
        assert_eq!(
            game.hull_cap_for_stacks(2) - game.hull_cap_for_stacks(0),
            VC21_VITAL_SPARK_HULL_BONUS * 2.0
        );
    }

    #[test]
    fn vital_spark_bonus_is_meaningful_but_not_an_extra_health_bar() {
        assert!(VC21_VITAL_SPARK_HULL_BONUS > 0.0);
        assert!(VC21_VITAL_SPARK_HULL_BONUS < 50.0);
    }
}