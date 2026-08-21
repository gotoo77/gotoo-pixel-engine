#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChoiceFocusKind {
    Chassis,
    Upgrade,
    Mutation,
    Support,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ChoiceFocus {
    kind: ChoiceFocusKind,
    index: usize,
}

#[derive(Clone, Copy)]
struct ChoiceSnapshot {
    focus: ChoiceFocus,
    label: &'static str,
    accent: Pixel,
    hover_sound: Option<SoundId>,
    confirm_sound: Option<SoundId>,
    synergy_mask_before: u8,
}

impl ChoiceSnapshot {
    fn from_profile(
        focus: ChoiceFocus,
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
struct ChoiceConfirmation {
    label: &'static str,
    accent: Pixel,
    synergy: Option<&'static str>,
    remaining: f32,
    duration: f32,
}

struct VoidCanticleChoicePresentation {
    presentation: VoidCanticlePresentation,
    last_focus: Option<ChoiceFocus>,
    confirmation: Option<ChoiceConfirmation>,
}

impl VoidCanticleChoicePresentation {
    fn new() -> Self {
        let mut presentation = VoidCanticlePresentation::new();
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
        let progression = self.presentation.game.presentation_progression();
        synergy_mask(progression.progression.build, progression.mutations)
    }

    fn current_choice_snapshot(&self) -> Option<ChoiceSnapshot> {
        let synergy_mask_before = self.current_synergy_mask();

        if self.presentation.chassis_selection_active() {
            let (index, chassis) = self
                .presentation
                .game
                .presentation_selected_chassis_choice()?;
            return Some(ChoiceSnapshot::from_profile(
                ChoiceFocus {
                    kind: ChoiceFocusKind::Chassis,
                    index,
                },
                vc27_chassis_profile(chassis),
                synergy_mask_before,
            ));
        }

        match self.presentation.visual_mode() {
            VcVisualMode::LevelChoice => {
                let progression = self.presentation.game.presentation_progression();
                let choice = progression.progression.level_choice.as_ref()?;
                let index = choice.menu.selected()?;
                let upgrade = choice.offers.get(index).copied()?;
                Some(ChoiceSnapshot::from_profile(
                    ChoiceFocus {
                        kind: ChoiceFocusKind::Upgrade,
                        index,
                    },
                    vc27_upgrade_profile(upgrade),
                    synergy_mask_before,
                ))
            }
            VcVisualMode::MutationChoice => {
                let progression = self.presentation.game.presentation_progression();
                let choice = progression.mutation_choice.as_ref()?;
                let index = choice.menu.selected()?;
                let mutation = choice.offers.get(index).copied()?;
                Some(ChoiceSnapshot::from_profile(
                    ChoiceFocus {
                        kind: ChoiceFocusKind::Mutation,
                        index,
                    },
                    vc27_mutation_profile(mutation),
                    synergy_mask_before,
                ))
            }
            VcVisualMode::SupportChoice => {
                let (index, augment) = self
                    .presentation
                    .game
                    .presentation_selected_support_choice()?;
                Some(ChoiceSnapshot::from_profile(
                    ChoiceFocus {
                        kind: ChoiceFocusKind::Support,
                        index,
                    },
                    vc27_support_profile(augment),
                    synergy_mask_before,
                ))
            }
            _ => None,
        }
    }

    fn current_choice_focus(&self) -> Option<(ChoiceFocus, Option<SoundId>)> {
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
        before: Option<ChoiceSnapshot>,
    ) -> (bool, Option<SoundId>) {
        let Some(before) = before else {
            return (false, None);
        };
        let after_focus = self.current_choice_snapshot().map(|snapshot| snapshot.focus);
        if !choice_was_confirmed(before.focus, after_focus) {
            return (false, None);
        }

        let synergy = vc27_new_synergy_name(before.synergy_mask_before, self.current_synergy_mask());
        let duration = if synergy.is_some() {
            VC27_SYNERGY_REVEAL_DURATION
        } else {
            VC27_CHOICE_CONFIRM_DURATION
        };
        self.confirmation = Some(ChoiceConfirmation {
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

impl Game for VoidCanticleChoicePresentation {
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

fn choice_was_confirmed(before: ChoiceFocus, after: Option<ChoiceFocus>) -> bool {
    match after {
        Some(after) => after.kind != before.kind,
        None => true,
    }
}

#[cfg(test)]
mod choice_runtime_tests {
    use super::*;

    #[test]
    fn choice_runtime_keeps_presentation_dimensions() {
        let game = VoidCanticleChoicePresentation::new();
        assert_eq!(game.presentation.legacy_sink.width(), FRAMEBUFFER_WIDTH);
        assert_eq!(VC_VISUAL_PRESENTATION_WIDTH, FRAMEBUFFER_WIDTH * 2);
        assert_eq!(VC_VISUAL_PRESENTATION_HEIGHT, FRAMEBUFFER_HEIGHT * 2);
    }

    #[test]
    fn hover_event_is_edge_triggered_per_focused_choice() {
        let mut game = VoidCanticleChoicePresentation::new();
        assert_eq!(
            game.choice_hover_event(),
            Some(Vc27ChoiceArtId::Bulwark.hover_override_sound())
        );
        assert_eq!(game.choice_hover_event(), None);

        game.presentation
            .game
            .presentation_select_next_chassis_for_test();
        assert_eq!(game.choice_hover_event(), Some(VC27_CHOICE_HOVER_SOUND));
        assert_eq!(game.choice_hover_event(), None);
    }

    #[test]
    fn chassis_snapshot_uses_shared_choice_profile() {
        let profile = vc27_chassis_profile(ExosuitChassis::Bulwark);
        let snapshot = ChoiceSnapshot::from_profile(
            ChoiceFocus {
                kind: ChoiceFocusKind::Chassis,
                index: 0,
            },
            profile,
            0,
        );
        assert_eq!(snapshot.label, profile.label());
        assert_eq!(snapshot.accent, profile.accent());
        assert_eq!(snapshot.hover_sound, profile.hover_sound());
        assert_eq!(snapshot.confirm_sound, profile.confirm_sound());
    }

    #[test]
    fn profile_snapshot_carries_visual_and_audio_identity() {
        let profile = vc27_mutation_profile(MutationKind::DeathNova);
        let snapshot = ChoiceSnapshot::from_profile(
            ChoiceFocus {
                kind: ChoiceFocusKind::Mutation,
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
        let before = ChoiceFocus {
            kind: ChoiceFocusKind::Upgrade,
            index: 1,
        };
        assert!(!choice_was_confirmed(
            before,
            Some(ChoiceFocus {
                kind: ChoiceFocusKind::Upgrade,
                index: 2,
            })
        ));
        assert!(choice_was_confirmed(
            before,
            Some(ChoiceFocus {
                kind: ChoiceFocusKind::Mutation,
                index: 0,
            })
        ));
        assert!(choice_was_confirmed(before, None));
    }

    #[test]
    fn current_choice_runtime_does_not_traverse_legacy_wrapper_chain() {
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/void_canticle/presentation/choice_runtime.rs"
        ));
        let forbidden = [".game", ".game"].concat();
        assert!(!source.contains(&forbidden));
    }
}
