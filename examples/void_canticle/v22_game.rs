const VC22_VERSION: &str = "VC2.2";
const VC22_PIXEL_VERSION: &str = "VC2-2";

const VC22_CHASSIS_UP: ActionId = ActionId::new("void_canticle.chassis.up");
const VC22_CHASSIS_DOWN: ActionId = ActionId::new("void_canticle.chassis.down");
const VC22_CHASSIS_CONFIRM: ActionId = ActionId::new("void_canticle.chassis.confirm");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExosuitChassis {
    Bulwark,
    Pilgrim,
    Wraith,
}

const VC22_CHASSIS: [ExosuitChassis; 3] = [
    ExosuitChassis::Bulwark,
    ExosuitChassis::Pilgrim,
    ExosuitChassis::Wraith,
];

fn vc22_run_restarted(stage_time_before: f32, stage_time_after: f32) -> bool {
    stage_time_after + 0.05 < stage_time_before
}

#[derive(Debug, Clone, Copy)]
struct ChassisProfile {
    hull_multiplier: f32,
    shield_multiplier: f32,
    move_multiplier: f32,
    focus_multiplier: f32,
}

impl ExosuitChassis {
    fn name(self) -> &'static str {
        match self {
            Self::Bulwark => "BULWARK",
            Self::Pilgrim => "PILGRIM",
            Self::Wraith => "WRAITH",
        }
    }

    fn role(self) -> &'static str {
        match self {
            Self::Bulwark => "HEAVY / ARMORED",
            Self::Pilgrim => "BALANCED FRAME",
            Self::Wraith => "LIGHT / AGILE",
        }
    }

    fn profile(self) -> ChassisProfile {
        match self {
            Self::Bulwark => ChassisProfile {
                hull_multiplier: 1.50,
                shield_multiplier: 1.60,
                move_multiplier: 0.72,
                focus_multiplier: 0.84,
            },
            Self::Pilgrim => ChassisProfile {
                hull_multiplier: 1.00,
                shield_multiplier: 1.00,
                move_multiplier: 1.00,
                focus_multiplier: 1.00,
            },
            Self::Wraith => ChassisProfile {
                hull_multiplier: 0.67,
                shield_multiplier: 0.60,
                move_multiplier: 1.28,
                focus_multiplier: 1.18,
            },
        }
    }
}

struct VoidCanticleV22 {
    game: VoidCanticleV21SurvivalCleanup,
    chassis: Option<ExosuitChassis>,
    menu: gotoo_pixel_engine::ui::MenuState,
    controls: ControlMap,
    baseline_hull: f32,
    baseline_shield: f32,
}

impl VoidCanticleV22 {
    fn new() -> Self {
        let game = VoidCanticleV21SurvivalCleanup::new();
        let baseline_hull = game.game.tuning.player_hull;
        let baseline_shield = game.game.tuning.player_shield;
        let controls = gotoo_pixel_engine::ui::standard_menu_controls(
            VC22_CHASSIS_UP,
            VC22_CHASSIS_DOWN,
            VC22_CHASSIS_CONFIRM,
        );

        Self {
            game,
            chassis: None,
            menu: gotoo_pixel_engine::ui::MenuState::new(VC22_CHASSIS.len()),
            controls,
            baseline_hull,
            baseline_shield,
        }
    }

    fn selected_chassis(&self) -> ExosuitChassis {
        self.menu
            .selected()
            .and_then(|index| VC22_CHASSIS.get(index).copied())
            .unwrap_or(ExosuitChassis::Bulwark)
    }

    fn chassis_limits(&self, chassis: ExosuitChassis) -> (f32, f32) {
        let profile = chassis.profile();
        (
            (self.baseline_hull * profile.hull_multiplier).max(1.0),
            (self.baseline_shield * profile.shield_multiplier).max(0.0),
        )
    }

    fn apply_chassis(&mut self, chassis: ExosuitChassis) {
        let (hull, shield) = self.chassis_limits(chassis);
        self.chassis = Some(chassis);
        self.game.base_hull_cap = hull;
        self.game.game.tuning.player_hull = hull;
        self.game.game.tuning.player_shield = shield;
        self.game.game.reset_runtime_models();
    }

