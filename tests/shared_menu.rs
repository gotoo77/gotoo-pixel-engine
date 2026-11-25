use gotoo_pixel_engine::{
    Audio, AudioBus, Framebuffer, NoopAudio,
    ui::{
        UiTheme,
        experimental::UiNavInput,
        menu::{AudioSettings, MenuBack, MenuStack, PauseSettingsMenu, SettingsIntent},
    },
};

#[test]
fn public_stack_retains_root_and_requests_close() {
    let mut stack = MenuStack::new("consumer-root");
    stack.push("consumer-options");
    assert_eq!(stack.depth(), 2);
    assert_eq!(stack.back(), MenuBack::ReturnedToParent);
    assert_eq!(stack.current(), &"consumer-root");
    assert_eq!(stack.back(), MenuBack::CloseRequested);
    assert_eq!(stack.depth(), 1);
}

#[test]
fn public_menu_only_emits_intents_and_applies_four_audio_values_explicitly() {
    let mut menu = PauseSettingsMenu::new();
    let mut framebuffer = Framebuffer::new(640, 360);
    let mut audio = NoopAudio::default();
    menu.audio = AudioSettings {
        master: 0.8,
        music: 0.6,
        sfx: 0.4,
        ui: 0.2,
    };
    let intent = menu.update(
        &mut framebuffer,
        UiNavInput {
            confirm: true,
            ..Default::default()
        },
        UiTheme::default(),
    );
    assert_eq!(intent, SettingsIntent::Resume);
    assert_eq!(audio.master_volume(), 1.0);
    assert_eq!(audio.bus_volume(AudioBus::Ui), 1.0);
    menu.apply_audio(&mut audio).unwrap();
    assert_eq!(audio.master_volume(), 0.8);
    assert_eq!(audio.bus_volume(AudioBus::Music), 0.6);
    assert_eq!(audio.bus_volume(AudioBus::Sfx), 0.4);
    assert_eq!(audio.bus_volume(AudioBus::Ui), 0.2);
}

use gotoo_pixel_engine::ui::experimental::{self, UiInput, UiPointerInput, UiStateStore};
use gotoo_pixel_engine::ui::{
    UiStyleOverride, UiStyleSheet,
    menu::{SettingsLabels, SettingsPage},
};
use gotoo_pixel_engine::{AudioError, Pixel, Rect, Size, SoundId, Touch, TouchPhase};

fn nav(menu: &mut PauseSettingsMenu, input: UiNavInput) -> SettingsIntent {
    menu.update(&mut Framebuffer::new(640, 360), input, UiTheme::default())
}

fn enter_audio(menu: &mut PauseSettingsMenu) {
    nav(menu, UiNavInput::default());
    nav(
        menu,
        UiNavInput {
            down: true,
            ..Default::default()
        },
    );
    nav(
        menu,
        UiNavInput {
            confirm: true,
            ..Default::default()
        },
    );
    assert_eq!(menu.current(), SettingsPage::Settings);
    nav(menu, UiNavInput::default());
    nav(
        menu,
        UiNavInput {
            confirm: true,
            ..Default::default()
        },
    );
    assert_eq!(menu.current(), SettingsPage::Audio);
    nav(menu, UiNavInput::default());
}

#[test]
fn nested_back_restores_focus_and_invalidates_previous_page_geometry() {
    let mut menu = PauseSettingsMenu::new();
    enter_audio(&mut menu);
    assert_eq!(
        menu.presentation_snapshot()
            .controls
            .iter()
            .flatten()
            .count(),
        5
    );
    nav(
        &mut menu,
        UiNavInput {
            cancel: true,
            ..Default::default()
        },
    );
    assert_eq!(menu.current(), SettingsPage::Settings);
    assert_eq!(menu.presentation_snapshot().controls, [None; 5]);
    nav(
        &mut menu,
        UiNavInput {
            cancel: true,
            ..Default::default()
        },
    );
    nav(&mut menu, UiNavInput::default());
    assert_eq!(menu.current(), SettingsPage::Pause);
    assert_eq!(menu.presentation_snapshot().selected, 1);
    nav(
        &mut menu,
        UiNavInput {
            down: true,
            ..Default::default()
        },
    );
    assert_eq!(
        nav(
            &mut menu,
            UiNavInput {
                confirm: true,
                ..Default::default()
            }
        ),
        SettingsIntent::Exit
    );
    menu.reset_to_pause();
    assert_eq!(menu.presentation_snapshot().controls, [None; 5]);
    nav(&mut menu, UiNavInput::default());
    assert_eq!(menu.presentation_snapshot().selected, 0);
}

