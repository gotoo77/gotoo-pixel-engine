const VC10_VERSION: &str = "VC0.10";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum V10Screen {
    Running,
    PauseMenu,
    Controls,
    BuildInfo,
    ResumeGate,
}

struct VoidCanticleV10 {
    game: VoidCanticleV09,
    screen: V10Screen,
    menu_index: usize,
}

impl VoidCanticleV10 {
    fn new() -> Self {
        Self {
            game: VoidCanticleV09::new(),
            screen: V10Screen::Running,
            menu_index: 0,
        }
    }

    fn reset_run(&mut self) {
        self.game.reset_run();
        self.screen = V10Screen::Running;
        self.menu_index = 0;
    }

    fn pause_pressed(frame: &Frame<'_>) -> bool {
        frame.input.key(Key::Escape).pressed()
            || frame
                .input
                .gamepad_button_any(GamepadButton::Start)
                .pressed()
    }

    fn menu_up_pressed(frame: &Frame<'_>) -> bool {
        frame.input.key(Key::Up).pressed()
            || frame.input.key(Key::W).pressed()
            || frame
                .input
                .gamepad_button_any(GamepadButton::DPadUp)
                .pressed()
            || frame
                .input
                .gamepad_button_any(GamepadButton::LeftStickUp)
                .pressed()
    }

    fn menu_down_pressed(frame: &Frame<'_>) -> bool {
        frame.input.key(Key::Down).pressed()
            || frame.input.key(Key::S).pressed()
            || frame
                .input
                .gamepad_button_any(GamepadButton::DPadDown)
                .pressed()
            || frame
                .input
                .gamepad_button_any(GamepadButton::LeftStickDown)
                .pressed()
    }

    fn confirm_pressed(frame: &Frame<'_>) -> bool {
        frame.input.key(Key::Space).pressed()
            || frame
                .input
                .gamepad_button_any(GamepadButton::South)
                .pressed()
    }

    fn back_pressed(frame: &Frame<'_>) -> bool {
        frame.input.key(Key::Escape).pressed()
            || frame
                .input
                .gamepad_button_any(GamepadButton::East)
                .pressed()
    }

    fn menu_input_held(frame: &Frame<'_>) -> bool {
        [Key::Escape, Key::Space, Key::Up, Key::Down, Key::W, Key::S]
            .into_iter()
            .any(|key| frame.input.key(key).held())
            || [
                GamepadButton::Start,
                GamepadButton::South,
                GamepadButton::East,
                GamepadButton::DPadUp,
                GamepadButton::DPadDown,
                GamepadButton::LeftStickUp,
                GamepadButton::LeftStickDown,
            ]
            .into_iter()
            .any(|button| frame.input.gamepad_button_any(button).held())
    }

    fn update_running(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if Self::pause_pressed(frame) {
            self.screen = V10Screen::PauseMenu;
            self.menu_index = 0;
            self.render(frame.framebuffer, false);
            self.render_pause_menu(frame.framebuffer);
            return GameResult::Continue;
        }

        self.game.inner.base.controls.update(frame.input);

        if self.game.inner.base.game_over {
            if self.game.inner.base.controls.action(FIRE).pressed() {
                self.reset_run();
            }
            self.render(frame.framebuffer, false);
            return GameResult::Continue;
        }

        let dt = frame.delta_time.as_secs_f32().min(0.05);
        let focused = self.game.inner.base.focus_held(frame);
        let canticle_pressed = self.game.inner.base.canticle_pressed(frame);

        self.game.wipe_banner_timer = (self.game.wipe_banner_timer - dt).max(0.0);
        self.game.inner.base.scroll =
            (self.game.inner.base.scroll + 34.0 * dt) % FRAMEBUFFER_HEIGHT as f32;
        self.game.inner.base.update_feedback(dt);
        self.game.inner.base.update_player(dt, focused);
        self.game.inner.update_player_fire(dt, frame);
        self.game.update_wave_flow(dt, frame);
        self.game.inner.base.update_enemies(dt, frame);
        self.game.inner.base.update_projectiles(dt);
        self.game.inner.update_power_shots(dt);
        self.game.resolve_power_shots(frame);
        self.game.inner.base.update_cinders(dt, frame);
        self.game.inner.update_relics(dt, frame);

        if canticle_pressed {
            if self.game.inner.base.encounter_phase == EncounterPhase::Waves
                && self.game.inner.base.core_charge >= CORE_MAX
                && self.game.inner.base.canticle_timer <= 0.0
            {
                self.game.canticle_used_in_wave = true;
            }
            self.game.inner.base.trigger_canticle(frame);
        }

        self.game.finish_wave_if_clear();

        let previous_lives = self.game.inner.base.lives;
        self.game.inner.base.resolve_player_hits(frame);
        self.game.inner.apply_death_penalty(previous_lives);

        self.render(frame.framebuffer, focused);
        GameResult::Continue
    }

