const VC10_VERSION: &str = "VC1.0";
const VC10_WAVE_TOAST_DURATION: f32 = 1.45;

const VC_PAUSE_TOGGLE: ActionId = ActionId::new("void_canticle.pause.toggle");
const VC_PAUSE_UP: ActionId = ActionId::new("void_canticle.pause.up");
const VC_PAUSE_DOWN: ActionId = ActionId::new("void_canticle.pause.down");
const VC_PAUSE_CONFIRM: ActionId = ActionId::new("void_canticle.pause.confirm");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VcPauseState {
    Running,
    Menu,
    Controls,
    BuildInfo,
    ResumeGate,
}

struct VoidCanticleV10 {
    inner: VoidCanticleV09,
    announced_wave: Option<usize>,
    wave_toast_timer: f32,
}

impl VoidCanticleV10 {
    fn new() -> Self {
        Self {
            inner: VoidCanticleV09::new(),
            announced_wave: None,
            wave_toast_timer: 0.0,
        }
    }

    fn reset_run(&mut self) {
        self.inner.reset_run();
        self.announced_wave = None;
        self.wave_toast_timer = 0.0;
    }

    fn update_wave_toast(&mut self, dt: f32) {
        self.wave_toast_timer = (self.wave_toast_timer - dt).max(0.0);

        if self.inner.inner.base.encounter_phase != EncounterPhase::Waves
            || self.inner.wave_index >= V09_WAVES.len()
            || self.inner.intermission > 0.0
        {
            return;
        }

        if self.announced_wave != Some(self.inner.wave_index) {
            self.announced_wave = Some(self.inner.wave_index);
            self.wave_toast_timer = VC10_WAVE_TOAST_DURATION;
        }
    }

    fn render(&self, framebuffer: &mut Framebuffer, focused: bool) {
        let game = &self.inner.inner;
        let base = &game.base;

        framebuffer.clear(if base.canticle_timer > 0.46 {
            BG_CANTICLE
        } else {
            BG
        });
        render_grave_orbit_background(framebuffer, base.scroll);

        for cinder in &base.cinders {
            let x = cinder.x.round() as i32;
            let y = cinder.y.round() as i32;
            framebuffer.fill_circle(x, y, 2, CINDER);
            framebuffer.draw(x, y - 4, CANTICLE_COLOR);
        }

        for relic in &game.relics {
            render_relic(framebuffer, *relic);
        }

        for enemy in &base.enemies {
            base.visuals.render_carrion(framebuffer, *enemy);
        }

        if let Some(boss) = base.boss {
            if base.encounter_phase != EncounterPhase::Cleared {
                base.visuals.render_bellkeeper(framebuffer, boss);
            }
        }

        for shot in &game.power_shots {
            render_power_shot(framebuffer, *shot, game.power_level);
        }

        for bullet in &base.enemy_bullets {
            let color = if bullet.alternate {
                ENEMY_SHOT_ALT
            } else {
                ENEMY_SHOT
            };
            framebuffer.fill_circle(
                bullet.x.round() as i32,
                bullet.y.round() as i32,
                2,
                color,
            );
        }

        for burst in &base.bursts {
            render_burst(framebuffer, *burst);
        }

        game.pilgrim_visuals.render(
            framebuffer,
            base.player_x.round() as i32,
            base.player_y.round() as i32,
            focused,
            base.invulnerability,
            base.animation_time,
        );

        if base.canticle_timer > 0.0 {
            render_canticle(
                framebuffer,
                base.player_x.round() as i32,
                base.player_y.round() as i32,
                base.canticle_timer,
            );
        }

        self.render_minimal_hud(framebuffer);
        self.render_side_notifications(framebuffer);

        if base.game_over {
            framebuffer.fill_rect(4, 132, 92, 42, Pixel::rgb(10, 8, 16));
            framebuffer.draw_rect(4, 132, 92, 42, DANGER);
            framebuffer.draw_text(12, 141, "PILGRIM FALLEN", DANGER);
            framebuffer.draw_text(12, 158, "SPACE RETRY", TEXT);
        }
    }

