//! T1.5 characterization of ordinal widget identity.
//!
//! Within one page, interactive widget order is expected to stay stable. These
//! tests deliberately show the raw rebinding behavior after structural changes,
//! then verify that `UiState::reset_interaction()` before a deliberate page/shape
//! change cleanly invalidates focus, pointer capture, and repeat ownership.

use std::time::Duration;

use super::{Ui, UiResponse, UiState, UiTheme};
use crate::{Framebuffer, Input, Key, MouseButton};

fn compact_theme() -> UiTheme {
    UiTheme {
        padding: 0,
        row_height: 20,
        row_spacing: 0,
        ..UiTheme::default()
    }
}

fn focus_second_of_three(state: &mut UiState) {
    let theme = compact_theme();
    let mut framebuffer = Framebuffer::new(200, 60);
    let idle = Input::default();
    {
        let mut ui = Ui::new(
            &mut framebuffer,
            &idle,
            Duration::from_millis(16),
            state,
            theme,
        );
        ui.button("A");
        ui.button("B");
        let mut value = 0.5;
        ui.slider_f32("C", &mut value, 0.0..=1.0, 0.1);
    }

    let mut down = Input::default();
    down.press_key(Key::Down);
    {
        let mut ui = Ui::new(
            &mut framebuffer,
            &down,
            Duration::from_millis(16),
            state,
            theme,
        );
        ui.button("A");
        ui.button("B");
        let mut value = 0.5;
        ui.slider_f32("C", &mut value, 0.0..=1.0, 0.1);
    }
    assert_eq!(state.focused_index(), Some(1));
}

#[test]
fn focus_shift_rebinds_to_the_new_widget_at_the_same_ordinal() {
    let theme = compact_theme();
    let mut state = UiState::default();
    focus_second_of_three(&mut state);

    let mut framebuffer = Framebuffer::new(200, 40);
    let input = Input::default();
    let mut value = 0.5;
    let slider_c = {
        let mut ui = Ui::new(
            &mut framebuffer,
            &input,
            Duration::from_millis(16),
            &mut state,
            theme,
        );
        let button_a = ui.button("A");
        assert!(!button_a.focused);
        ui.slider_f32("C", &mut value, 0.0..=1.0, 0.1)
    };

    assert!(slider_c.focused);
    assert_eq!(state.focused_index(), Some(1));
}

#[test]
fn active_pointer_rebinds_to_the_new_slider_at_the_same_ordinal() {
    let theme = compact_theme();
    let mut state = UiState::default();
    let mut framebuffer = Framebuffer::new(200, 60);
    let mut input = Input::default();
    let mut a = 0.1;
    let mut b = 0.1;
    let mut c = 0.1;

    input.set_mouse_position(Some((150, 30)));
    input.press_mouse_button(MouseButton::Left);
    let b_response = {
        let mut ui = Ui::new(
            &mut framebuffer,
            &input,
            Duration::from_millis(16),
            &mut state,
            theme,
        );
        ui.slider_f32("A", &mut a, 0.0..=1.0, 0.1);
        let response = ui.slider_f32("B", &mut b, 0.0..=1.0, 0.1);
        ui.slider_f32("C", &mut c, 0.0..=1.0, 0.1);
        response
    };
    assert!(b_response.active);

    input.advance_frame();
    c = 0.1;
    let c_response = {
        let mut ui = Ui::new(
            &mut framebuffer,
            &input,
            Duration::from_millis(16),
            &mut state,
            theme,
        );
        ui.slider_f32("A", &mut a, 0.0..=1.0, 0.1);
        ui.slider_f32("C", &mut c, 0.0..=1.0, 0.1)
    };

    assert!(c_response.active);
    assert!(c_response.changed);
    assert!(c > 0.1);
}

