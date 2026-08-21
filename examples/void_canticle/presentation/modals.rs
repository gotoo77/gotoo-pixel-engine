include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/presentation/modals/build_overview.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/presentation/modals/pause_menu.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/presentation/modals/controls.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/presentation/modals/run_summary.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/presentation/modals/stage_clear.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/presentation/modals/death.rs"
));

impl VoidCanticlePresentation {
    fn render_clean_modal(&mut self, framebuffer: &mut Framebuffer, mode: VcVisualMode) {
        self.render_clean_background(framebuffer);

        match mode {
            VcVisualMode::LevelChoice => {
                let progression = self.game.presentation_progression();
                if let Some(choice) = progression.progression.level_choice.as_ref() {
                    render_upgrade_showcase(
                        framebuffer,
                        &progression.progression,
                        &progression.mutations,
                        choice,
                        self.presentation_time,
                    );
                }
            }
            VcVisualMode::MutationChoice => {
                let progression = self.game.presentation_progression();
                if let Some(choice) = progression.mutation_choice.as_ref() {
                    render_mutation_showcase(
                        framebuffer,
                        progression,
                        choice,
                        self.presentation_time,
                    );
                }
            }
            VcVisualMode::SupportChoice => {
                render_support_showcase(framebuffer, &self.game, self.presentation_time);
            }
            VcVisualMode::Pause => {
                let pause = self.game.survival_model().game.pause_ui();
                match &pause.state {
                    VcPauseState::BuildInfo => {
                        render_build_overview(framebuffer, &self.game, self.presentation_time);
                    }
                    VcPauseState::Controls => {
                        render_controls_reference(framebuffer, self.presentation_time);
                    }
                    VcPauseState::Menu | VcPauseState::ResumeGate | VcPauseState::Running => {
                        render_pause_menu(
                            framebuffer,
                            &self.game,
                            pause,
                            self.presentation_time,
                        );
                    }
                }
            }
            VcVisualMode::StageClear => {
                render_stage_clear_presentation(framebuffer, &self.game, self.presentation_time);
            }
            VcVisualMode::Combat | VcVisualMode::Death => {}
        }
    }

    fn render_death_presentation(&mut self, framebuffer: &mut Framebuffer) {
        self.render_clean_background(framebuffer);
        render_death_screen(framebuffer, &self.game, self.presentation_time);
    }
}
