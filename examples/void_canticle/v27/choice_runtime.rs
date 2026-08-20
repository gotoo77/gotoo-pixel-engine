struct VoidCanticleV27ChoicePresentation {
    presentation: VoidCanticleV27DirectPresentation,
}

impl VoidCanticleV27ChoicePresentation {
    fn new() -> Self {
        Self {
            presentation: VoidCanticleV27DirectPresentation::new(),
        }
    }

    fn render_hd_choice_override(&mut self, framebuffer: &mut Framebuffer) {
        let mode = self.presentation.visual_mode();
        if !matches!(mode, VcVisualMode::MutationChoice | VcVisualMode::SupportChoice) {
            return;
        }

        self.presentation.render_clean_background(framebuffer);
        let time = self.presentation.presentation_time;

        match mode {
            VcVisualMode::MutationChoice => {
                let v14 = self.presentation.game.game.v20().game.v14();
                if let Some(choice) = v14.mutation_choice.as_ref() {
                    vc27_render_mutation_showcase(framebuffer, v14, choice, time);
                }
            }
            VcVisualMode::SupportChoice => {
                vc27_render_support_showcase(framebuffer, &self.presentation.game, time);
            }
            _ => {}
        }
    }
}

impl Game for VoidCanticleV27ChoicePresentation {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let result = self.presentation.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        self.render_hd_choice_override(frame.framebuffer);
        GameResult::Continue
    }
}

pub fn run_v27_showcase_presentation_with_obs_mirror() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: format!(
                "Void Canticle {VC27_PRESENTATION_VERSION} - Gotoo Pixel Engine"
            ),
            framebuffer_width: VC_VISUAL_PRESENTATION_WIDTH,
            framebuffer_height: VC_VISUAL_PRESENTATION_HEIGHT,
            window_width,
            window_height,
        },
        gotoo_pixel_engine::ObsMirrorGame::from_env(
            VoidCanticleV27ChoicePresentation::new(),
            VC_VISUAL_PRESENTATION_WIDTH,
            VC_VISUAL_PRESENTATION_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod choice_runtime_tests {
    use super::*;

    #[test]
    fn choice_runtime_keeps_vc27_presentation_dimensions() {
        let game = VoidCanticleV27ChoicePresentation::new();
        assert_eq!(game.presentation.legacy_sink.width(), FRAMEBUFFER_WIDTH);
        assert_eq!(VC_VISUAL_PRESENTATION_WIDTH, FRAMEBUFFER_WIDTH * 2);
        assert_eq!(VC_VISUAL_PRESENTATION_HEIGHT, FRAMEBUFFER_HEIGHT * 2);
    }
}
