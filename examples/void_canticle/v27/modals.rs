include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/v27/modals/build_overview.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/v27/modals/pause_menu.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/v27/modals/controls.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/v27/modals/run_summary.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/v27/modals/stage_clear.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/v27/modals/death.rs"
));

impl VoidCanticleV27DirectPresentation {
    fn render_clean_modal(&mut self, framebuffer: &mut Framebuffer, mode: VcVisualMode) {
        self.render_clean_background(framebuffer);

        match mode {
            VcVisualMode::LevelChoice => {
                let v14 = self.game.game.v20().game.v14();
                if let Some(choice) = v14.progression.level_choice.as_ref() {
                    vc27_render_upgrade_showcase(
                        framebuffer,
                        &v14.progression,
                        &v14.mutations,
                        choice,
                        self.presentation_time,
                    );
                }
                return;
            }
            VcVisualMode::MutationChoice => {
                let v14 = self.game.game.v20().game.v14();
                if let Some(choice) = v14.mutation_choice.as_ref() {
                    vc27_render_mutation_showcase(
                        framebuffer,
                        v14,
                        choice,
                        self.presentation_time,
                    );
                }
                return;
            }
            VcVisualMode::SupportChoice => {
                vc27_render_support_showcase(framebuffer, &self.game, self.presentation_time);
                return;
            }
            VcVisualMode::Pause => {
                let pause = self.game.survival_model().game.pause_ui();
                match &pause.state {
                    VcPauseState::BuildInfo => {
                        vc27_render_build_overview(framebuffer, &self.game, self.presentation_time);
                    }
                    VcPauseState::Controls => {
                        vc27_render_controls_reference(framebuffer, self.presentation_time);
                    }
                    VcPauseState::Menu | VcPauseState::ResumeGate | VcPauseState::Running => {
                        vc27_render_pause_menu(
                            framebuffer,
                            &self.game,
                            pause,
                            self.presentation_time,
                        );
                    }
                }
                return;
            }
            VcVisualMode::StageClear => {
                vc27_render_stage_clear_presentation(framebuffer, &self.game, self.presentation_time);
                return;
            }
            VcVisualMode::Combat | VcVisualMode::Death => {}
        }
    }

    fn render_death_presentation(&mut self, framebuffer: &mut Framebuffer) {
        self.render_clean_background(framebuffer);
        vc27_render_death_screen(framebuffer, &self.game, self.presentation_time);
    }
}
