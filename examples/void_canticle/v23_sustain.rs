const VC23_SUPPORT_LEVEL: u32 = 3;
const VC23_NANITE_DELAY: f32 = 4.0;
const VC23_NANITE_REPAIR_PER_SECOND: f32 = 0.75;
const VC23_CAPACITOR_DELAY: f32 = 2.5;
const VC23_CAPACITOR_REGEN_PER_SECOND: f32 = 8.0;
const VC23_SUSTAIN_FLASH_DURATION: f32 = 0.32;

const VC23_SUPPORT_UP: ActionId = ActionId::new("void_canticle.support.up");
const VC23_SUPPORT_DOWN: ActionId = ActionId::new("void_canticle.support.down");
const VC23_SUPPORT_CONFIRM: ActionId = ActionId::new("void_canticle.support.confirm");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Vc23SustainAugment {
    NaniteRepair,
    ShieldCapacitor,
}

const VC23_SUSTAIN_AUGMENTS: [Vc23SustainAugment; 2] = [
    Vc23SustainAugment::NaniteRepair,
    Vc23SustainAugment::ShieldCapacitor,
];

impl Vc23SustainAugment {
    fn name(self) -> &'static str {
        match self {
            Self::NaniteRepair => "NANITE REPAIR",
            Self::ShieldCapacitor => "SHIELD CAPACITOR",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::NaniteRepair => "HULL REPAIR AFTER 4S",
            Self::ShieldCapacitor => "SHIELD REGEN AFTER 2S",
        }
    }
}

struct VoidCanticleV23Sustain {
    game: VoidCanticleV23,
    augment: Option<Vc23SustainAugment>,
    support_offered: bool,
    choosing_support: bool,
    choice_input_armed: bool,
    menu: gotoo_pixel_engine::ui::MenuState,
    controls: ControlMap,
    quiet_timer: f32,
    sustain_flash_timer: f32,
}

impl VoidCanticleV23Sustain {
    fn new() -> Self {
        Self {
            game: VoidCanticleV23::new(),
            augment: None,
            support_offered: false,
            choosing_support: false,
            choice_input_armed: false,
            menu: gotoo_pixel_engine::ui::MenuState::new(VC23_SUSTAIN_AUGMENTS.len()),
            controls: gotoo_pixel_engine::ui::standard_menu_controls(
                VC23_SUPPORT_UP,
                VC23_SUPPORT_DOWN,
                VC23_SUPPORT_CONFIRM,
            ),
            quiet_timer: 0.0,
            sustain_flash_timer: 0.0,
        }
    }

    fn combat_model(&self) -> &VoidCanticleV21 {
        &self.game.game.game.game.game.game.game.game
    }

    fn combat_model_mut(&mut self) -> &mut VoidCanticleV21 {
        &mut self.game.game.game.game.game.game.game.game
    }

    fn survival_model(&self) -> &VoidCanticleV21SurvivalCleanup {
        &self.game.game.game.game.game
    }

    fn hull_cap(&self) -> f32 {
        let survival = self.survival_model();
        survival.hull_cap_for_stacks(survival.vital_spark_stacks())
    }

    fn shield_cap(&self) -> f32 {
        self.game.game.game.game.game.game.tuning.player_shield
    }

    fn progression_level(&self) -> u32 {
        self.game.v20().game.v14().progression.level
    }

    fn support_choice_can_start(&self) -> bool {
        !self.support_offered
            && !self.choosing_support
            && self.progression_level() >= VC23_SUPPORT_LEVEL
            && self.game.active_combat()
            && self.game.v20().game.v14().progression.level_choice.is_none()
            && self.game.v20().game.v14().mutation_choice.is_none()
    }

    fn maybe_start_support_choice(&mut self) {
        if self.support_choice_can_start() {
            self.support_offered = true;
            self.choosing_support = true;
            self.choice_input_armed = false;
            self.menu = gotoo_pixel_engine::ui::MenuState::new(VC23_SUSTAIN_AUGMENTS.len());
            self.quiet_timer = 0.0;
        }
    }

