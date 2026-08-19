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

        // `lives` remains a temporary internal adapter used by the historical
        // collision layer. VC2.1 exposes exactly one life: Hull reaching zero
        // is death. Keep the adapter at its neutral sentinel while alive.
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

    fn render_survival_hud(&self, framebuffer: &mut Framebuffer) {
        if self.game.stage_clear_visible() {
            return;
        }

        let vc21 = &self.game.game.game;
        let hull_cap = self.hull_cap_for_stacks(self.vital_spark_stacks());
        let shield_cap = self.game.tuning.player_shield.max(1.0);
        let hull_segments = ((hull_cap / 10.0).ceil() as u32).clamp(4, 12);
        let shield_segments = ((shield_cap / 5.0).ceil() as u32).clamp(3, 10);

        // VC2.3 presentation rule: survival is read graphically. Remove the
        // historical lives/build text and replace it with compact segmented
        // bars inspired by MOBA health/mana readability.
        framebuffer.fill_rect(0, 24, FRAMEBUFFER_WIDTH, 16, BG);
        framebuffer.draw_text(4, 27, "H", VC20_HULL);
        vc21_render_segmented_bar(
            framebuffer,
            12,
            27,
            72,
            7,
            vc21.player_hull,
            hull_cap,
            hull_segments,
            if vc21.player_hull_flash_timer > 0.0 {
                DANGER
            } else {
                VC20_HULL
            },
            CORE_BG,
        );

        framebuffer.draw_text(92, 27, "S", VC20_ARMOR_LIGHT);
        vc21_render_segmented_bar(
            framebuffer,
            100,
            27,
            72,
            7,
            vc21.player_shield,
            shield_cap,
            shield_segments,
            if vc21.player_shield_flash_timer > 0.0 {
                VC20_ARMOR_LIGHT
            } else {
                VC20_ARMOR
            },
            VC20_ARMOR_BG,
        );
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

fn vc21_render_segmented_bar(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    value: f32,
    max_value: f32,
    segments: u32,
    fill: Pixel,
    background: Pixel,
) {
    let max_value = max_value.max(1.0);
    let ratio = (value / max_value).clamp(0.0, 1.0);
    framebuffer.fill_rect(x, y, width, height, background);
    let filled = (width as f32 * ratio).round() as u32;
    if filled > 0 {
        framebuffer.fill_rect(x, y, filled.min(width), height, fill);
    }

    let segments = segments.max(1);
    for segment in 1..segments {
        let separator_x = x + (width.saturating_mul(segment) / segments) as i32;
        framebuffer.fill_rect(separator_x, y, 1, height, BG);
    }
    framebuffer.draw_rect(x - 1, y - 1, width + 2, height + 2, WRECK_LIGHT);
}

impl Game for VoidCanticleV21SurvivalCleanup {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let vital_spark_before = self.prepare_frame();
        let result = self.game.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        self.reconcile_vital_spark(vital_spark_before);
        self.render_survival_hud(frame.framebuffer);
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

    #[test]
    fn segmented_bar_count_scales_with_survival_budget() {
        let low = ((40.0_f32 / 10.0).ceil() as u32).clamp(4, 12);
        let high = ((90.0_f32 / 10.0).ceil() as u32).clamp(4, 12);
        assert!(low < high);
    }
}