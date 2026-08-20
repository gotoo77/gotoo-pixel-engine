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

#[derive(Clone, Copy)]
struct Vc27ChoiceSnapshot {
    focus: Vc27ChoiceFocus,
    label: &'static str,
    accent: Pixel,
    hover_sound: Option<SoundId>,
    confirm_sound: Option<SoundId>,
    synergy_mask_before: u8,
}

impl Vc27ChoiceSnapshot {
    fn from_profile(
        focus: Vc27ChoiceFocus,
        profile: Vc27ChoiceProfile<'static>,
        synergy_mask_before: u8,
    ) -> Self {
        Self {
            focus,
            label: profile.label(),
            accent: profile.accent(),
            hover_sound: profile.hover_sound(),
            confirm_sound: profile.confirm_sound(),
            synergy_mask_before,
        }
    }
}

#[derive(Clone, Copy)]
struct Vc27ChoiceConfirmation {
    label: &'static str,
    accent: Pixel,
    synergy: Option<&'static str>,
    remaining: f32,
    duration: f32,
}

struct VoidCanticleV27ChoicePresentation {
    presentation: VoidCanticleV27DirectPresentation,
    last_focus: Option<Vc27ChoiceFocus>,
    confirmation: Option<Vc27ChoiceConfirmation>,
}

impl VoidCanticleV27ChoicePresentation {
    fn new() -> Self {
        let mut presentation = VoidCanticleV27DirectPresentation::new();
        vc27_register_choice_sounds(
            &mut presentation.game.combat_model_mut().base_mut().sounds,
        );
        Self {
            presentation,
            last_focus: None,
            confirmation: None,
        }
    }

    fn current_synergy_mask(&self) -> u8 {
        let v14 = self.presentation.game.game.v20().game.v14();
        synergy_mask(v14.progression.build, v14.mutations)
    }

    fn current_choice_snapshot(&self) -> Option<Vc27ChoiceSnapshot> {
        let synergy_mask_before = self.current_synergy_mask();

        if self.presentation.chassis_selection_active() {
            let selector = &self.presentation.game.game.game.game.game;
            let index = selector.menu.selected()?;
            let chassis = VC22_CHASSIS.get(index).copied()?;
            return Some(Vc27ChoiceSnapshot {
                focus: Vc27ChoiceFocus {
                    kind: Vc27ChoiceFocusKind::Chassis,
                    index,
                },
                label: chassis.name(),
                accent: vc27_chassis_accent(chassis),
                hover_sound: Some(VC27_CHOICE_HOVER_SOUND),
                confirm_sound: Some(VC27_CHOICE_CONFIRM_SOUND),
                synergy_mask_before,
            });
        }

        match self.presentation.visual_mode() {
            VcVisualMode::LevelChoice => {
                let v14 = self.presentation.game.game.v20().game.v14();
                let choice = v14.progression.level_choice.as_ref()?;
                let index = choice.menu.selected()?;
                let upgrade = choice.offers.get(index).copied()?;
                Some(Vc27ChoiceSnapshot::from_profile(
                    Vc27ChoiceFocus {
                        kind: Vc27ChoiceFocusKind::Upgrade,
                        index,
                    },
                    vc27_upgrade_profile(upgrade),
                    synergy_mask_before,
                ))
            }
            VcVisualMode::MutationChoice => {
                let v14 = self.presentation.game.game.v20().game.v14();
                let choice = v14.mutation_choice.as_ref()?;
                let index = choice.menu.selected()?;
                let mutation = choice.offers.get(index).copied()?;
                Some(Vc27ChoiceSnapshot::from_profile(
                    Vc27ChoiceFocus {
                        kind: Vc27ChoiceFocusKind::Mutation,
                        index,
                    },
                    vc27_mutation_profile(mutation),
                    synergy_mask_before,
                ))
            }
            VcVisualMode::SupportChoice => {
                let index = self.presentation.game.menu.selected()?;
                let augment = VC23_SUSTAIN_AUGMENTS.get(index).copied()?;
                Some(Vc27ChoiceSnapshot::from_profile(
                    Vc27ChoiceFocus {
                        kind: Vc27ChoiceFocusKind::Support,
                        index,
                    },
                    vc27_support_profile(augment),
                    synergy_mask_before,
                ))
            }
            _ => None,
        }
    }

