const VC23_VERSION: &str = "VC2.3";
const VC23_PIXEL_VERSION: &str = "VC2-3";

const VC23_EMP: ActionId = ActionId::new("void_canticle.emp");
const VC23_EMP_COOLDOWN: f32 = 8.0;
const VC23_EMP_BASE_DAMAGE: u32 = 20;
const VC23_EMP_FLASH_DURATION: f32 = 0.52;
const VC23_EMP_WAVE_RADIUS: f32 = 340.0;
const VC23_EMP_SOUND: SoundId = SoundId::new("void_canticle.emp_cast");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Vc23DamageKind {
    Kinetic,
    Emp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Vc23DefenseLayer {
    Shield,
    Armor,
    Hull,
}

fn vc23_damage_multiplier(kind: Vc23DamageKind, layer: Vc23DefenseLayer) -> f32 {
    match (kind, layer) {
        (Vc23DamageKind::Kinetic, _) => 1.0,
        (Vc23DamageKind::Emp, Vc23DefenseLayer::Shield) => 3.0,
        (Vc23DamageKind::Emp, Vc23DefenseLayer::Armor | Vc23DefenseLayer::Hull) => 0.0,
    }
}

fn vc23_scaled_damage(amount: u32, kind: Vc23DamageKind, layer: Vc23DefenseLayer) -> u32 {
    ((amount as f32) * vc23_damage_multiplier(kind, layer)).round() as u32
}

struct VoidCanticleV23 {
    game: VoidCanticleV22Passives,
    controls: ControlMap,
    emp_cooldown: f32,
    emp_flash_timer: f32,
    emp_hit_flash_timer: f32,
}

impl VoidCanticleV23 {
    fn new() -> Self {
        let mut game = VoidCanticleV22Passives::new();
        game.game
            .base_mut()
            .sounds
            .insert_wav(
                VC23_EMP_SOUND,
                synthesize_chirp(1_760.0, 110.0, 0.240, 0.24),
            )
            .expect("VC2.3 EMP sound id should be unique");

        let mut controls = ControlMap::new();
        controls
            .bind_key(VC23_EMP, Key::C)
            .bind_gamepad(VC23_EMP, GamepadButton::West);

        Self {
            game,
            controls,
            emp_cooldown: 0.0,
            emp_flash_timer: 0.0,
            emp_hit_flash_timer: 0.0,
        }
    }

    fn base(&self) -> &VoidCanticleGame {
        self.game.game.base()
    }

    fn base_mut(&mut self) -> &mut VoidCanticleGame {
        self.game.game.base_mut()
    }

    fn v20(&self) -> &VoidCanticleV20 {
        &self.game.game.game.game.game.game.game.combat
    }

    fn v20_mut(&mut self) -> &mut VoidCanticleV20 {
        &mut self.game.game.game.game.game.game.game.combat
    }

    fn active_combat(&self) -> bool {
        self.game.gameplay_running()
            && !self.base().game_over
            && self.base().encounter_phase != EncounterPhase::Cleared
    }

    fn reset_emp(&mut self) {
        self.emp_cooldown = 0.0;
        self.emp_flash_timer = 0.0;
        self.emp_hit_flash_timer = 0.0;
    }

    fn update_emp_timers(&mut self, dt: f32) {
        if !self.active_combat() {
            return;
        }
        self.emp_cooldown = (self.emp_cooldown - dt).max(0.0);
        self.emp_flash_timer = (self.emp_flash_timer - dt).max(0.0);
        self.emp_hit_flash_timer = (self.emp_hit_flash_timer - dt).max(0.0);
    }

    fn trigger_emp(&mut self, frame: &mut Frame<'_>) {
        if !self.active_combat() || self.emp_cooldown > 0.0 {
            return;
        }

        let emp_shield_damage = vc23_scaled_damage(
            VC23_EMP_BASE_DAMAGE,
            Vc23DamageKind::Emp,
            Vc23DefenseLayer::Shield,
        );
        let mut hit_shield = false;

        {
            let v20 = self.v20_mut();
            v20.sync_enemy_armor();
            v20.prepare_boss_shield();

            let wraith_keys: Vec<SpecialDefenseKey> = v20
                .game
                .v12()
                .combat
                .specials
                .iter()
                .filter(|enemy| enemy.alive && enemy.kind == SpecialKind::BellWraith)
                .map(vc20_special_key)
                .collect();

            for key in wraith_keys {
                if let Some(shield) = v20.special_armor.get_mut(&key)
                    && *shield > 0
                {
                    *shield = shield.saturating_sub(emp_shield_damage);
                    hit_shield = true;
                }
            }

            if v20.boss_shield > 0 {
                let before = v20.boss_shield;
                v20.boss_shield = v20.boss_shield.saturating_sub(emp_shield_damage);
                hit_shield = true;
                v20.boss_shield_flash_timer = VC20_BOSS_SHIELD_FLASH_DURATION.max(0.18);
                if before > 0 && v20.boss_shield == 0 {
                    v20.boss_shield_break_timer = VC20_BOSS_SHIELD_BREAK_DURATION;
                }
            }
        }

        self.emp_cooldown = VC23_EMP_COOLDOWN;
        self.emp_flash_timer = VC23_EMP_FLASH_DURATION;
        if hit_shield {
            self.emp_hit_flash_timer = VC23_EMP_FLASH_DURATION;
        }
        let _ = self.base_mut().sounds.play(frame.audio, VC23_EMP_SOUND);
    }

    fn render_wraith_shields(&self, framebuffer: &mut Framebuffer) {
        if !self.active_combat() {
            return;
        }

        for enemy in &self.v20().game.v12().combat.specials {
            if !enemy.alive || enemy.kind != SpecialKind::BellWraith {
                continue;
            }
            let key = vc20_special_key(enemy);
            let shield = self
                .v20()
                .special_armor
                .get(&key)
                .copied()
                .unwrap_or_else(|| vc20_special_armor_max(SpecialKind::BellWraith));
            if shield == 0 {
                continue;
            }

            let x = enemy.x.round() as i32;
            let y = enemy.y.round() as i32;
            let pulse = ((self.base().animation_time * 8.0).sin().abs() * 2.0) as u32;
            framebuffer.draw_circle(x, y, 16 + pulse, VC20_ARMOR_LIGHT);
            framebuffer.draw_circle(x, y, 19 + pulse, VC20_ARMOR_BG);
        }
    }

    fn render_emp_wave(&self, framebuffer: &mut Framebuffer) {
        if self.emp_flash_timer <= 0.0 {
            return;
        }
        let progress =
            (1.0 - self.emp_flash_timer / VC23_EMP_FLASH_DURATION).clamp(0.0, 1.0);
        let radius = (10.0 + progress * VC23_EMP_WAVE_RADIUS).round() as u32;
        let x = self.base().player_x.round() as i32;
        let y = self.base().player_y.round() as i32;
        framebuffer.draw_circle(x, y, radius, ART_CYAN_LIGHT);
        if radius > 5 {
            framebuffer.draw_circle(x, y, radius - 4, ART_CYAN);
        }
    }

    fn render_emp_hud(&self, framebuffer: &mut Framebuffer) {
        if !self.active_combat() {
            return;
        }

        const CENTER_X: i32 = 164;
        const CENTER_Y: i32 = 250;
        const RADIUS: f32 = 9.0;
        const PIPS: u32 = 8;

        let ready_ratio = (1.0 - self.emp_cooldown / VC23_EMP_COOLDOWN).clamp(0.0, 1.0);
        let lit = (ready_ratio * PIPS as f32).floor() as u32;
        let gauge_color = if self.emp_hit_flash_timer > 0.0 {
            VC20_ARMOR_LIGHT
        } else if self.emp_cooldown <= 0.0 {
            CANTICLE_COLOR
        } else {
            ART_CYAN
        };

        framebuffer.fill_circle(CENTER_X, CENTER_Y, 6, BG);
        framebuffer.draw_circle(CENTER_X, CENTER_Y, 7, VC20_ARMOR_BG);
        for index in 0..PIPS {
            let angle = -std::f32::consts::FRAC_PI_2
                + index as f32 * std::f32::consts::TAU / PIPS as f32;
            let x = CENTER_X + (angle.cos() * RADIUS).round() as i32;
            let y = CENTER_Y + (angle.sin() * RADIUS).round() as i32;
            framebuffer.fill_rect(
                x - 1,
                y - 1,
                2,
                2,
                if index < lit { gauge_color } else { WRECK_MID },
            );
        }
        framebuffer.draw_text(
            CENTER_X - 2,
            CENTER_Y - 3,
            if self.emp_cooldown <= 0.0 { "R" } else { "E" },
            gauge_color,
        );
    }
}

impl Game for VoidCanticleV23 {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let dt = frame.delta_time.as_secs_f32().min(0.05);
        let stage_time_before = self.base().stage_time;

        self.controls.update(frame.input);
        self.update_emp_timers(dt);
        if self.controls.action(VC23_EMP).pressed() {
            self.trigger_emp(frame);
        }

        let result = self.game.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        if self.base().stage_time + 0.05 < stage_time_before {
            self.reset_emp();
        }

        self.render_wraith_shields(frame.framebuffer);
        self.render_emp_wave(frame.framebuffer);
        self.render_emp_hud(frame.framebuffer);
        GameResult::Continue
    }
}

