impl VoidCanticleV27DirectPresentation {
    fn new() -> Self {
        Self {
            game: VoidCanticleV23Sustain::new(),
            legacy_sink: Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT),
            clean_background: Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT),
            presentation_time: 0.0,
            hit_reactions: Vc27HitReactionState::default(),
            projectile_provenance: Vc27ProjectileProvenance::default(),
        }
    }

    fn chassis_selection_active(&self) -> bool {
        self.game.game.game.game.game.chassis.is_none()
    }

    fn reset_to_fresh_launch(&mut self) {
        self.game = VoidCanticleV23Sustain::new();
        self.presentation_time = 0.0;
        self.hit_reactions = Vc27HitReactionState::default();
        self.projectile_provenance = Vc27ProjectileProvenance::default();
    }

    fn render_chassis_selection_presentation(&mut self, framebuffer: &mut Framebuffer) {
        let selector = &self.game.game.game.game.game;
        vc27_render_chassis_showcase(
            &mut self.clean_background,
            framebuffer,
            selector,
            self.presentation_time,
        );
    }

    fn visual_mode(&self) -> VcVisualMode {
        if self.game.choosing_support {
            return VcVisualMode::SupportChoice;
        }

        let v14 = self.game.game.v20().game.v14();
        if v14.progression.level_choice.is_some() {
            return VcVisualMode::LevelChoice;
        }
        if v14.mutation_choice.is_some() {
            return VcVisualMode::MutationChoice;
        }
        if self.game.game.base().game_over {
            return VcVisualMode::Death;
        }

        let stabilized = &self.game.survival_model().game;
        if stabilized.stage_clear_visible() {
            return VcVisualMode::StageClear;
        }
        if !matches!(&stabilized.pause_ui().state, VcPauseState::Running) {
            return VcVisualMode::Pause;
        }

        VcVisualMode::Combat
    }

    fn render_clean_background(&mut self, framebuffer: &mut Framebuffer) {
        let base = self.game.game.base();
        let color = if base.canticle_timer > 0.46 {
            BG_CANTICLE
        } else {
            BG
        };
        let scroll = base.scroll;

        self.clean_background.clear(color);
        render_grave_orbit_background(&mut self.clean_background, scroll);
        vc_visual_blit_nearest(
            &self.clean_background,
            framebuffer,
            VC_VISUAL_PRESENTATION_SCALE,
            false,
        );
    }
}

fn vc27_full_restart_completed(
    was_game_over: bool,
    is_game_over: bool,
    was_stage_clear: bool,
    encounter_phase: EncounterPhase,
    pause_restart_selected: bool,
    pause_confirm_pressed: bool,
) -> bool {
    let death_restart = was_game_over && !is_game_over;
    let stage_restart = was_stage_clear && encounter_phase != EncounterPhase::Cleared;
    let pause_restart = pause_restart_selected && pause_confirm_pressed;

    death_restart || stage_restart || pause_restart
}

impl Game for VoidCanticleV27DirectPresentation {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let dt = frame.delta_time.as_secs_f32().min(0.05);
        self.presentation_time += dt;

        let was_game_over = self.game.game.base().game_over;
        let was_stage_clear = self.game.survival_model().game.stage_clear_visible();
        let pause_restart_selected = {
            let pause = self.game.survival_model().game.pause_ui();
            matches!(&pause.state, VcPauseState::Menu) && pause.menu.selected() == Some(1)
        };
        let hit_snapshot = Vc27HitSnapshot::capture(&self.game);
        let projectile_snapshot = Vc27ProjectileSourceSnapshot::capture(&self.game);

        let result = {
            let mut legacy_audio = Vc27LegacyAttackAudioFilter::new(&mut *frame.audio);
            let mut legacy_frame = Frame {
                framebuffer: &mut self.legacy_sink,
                input: frame.input,
                delta_time: frame.delta_time,
                storage: &mut *frame.storage,
                audio: &mut legacy_audio,
                surface_size: frame.surface_size,
                viewport: gotoo_pixel_engine::Viewport::new(
                    frame.surface_size,
                    Size {
                        width: FRAMEBUFFER_WIDTH,
                        height: FRAMEBUFFER_HEIGHT,
                    },
                ),
            };
            self.game.update(&mut legacy_frame)
        };
        if result == GameResult::Exit {
            return result;
        }

