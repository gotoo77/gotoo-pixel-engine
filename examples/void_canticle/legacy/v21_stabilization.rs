const VC21_STABLE_VERSION: &str = "VC2.1";
const VC21_PIXEL_VERSION: &str = "VC2-1";
const VC21_SHOT_ESCAPE_Y: f32 = 18.0;
const VC21_SHOT_DESPAWN_Y: f32 = -12.0;

const VC21_STAGE_RESTART: ActionId = ActionId::new("void_canticle.stage.restart");
const VC21_STAGE_MENU: ActionId = ActionId::new("void_canticle.stage.menu");

#[derive(Debug, Clone, Copy)]
struct Vc21EscapedShot {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    damage: u32,
    radius: u32,
}

impl From<PowerShot> for Vc21EscapedShot {
    fn from(shot: PowerShot) -> Self {
        Self {
            x: shot.x,
            y: shot.y,
            vx: shot.vx,
            vy: shot.vy,
            damage: shot.damage,
            radius: shot.radius,
        }
    }
}

struct VoidCanticleV21Stabilized {
    game: VoidCanticleV21Runtime,
    escaped_shots: Vec<Vc21EscapedShot>,
    stage_clear_seen: bool,
    stage_controls: ControlMap,
    tuning: Vc21Tuning,
}

impl VoidCanticleV21Stabilized {
    fn new() -> Self {
        let mut stage_controls = ControlMap::new();
        stage_controls
            .bind_key(VC21_STAGE_RESTART, Key::Space)
            .bind_gamepad(VC21_STAGE_RESTART, GamepadButton::South)
            .bind_key(VC21_STAGE_MENU, Key::Escape)
            .bind_gamepad(VC21_STAGE_MENU, GamepadButton::Start);

        let mut game = Self {
            game: VoidCanticleV21Runtime::new(),
            escaped_shots: Vec::new(),
            stage_clear_seen: false,
            stage_controls,
            tuning: Vc21Tuning::load(),
        };
        game.apply_tuning_to_fresh_run();
        game
    }

    fn power_shots(&self) -> &[PowerShot] {
        &self
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
            .progression
            .combat
            .combat
            .ui
            .inner
            .inner
            .power_shots
    }

    fn power_shots_mut(&mut self) -> &mut Vec<PowerShot> {
        &mut self
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
            .progression
            .combat
            .combat
            .ui
            .inner
            .inner
            .power_shots
    }

    fn pause_ui(&self) -> &VoidCanticlePauseV17 {
        &self.game.game.combat.game.ui
    }

    fn pause_ui_mut(&mut self) -> &mut VoidCanticlePauseV17 {
        &mut self.game.game.combat.game.ui
    }

    fn gameplay_running(&self) -> bool {
        self.game.game.combat.game.art_can_overlay_game()
    }

    fn stage_clear_visible(&self) -> bool {
        self.game.game.base().encounter_phase == EncounterPhase::Cleared
            && matches!(&self.pause_ui().state, VcPauseState::Running)
    }

    fn apply_tuning_to_fresh_run(&mut self) {
        self.game.game.player_hull = self.tuning.player_hull;
        self.game.game.player_shield = self.tuning.player_shield;
        if !self.tuning.shield_regen {
            self.game.game.shield_regen_delay = f32::MAX;
        }
    }

    fn reset_v20_defenses(&mut self) {
        let combat = &mut self.game.game.combat;
        combat.carrion_armor.clear();
        combat.special_armor.clear();
        combat.threat_armor.clear();
        combat.boss_shield = 0;
        combat.boss_defense_armed = false;
        combat.boss_shield_flash_timer = 0.0;
        combat.boss_shield_break_timer = 0.0;
    }

    fn reset_runtime_models(&mut self) {
        self.reset_v20_defenses();
        self.game.game.reset_combat_model();
        self.apply_tuning_to_fresh_run();
        self.escaped_shots.clear();
    }

    fn restart_run_from_stage_clear(&mut self) {
        self.pause_ui_mut().game.reset_run();
        self.pause_ui_mut().state = VcPauseState::Running;
        self.reset_runtime_models();
        self.stage_clear_seen = false;
    }

