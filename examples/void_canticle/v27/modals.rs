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

impl VoidCanticleV27DirectPresentation {
    fn render_clean_modal(&mut self, framebuffer: &mut Framebuffer, mode: VcVisualMode) {
        self.render_clean_background(framebuffer);

        if matches!(&mode, VcVisualMode::LevelChoice) {
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

        if matches!(&mode, VcVisualMode::Pause) {
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

        self.clean_background.clear(BG);
        render_grave_orbit_background(&mut self.clean_background, self.game.game.base().scroll);

        match mode {
            VcVisualMode::SupportChoice => {
                self.game.render_support_choice(&mut self.clean_background);
            }
            VcVisualMode::LevelChoice => {}
            VcVisualMode::MutationChoice => {
                let v14 = self.game.game.v20().game.v14();
                if let Some(choice) = v14.mutation_choice.as_ref() {
                    v14.render_mutation_choice(&mut self.clean_background, choice);
                }
            }
            VcVisualMode::Pause => {}
            VcVisualMode::StageClear => {
                self.game
                    .survival_model()
                    .game
                    .render_stage_clear(&mut self.clean_background);
            }
            VcVisualMode::Combat | VcVisualMode::Death => {}
        }

        vc_visual_blit_nearest(
            &self.clean_background,
            framebuffer,
            VC_VISUAL_PRESENTATION_SCALE,
            false,
        );
    }

    fn render_death_presentation(&mut self, framebuffer: &mut Framebuffer) {
        let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
        self.clean_background.clear(BG);
        render_grave_orbit_background(&mut self.clean_background, self.game.game.base().scroll);
        vc_visual_blit_nearest(
            &self.clean_background,
            framebuffer,
            VC_VISUAL_PRESENTATION_SCALE,
            false,
        );

        let panel_width = 150 * scale;
        let panel_height = 62 * scale;
        let panel_x = ((VC_VISUAL_PRESENTATION_WIDTH - panel_width) / 2) as i32;
        let panel_y = (118 * scale) as i32;
        framebuffer.fill_rect(
            panel_x,
            panel_y,
            panel_width,
            panel_height,
            Pixel::rgb(9, 8, 15),
        );
        framebuffer.draw_rect(panel_x, panel_y, panel_width, panel_height, DANGER);
        vc_visual_draw_centered_text(
            framebuffer,
            panel_y + 15 * scale as i32,
            "PILGRIM FALLEN",
            scale,
            DANGER,
        );
        vc_visual_draw_centered_text(
            framebuffer,
            panel_y + 39 * scale as i32,
            "SPACE TO RETURN",
            scale,
            TEXT,
        );
    }
}

#[cfg(test)]
mod v27_modal_tests {
    use super::*;

    #[test]
    fn death_panel_fits_inside_presentation_space() {
        let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
        let panel_width = 150 * scale;
        let panel_height = 62 * scale;
        let panel_x = (VC_VISUAL_PRESENTATION_WIDTH - panel_width) / 2;
        let panel_y = 118 * scale;
        assert!(panel_x + panel_width <= VC_VISUAL_PRESENTATION_WIDTH);
        assert!(panel_y + panel_height <= VC_VISUAL_PRESENTATION_HEIGHT);
    }
}