#[test]
fn reset_interaction_prevents_active_pointer_rebind() {
    let theme = compact_theme();
    let mut state = UiState::default();
    let mut framebuffer = Framebuffer::new(200, 60);
    let mut input = Input::default();
    let mut a = 0.1;
    let mut b = 0.1;
    let mut c = 0.1;

    input.set_mouse_position(Some((150, 30)));
    input.press_mouse_button(MouseButton::Left);
    {
        let mut ui = Ui::new(
            &mut framebuffer,
            &input,
            Duration::from_millis(16),
            &mut state,
            theme,
        );
        ui.slider_f32("A", &mut a, 0.0..=1.0, 0.1);
        let response = ui.slider_f32("B", &mut b, 0.0..=1.0, 0.1);
        ui.slider_f32("C", &mut c, 0.0..=1.0, 0.1);
        assert!(response.active);
    }

    input.advance_frame();
    state.reset_interaction();
    c = 0.1;
    let c_response = {
        let mut ui = Ui::new(
            &mut framebuffer,
            &input,
            Duration::from_millis(16),
            &mut state,
            theme,
        );
        ui.slider_f32("A", &mut a, 0.0..=1.0, 0.1);
        ui.slider_f32("C", &mut c, 0.0..=1.0, 0.1)
    };

    assert!(!c_response.active);
    assert!(!c_response.changed);
    assert!((c - 0.1).abs() < f32::EPSILON);
}

#[test]
fn horizontal_repeat_owner_rebinds_to_the_new_slider_at_the_same_ordinal() {
    let theme = compact_theme();
    let mut state = UiState::default();
    let mut framebuffer = Framebuffer::new(200, 60);
    let mut a = 0.5;
    let mut b = 0.5;
    let mut c = 0.5;

    let idle = Input::default();
    {
        let mut ui = Ui::new(
            &mut framebuffer,
            &idle,
            Duration::from_millis(16),
            &mut state,
            theme,
        );
        ui.slider_f32("A", &mut a, 0.0..=1.0, 0.1);
        ui.slider_f32("B", &mut b, 0.0..=1.0, 0.1);
        ui.slider_f32("C", &mut c, 0.0..=1.0, 0.1);
    }

    let mut held_right = Input::default();
    held_right.press_key(Key::Down);
    held_right.press_key(Key::Right);
    {
        let mut ui = Ui::new(
            &mut framebuffer,
            &held_right,
            Duration::ZERO,
            &mut state,
            theme,
        );
        ui.slider_f32("A", &mut a, 0.0..=1.0, 0.1);
        ui.slider_f32("B", &mut b, 0.0..=1.0, 0.1);
        ui.slider_f32("C", &mut c, 0.0..=1.0, 0.1);
    }
    assert_eq!(state.focused_index(), Some(1));

    held_right.advance_frame();
    {
        let mut ui = Ui::new(
            &mut framebuffer,
            &held_right,
            Duration::from_millis(299),
            &mut state,
            theme,
        );
        ui.slider_f32("A", &mut a, 0.0..=1.0, 0.1);
        ui.slider_f32("B", &mut b, 0.0..=1.0, 0.1);
        ui.slider_f32("C", &mut c, 0.0..=1.0, 0.1);
    }

    let before = c;
    let c_response = {
        let mut ui = Ui::new(
            &mut framebuffer,
            &held_right,
            Duration::from_millis(1),
            &mut state,
            theme,
        );
        ui.slider_f32("A", &mut a, 0.0..=1.0, 0.1);
        ui.slider_f32("C", &mut c, 0.0..=1.0, 0.1)
    };

    assert!(c_response.focused);
    assert!(c_response.changed);
    assert!(c > before);
}

#[test]
fn reset_interaction_prevents_horizontal_repeat_rebind() {
    let theme = compact_theme();
    let mut state = UiState::default();
    let mut framebuffer = Framebuffer::new(200, 60);
    let mut a = 0.5;
    let mut b = 0.5;
    let mut c = 0.5;

    let idle = Input::default();
    {
        let mut ui = Ui::new(
            &mut framebuffer,
            &idle,
            Duration::from_millis(16),
            &mut state,
            theme,
        );
        ui.slider_f32("A", &mut a, 0.0..=1.0, 0.1);
        ui.slider_f32("B", &mut b, 0.0..=1.0, 0.1);
        ui.slider_f32("C", &mut c, 0.0..=1.0, 0.1);
    }

    let mut held_right = Input::default();
    held_right.press_key(Key::Down);
    held_right.press_key(Key::Right);
    {
        let mut ui = Ui::new(
            &mut framebuffer,
            &held_right,
            Duration::ZERO,
            &mut state,
            theme,
        );
        ui.slider_f32("A", &mut a, 0.0..=1.0, 0.1);
        ui.slider_f32("B", &mut b, 0.0..=1.0, 0.1);
        ui.slider_f32("C", &mut c, 0.0..=1.0, 0.1);
    }

    held_right.advance_frame();
    {
        let mut ui = Ui::new(
            &mut framebuffer,
            &held_right,
            Duration::from_millis(299),
            &mut state,
            theme,
        );
        ui.slider_f32("A", &mut a, 0.0..=1.0, 0.1);
        ui.slider_f32("B", &mut b, 0.0..=1.0, 0.1);
        ui.slider_f32("C", &mut c, 0.0..=1.0, 0.1);
    }

    state.reset_interaction();
    let before = c;
    let c_response = {
        let mut ui = Ui::new(
            &mut framebuffer,
            &held_right,
            Duration::from_millis(1),
            &mut state,
            theme,
        );
        ui.slider_f32("A", &mut a, 0.0..=1.0, 0.1);
        ui.slider_f32("C", &mut c, 0.0..=1.0, 0.1)
    };

    assert!(!c_response.focused);
    assert!(!c_response.changed);
    assert_eq!(c.to_bits(), before.to_bits());
}