    fn stage_time(&self) -> f32 {
        self.game.game.game.game.base().stage_time
    }

    fn reset_chassis_selection_for_new_run(&mut self) {
        self.chassis = None;
        self.menu = gotoo_pixel_engine::ui::MenuState::new(VC22_CHASSIS.len());
        self.game.base_hull_cap = self.baseline_hull;
        self.game.game.tuning.player_hull = self.baseline_hull;
        self.game.game.tuning.player_shield = self.baseline_shield;
    }

    fn reconcile_inner_restart(&mut self, stage_time_before: f32) -> bool {
        if !vc22_run_restarted(stage_time_before, self.stage_time()) {
            return false;
        }

        self.reset_chassis_selection_for_new_run();
        true
    }

    fn update_chassis_selection(&mut self, frame: &mut Frame<'_>) -> bool {
        self.controls.update(frame.input);
        if self.controls.action(VC22_CHASSIS_UP).pressed() {
            self.menu.select_previous();
        }
        if self.controls.action(VC22_CHASSIS_DOWN).pressed() {
            self.menu.select_next();
        }
        if self.controls.action(VC22_CHASSIS_CONFIRM).pressed() {
            let chassis = self.selected_chassis();
            self.apply_chassis(chassis);
            return true;
        }
        false
    }

    fn render_chassis_selection(&self, framebuffer: &mut Framebuffer) {
        framebuffer.clear(BG);
        render_grave_orbit_background(framebuffer, 0.0);
        gotoo_pixel_engine::ui::draw_panel(
            framebuffer,
            gotoo_pixel_engine::Rect {
                x: 8,
                y: 34,
                width: 164,
                height: 250,
            },
            Pixel::rgb(7, 9, 18),
            PILGRIM_VIOLET,
        );
        framebuffer.draw_text_scaled(39, 48, "EXOSUIT", 2, POWER_RELIC_LIGHT);
        framebuffer.draw_text(45, 70, "CHOOSE CHASSIS", WRECK_LIGHT);

        for (index, chassis) in VC22_CHASSIS.iter().copied().enumerate() {
            let y = 91 + index as i32 * 52;
            let selected = self.menu.selected() == Some(index);
            gotoo_pixel_engine::ui::draw_menu_item(
                framebuffer,
                gotoo_pixel_engine::Rect {
                    x: 18,
                    y,
                    width: 144,
                    height: 18,
                },
                chassis.name(),
                selected,
                1,
                TEXT,
                POWER_RELIC_LIGHT,
            );

            let (hull, shield) = self.chassis_limits(chassis);
            let profile = chassis.profile();
            framebuffer.draw_text(28, y + 22, chassis.role(), WRECK_LIGHT);
            framebuffer.draw_text(
                28,
                y + 33,
                &format!(
                    "H{} S{} MOVE {}",
                    hull.round() as u32,
                    shield.round() as u32,
                    (profile.move_multiplier * 100.0).round() as u32
                ),
                if selected { CANTICLE_COLOR } else { WRECK_LIGHT },
            );
        }

        let selected = self.selected_chassis();
        framebuffer.fill_rect(18, 241, 144, 15, Pixel::rgb(7, 9, 18));
        framebuffer.draw_text(
            22,
            244,
            &format!("PASSIVE {}", selected.passive_name()),
            CANTICLE_COLOR,
        );
        framebuffer.draw_text(39, 260, "SPACE SOUTH SELECT", TEXT);
        framebuffer.fill_rect(18, 270, 144, 11, Pixel::rgb(7, 9, 18));
        framebuffer.draw_text(22, 273, selected.passive_description(), WRECK_LIGHT);
    }

    fn gameplay_running(&self) -> bool {
        self.chassis.is_some() && self.game.game.gameplay_running()
    }

    fn player_position(&self) -> (f32, f32) {
        let base = self.game.game.game.game.base();
        (base.player_x, base.player_y)
    }