#[test]
fn four_sliders_adjust_independently_and_clamp_at_both_ends() {
    let mut menu = PauseSettingsMenu::new();
    enter_audio(&mut menu);
    for _ in 0..3 {
        nav(
            &mut menu,
            UiNavInput {
                down: true,
                ..Default::default()
            },
        );
    }
    assert_eq!(menu.presentation_snapshot().selected, 3);
    nav(
        &mut menu,
        UiNavInput {
            left: true,
            ..Default::default()
        },
    );
    assert!((menu.audio.ui - 0.95).abs() < 0.000001);
    assert_eq!(
        (menu.audio.master, menu.audio.music, menu.audio.sfx),
        (1.0, 1.0, 1.0)
    );
    for _ in 0..30 {
        nav(
            &mut menu,
            UiNavInput {
                left: true,
                ..Default::default()
            },
        );
    }
    assert_eq!(menu.audio.ui, 0.0);
    for _ in 0..30 {
        nav(
            &mut menu,
            UiNavInput {
                right: true,
                ..Default::default()
            },
        );
    }
    assert_eq!(menu.audio.ui, 1.0);
    let before = menu.audio;
    nav(
        &mut menu,
        UiNavInput {
            left: true,
            right: true,
            ..Default::default()
        },
    );
    assert_eq!(menu.audio, before);
    nav(
        &mut menu,
        UiNavInput {
            down: true,
            ..Default::default()
        },
    );
    assert_eq!(menu.presentation_snapshot().selected, 4);
    assert_eq!(
        nav(
            &mut menu,
            UiNavInput {
                confirm: true,
                ..Default::default()
            }
        ),
        SettingsIntent::None
    );
    assert_eq!(menu.current(), SettingsPage::Settings);
}

#[test]
fn cancel_wins_over_concurrent_edit_and_activation() {
    let mut menu = PauseSettingsMenu::new();
    enter_audio(&mut menu);
    let before = menu.audio;
    nav(
        &mut menu,
        UiNavInput {
            cancel: true,
            left: true,
            confirm: true,
            ..Default::default()
        },
    );
    assert_eq!(menu.audio, before);
    assert_eq!(menu.current(), SettingsPage::Settings);
    nav(
        &mut menu,
        UiNavInput {
            cancel: true,
            confirm: true,
            ..Default::default()
        },
    );
    assert_eq!(menu.current(), SettingsPage::Pause);
    nav(
        &mut menu,
        UiNavInput {
            down: true,
            ..Default::default()
        },
    );
    assert_eq!(menu.presentation_snapshot().selected, 2);
    assert_eq!(
        nav(
            &mut menu,
            UiNavInput {
                cancel: true,
                confirm: true,
                ..Default::default()
            }
        ),
        SettingsIntent::Resume
    );
}

fn center(rect: Rect) -> (i32, i32) {
    (
        rect.x + rect.width as i32 / 2,
        rect.y + rect.height as i32 / 2,
    )
}

#[test]
fn pointer_and_touch_use_snapshot_hitboxes_and_do_not_activate_twice() {
    let mut menu = PauseSettingsMenu::new();
    let mut fb = Framebuffer::new(640, 360);
    menu.update(&mut fb, UiNavInput::default(), UiTheme::default());
    let point = center(menu.presentation_snapshot().controls[1].unwrap());
    let pointer = |pressed, released| UiInput {
        pointer: UiPointerInput {
            position: Some(point),
            pressed,
            released,
        },
        ..Default::default()
    };
    assert_eq!(
        menu.update_with_input(&mut fb, pointer(true, false), UiTheme::default()),
        SettingsIntent::None
    );
    assert_eq!(menu.current(), SettingsPage::Pause);
    assert_eq!(
        menu.update_with_input(
            &mut fb,
            UiInput {
                nav: UiNavInput {
                    confirm: true,
                    ..Default::default()
                },
                ..pointer(false, true)
            },
            UiTheme::default()
        ),
        SettingsIntent::None
    );
    assert_eq!(menu.current(), SettingsPage::Settings);
    menu.update(&mut fb, UiNavInput::default(), UiTheme::default());
    assert_eq!(menu.current(), SettingsPage::Settings);
    let point = center(menu.presentation_snapshot().controls[0].unwrap());
    for phase in [TouchPhase::Started, TouchPhase::Ended] {
        let touches = [Touch {
            id: 9,
            phase,
            position: Some(point),
        }];
        menu.update_with_input(
            &mut fb,
            UiInput {
                touches: &touches,
                ..Default::default()
            },
            UiTheme::default(),
        );
    }
    assert_eq!(menu.current(), SettingsPage::Audio);
    // A release without a preceding press on the new page cannot activate Back.
    assert_eq!(
        menu.update_with_input(&mut fb, pointer(false, true), UiTheme::default()),
        SettingsIntent::None
    );
    assert_eq!(menu.current(), SettingsPage::Audio);
}