    fn render_minimal_hud(&self, framebuffer: &mut Framebuffer) {
        let game = &self.inner.inner;
        let base = &game.base;

        framebuffer.draw_text(4, 4, &format!("L {}", base.lives), TEXT);
        let score = format!("{}", base.score);
        let (score_width, _) = Framebuffer::text_size(&score, 1);
        let score_x = FRAMEBUFFER_WIDTH as i32 - score_width as i32 - 4;
        framebuffer.draw_text(score_x, 4, &score, PILGRIM_CORE);

        if let Some(boss) = base.boss {
            if base.encounter_phase == EncounterPhase::BossFight {
                framebuffer.fill_rect(24, 15, 132, 5, CORE_BG);
                let width = 132_u32.saturating_mul(boss.hp) / BELLKEEPER_MAX_HP;
                framebuffer.fill_rect(24, 15, width, 5, DANGER);
            }
        }

        framebuffer.draw_text(4, 302, "CORE", TEXT);
        framebuffer.fill_rect(34, 303, 76, 5, CORE_BG);
        let core_color = if base.core_charge >= CORE_MAX {
            CANTICLE_COLOR
        } else {
            CINDER
        };
        let charge_width = 76_u32.saturating_mul(base.core_charge) / CORE_MAX;
        framebuffer.fill_rect(34, 303, charge_width, 5, core_color);

        for index in 0..MAX_POWER_LEVEL {
            let color = if index < game.power_level {
                POWER_RELIC
            } else {
                CORE_BG
            };
            framebuffer.fill_rect(143 + i32::from(index) * 7, 303, 5, 5, color);
        }
    }

    fn render_side_notifications(&self, framebuffer: &mut Framebuffer) {
        let base = &self.inner.inner.base;

        if self.wave_toast_timer > 0.0 && self.inner.wave_index < V09_WAVES.len() {
            let wave = V09_WAVES[self.inner.wave_index];
            framebuffer.fill_rect(102, 34, 74, 31, Pixel::rgb(9, 8, 15));
            framebuffer.draw_rect(102, 34, 74, 31, WRECK_LIGHT);
            framebuffer.draw_text(
                108,
                40,
                &format!("W {}/15", self.inner.wave_index + 1),
                POWER_RELIC_LIGHT,
            );
            framebuffer.draw_text(108, 52, wave.name, WRECK_LIGHT);
        }

        if self.inner.wipe_banner_timer > 0.0 {
            framebuffer.fill_rect(105, 76, 71, 31, Pixel::rgb(9, 8, 15));
            framebuffer.draw_rect(105, 76, 71, 31, POWER_RELIC_LIGHT);
            framebuffer.draw_text(113, 82, "FULL WIPE", POWER_RELIC_LIGHT);
            framebuffer.draw_text(
                113,
                94,
                &format!("CHAIN {}", self.inner.wipe_banner_chain),
                POWER_RELIC,
            );
        }

        match base.encounter_phase {
            EncounterPhase::BossIntro => {
                framebuffer.fill_rect(4, 90, 92, 45, Pixel::rgb(9, 8, 15));
                framebuffer.draw_rect(4, 90, 92, 45, BELL_LIGHT);
                framebuffer.draw_text(12, 98, "WARNING", DANGER);
                framebuffer.draw_text(12, 111, "BELLKEEPER", BELL_LIGHT);
                framebuffer.draw_text(12, 123, "TOLLS", TEXT);
            }
            EncounterPhase::Cleared => {
                framebuffer.fill_rect(4, 128, 94, 40, Pixel::rgb(9, 8, 15));
                framebuffer.draw_rect(4, 128, 94, 40, CANTICLE_COLOR);
                framebuffer.draw_text(10, 137, "ORBIT CLEARED", CANTICLE_COLOR);
                framebuffer.draw_text(10, 153, "PATH OPENS", TEXT);
            }
            EncounterPhase::Waves | EncounterPhase::BossFight => {}
        }
    }
}

impl Game for VoidCanticleV10 {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.inner.inner.base.controls.update(frame.input);

        if self.inner.inner.base.game_over {
            if self.inner.inner.base.controls.action(FIRE).pressed() {
                self.reset_run();
            }
            self.render(frame.framebuffer, false);
            return GameResult::Continue;
        }

        let dt = frame.delta_time.as_secs_f32().min(0.05);
        let focused = self.inner.inner.base.focus_held(frame);
        let canticle_pressed = self.inner.inner.base.canticle_pressed(frame);

