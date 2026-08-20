#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Vc27ChoiceFocusKind {
    Chassis,
    Upgrade,
    Mutation,
    Support,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Vc27ChoiceFocus {
    kind: Vc27ChoiceFocusKind,
    index: usize,
}

struct VoidCanticleV27ChoicePresentation {
    presentation: VoidCanticleV27DirectPresentation,
    last_focus: Option<Vc27ChoiceFocus>,
}

impl VoidCanticleV27ChoicePresentation {
    fn new() -> Self {
        let mut presentation = VoidCanticleV27DirectPresentation::new();
        vc27_register_choice_hover_sound(
            &mut presentation.game.combat_model_mut().base_mut().sounds,
        );
        Self {
            presentation,
            last_focus: None,
        }
    }

    fn current_choice_focus(&self) -> Option<(Vc27ChoiceFocus, Option<SoundId>)> {
        if self.presentation.chassis_selection_active() {
            let selector = &self.presentation.game.game.game.game.game;
            let index = selector.menu.selected()?;
            return Some((
                Vc27ChoiceFocus {
                    kind: Vc27ChoiceFocusKind::Chassis,
                    index,
                },
                Some(VC27_CHOICE_HOVER_SOUND),
            ));
        }

        match self.presentation.visual_mode() {
            VcVisualMode::LevelChoice => {
                let v14 = self.presentation.game.game.v20().game.v14();
                let choice = v14.progression.level_choice.as_ref()?;
                let index = choice.menu.selected()?;
                let upgrade = choice.offers.get(index).copied()?;
                Some((
                    Vc27ChoiceFocus {
                        kind: Vc27ChoiceFocusKind::Upgrade,
                        index,
                    },
                    vc27_upgrade_assets(upgrade).hover_sound(),
                ))
            }
            VcVisualMode::MutationChoice => {
                let v14 = self.presentation.game.game.v20().game.v14();
                let choice = v14.mutation_choice.as_ref()?;
                let index = choice.menu.selected()?;
                let mutation = choice.offers.get(index).copied()?;
                Some((
                    Vc27ChoiceFocus {
                        kind: Vc27ChoiceFocusKind::Mutation,
                        index,
                    },
                    vc27_mutation_assets(mutation).hover_sound(),
                ))
            }
            VcVisualMode::SupportChoice => {
                let index = self.presentation.game.menu.selected()?;
                let augment = VC23_SUSTAIN_AUGMENTS.get(index).copied()?;
                Some((
                    Vc27ChoiceFocus {
                        kind: Vc27ChoiceFocusKind::Support,
                        index,
                    },
                    vc27_support_assets(augment).hover_sound(),
                ))
            }
            _ => None,
        }
    }

    fn choice_hover_event(&mut self) -> Option<SoundId> {
        let Some((focus, sound)) = self.current_choice_focus() else {
            self.last_focus = None;
            return None;
        };

        if self.last_focus == Some(focus) {
            return None;
        }

        self.last_focus = Some(focus);
        sound
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

        if let Some(sound) = self.choice_hover_event() {
            let _ = self
                .presentation
                .game
                .combat_model_mut()
                .base_mut()
                .sounds
                .play(frame.audio, sound);
        }

        self.render_hd_choice_override(frame.framebuffer);
        GameResult::Continue
    }
}

pub fn run_v27_showcase_presentation_with_obs_mirror() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: format!("Void Canticle {VC27_PRESENTATION_VERSION} - Gotoo Pixel Engine"),
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

    #[test]
    fn hover_event_is_edge_triggered_per_focused_choice() {
        let mut game = VoidCanticleV27ChoicePresentation::new();
        assert_eq!(game.choice_hover_event(), Some(VC27_CHOICE_HOVER_SOUND));
        assert_eq!(game.choice_hover_event(), None);

        let selector = &mut game.presentation.game.game.game.game.game;
        selector.menu.select_next();
        assert_eq!(game.choice_hover_event(), Some(VC27_CHOICE_HOVER_SOUND));
        assert_eq!(game.choice_hover_event(), None);
    }
}