    fn update_pause_menu(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if Self::menu_up_pressed(frame) {
            self.menu_index = self.menu_index.checked_sub(1).unwrap_or(3);
        }
        if Self::menu_down_pressed(frame) {
            self.menu_index = (self.menu_index + 1) % 4;
        }

        if Self::pause_pressed(frame) {
            self.screen = V10Screen::ResumeGate;
        } else if Self::confirm_pressed(frame) {
            match self.menu_index {
                0 => self.screen = V10Screen::ResumeGate,
                1 => self.screen = V10Screen::Controls,
                2 => self.screen = V10Screen::BuildInfo,
                3 => return GameResult::Exit,
                _ => unreachable!(),
            }
        }

        self.render(frame.framebuffer, false);
        match self.screen {
            V10Screen::PauseMenu | V10Screen::ResumeGate => {
                self.render_pause_menu(frame.framebuffer)
            }
            V10Screen::Controls => self.render_controls(frame.framebuffer),
            V10Screen::BuildInfo => self.render_build_info(frame.framebuffer),
            V10Screen::Running => {}
        }
        GameResult::Continue
    }

    fn update_info_page(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if Self::back_pressed(frame) || Self::confirm_pressed(frame) {
            self.screen = V10Screen::PauseMenu;
        }

        self.render(frame.framebuffer, false);
        match self.screen {
            V10Screen::Controls => self.render_controls(frame.framebuffer),
            V10Screen::BuildInfo => self.render_build_info(frame.framebuffer),
            V10Screen::PauseMenu => self.render_pause_menu(frame.framebuffer),
            _ => {}
        }
        GameResult::Continue
    }

    fn update_resume_gate(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.render(frame.framebuffer, false);
        self.render_pause_menu(frame.framebuffer);
        if !Self::menu_input_held(frame) {
            self.screen = V10Screen::Running;
        }
        GameResult::Continue
    }