    fn rescale_player_movement(&mut self, before: (f32, f32), frame: &Frame<'_>) {
        if !self.gameplay_running() {
            return;
        }
        let Some(chassis) = self.chassis else {
            return;
        };

        let focused = {
            let base = self.game.game.game.game.base();
            base.controls.action(FOCUS).held() || frame.input.mouse_button(MouseButton::Right).held()
        };
        let profile = chassis.profile();
        let scale = if focused {
            profile.focus_multiplier
        } else {
            profile.move_multiplier
        };
        if (scale - 1.0).abs() <= f32::EPSILON {
            return;
        }

        let (after_x, after_y) = self.player_position();
        let base = self.game.game.game.game.base_mut();
        base.player_x = (before.0 + (after_x - before.0) * scale)
            .clamp(8.0, FRAMEBUFFER_WIDTH as f32 - 8.0);
        base.player_y = (before.1 + (after_y - before.1) * scale)
            .clamp(30.0, FRAMEBUFFER_HEIGHT as f32 - 16.0);
    }
}

impl Game for VoidCanticleV22 {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if self.chassis.is_none() {
            let selected = self.update_chassis_selection(frame);
            if self.chassis.is_none() || selected {
                self.render_chassis_selection(frame.framebuffer);
                return GameResult::Continue;
            }
        }

        let stage_time_before = self.stage_time();
        let before = self.player_position();
        let result = self.game.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        if self.reconcile_inner_restart(stage_time_before) {
            self.render_chassis_selection(frame.framebuffer);
            return GameResult::Continue;
        }

        self.rescale_player_movement(before, frame);
        GameResult::Continue
    }
}

pub fn run_v22_with_obs_mirror() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: format!("Void Canticle {VC22_VERSION} - Gotoo Pixel Engine"),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width,
            window_height,
        },
        gotoo_pixel_engine::ObsMirrorGame::from_env(
            VoidCanticleV22::new(),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v22_tests {
    use super::*;

    #[test]
    fn chassis_form_a_real_mobility_survival_tradeoff() {
        let bulwark = ExosuitChassis::Bulwark.profile();
        let pilgrim = ExosuitChassis::Pilgrim.profile();
        let wraith = ExosuitChassis::Wraith.profile();

        assert!(bulwark.hull_multiplier > pilgrim.hull_multiplier);
        assert!(pilgrim.hull_multiplier > wraith.hull_multiplier);
        assert!(bulwark.shield_multiplier > pilgrim.shield_multiplier);
        assert!(pilgrim.shield_multiplier > wraith.shield_multiplier);
        assert!(bulwark.move_multiplier < pilgrim.move_multiplier);
        assert!(pilgrim.move_multiplier < wraith.move_multiplier);
    }

    #[test]
    fn pilgrim_preserves_external_tuning_baseline() {
        let game = VoidCanticleV22::new();
        let (hull, shield) = game.chassis_limits(ExosuitChassis::Pilgrim);
        assert_eq!(hull, game.baseline_hull);
        assert_eq!(shield, game.baseline_shield);
    }

    #[test]
    fn inner_run_restart_returns_to_chassis_selection() {
        let mut game = VoidCanticleV22::new();
        game.apply_chassis(ExosuitChassis::Wraith);
        game.game.game.game.game.base_mut().stage_time = 12.0;
        let stage_time_before = game.stage_time();

        game.game.game.pause_ui_mut().game.reset_run();

        assert!(game.reconcile_inner_restart(stage_time_before));
        assert_eq!(game.chassis, None);
        assert_eq!(game.game.base_hull_cap, game.baseline_hull);
        assert_eq!(game.game.game.tuning.player_hull, game.baseline_hull);
        assert_eq!(game.game.game.tuning.player_shield, game.baseline_shield);
    }

    #[test]
    fn restart_detection_requires_a_real_timeline_drop() {
        assert!(vc22_run_restarted(12.0, 0.0));
        assert!(!vc22_run_restarted(12.0, 11.98));
        assert!(!vc22_run_restarted(0.0, 0.0));
    }

    #[test]
    fn framebuffer_version_avoids_unsupported_dot() {
        assert_eq!(VC22_PIXEL_VERSION, "VC2-2");
        assert!(!VC22_PIXEL_VERSION.contains('.'));
    }
}