fn page_a(ui: &mut Ui<'_>, first: &mut f32, second: &mut f32) -> [UiResponse; 3] {
    [
        ui.button("BUTTON"),
        ui.slider_f32("FIRST", first, 0.0..=1.0, 0.1),
        ui.slider_f32("SECOND", second, 0.0..=1.0, 0.1),
    ]
}

fn page_b(ui: &mut Ui<'_>, enabled: &mut bool) -> [UiResponse; 2] {
    [ui.toggle("TOGGLE", enabled), ui.button("BUTTON")]
}

#[test]
fn page_shape_change_with_out_of_range_focus_has_a_one_frame_focus_gap_then_clamps() {
    let theme = compact_theme();
    let mut state = UiState::default();
    let mut framebuffer = Framebuffer::new(200, 60);
    let mut first = 0.5;
    let mut second = 0.5;

    let idle = Input::default();
    {
        let mut ui = Ui::new(
            &mut framebuffer,
            &idle,
            Duration::from_millis(16),
            &mut state,
            theme,
        );
        page_a(&mut ui, &mut first, &mut second);
    }

    for _ in 0..2 {
        let mut down = Input::default();
        down.press_key(Key::Down);
        let mut ui = Ui::new(
            &mut framebuffer,
            &down,
            Duration::from_millis(16),
            &mut state,
            theme,
        );
        page_a(&mut ui, &mut first, &mut second);
    }
    assert_eq!(state.focused_index(), Some(2));

    let mut enabled = false;
    let first_page_b_frame = {
        let mut ui = Ui::new(
            &mut framebuffer,
            &idle,
            Duration::from_millis(16),
            &mut state,
            theme,
        );
        page_b(&mut ui, &mut enabled)
    };
    assert!(!first_page_b_frame[0].focused);
    assert!(!first_page_b_frame[1].focused);
    assert_eq!(state.focused_index(), Some(1));

    let second_page_b_frame = {
        let mut ui = Ui::new(
            &mut framebuffer,
            &idle,
            Duration::from_millis(16),
            &mut state,
            theme,
        );
        page_b(&mut ui, &mut enabled)
    };
    assert!(!second_page_b_frame[0].focused);
    assert!(second_page_b_frame[1].focused);
}

#[test]
fn reset_interaction_before_page_change_restarts_focus_on_first_widget() {
    let theme = compact_theme();
    let mut state = UiState::default();
    let mut framebuffer = Framebuffer::new(200, 60);
    let mut first = 0.5;
    let mut second = 0.5;

    let idle = Input::default();
    {
        let mut ui = Ui::new(
            &mut framebuffer,
            &idle,
            Duration::from_millis(16),
            &mut state,
            theme,
        );
        page_a(&mut ui, &mut first, &mut second);
    }

    for _ in 0..2 {
        let mut down = Input::default();
        down.press_key(Key::Down);
        let mut ui = Ui::new(
            &mut framebuffer,
            &down,
            Duration::from_millis(16),
            &mut state,
            theme,
        );
        page_a(&mut ui, &mut first, &mut second);
    }
    assert_eq!(state.focused_index(), Some(2));

    state.reset_interaction();
    let mut enabled = false;
    let page_b_frame = {
        let mut ui = Ui::new(
            &mut framebuffer,
            &idle,
            Duration::from_millis(16),
            &mut state,
            theme,
        );
        page_b(&mut ui, &mut enabled)
    };

    assert!(page_b_frame[0].focused);
    assert!(!page_b_frame[1].focused);
    assert_eq!(state.focused_index(), Some(0));
}
