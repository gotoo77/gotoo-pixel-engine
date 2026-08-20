const VC22_BULWARK_IMPACT_MULTIPLIER: f32 = 0.80;
const VC22_PILGRIM_CORE_BONUS_NUMERATOR: u32 = 1;
const VC22_PILGRIM_CORE_BONUS_DENOMINATOR: u32 = 4;
const VC22_WRAITH_REDLINE_COOLDOWN_BONUS: f32 = 0.30;
const VC22_PASSIVE_FLASH_DURATION: f32 = 0.28;

impl ExosuitChassis {
    fn passive_name(self) -> &'static str {
        match self {
            Self::Bulwark => "IMPACT DAMPERS",
            Self::Pilgrim => "RESONANT CORE",
            Self::Wraith => "REDLINE WEAPONS",
        }
    }

    fn passive_description(self) -> &'static str {
        match self {
            Self::Bulwark => "IMPACT DAMAGE -20 PCT",
            Self::Pilgrim => "CORE GAINS +25 PCT",
            Self::Wraith => "SHIELD DOWN FIRE +25 PCT",
        }
    }
}

struct VoidCanticleV22Passives {
    game: VoidCanticleV22Movement,
    baseline_impact_damage: f32,
    passive_flash_timer: f32,
}

impl VoidCanticleV22Passives {
    fn new() -> Self {
        let game = VoidCanticleV22Movement::new();
        let baseline_impact_damage = game.game.game.game.tuning.impact_damage;
        Self {
            game,
            baseline_impact_damage,
            passive_flash_timer: 0.0,
        }
    }

    fn chassis(&self) -> Option<ExosuitChassis> {
        self.game.game.chassis
    }

    fn gameplay_running(&self) -> bool {
        self.game.game.gameplay_running()
    }

    fn player_hull(&self) -> f32 {
        self.game.game.game.game.game.game.player_hull
    }

    fn player_shield(&self) -> f32 {
        self.game.game.game.game.game.game.player_shield
    }

    fn apply_pre_update_passive(&mut self, dt: f32) {
        self.passive_flash_timer = (self.passive_flash_timer - dt).max(0.0);

        let Some(chassis) = self.chassis() else {
            return;
        };

        let impact_damage = match chassis {
            ExosuitChassis::Bulwark => vc22_bulwark_impact_damage(self.baseline_impact_damage),
            ExosuitChassis::Pilgrim | ExosuitChassis::Wraith => self.baseline_impact_damage,
        };
        self.game.game.game.game.tuning.impact_damage = impact_damage;

        if vc22_wraith_redline_active(chassis, self.player_shield(), self.gameplay_running()) {
            let base = self.game.base_mut();
            base.fire_cooldown =
                (base.fire_cooldown - dt * VC22_WRAITH_REDLINE_COOLDOWN_BONUS).max(0.0);
        }
    }

    fn apply_post_update_passive(
        &mut self,
        chassis: Option<ExosuitChassis>,
        hull_before: f32,
        core_before: u32,
    ) {
        let Some(chassis) = chassis else {
            return;
        };

        match chassis {
            ExosuitChassis::Bulwark => {
                if self.player_hull() + f32::EPSILON < hull_before {
                    self.passive_flash_timer = VC22_PASSIVE_FLASH_DURATION;
                }
            }
            ExosuitChassis::Pilgrim => {
                let core_after = self.game.base().core_charge;
                if core_after > core_before {
                    let gained = core_after - core_before;
                    let bonus = vc22_pilgrim_core_bonus(gained);
                    if bonus > 0 {
                        let base = self.game.base_mut();
                        base.core_charge = base.core_charge.saturating_add(bonus).min(CORE_MAX);
                        self.passive_flash_timer = VC22_PASSIVE_FLASH_DURATION;
                    }
                }
            }
            ExosuitChassis::Wraith => {}
        }
    }