    fn open_stage_clear_menu(&mut self, framebuffer: &mut Framebuffer) {
        let ui = self.pause_ui_mut();
        ui.state = VcPauseState::Menu;
        ui.menu = gotoo_pixel_engine::ui::MenuState::new(5);
        ui.menu.select_next();
        ui.render_menu(framebuffer);
    }

    fn collect_shots_crossing_hud_cutoff(&mut self, dt: f32) {
        if !self.gameplay_running() {
            return;
        }

        let crossing: Vec<Vc21EscapedShot> = self
            .power_shots()
            .iter()
            .filter(|shot| {
                shot.alive
                    && shot.y > VC21_SHOT_ESCAPE_Y
                    && shot.y + shot.vy * dt <= VC21_SHOT_ESCAPE_Y
            })
            .copied()
            .map(Vc21EscapedShot::from)
            .collect();
        self.escaped_shots.extend(crossing);
    }

    fn update_escaped_shots(&mut self, dt: f32) {
        if !self.gameplay_running() {
            return;
        }
        for shot in &mut self.escaped_shots {
            shot.x += shot.vx * dt;
            shot.y += shot.vy * dt;
        }
        self.escaped_shots.retain(|shot| {
            shot.y > VC21_SHOT_DESPAWN_Y
                && shot.x > -12.0
                && shot.x < FRAMEBUFFER_WIDTH as f32 + 12.0
        });
    }

    fn render_escaped_shots(&self, framebuffer: &mut Framebuffer) {
        if !self.gameplay_running() {
            return;
        }
        let power_level = self.game.game.power_level();
        for shot in &self.escaped_shots {
            render_power_shot(
                framebuffer,
                PowerShot {
                    x: shot.x,
                    y: shot.y,
                    vx: shot.vx,
                    vy: shot.vy,
                    damage: shot.damage,
                    radius: shot.radius,
                    alive: true,
                },
                power_level,
            );
        }
    }

    fn apply_damage_tuning(
        &mut self,
        shield_before: f32,
        hull_before: f32,
        frame: &mut Frame<'_>,
    ) {
        let health_before = shield_before + hull_before;
        let health_after = self.game.game.player_shield + self.game.game.player_hull;
        let damage_taken = (health_before - health_after).max(0.0);
        let legacy_hit = damage_taken > 0.0
            && self.game.game.base().invulnerability >= PLAYER_INVULNERABILITY * 0.90;

        if legacy_hit && self.tuning.impact_damage > damage_taken {
            let extra_damage = self.tuning.impact_damage - damage_taken;
            self.game.game.apply_player_damage(extra_damage, false);
            self.game.game.process_combat_events(frame);
        }

        if damage_taken > 0.0 {
            self.game.game.base_mut().invulnerability = self.tuning.post_hit_invulnerability;
        }

        if !self.tuning.shield_regen {
            self.game.game.player_shield = self.game.game.player_shield.min(shield_before);
            self.game.game.shield_regen_delay = f32::MAX;
        }
        self.game.game.player_shield = self
            .game
            .game
            .player_shield
            .min(self.tuning.player_shield);
        self.game.game.player_hull = self.game.game.player_hull.min(self.tuning.player_hull);
        self.game.sync_fatal_hull_to_legacy_game_over();
    }

    fn handle_stage_clear_transition(&mut self) {
        let cleared = self.game.game.base().encounter_phase == EncounterPhase::Cleared;
        if !cleared {
            if self.stage_clear_seen {
                self.reset_runtime_models();
            }
            self.stage_clear_seen = false;
            return;
        }
        if self.stage_clear_seen {
            return;
        }

        self.stage_clear_seen = true;
        self.escaped_shots.clear();
        self.game.game.base_mut().player_bullets.clear();
        self.power_shots_mut().clear();
        self.game.game.base_mut().enemy_bullets.clear();
    }