        let pause_confirm_pressed = self
            .game
            .survival_model()
            .game
            .pause_ui()
            .controls
            .action(VC_PAUSE_CONFIRM)
            .pressed();
        if vc27_full_restart_completed(
            was_game_over,
            self.game.game.base().game_over,
            was_stage_clear,
            self.game.game.base().encounter_phase,
            pause_restart_selected,
            pause_confirm_pressed,
        ) {
            self.reset_to_fresh_launch();
            self.render_chassis_selection_presentation(frame.framebuffer);
            return GameResult::Continue;
        }

        self.hit_reactions.update(dt, &hit_snapshot, &self.game);
        let attack_sounds =
            self.projectile_provenance
                .reconcile(dt, &projectile_snapshot, &self.game);
        self.game.play_attack_sounds(frame, &attack_sounds);

        if self.chassis_selection_active() {
            self.render_chassis_selection_presentation(frame.framebuffer);
            return GameResult::Continue;
        }

        let mode = self.visual_mode();
        match mode {
            VcVisualMode::Combat => self.render_combat_presentation(frame.framebuffer),
            VcVisualMode::Death => self.render_death_presentation(frame.framebuffer),
            VcVisualMode::Pause
            | VcVisualMode::LevelChoice
            | VcVisualMode::MutationChoice
            | VcVisualMode::SupportChoice
            | VcVisualMode::StageClear => self.render_clean_modal(frame.framebuffer, mode),
        }

        GameResult::Continue
    }
}

pub fn run_v27_direct_presentation_with_obs_mirror() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: format!(
                "Void Canticle {VC27_PRESENTATION_VERSION} Direct HUD - Gotoo Pixel Engine"
            ),
            framebuffer_width: VC_VISUAL_PRESENTATION_WIDTH,
            framebuffer_height: VC_VISUAL_PRESENTATION_HEIGHT,
            window_width,
            window_height,
        },
        gotoo_pixel_engine::ObsMirrorGame::from_env(
            VoidCanticleV27DirectPresentation::new(),
            VC_VISUAL_PRESENTATION_WIDTH,
            VC_VISUAL_PRESENTATION_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v27_runtime_tests {
    use super::*;

    #[test]
    fn version_is_explicit() {
        assert_eq!(VC27_PRESENTATION_VERSION, "VC3.1");
    }

    #[test]
    fn chassis_selection_is_explicit_precombat_state() {
        let game = VoidCanticleV27DirectPresentation::new();
        assert!(game.chassis_selection_active());
    }

    #[test]
    fn every_restart_path_requests_a_fresh_launch() {
        assert!(vc27_full_restart_completed(
            true,
            false,
            false,
            EncounterPhase::Waves,
            false,
            false,
        ));
        assert!(vc27_full_restart_completed(
            false,
            false,
            true,
            EncounterPhase::Waves,
            false,
            false,
        ));
        assert!(vc27_full_restart_completed(
            false,
            false,
            false,
            EncounterPhase::Waves,
            true,
            true,
        ));
        assert!(!vc27_full_restart_completed(
            false,
            false,
            false,
            EncounterPhase::Waves,
            true,
            false,
        ));
    }

    #[test]
    fn fresh_launch_resets_presentation_state_and_returns_to_chassis_select() {
        let mut game = VoidCanticleV27DirectPresentation::new();
        game.presentation_time = 42.0;
        game.reset_to_fresh_launch();

        assert_eq!(game.presentation_time, 0.0);
        assert!(game.chassis_selection_active());
    }

    #[test]
    fn simulation_coordinates_map_to_presentation_space() {
        assert_eq!(vc27_present(0.0), 0);
        assert_eq!(vc27_present(90.0), 180);
        assert_eq!(vc27_present(319.5), 639);
    }
}