#[test]
fn translated_labels_keep_semantic_focus_and_styles_drive_geometry_and_pixels() {
    let mut menu = PauseSettingsMenu::new();
    let mut fb = Framebuffer::new(640, 360);
    nav(&mut menu, UiNavInput::default());
    nav(
        &mut menu,
        UiNavInput {
            down: true,
            ..Default::default()
        },
    );
    let mut style = UiStyleSheet::default();
    let color = Pixel::rgb(101, 20, 80);
    style.button.focused = UiStyleOverride {
        background: Some(color),
        ..Default::default()
    };
    let theme = UiTheme {
        row_height: 40,
        ..Default::default()
    };
    let title = String::from("EN PAUSE");
    let labels = SettingsLabels {
        paused: &title,
        resume: "REPRENDRE",
        settings: "OPTIONS",
        quit: "QUITTER",
        ..Default::default()
    };
    menu.update_with_input_styled_labels(&mut fb, UiInput::default(), theme, style, labels);
    assert_eq!(menu.presentation_snapshot().selected, 1);
    let rect = menu.presentation_snapshot().controls[1].unwrap();
    assert_eq!(rect.height, 40);
    assert_eq!(fb.pixel(rect.x + 2, rect.y + 2), Some(color));
    let wide = menu.presentation_snapshot().controls[0].unwrap();
    fb = Framebuffer::new(320, 180);
    menu.update_styled(&mut fb, UiNavInput::default(), theme, style);
    let narrow = menu.presentation_snapshot().controls[0].unwrap();
    assert!(narrow.width < wide.width);
    assert_eq!(menu.presentation_snapshot().selected, 1);
}

#[test]
fn output_geometry_rejects_handles_from_previous_transactions() {
    let mut state = UiStateStore::default();
    let size = Size {
        width: 320,
        height: 180,
    };
    let (first, old) = experimental::run_headless(
        size,
        &mut state,
        UiNavInput::default(),
        UiTheme::default(),
        |ui| ui.button("ACTION"),
    );
    assert!(first.rect(old).is_some());
    let (second, new) = experimental::run_headless(
        size,
        &mut state,
        UiNavInput::default(),
        UiTheme::default(),
        |ui| ui.button("ACTION"),
    );
    assert_eq!(second.rect(old), None);
    assert!(second.rect(new).is_some());
}

struct FailingAudio {
    inner: NoopAudio,
    fail: usize,
    calls: usize,
    error: AudioError,
}
impl Audio for FailingAudio {
    fn register_wav(&mut self, id: SoundId, bytes: &[u8]) -> Result<(), AudioError> {
        self.inner.register_wav(id, bytes)
    }
    fn play(&mut self, id: SoundId) -> Result<(), AudioError> {
        self.inner.play(id)
    }
    fn set_master_volume(&mut self, value: f32) -> Result<(), AudioError> {
        self.calls += 1;
        if self.fail == self.calls {
            Err(self.error.clone())
        } else {
            self.inner.set_master_volume(value)
        }
    }
    fn set_bus_volume(&mut self, bus: AudioBus, value: f32) -> Result<(), AudioError> {
        self.calls += 1;
        if self.fail == self.calls {
            Err(self.error.clone())
        } else {
            self.inner.set_bus_volume(bus, value)
        }
    }
}