pub fn run_v23_with_obs_mirror() -> Result<(), EngineError> {
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
            VoidCanticleV23::new(),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v23_tests {
    use super::*;

    #[test]
    fn emp_is_specialized_for_shields() {
        assert_eq!(
            vc23_damage_multiplier(Vc23DamageKind::Emp, Vc23DefenseLayer::Shield),
            3.0
        );
        assert_eq!(
            vc23_damage_multiplier(Vc23DamageKind::Emp, Vc23DefenseLayer::Armor),
            0.0
        );
        assert_eq!(
            vc23_damage_multiplier(Vc23DamageKind::Emp, Vc23DefenseLayer::Hull),
            0.0
        );
    }

    #[test]
    fn kinetic_remains_neutral_against_all_layers() {
        for layer in [
            Vc23DefenseLayer::Shield,
            Vc23DefenseLayer::Armor,
            Vc23DefenseLayer::Hull,
        ] {
            assert_eq!(vc23_damage_multiplier(Vc23DamageKind::Kinetic, layer), 1.0);
        }
    }

    #[test]
    fn one_emp_breaks_a_bell_wraith_shield_but_not_bellkeeper_shield() {
        let damage = vc23_scaled_damage(
            VC23_EMP_BASE_DAMAGE,
            Vc23DamageKind::Emp,
            Vc23DefenseLayer::Shield,
        );
        assert!(damage >= vc20_special_armor_max(SpecialKind::BellWraith));
        assert!(damage < VC20_BOSS_SHIELD_MAX);
    }

    #[test]
    fn emp_has_real_opportunity_cost() {
        assert!(VC23_EMP_COOLDOWN >= 5.0);
        assert!(VC23_EMP_COOLDOWN <= 12.0);
    }

    #[test]
    fn checked_in_manifest_names_emp_event() {
        let manifest = gotoo_pixel_engine::SfxManifest::parse(VC20_SFX_MANIFEST)
            .expect("VC2.3 SFX manifest should parse");
        assert_eq!(
            manifest.path("emp_cast").unwrap(),
            "assets/void_canticle/sfx/emp_cast.wav"
        );
    }

    #[test]
    fn framebuffer_version_is_pixel_safe() {
        assert_eq!(VC23_PIXEL_VERSION, "VC2-3");
        assert!(!VC23_PIXEL_VERSION.contains('.'));
    }
}