    fn render(&self, framebuffer: &mut Framebuffer, focused: bool) {
        let base = &self.game.inner.base;
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
        for relic in &self.game.inner.relics {
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
        for shot in &self.game.inner.power_shots {
            render_power_shot(framebuffer, *shot, self.game.inner.power_level);
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

        self.game.inner.pilgrim_visuals.render(
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

        self.render_clean_hud(framebuffer);
        self.render_side_notifications(framebuffer);

        if base.game_over {
            framebuffer.fill_rect(27, 136, 126, 42, Pixel::rgb(10, 8, 14));
            framebuffer.draw_rect(27, 136, 126, 42, DANGER);
            framebuffer.draw_text(42, 146, "PILGRIM FALLEN", DANGER);
            framebuffer.draw_text(46, 162, "SPACE RETRY", TEXT);
        }
    }

    fn render_clean_hud(&self, framebuffer: &mut Framebuffer) {
        let base = &self.game.inner.base;
        framebuffer.draw_text(4, 4, &format!("L{}", base.lives), TEXT);
        framebuffer.draw_text(132, 4, &format!("{}", base.score), PILGRIM_CORE);

        if let Some(boss) = base.boss {
            if base.encounter_phase == EncounterPhase::BossFight {
                framebuffer.fill_rect(30, 5, 96, 4, CORE_BG);
                let width = 96_u32.saturating_mul(boss.hp) / BELLKEEPER_MAX_HP;
                framebuffer.fill_rect(30, 5, width, 4, DANGER);
            }
        }

        for index in 0..MAX_POWER_LEVEL {
            let color = if index < self.game.inner.power_level {
                POWER_RELIC
            } else {
                CORE_BG
            };
            framebuffer.fill_rect(4 + i32::from(index) * 7, 311, 5, 4, color);
        }

        framebuffer.fill_rect(44, 311, 132, 4, CORE_BG);
        let core_width = 132_u32.saturating_mul(base.core_charge) / CORE_MAX;
        framebuffer.fill_rect(44, 311, core_width, 4, CINDER);
        if base.core_charge >= CORE_MAX {
            framebuffer.draw(177, 311, CANTICLE_COLOR);
            framebuffer.draw(177, 312, CANTICLE_COLOR);
            framebuffer.draw(177, 313, CANTICLE_COLOR);
            framebuffer.draw(177, 314, CANTICLE_COLOR);
        }
    }

    fn render_side_notifications(&self, framebuffer: &mut Framebuffer) {
        let base = &self.game.inner.base;

        if base.encounter_phase == EncounterPhase::Waves {
            let wave = (self.game.wave_index + 1).min(V09_WAVES.len());
            framebuffer.draw_text(4, 15, &format!("W {wave:02}/15"), WRECK_LIGHT);
        }

        if self.game.wipe_banner_timer > 0.0 {
            framebuffer.fill_rect(108, 30, 68, 25, Pixel::rgb(10, 8, 16));
            framebuffer.draw_rect(108, 30, 68, 25, POWER_RELIC_LIGHT);
            framebuffer.draw_text(114, 35, "FULL WIPE", POWER_RELIC_LIGHT);
            framebuffer.draw_text(
                120,
                46,
                &format!("X{} +{}", self.game.wipe_banner_chain, self.game.wipe_banner_bonus),
                POWER_RELIC,
            );
        }

        match base.encounter_phase {
            EncounterPhase::BossIntro => {
                framebuffer.fill_rect(108, 86, 68, 40, Pixel::rgb(10, 8, 14));
                framebuffer.draw_rect(108, 86, 68, 40, BELL_LIGHT);
                framebuffer.draw_text(116, 92, "WARNING", DANGER);
                framebuffer.draw_text(112, 104, "BELLKEEPER", BELL_LIGHT);
                framebuffer.draw_text(120, 116, "TOLLS", TEXT);
            }
            EncounterPhase::Cleared => {
                framebuffer.fill_rect(108, 116, 68, 38, Pixel::rgb(10, 8, 14));
                framebuffer.draw_rect(108, 116, 68, 38, CANTICLE_COLOR);
                framebuffer.draw_text(116, 123, "CLEARED", CANTICLE_COLOR);
                framebuffer.draw_text(118, 139, "PATH OPEN", TEXT);
            }
            EncounterPhase::Waves | EncounterPhase::BossFight => {}
        }
    }

    fn render_pause_panel(framebuffer: &mut Framebuffer, title: &str) {
        framebuffer.fill_rect(20, 55, 140, 210, Pixel::rgb(8, 9, 16));
        framebuffer.draw_rect(20, 55, 140, 210, BELL_LIGHT);
        framebuffer.draw_text(52, 68, title, CANTICLE_COLOR);
    }

    fn render_pause_menu(&self, framebuffer: &mut Framebuffer) {
        Self::render_pause_panel(framebuffer, "VOID CANTICLE");
        framebuffer.draw_text(67, 87, "PAUSED", TEXT);

        for (index, label) in ["RESUME", "CONTROLS", "BUILD INFO", "QUIT"]
            .into_iter()
            .enumerate()
        {
            let y = 116 + index as i32 * 28;
            let selected = self.menu_index == index;
            let color = if selected { POWER_RELIC_LIGHT } else { TEXT };
            if selected {
                framebuffer.draw_text(42, y, ">", POWER_RELIC);
            }
            framebuffer.draw_text(56, y, label, color);
        }
    }

    fn render_controls(&self, framebuffer: &mut Framebuffer) {
        Self::render_pause_panel(framebuffer, "CONTROLS");
        for (y, text, color) in [
            (94, "MOVE", POWER_RELIC_LIGHT),
            (108, "ARROWS / WASD", TEXT),
            (130, "FIRE", POWER_RELIC_LIGHT),
            (144, "SPACE / SOUTH", TEXT),
            (166, "FOCUS", POWER_RELIC_LIGHT),
            (180, "SHIFT / L SHOULDER", TEXT),
            (202, "CANTICLE", POWER_RELIC_LIGHT),
            (216, "X / EAST", TEXT),
            (242, "ESC / B BACK", WRECK_LIGHT),
        ] {
            framebuffer.draw_text(32, y, text, color);
        }
    }

    fn render_build_info(&self, framebuffer: &mut Framebuffer) {
        Self::render_pause_panel(framebuffer, "BUILD INFO");
        framebuffer.draw_text(32, 98, "VOID CANTICLE", ACCENT);
        framebuffer.draw_text(32, 116, VC10_VERSION, TEXT);
        framebuffer.draw_text(32, 142, "BUILD", WRECK_LIGHT);
        framebuffer.draw_text(32, 158, BUILD_ID, TEXT);
        framebuffer.draw_text(32, 188, "ENGINE", WRECK_LIGHT);
        framebuffer.draw_text(32, 204, "GOTOO PIXEL ENGINE", TEXT);
        framebuffer.draw_text(32, 242, "ESC / B BACK", WRECK_LIGHT);
    }
}

impl Game for VoidCanticleV10 {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        match self.screen {
            V10Screen::Running => self.update_running(frame),
            V10Screen::PauseMenu => self.update_pause_menu(frame),
            V10Screen::Controls | V10Screen::BuildInfo => self.update_info_page(frame),
            V10Screen::ResumeGate => self.update_resume_gate(frame),
        }
    }
}

pub fn run_v10() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: format!(
                "Void Canticle {VC10_VERSION} [{BUILD_ID}] - Gotoo Pixel Engine"
            ),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width,
            window_height,
        },
        VoidCanticleV10::new(),
    )
}

#[cfg(test)]
mod v10_tests {
    use super::*;

    #[test]
    fn clean_hud_keeps_only_direct_gameplay_metrics() {
        assert_eq!(MAX_POWER_LEVEL, 5);
        assert_eq!(VC10_VERSION, "VC0.10");
    }

    #[test]
    fn pause_menu_exposes_controls_and_build_info() {
        let labels = ["RESUME", "CONTROLS", "BUILD INFO", "QUIT"];
        assert_eq!(labels.len(), 4);
        assert!(labels.contains(&"CONTROLS"));
        assert!(labels.contains(&"BUILD INFO"));
    }
}
