//! The actual presentation example's consumer, tested with crate-private input injection.
use crate as gpe;
#[path = "../../examples/support/menu_consumer.rs"]
mod consumer;

use crate::ui::menu::SettingsPage;
use crate::{
    Frame, Framebuffer, GamepadButton, GamepadId, Input, Key, NoopAudio, NoopStorage, Size,
    Viewport,
};
use consumer::{MenuConsumer, MenuFrame};

#[derive(Default)]
struct Demo {
    owner: MenuConsumer,
    ticks: usize,
}
impl Demo {
    fn tick(&mut self, input: &Input) -> MenuFrame {
        let size = Size {
            width: 640,
            height: 360,
        };
        let mut fb = Framebuffer::new(size.width, size.height);
        let mut audio = NoopAudio::default();
        let mut storage = NoopStorage;
        let outcome = self.owner.update(&mut Frame {
            framebuffer: &mut fb,
            input,
            audio: &mut audio,
            storage: &mut storage,
            delta_time: std::time::Duration::from_millis(16),
            surface_size: size,
            viewport: Viewport::new(size, size),
        });
        if outcome == MenuFrame::Advance {
            self.ticks += 1;
        }
        outcome
    }
    fn tap(&mut self, input: &mut Input, key: Key) -> MenuFrame {
        input.advance_frame();
        input.press_key(key);
        let outcome = self.tick(input);
        input.advance_frame();
        input.release_key(key);
        self.tick(input);
        input.advance_frame();
        outcome
    }
}

#[test]
fn shared_settings_pause_navigation_keeps_child_suspended() {
    let mut demo = Demo::default();
    let mut input = Input::default();
    demo.tap(&mut input, Key::Escape);
    demo.tap(&mut input, Key::Down);
    demo.tap(&mut input, Key::Space);
    assert_eq!(demo.owner.menu.current(), SettingsPage::Settings);
    assert_eq!(demo.ticks, 0);
    demo.tap(&mut input, Key::Space);
    assert_eq!(demo.owner.menu.current(), SettingsPage::Audio);
    assert_eq!(demo.ticks, 0);
    demo.tap(&mut input, Key::Escape);
    assert_eq!(demo.owner.menu.current(), SettingsPage::Settings);
    assert_eq!(demo.ticks, 0);
}

#[test]
fn shared_settings_resume_gate_waits_for_confirmation_release() {
    let mut demo = Demo::default();
    let mut input = Input::default();
    demo.tap(&mut input, Key::Escape);
    input.press_key(Key::Space);
    demo.tick(&input);
    assert_eq!(demo.ticks, 0);
    input.advance_frame();
    assert_eq!(demo.tick(&input), MenuFrame::Suspended);
    assert_eq!(demo.ticks, 0);
    input.release_key(Key::Space);
    demo.tick(&input);
    assert_eq!(demo.ticks, 0);
    input.advance_frame();
    demo.tick(&input);
    assert_eq!(demo.ticks, 1);
}

#[test]
fn entry_consumes_toggle_and_held_confirm_does_not_resume_or_open_twice() {
    let mut demo = Demo::default();
    let mut input = Input::default();
    input.press_key(Key::Escape);
    input.press_key(Key::Space);
    demo.tick(&input);
    input.advance_frame();
    input.release_key(Key::Escape);
    for _ in 0..4 {
        demo.tick(&input);
        input.advance_frame();
    }
    assert_eq!(demo.ticks, 0);
    assert_eq!(demo.owner.menu.current(), SettingsPage::Pause);
    input.release_key(Key::Space);
    demo.tick(&input);
    input.advance_frame();
    demo.tap(&mut input, Key::Down);
    input.press_key(Key::Space);
    demo.tick(&input);
    input.advance_frame();
    for _ in 0..4 {
        demo.tick(&input);
        input.advance_frame();
    }
    assert_eq!(demo.owner.menu.current(), SettingsPage::Settings);
    assert_eq!(demo.ticks, 0);
}

#[test]
fn resume_requires_all_keyboard_and_pad_sources_and_a_clean_release_frame() {
    let mut demo = Demo::default();
    let mut input = Input::default();
    let first = GamepadId::new(1);
    let second = GamepadId::new(2);
    demo.tap(&mut input, Key::Escape);
    input.press_key(Key::Space);
    input.press_key(Key::Left);
    input.set_gamepad_button(first, GamepadButton::South, true);
    input.set_gamepad_button(second, GamepadButton::DPadRight, true);
    assert_eq!(demo.tick(&input), MenuFrame::Suspended);
    input.advance_frame();
    input.release_key(Key::Space);
    for _ in 0..3 {
        demo.tick(&input);
        input.advance_frame();
    }
    assert_eq!(demo.ticks, 0);
    input.release_key(Key::Left);
    demo.tick(&input);
    input.advance_frame();
    assert_eq!(demo.ticks, 0);
    input.set_gamepad_button(first, GamepadButton::South, false);
    demo.tick(&input);
    input.advance_frame();
    assert_eq!(demo.ticks, 0);
    input.disconnect_gamepad(second);
    assert_eq!(demo.tick(&input), MenuFrame::Suspended);
    assert_eq!(demo.ticks, 0);
    input.advance_frame();
    assert_eq!(demo.tick(&input), MenuFrame::Advance);
    assert_eq!(demo.ticks, 1);
}

#[test]
fn gamepad_navigation_and_exit_share_the_single_consumer_owner() {
    let mut demo = Demo::default();
    let mut input = Input::default();
    let pad = GamepadId::new(4);
    let tap = |demo: &mut Demo, input: &mut Input, button| {
        input.set_gamepad_button(pad, button, true);
        let outcome = demo.tick(input);
        input.advance_frame();
        input.set_gamepad_button(pad, button, false);
        demo.tick(input);
        input.advance_frame();
        outcome
    };
    tap(&mut demo, &mut input, GamepadButton::Start);
    tap(&mut demo, &mut input, GamepadButton::DPadDown);
    tap(&mut demo, &mut input, GamepadButton::DPadDown);
    assert_eq!(
        tap(&mut demo, &mut input, GamepadButton::South),
        MenuFrame::Exit
    );
    assert_eq!(demo.ticks, 0);
}