    fn update_support_choice(&mut self, frame: &mut Frame<'_>) {
        self.controls.update(frame.input);

        if !self.choice_input_armed {
            let any_held = [VC23_SUPPORT_UP, VC23_SUPPORT_DOWN, VC23_SUPPORT_CONFIRM]
                .into_iter()
                .any(|action| self.controls.action(action).held());
            if !any_held {
                self.choice_input_armed = true;
            }
            return;
        }

        if self.controls.action(VC23_SUPPORT_UP).pressed() {
            self.menu.select_previous();
        }
        if self.controls.action(VC23_SUPPORT_DOWN).pressed() {
            self.menu.select_next();
        }
        if self.controls.action(VC23_SUPPORT_CONFIRM).pressed()
            && let Some(index) = self.menu.selected()
            && let Some(augment) = VC23_SUSTAIN_AUGMENTS.get(index).copied()
        {
            self.augment = Some(augment);
            self.choosing_support = false;
            self.choice_input_armed = false;
            self.quiet_timer = 0.0;
            self.sustain_flash_timer = VC23_SUSTAIN_FLASH_DURATION;
        }
    }

    fn update_sustain(&mut self, dt: f32, hull_before: f32, shield_before: f32) {
        self.sustain_flash_timer = (self.sustain_flash_timer - dt).max(0.0);
        let hull_now = self.combat_model().player_hull;
        let shield_now = self.combat_model().player_shield;
        let damaged = hull_now + f32::EPSILON < hull_before
            || shield_now + f32::EPSILON < shield_before;

        if damaged {
            self.quiet_timer = 0.0;
            return;
        }
        if !self.game.active_combat() {
            return;
        }

        self.quiet_timer += dt;
        let Some(augment) = self.augment else {
            return;
        };

        match augment {
            Vc23SustainAugment::NaniteRepair if self.quiet_timer >= VC23_NANITE_DELAY => {
                let cap = self.hull_cap();
                let model = self.combat_model_mut();
                let before = model.player_hull;
                model.player_hull =
                    (model.player_hull + VC23_NANITE_REPAIR_PER_SECOND * dt).min(cap);
                if model.player_hull > before + f32::EPSILON {
                    self.sustain_flash_timer = VC23_SUSTAIN_FLASH_DURATION;
                }
            }
            Vc23SustainAugment::ShieldCapacitor
                if self.quiet_timer >= VC23_CAPACITOR_DELAY =>
            {
                let cap = self.shield_cap();
                let model = self.combat_model_mut();
                let before = model.player_shield;
                model.player_shield =
                    (model.player_shield + VC23_CAPACITOR_REGEN_PER_SECOND * dt).min(cap);
                if model.player_shield > before + f32::EPSILON {
                    self.sustain_flash_timer = VC23_SUSTAIN_FLASH_DURATION;
                }
            }
            _ => {}
        }
    }

    fn reset_for_new_run(&mut self) {
        self.augment = None;
        self.support_offered = false;
        self.choosing_support = false;
        self.choice_input_armed = false;
        self.menu = gotoo_pixel_engine::ui::MenuState::new(VC23_SUSTAIN_AUGMENTS.len());
        self.quiet_timer = 0.0;
        self.sustain_flash_timer = 0.0;
    }

    fn render_support_choice(&self, framebuffer: &mut Framebuffer) {
        framebuffer.fill_rect(8, 72, 164, 176, Pixel::rgb(7, 10, 19));
        framebuffer.draw_rect(8, 72, 164, 176, ART_CYAN);
        framebuffer.draw_text(43, 86, "SUPPORT AUGMENT", ART_CYAN_LIGHT);
        framebuffer.draw_text(54, 101, "CHOOSE MODULE", WRECK_LIGHT);

        for (index, augment) in VC23_SUSTAIN_AUGMENTS.iter().copied().enumerate() {
            let y = 126 + index as i32 * 52;
            gotoo_pixel_engine::ui::draw_menu_item(
                framebuffer,
                gotoo_pixel_engine::Rect {
                    x: 18,
                    y,
                    width: 144,
                    height: 18,
                },
                augment.name(),
                self.menu.selected() == Some(index),
                1,
                TEXT,
                ART_CYAN_LIGHT,
            );
            framebuffer.draw_text(28, y + 24, augment.description(), WRECK_LIGHT);
        }

        framebuffer.draw_text(39, 228, "SPACE SOUTH SELECT", TEXT);
    }