    fn render_stage_clear(&self, framebuffer: &mut Framebuffer) {
        if !self.stage_clear_visible() {
            return;
        }

        framebuffer.clear(BG);
        render_grave_orbit_background(framebuffer, self.game.game.base().scroll);
        framebuffer.fill_rect(12, 55, 156, 206, Pixel::rgb(9, 8, 15));
        framebuffer.draw_rect(12, 55, 156, 206, CANTICLE_COLOR);
        framebuffer.draw_rect(16, 59, 148, 198, ART_GOLD);

        framebuffer.draw_text_scaled(30, 73, "STAGE CLEAR", 2, CANTICLE_COLOR);
        framebuffer.draw_text(54, 98, "GRAVE ORBIT", TEXT);
        framebuffer.draw_text(60, 116, "PATH OPENS", ART_GOLD);

        framebuffer.draw_text(32, 145, "SCORE", WRECK_LIGHT);
        framebuffer.draw_text(
            95,
            145,
            &format!("{}", self.game.game.base().score),
            PILGRIM_CORE,
        );
        framebuffer.draw_text(32, 162, "LEVEL", WRECK_LIGHT);
        framebuffer.draw_text(
            95,
            162,
            &format!("{}", self.game.game.combat.game.v14().progression.level),
            XP_ORB_CORE,
        );
        framebuffer.draw_text(32, 179, "ECHOES", WRECK_LIGHT);
        framebuffer.draw_text(
            95,
            179,
            &format!("{}", self.game.game.combat.game.v14().progression.xp),
            XP_ORB_CORE,
        );
        framebuffer.draw_text(32, 196, "HULL", WRECK_LIGHT);
        framebuffer.draw_text(
            95,
            196,
            &format!("{}", self.game.game.player_hull.round() as u32),
            VC20_HULL,
        );
        framebuffer.draw_text(32, 213, "SHIELD", WRECK_LIGHT);
        framebuffer.draw_text(
            95,
            213,
            &format!("{}", self.game.game.player_shield.round() as u32),
            VC20_ARMOR_LIGHT,
        );

        framebuffer.draw_text(35, 235, "SPACE RESTART", CANTICLE_COLOR);
        framebuffer.draw_text(49, 249, "ESC MENU", TEXT);
    }
}

impl Game for VoidCanticleV21Stabilized {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let dt = frame.delta_time.as_secs_f32().min(0.05);
        self.stage_controls.update(frame.input);

        if self.stage_clear_visible() {
            if self.stage_controls.action(VC21_STAGE_RESTART).pressed() {
                self.restart_run_from_stage_clear();
            } else if self.stage_controls.action(VC21_STAGE_MENU).pressed() {
                self.open_stage_clear_menu(frame.framebuffer);
                return GameResult::Continue;
            }
        }

        self.collect_shots_crossing_hud_cutoff(dt);
        let shield_before = self.game.game.player_shield;
        let hull_before = self.game.game.player_hull;

        let result = self.game.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        self.apply_damage_tuning(shield_before, hull_before, frame);
        self.update_escaped_shots(dt);
        self.handle_stage_clear_transition();
        self.render_escaped_shots(frame.framebuffer);
        self.render_stage_clear(frame.framebuffer);

        GameResult::Continue
    }
}

pub fn run_v21_stabilized_with_obs_mirror() -> Result<(), EngineError> {
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
            VoidCanticleV21Stabilized::new(),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v21_stabilization_tests {
    use super::*;

    #[test]
    fn framebuffer_version_uses_supported_punctuation() {
        assert_eq!(VC21_PIXEL_VERSION, "VC2-1");
        assert!(!VC21_PIXEL_VERSION.contains('.'));
    }

    #[test]
    fn escaped_player_shots_have_real_screen_margin() {
        assert!(VC21_SHOT_DESPAWN_Y < 0.0);
        assert!(VC21_SHOT_ESCAPE_Y > 0.0);
    }

    #[test]
    fn default_tuning_removes_post_hit_invulnerability() {
        let tuning = Vc21Tuning::default();
        assert_eq!(tuning.post_hit_invulnerability, 0.0);
        assert!(tuning.impact_damage >= VC21_LEGACY_IMPACT_DAMAGE);
    }
}