    fn render_passive_fx(&self, framebuffer: &mut Framebuffer) {
        if !self.gameplay_running() {
            return;
        }
        let Some(chassis) = self.chassis() else {
            return;
        };

        let base = self.game.base();
        let x = base.player_x.round() as i32;
        let y = base.player_y.round() as i32;
        match chassis {
            ExosuitChassis::Bulwark if self.passive_flash_timer > 0.0 => {
                framebuffer.draw_circle(x, y, 13, ART_GOLD);
                framebuffer.draw_circle(x, y, 15, BELL_METAL);
            }
            ExosuitChassis::Pilgrim if self.passive_flash_timer > 0.0 => {
                framebuffer.draw_circle(x, y, 12, CINDER);
                framebuffer.draw_circle(x, y, 15, CANTICLE_COLOR);
            }
            ExosuitChassis::Wraith
                if vc22_wraith_redline_active(chassis, self.player_shield(), true) =>
            {
                framebuffer.draw_line(x - 5, y + 9, x - 7, y + 19, ART_CYAN_LIGHT);
                framebuffer.draw_line(x + 5, y + 9, x + 7, y + 19, ART_CYAN_LIGHT);
                framebuffer.draw_circle(x, y, 11, ART_CYAN);
            }
            _ => {}
        }
    }
}

impl Game for VoidCanticleV22Passives {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let dt = frame.delta_time.as_secs_f32().min(0.05);
        let chassis_before = self.chassis();
        let hull_before = self.player_hull();
        let core_before = self.game.base().core_charge;

        self.apply_pre_update_passive(dt);
        let result = self.game.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        self.apply_post_update_passive(chassis_before, hull_before, core_before);
        self.render_passive_fx(frame.framebuffer);
        GameResult::Continue
    }
}

fn vc22_bulwark_impact_damage(baseline: f32) -> f32 {
    (baseline * VC22_BULWARK_IMPACT_MULTIPLIER).max(VC21_LEGACY_IMPACT_DAMAGE)
}

fn vc22_pilgrim_core_bonus(gained: u32) -> u32 {
    gained
        .saturating_mul(VC22_PILGRIM_CORE_BONUS_NUMERATOR)
        .saturating_add(VC22_PILGRIM_CORE_BONUS_DENOMINATOR - 1)
        / VC22_PILGRIM_CORE_BONUS_DENOMINATOR
}

fn vc22_wraith_redline_active(
    chassis: ExosuitChassis,
    player_shield: f32,
    gameplay_running: bool,
) -> bool {
    chassis == ExosuitChassis::Wraith && gameplay_running && player_shield <= 0.0
}

pub fn run_v22_passives_with_obs_mirror() -> Result<(), EngineError> {
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
            VoidCanticleV22Passives::new(),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v22_passive_tests {
    use super::*;

    #[test]
    fn chassis_passives_are_distinct_and_pixel_safe() {
        let names = VC22_CHASSIS.map(ExosuitChassis::passive_name);
        assert_ne!(names[0], names[1]);
        assert_ne!(names[0], names[2]);
        assert_ne!(names[1], names[2]);
        for chassis in VC22_CHASSIS {
            assert!(!chassis.passive_description().contains('.'));
            assert!(!chassis.passive_description().contains('%'));
        }
    }

    #[test]
    fn bulwark_dampers_reduce_tuned_impact_damage() {
        let baseline = 35.0;
        let damped = vc22_bulwark_impact_damage(baseline);
        assert!(damped < baseline);
        assert!(damped >= VC21_LEGACY_IMPACT_DAMAGE);
    }

    #[test]
    fn pilgrim_resonance_rounds_small_core_gains_up() {
        assert_eq!(vc22_pilgrim_core_bonus(1), 1);
        assert_eq!(vc22_pilgrim_core_bonus(4), 1);
        assert_eq!(vc22_pilgrim_core_bonus(20), 5);
    }

    #[test]
    fn wraith_redline_requires_depleted_shield() {
        assert!(vc22_wraith_redline_active(
            ExosuitChassis::Wraith,
            0.0,
            true
        ));
        assert!(!vc22_wraith_redline_active(
            ExosuitChassis::Wraith,
            1.0,
            true
        ));
        assert!(!vc22_wraith_redline_active(
            ExosuitChassis::Pilgrim,
            0.0,
            true
        ));
    }
}
