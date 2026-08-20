// Transitional compatibility seam for the HD frontend. The historical procedural
// title flow has been retired; current launches are owned by frontend.rs.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Vc27FrontState {
    Run,
}

impl VoidCanticleV27DirectPresentation {
    fn new() -> Self {
        Self {
            game: VoidCanticleV23Sustain::new(),
            legacy_sink: Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT),
            clean_background: Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT),
            presentation_time: 0.0,
            front_state: Vc27FrontState::Run,
            hit_reactions: Vc27HitReactionState::default(),
            projectile_provenance: Vc27ProjectileProvenance::default(),
        }
    }

    fn chassis_runtime(&self) -> &VoidCanticleV22 {
        &self.game.game.game.game.game
    }

    fn chassis_runtime_mut(&mut self) -> &mut VoidCanticleV22 {
        &mut self.game.game.game.game.game
    }

    fn chassis_selection_active(&self) -> bool {
        self.chassis_runtime().chassis.is_none()
    }

    // The lower gameplay runtime owns the actual run reset. This method only
    // resets the state layered above it and explicitly returns the chassis owner
    // to its pre-run selection state. It must not replace `self.game`.
    fn reset_gameplay_selection_after_restart(&mut self) {
        self.game.reset_for_new_run();
        self.chassis_runtime_mut()
            .reset_chassis_selection_for_new_run();
    }

    fn reset_presentation_after_restart(&mut self) {
        self.presentation_time = 0.0;
        self.front_state = Vc27FrontState::Run;
        self.hit_reactions = Vc27HitReactionState::default();
        self.projectile_provenance = Vc27ProjectileProvenance::default();
    }

    fn render_chassis_selection_presentation(&mut self, framebuffer: &mut Framebuffer) {
        let Self {
            game,
            clean_background,
            presentation_time,
            ..
        } = self;
        let selector = &game.game.game.game.game;
        vc27_render_chassis_showcase(
            clean_background,
            framebuffer,
            selector,
            *presentation_time,
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
        let pause_restart_requested = pause_restart_selected
            && gotoo_pixel_engine::ui::menu_confirm_pressed(frame.input);
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

        let pause_restart_completed = pause_restart_requested
            && matches!(
                &self.game.survival_model().game.pause_ui().state,
                VcPauseState::Running
            );
        if pause_restart_completed {
            self.reset_gameplay_selection_after_restart();
        }

        if restart_completed(
            was_game_over,
            self.game.game.base().game_over,
            was_stage_clear,
            self.game.game.base().encounter_phase,
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
        let game = VoidCanticleV27DirectPresentation::new();
        assert!(game.chassis_selection_active());
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
        let mut presentation = VoidCanticleV27DirectPresentation::new();
        presentation.chassis_runtime_mut().confirm_armed = true;
        presentation
            .chassis_runtime_mut()
            .apply_chassis(ExosuitChassis::Wraith);

        presentation.reset_gameplay_selection_after_restart();

        assert!(presentation.chassis_selection_active());
        assert_eq!(presentation.chassis_runtime().chassis, None);
        assert!(!presentation.chassis_runtime().confirm_armed);
    }

    #[test]
    fn presentation_restart_cleanup_only_resets_presentation_state() {
        let mut presentation = VoidCanticleV27DirectPresentation::new();
        presentation
            .chassis_runtime_mut()
            .apply_chassis(ExosuitChassis::Wraith);
        presentation.presentation_time = 42.0;

        presentation.reset_presentation_after_restart();

        assert_eq!(presentation.presentation_time, 0.0);
        assert_eq!(presentation.front_state, Vc27FrontState::Run);
        assert_eq!(
            presentation.chassis_runtime().chassis,
            Some(ExosuitChassis::Wraith)
        );
    }

    #[test]
    fn simulation_coordinates_map_to_presentation_space() {
        assert_eq!(vc27_present(0.0), 0);
        assert_eq!(vc27_present(90.0), 180);
        assert_eq!(vc27_present(319.5), 639);
    }
}