#[test]
fn audio_errors_are_returned_unchanged_and_stop_at_each_failed_setter() {
    let mut menu = PauseSettingsMenu::new();
    menu.audio = AudioSettings {
        master: 0.8,
        music: 0.6,
        sfx: 0.4,
        ui: 0.2,
    };
    let proposed = menu.audio;
    for (fail, expected) in [
        (1, [1.0, 1.0, 1.0, 1.0]),
        (2, [0.8, 1.0, 1.0, 1.0]),
        (3, [0.8, 0.6, 1.0, 1.0]),
        (4, [0.8, 0.6, 0.4, 1.0]),
    ] {
        let error = AudioError::new(format!("backend failure {fail}"));
        let mut backend = FailingAudio {
            inner: NoopAudio::default(),
            fail,
            calls: 0,
            error: error.clone(),
        };
        assert_eq!(menu.apply_audio(&mut backend), Err(error));
        assert_eq!(backend.calls, fail);
        assert_eq!(
            [
                backend.inner.master_volume(),
                backend.inner.bus_volume(AudioBus::Music),
                backend.inner.bus_volume(AudioBus::Sfx),
                backend.inner.bus_volume(AudioBus::Ui)
            ],
            expected
        );
        assert_eq!(menu.audio, proposed);
    }
}

#[test]
fn invalid_audio_proposals_preserve_backend_validation_and_values() {
    for value in [f32::NAN, f32::INFINITY, -0.1, 1.1] {
        let mut menu = PauseSettingsMenu::new();
        menu.audio.music = value;
        let mut backend = NoopAudio::default();
        assert!(menu.apply_audio(&mut backend).is_err());
        assert_eq!(backend.bus_volume(AudioBus::Music), 1.0);
        assert_eq!(backend.bus_volume(AudioBus::Ui), 1.0);
    }
}

#[cfg(feature = "outline-fonts")]
#[test]
fn outline_uses_same_interaction_and_geometry_with_nonfinite_size_normalized() {
    use gotoo_pixel_engine::ui::fonts::BuiltinOutlineFont;
    let mut menu = PauseSettingsMenu::new().with_outline_font(
        BuiltinOutlineFont::Exo2.load().unwrap(),
        f32::NAN,
        0,
    );
    let mut fb = Framebuffer::new(640, 360);
    menu.update(&mut fb, UiNavInput::default(), UiTheme::default());
    let point = center(menu.presentation_snapshot().controls[1].unwrap());
    for (pressed, released) in [(true, false), (false, true)] {
        menu.update_with_input(
            &mut fb,
            UiInput {
                pointer: UiPointerInput {
                    position: Some(point),
                    pressed,
                    released,
                },
                ..Default::default()
            },
            UiTheme::default(),
        );
    }
    assert_eq!(menu.current(), SettingsPage::Settings);
    assert_eq!(menu.presentation_snapshot().controls, [None; 5]);
    menu.update(&mut fb, UiNavInput::default(), UiTheme::default());
    assert!(menu.presentation_snapshot().controls[0].is_some());
}

#[test]
fn reset_retains_all_consumer_audio_values_without_applying_them() {
    let mut menu = PauseSettingsMenu::new();
    let audio = AudioSettings {
        master: 0.13,
        music: 0.27,
        sfx: 0.46,
        ui: 0.71,
    };
    menu.audio = audio;
    enter_audio(&mut menu);
    menu.reset_to_pause();
    assert_eq!(menu.audio, audio);
    assert_eq!(menu.current(), SettingsPage::Pause);
    assert_eq!(menu.presentation_snapshot().controls, [None; 5]);
}

#[test]
fn consumer_pages_compose_existing_widgets_with_domain_values_outside_core() {
    let mut pages = MenuStack::new("consumer-main");
    let mut fb = Framebuffer::new(640, 360);
    let (output, options) = experimental::run(
        &mut fb,
        pages.ui_state_mut(),
        UiNavInput {
            confirm: true,
            ..Default::default()
        },
        UiTheme::default(),
        |ui| ui.keyed("options", |ui| ui.button("GAME OPTIONS")),
    );
    assert!(output.activated(options));
    pages.push("consumer-options");
    let old_value = false;
    let (output, option) = experimental::run(
        &mut fb,
        pages.ui_state_mut(),
        UiNavInput {
            confirm: true,
            ..Default::default()
        },
        UiTheme::default(),
        |ui| ui.keyed("domain-option", |ui| ui.toggle("GAME RULE", old_value)),
    );
    assert_eq!(output.changed(option), Some(true));
    assert!(
        !old_value,
        "UI proposes values; the consumer owns their acceptance"
    );
    assert_eq!(pages.back(), MenuBack::ReturnedToParent);
    assert_eq!(pages.current(), &"consumer-main");
}