    fn current_choice_focus(&self) -> Option<(Vc27ChoiceFocus, Option<SoundId>)> {
        self.current_choice_snapshot()
            .map(|snapshot| (snapshot.focus, snapshot.hover_sound))
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

    fn capture_confirmation(
        &mut self,
        before: Option<Vc27ChoiceSnapshot>,
    ) -> (bool, Option<SoundId>) {
        let Some(before) = before else {
            return (false, None);
        };
        let after_focus = self.current_choice_snapshot().map(|snapshot| snapshot.focus);
        if !vc27_choice_was_confirmed(before.focus, after_focus) {
            return (false, None);
        }

        let synergy = vc27_new_synergy_name(before.synergy_mask_before, self.current_synergy_mask());
        let duration = if synergy.is_some() {
            VC27_SYNERGY_REVEAL_DURATION
        } else {
            VC27_CHOICE_CONFIRM_DURATION
        };
        self.confirmation = Some(Vc27ChoiceConfirmation {
            label: before.label,
            accent: before.accent,
            synergy,
            remaining: duration,
            duration,
        });
        self.last_focus = after_focus;
        (true, before.confirm_sound)
    }

    fn tick_confirmation(&mut self, dt: f32) {
        let mut expired = false;
        if let Some(confirmation) = self.confirmation.as_mut() {
            confirmation.remaining = (confirmation.remaining - dt).max(0.0);
            expired = confirmation.remaining <= 0.0;
        }
        if expired {
            self.confirmation = None;
        }
    }

    fn render_confirmation(&self, framebuffer: &mut Framebuffer) {
        let Some(confirmation) = self.confirmation else {
            return;
        };
        vc27_render_choice_confirmation(
            framebuffer,
            confirmation.label,
            confirmation.accent,
            confirmation.synergy,
            confirmation.remaining,
            confirmation.duration,
        );
    }
}

impl Game for VoidCanticleV27ChoicePresentation {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let dt = frame.delta_time.as_secs_f32().min(0.05);
        self.tick_confirmation(dt);
        let before = self.current_choice_snapshot();

        let result = self.presentation.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        let (confirmed, confirm_sound) = self.capture_confirmation(before);
        if let Some(sound) = confirm_sound {
            let _ = self
                .presentation
                .game
                .combat_model_mut()
                .base_mut()
                .sounds
                .play(frame.audio, sound);
        }

        if !confirmed
            && let Some(sound) = self.choice_hover_event()
        {
            let _ = self
                .presentation
                .game
                .combat_model_mut()
                .base_mut()
                .sounds
                .play(frame.audio, sound);
        }

        self.render_confirmation(frame.framebuffer);
        GameResult::Continue
    }
}

fn vc27_choice_was_confirmed(
    before: Vc27ChoiceFocus,
    after: Option<Vc27ChoiceFocus>,
) -> bool {
    match after {
        Some(after) => after.kind != before.kind,
        None => true,
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

    #[test]
    fn profile_snapshot_carries_visual_and_audio_identity() {
        let profile = vc27_mutation_profile(MutationKind::DeathNova);
        let snapshot = Vc27ChoiceSnapshot::from_profile(
            Vc27ChoiceFocus {
                kind: Vc27ChoiceFocusKind::Mutation,
                index: 0,
            },
            profile,
            0,
        );
        assert_eq!(snapshot.label, mutation_name(MutationKind::DeathNova));
        assert_eq!(snapshot.accent, profile.accent());
        assert_eq!(snapshot.hover_sound, profile.hover_sound());
        assert_eq!(snapshot.confirm_sound, profile.confirm_sound());
    }

    #[test]
    fn confirmation_requires_choice_kind_to_close_or_change() {
        let before = Vc27ChoiceFocus {
            kind: Vc27ChoiceFocusKind::Upgrade,
            index: 1,
        };
        assert!(!vc27_choice_was_confirmed(
            before,
            Some(Vc27ChoiceFocus {
                kind: Vc27ChoiceFocusKind::Upgrade,
                index: 2,
            })
        ));
        assert!(vc27_choice_was_confirmed(
            before,
            Some(Vc27ChoiceFocus {
                kind: Vc27ChoiceFocusKind::Mutation,
                index: 0,
            })
        ));
        assert!(vc27_choice_was_confirmed(before, None));
    }
}