    fn render_sustain_hud(&self, framebuffer: &mut Framebuffer) {
        if !self.game.active_combat() || self.choosing_support {
            return;
        }
        let Some(augment) = self.augment else {
            return;
        };

        // Sustain communicates through the survival gauges themselves. The
        // thin line fills while waiting for a safe window; once active the
        // corresponding Hull/Shield gauge receives a bright outline.
        let (x, delay, color) = match augment {
            Vc23SustainAugment::NaniteRepair => (12, VC23_NANITE_DELAY, CANTICLE_COLOR),
            Vc23SustainAugment::ShieldCapacitor => (100, VC23_CAPACITOR_DELAY, ART_CYAN_LIGHT),
        };
        let progress = (self.quiet_timer / delay).clamp(0.0, 1.0);
        framebuffer.fill_rect(x, 37, 72, 2, WRECK_MID);
        framebuffer.fill_rect(x, 37, (72.0 * progress).round() as u32, 2, color);

        if self.quiet_timer >= delay || self.sustain_flash_timer > 0.0 {
            framebuffer.draw_rect(x - 2, 25, 76, 11, color);
        }
    }
}

impl Game for VoidCanticleV23Sustain {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let dt = frame.delta_time.as_secs_f32().min(0.05);

        if self.choosing_support {
            self.update_support_choice(frame);
            self.render_support_choice(frame.framebuffer);
            return GameResult::Continue;
        }

        let stage_time_before = self.game.base().stage_time;
        let hull_before = self.combat_model().player_hull;
        let shield_before = self.combat_model().player_shield;

        let result = self.game.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        if self.game.base().stage_time + 0.05 < stage_time_before {
            self.reset_for_new_run();
        }

        self.update_sustain(dt, hull_before, shield_before);
        self.maybe_start_support_choice();

        if self.choosing_support {
            self.render_support_choice(frame.framebuffer);
        } else {
            self.render_sustain_hud(frame.framebuffer);
        }
        GameResult::Continue
    }
}

pub fn run_v23_sustain_with_obs_mirror() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: format!("Void Canticle {VC23_VERSION} - Gotoo Pixel Engine"),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width,
            window_height,
        },
        gotoo_pixel_engine::ObsMirrorGame::from_env(
            VoidCanticleV23Sustain::new(),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v23_sustain_tests {
    use super::*;

    #[test]
    fn sustain_augments_are_mutually_exclusive_choices() {
        assert_eq!(VC23_SUSTAIN_AUGMENTS.len(), 2);
        assert_ne!(VC23_SUSTAIN_AUGMENTS[0], VC23_SUSTAIN_AUGMENTS[1]);
    }

    #[test]
    fn nanites_are_slow_and_require_a_safe_window() {
        assert!(VC23_NANITE_DELAY >= 3.0);
        assert!(VC23_NANITE_REPAIR_PER_SECOND > 0.0);
        assert!(VC23_NANITE_REPAIR_PER_SECOND <= 1.0);
    }

    #[test]
    fn capacitor_is_faster_than_hull_repair_but_not_instant() {
        assert!(VC23_CAPACITOR_DELAY > 1.0);
        assert!(VC23_CAPACITOR_REGEN_PER_SECOND > VC23_NANITE_REPAIR_PER_SECOND);
        assert!(VC23_CAPACITOR_REGEN_PER_SECOND < 20.0);
    }

    #[test]
    fn support_choice_waits_until_progression_is_established() {
        assert!(VC23_SUPPORT_LEVEL >= 3);
    }
}