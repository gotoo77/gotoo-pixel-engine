impl VoidCanticlePresentation {
    fn new() -> Self {
        Self {
            game: GameplayRuntime::new(),
            legacy_sink: Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT),
            clean_background: Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT),
            presentation_time: 0.0,
            hit_reactions: HitReactionState::default(),
            projectile_provenance: ProjectileProvenance::default(),
        }
    }

    fn chassis_selection_active(&self) -> bool {
        self.game.presentation_chassis_selection_active()
    }

    // The lower gameplay runtime owns the actual run reset. This method only
    // resets the state layered above it and explicitly returns the chassis owner
    // to its pre-run selection state. It must not replace `self.game`.
    fn reset_gameplay_selection_after_restart(&mut self) {
        self.game.reset_for_new_run();
        self.game.presentation_reset_chassis_selection();
    }

    fn reset_presentation_after_restart(&mut self) {
        self.presentation_time = 0.0;
        self.hit_reactions = HitReactionState::default();
        self.projectile_provenance = ProjectileProvenance::default();
    }

    fn render_chassis_selection_presentation(&mut self, framebuffer: &mut Framebuffer) {
        let Self {
            game,
            clean_background,
            presentation_time,
            ..
        } = self;
        let selector = game.presentation_chassis_selector();
        vc27_render_chassis_showcase(
            clean_background,
            framebuffer,
            selector,
            *presentation_time,
        );
    }

    fn visual_mode(&self) -> VcVisualMode {
        if self.game.presentation_support_choice_active() {
            return VcVisualMode::SupportChoice;
        }

        let progression = self.game.presentation_progression();
        if progression.progression.level_choice.is_some() {
            return VcVisualMode::LevelChoice;
        }
        if progression.mutation_choice.is_some() {
            return VcVisualMode::MutationChoice;
        }
        if self.game.presentation_base().game_over {
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
        let base = self.game.presentation_base();
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

fn pause_restart_completed(
    restart_selected_before_update: bool,
    confirm_pressed: bool,
    toggle_pressed: bool,
    state_after_update: &VcPauseState,
) -> bool {
    restart_selected_before_update
        && confirm_pressed
        && !toggle_pressed
        && matches!(state_after_update, VcPauseState::ResumeGate)
}

fn restart_completed(
    was_game_over: bool,
    is_game_over: bool,
    was_stage_clear: bool,
    encounter_phase: EncounterPhase,
    pause_restart_completed: bool,
) -> bool {
    let death_restart = was_game_over && !is_game_over;
    let stage_restart = was_stage_clear && encounter_phase != EncounterPhase::Cleared;

    death_restart || stage_restart || pause_restart_completed
}

impl Game for VoidCanticlePresentation {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let dt = frame.delta_time.as_secs_f32().min(0.05);
        self.presentation_time += dt;

        let was_game_over = self.game.presentation_base().game_over;
        let was_stage_clear = self.game.survival_model().game.stage_clear_visible();
        let pause_restart_selected = {
            let pause = self.game.survival_model().game.pause_ui();
            matches!(&pause.state, VcPauseState::Menu) && pause.menu.selected() == Some(1)
        };
        let hit_snapshot = HitSnapshot::capture(&self.game);
        let projectile_snapshot = ProjectileSourceSnapshot::capture(&self.game);

        let result = {
            let mut legacy_audio = LegacyAttackAudioFilter::new(&mut *frame.audio);
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

        let pause_restart_completed = {
            let pause = self.game.survival_model().game.pause_ui();
            pause_restart_completed(
                pause_restart_selected,
                pause.controls.action(VC_PAUSE_CONFIRM).pressed(),
                pause.controls.action(VC_PAUSE_TOGGLE).pressed(),
                &pause.state,
            )
        };
        if pause_restart_completed {
            self.reset_gameplay_selection_after_restart();
        }

        if restart_completed(
            was_game_over,
            self.game.presentation_base().game_over,
            was_stage_clear,
            self.game.presentation_base().encounter_phase,
            pause_restart_completed,
        ) {
            self.reset_presentation_after_restart();
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

#[cfg(test)]
mod presentation_runtime_tests {
    use super::*;

    #[test]
    fn chassis_selection_is_explicit_precombat_state() {
        let game = VoidCanticlePresentation::new();
        assert!(game.chassis_selection_active());
    }

    #[test]
    fn pause_restart_is_completed_when_pause_enters_resume_gate_after_confirm() {
        assert!(pause_restart_completed(
            true,
            true,
            false,
            &VcPauseState::ResumeGate,
        ));
        assert!(!pause_restart_completed(
            true,
            true,
            false,
            &VcPauseState::Running,
        ));
        assert!(!pause_restart_completed(
            true,
            true,
            true,
            &VcPauseState::ResumeGate,
        ));
    }

    #[test]
    fn every_restart_path_is_detected() {
        assert!(restart_completed(
            true,
            false,
            false,
            EncounterPhase::Waves,
            false,
        ));
        assert!(restart_completed(
            false,
            false,
            true,
            EncounterPhase::Waves,
            false,
        ));
        assert!(restart_completed(
            false,
            false,
            false,
            EncounterPhase::Waves,
            true,
        ));
        assert!(!restart_completed(
            false,
            false,
            false,
            EncounterPhase::Waves,
            false,
        ));
    }

    #[test]
    fn restart_reopens_chassis_selection_without_auto_confirm() {
        let mut presentation = VoidCanticlePresentation::new();
        presentation.game.presentation_arm_chassis_confirm_for_test();
        presentation
            .game
            .presentation_apply_chassis_for_test(ExosuitChassis::Wraith);

        presentation.reset_gameplay_selection_after_restart();

        assert!(presentation.chassis_selection_active());
        assert_eq!(presentation.game.presentation_selected_chassis(), None);
        assert!(!presentation.game.presentation_chassis_confirm_armed());
    }

    #[test]
    fn presentation_restart_cleanup_only_resets_presentation_state() {
        let mut presentation = VoidCanticlePresentation::new();
        presentation
            .game
            .presentation_apply_chassis_for_test(ExosuitChassis::Wraith);
        presentation.presentation_time = 42.0;

        presentation.reset_presentation_after_restart();

        assert_eq!(presentation.presentation_time, 0.0);
        assert_eq!(
            presentation.game.presentation_selected_chassis(),
            Some(ExosuitChassis::Wraith)
        );
    }

    #[test]
    fn current_runtime_does_not_traverse_legacy_wrapper_chain() {
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/void_canticle/presentation/runtime.rs"
        ));
        let forbidden = [".game", ".game"].concat();
        assert!(!source.contains(&forbidden));
    }

    #[test]
    fn simulation_coordinates_map_to_presentation_space() {
        assert_eq!(vc27_present(0.0), 0);
        assert_eq!(vc27_present(90.0), 180);
        assert_eq!(vc27_present(319.5), 639);
    }
}