        self.inner.wipe_banner_timer = (self.inner.wipe_banner_timer - dt).max(0.0);
        self.inner.inner.base.scroll =
            (self.inner.inner.base.scroll + 34.0 * dt) % FRAMEBUFFER_HEIGHT as f32;
        self.inner.inner.base.update_feedback(dt);
        self.inner.inner.base.update_player(dt, focused);
        self.inner.inner.update_player_fire(dt, frame);
        self.inner.update_wave_flow(dt, frame);
        self.inner.inner.base.update_enemies(dt, frame);
        self.inner.inner.base.update_projectiles(dt);
        self.inner.inner.update_power_shots(dt);
        self.inner.resolve_power_shots(frame);
        self.inner.inner.base.update_cinders(dt, frame);
        self.inner.inner.update_relics(dt, frame);

        if canticle_pressed {
            if self.inner.inner.base.encounter_phase == EncounterPhase::Waves
                && self.inner.inner.base.core_charge >= CORE_MAX
                && self.inner.inner.base.canticle_timer <= 0.0
            {
                self.inner.canticle_used_in_wave = true;
            }
            self.inner.inner.base.trigger_canticle(frame);
        }

        self.inner.finish_wave_if_clear();
        self.update_wave_toast(dt);

        let previous_lives = self.inner.inner.base.lives;
        self.inner.inner.base.resolve_player_hits(frame);
        self.inner.inner.apply_death_penalty(previous_lives);

        self.render(frame.framebuffer, focused);
        GameResult::Continue
    }
}

struct VoidCanticlePause<G> {
    game: G,
    state: VcPauseState,
    menu: gotoo_pixel_engine::ui::MenuState,
    controls: ControlMap,
}

impl<G> VoidCanticlePause<G> {
    fn new(game: G) -> Self {
        let mut controls = gotoo_pixel_engine::ui::standard_menu_controls(
            VC_PAUSE_UP,
            VC_PAUSE_DOWN,
            VC_PAUSE_CONFIRM,
        );
        controls
            .bind_key(VC_PAUSE_TOGGLE, Key::Escape)
            .bind_gamepad(VC_PAUSE_TOGGLE, GamepadButton::Start);
        Self {
            game,
            state: VcPauseState::Running,
            menu: gotoo_pixel_engine::ui::MenuState::new(4),
            controls,
        }
    }

    fn pause_input_held(&self) -> bool {
        [VC_PAUSE_TOGGLE, VC_PAUSE_UP, VC_PAUSE_DOWN, VC_PAUSE_CONFIRM]
            .into_iter()
            .any(|action| self.controls.action(action).held())
    }

    fn render_menu(&self, framebuffer: &mut Framebuffer) {
        let panel = gotoo_pixel_engine::Rect { x: 18, y: 58, width: 144, height: 198 };
        gotoo_pixel_engine::ui::draw_panel(
            framebuffer,
            panel,
            Pixel::rgb(9, 8, 15),
            PILGRIM_VIOLET,
        );
        framebuffer.draw_text(60, 72, "PAUSED", POWER_RELIC_LIGHT);

        for (index, (y, label)) in [
            (101, "RESUME"),
            (130, "CONTROLS"),
            (159, "BUILD INFO"),
            (188, "QUIT"),
        ]
        .into_iter()
        .enumerate()
        {
            gotoo_pixel_engine::ui::draw_menu_item(
                framebuffer,
                gotoo_pixel_engine::Rect { x: 34, y, width: 112, height: 18 },
                label,
                self.menu.selected() == Some(index),
                1,
                TEXT,
                POWER_RELIC_LIGHT,
            );
        }
    }

    fn render_controls(&self, framebuffer: &mut Framebuffer) {
        let panel = gotoo_pixel_engine::Rect { x: 10, y: 46, width: 160, height: 224 };
        gotoo_pixel_engine::ui::draw_panel(
            framebuffer,
            panel,
            Pixel::rgb(9, 8, 15),
            PILGRIM_VIOLET,
        );
        framebuffer.draw_text(52, 59, "CONTROLS", POWER_RELIC_LIGHT);
        framebuffer.draw_text(20, 86, "MOVE  ARROWS WASD", TEXT);
        framebuffer.draw_text(20, 106, "FIRE  SPACE SOUTH", TEXT);
        framebuffer.draw_text(20, 126, "FOCUS SHIFT LB", TEXT);
        framebuffer.draw_text(20, 146, "CANTICLE X EAST", TEXT);
        framebuffer.draw_text(20, 166, "PAUSE ESC START", TEXT);
        framebuffer.draw_text(31, 231, "ESC START  BACK", WRECK_LIGHT);
    }

    fn render_build_info(&self, framebuffer: &mut Framebuffer) {
        let panel = gotoo_pixel_engine::Rect { x: 10, y: 58, width: 160, height: 196 };
        gotoo_pixel_engine::ui::draw_panel(
            framebuffer,
            panel,
            Pixel::rgb(9, 8, 15),
            PILGRIM_VIOLET,
        );
        framebuffer.draw_text(46, 72, "BUILD INFO", POWER_RELIC_LIGHT);
        framebuffer.draw_text(20, 103, &format!("VERSION {VC10_VERSION}"), TEXT);
        framebuffer.draw_text(20, 123, &format!("BUILD {BUILD_ID}"), TEXT);
        framebuffer.draw_text(20, 143, "STAGE GRAVE ORBIT", TEXT);
        framebuffer.draw_text(20, 163, "GPE DEV BUILD", WRECK_LIGHT);
        framebuffer.draw_text(31, 219, "ESC START  BACK", WRECK_LIGHT);
    }
}

impl<G: Game> Game for VoidCanticlePause<G> {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.controls.update(frame.input);

        match self.state {
            VcPauseState::Running => {
                if self.controls.action(VC_PAUSE_TOGGLE).pressed() {
                    self.state = VcPauseState::Menu;
                    self.menu = gotoo_pixel_engine::ui::MenuState::new(4);
                    self.render_menu(frame.framebuffer);
                    GameResult::Continue
                } else {
                    self.game.update(frame)
                }
            }
            VcPauseState::Menu => {
                if self.controls.action(VC_PAUSE_TOGGLE).pressed() {
                    self.state = VcPauseState::ResumeGate;
                } else {
                    if self.controls.action(VC_PAUSE_UP).pressed() {
                        self.menu.select_previous();
                    }
                    if self.controls.action(VC_PAUSE_DOWN).pressed() {
                        self.menu.select_next();
                    }
                    if self.controls.action(VC_PAUSE_CONFIRM).pressed() {
                        match self.menu.selected() {
                            Some(0) => self.state = VcPauseState::ResumeGate,
                            Some(1) => self.state = VcPauseState::Controls,
                            Some(2) => self.state = VcPauseState::BuildInfo,
                            Some(3) => return GameResult::Exit,
                            _ => {}
                        }
                    }
                }
                match self.state {
                    VcPauseState::Controls => self.render_controls(frame.framebuffer),
                    VcPauseState::BuildInfo => self.render_build_info(frame.framebuffer),
                    _ => self.render_menu(frame.framebuffer),
                }
                GameResult::Continue
            }
            VcPauseState::Controls => {
                if self.controls.action(VC_PAUSE_TOGGLE).pressed()
                    || self.controls.action(VC_PAUSE_CONFIRM).pressed()
                {
                    self.state = VcPauseState::Menu;
                    self.render_menu(frame.framebuffer);
                } else {
                    self.render_controls(frame.framebuffer);
                }
                GameResult::Continue
            }
            VcPauseState::BuildInfo => {
                if self.controls.action(VC_PAUSE_TOGGLE).pressed()
                    || self.controls.action(VC_PAUSE_CONFIRM).pressed()
                {
                    self.state = VcPauseState::Menu;
                    self.render_menu(frame.framebuffer);
                } else {
                    self.render_build_info(frame.framebuffer);
                }
                GameResult::Continue
            }
            VcPauseState::ResumeGate => {
                if self.pause_input_held() {
                    self.render_menu(frame.framebuffer);
                } else {
                    self.state = VcPauseState::Running;
                }
                GameResult::Continue
            }
        }
    }
}

pub fn run_v10() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: "Void Canticle - Gotoo Pixel Engine".to_string(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width,
            window_height,
        },
        VoidCanticlePause::new(VoidCanticleV10::new()),
    )
}

#[cfg(test)]
mod v10_tests {
    use super::*;

    #[test]
    fn player_hud_keeps_only_essential_resources() {
        assert_eq!(MAX_POWER_LEVEL, 5);
        assert_eq!(CORE_MAX, 100);
    }

    #[test]
    fn build_metadata_has_a_dedicated_version() {
        assert_eq!(VC10_VERSION, "VC1.0");
    }
}
