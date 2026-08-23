use std::time::Duration;

use super::{Ui, UiState, UiTheme};
use crate::{Framebuffer, Input, Key, MouseButton};

const TWO_TABS: [&str; 2] = ["CONTROLS", "OTHER"];
const THREE_TABS: [&str; 3] = ["ONE", "TWO", "THREE"];

fn compact_theme() -> UiTheme {
    UiTheme {
        padding: 0,
        row_height: 20,
        row_spacing: 0,
        ..UiTheme::default()
    }
}

fn tabs_only(
    input: &Input,
    state: &mut UiState,
    selected: usize,
    labels: &[&str],
) -> Option<usize> {
    let mut framebuffer = Framebuffer::new(180, 20);
    let mut ui = Ui::new(
        &mut framebuffer,
        input,
        Duration::from_millis(16),
        state,
        compact_theme(),
    );
    ui.tabs(selected, labels)
}

#[test]
fn tabs_left_requests_previous_and_wraps() {
    let mut state = UiState::default();

    let mut left = Input::default();
    left.press_key(Key::Left);
    assert_eq!(tabs_only(&left, &mut state, 1, &THREE_TABS), Some(0));

    state.reset_interaction();
    assert_eq!(tabs_only(&left, &mut state, 0, &THREE_TABS), Some(2));
}

#[test]
fn tabs_right_requests_next_and_wraps() {
    let mut state = UiState::default();

    let mut right = Input::default();
    right.press_key(Key::Right);
    assert_eq!(tabs_only(&right, &mut state, 1, &THREE_TABS), Some(2));

    state.reset_interaction();
    assert_eq!(tabs_only(&right, &mut state, 2, &THREE_TABS), Some(0));
}

#[test]
fn tabs_mouse_press_requests_clicked_tab() {
    let mut state = UiState::default();
    let mut input = Input::default();
    input.set_mouse_position(Some((150, 10)));
    input.press_mouse_button(MouseButton::Left);

    assert_eq!(tabs_only(&input, &mut state, 0, &THREE_TABS), Some(2));
    assert_eq!(state.focused_index(), Some(0));
}

#[test]
fn tabs_edge_cases_are_deterministic_without_extra_state() {
    let theme = compact_theme();
    let mut state = UiState::default();
    let mut framebuffer = Framebuffer::new(180, 40);
    let idle = Input::default();

    let button = {
        let mut ui = Ui::new(
            &mut framebuffer,
            &idle,
            Duration::from_millis(16),
            &mut state,
            theme,
        );
        assert_eq!(ui.tabs(0, &[]), None);
        ui.button("FIRST INTERACTIVE")
    };
    assert!(button.focused);
    assert_eq!(state.focused_index(), Some(0));

    state.reset_interaction();
    let mut right = Input::default();
    right.press_key(Key::Right);
    assert_eq!(tabs_only(&right, &mut state, 0, &["ONLY"]), None);

    state.reset_interaction();
    assert_eq!(
        tabs_only(&idle, &mut state, usize::MAX, &THREE_TABS),
        Some(2)
    );
}

#[test]
fn tab_mouse_transition_is_deferred_and_held_click_does_not_activate_new_page() {
    let theme = compact_theme();
    let mut state = UiState::default();
    let mut framebuffer = Framebuffer::new(200, 60);
    let mut selected_tab = 0;
    let mut input = Input::default();

    input.set_mouse_position(Some((150, 10)));
    input.press_mouse_button(MouseButton::Left);
    let request = {
        let mut ui = Ui::new(
            &mut framebuffer,
            &input,
            Duration::from_millis(16),
            &mut state,
            theme,
        );
        let request = ui.tabs(selected_tab, &TWO_TABS);
        let old_page = ui.button("PAGE A BUTTON");
        assert!(!old_page.clicked);
        request
    };

    assert_eq!(selected_tab, 0);
    assert_eq!(request, Some(1));

    state.reset_interaction();
    selected_tab = request.expect("tab B should be requested");
    input.advance_frame();

    let new_page = {
        let mut ui = Ui::new(
            &mut framebuffer,
            &input,
            Duration::from_millis(16),
            &mut state,
            theme,
        );
        assert_eq!(ui.tabs(selected_tab, &TWO_TABS), None);
        ui.button("PAGE B BUTTON")
    };

    assert_eq!(selected_tab, 1);
    assert_eq!(state.focused_index(), Some(0));
    assert!(!new_page.focused);
    assert!(!new_page.active);
    assert!(!new_page.clicked);
}

#[test]
fn reset_after_keyboard_tab_request_clears_old_pointer_capture() {
    let theme = compact_theme();
    let mut state = UiState::default();
    let mut framebuffer = Framebuffer::new(200, 60);
    let mut selected_tab = 0;
    let mut old_value = 0.0;
    let mut input = Input::default();

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
        assert_eq!(ui.tabs(selected_tab, &TWO_TABS), None);
        let slider = ui.slider_f32("OLD SLIDER", &mut old_value, 0.0..=1.0, 0.1);
        assert!(slider.active);
    }
    assert_eq!(state.focused_index(), Some(1));

    input.advance_frame();
    input.press_key(Key::Up);
    input.press_key(Key::Right);
    let request = {
        let mut ui = Ui::new(
            &mut framebuffer,
            &input,
            Duration::from_millis(16),
            &mut state,
            theme,
        );
        let request = ui.tabs(selected_tab, &TWO_TABS);
        let old_slider = ui.slider_f32("OLD SLIDER", &mut old_value, 0.0..=1.0, 0.1);
        assert!(old_slider.active);
        request
    };
    assert_eq!(request, Some(1));

    state.reset_interaction();
    selected_tab = request.expect("tab B should be requested");
    input.advance_frame();

    let new_button = {
        let mut ui = Ui::new(
            &mut framebuffer,
            &input,
            Duration::from_millis(16),
            &mut state,
            theme,
        );
        assert_eq!(ui.tabs(selected_tab, &TWO_TABS), None);
        ui.button("NEW PAGE BUTTON")
    };

    assert_eq!(state.focused_index(), Some(0));
    assert!(!new_button.focused);
    assert!(!new_button.active);
    assert!(!new_button.clicked);
}

#[test]
fn reset_after_normalization_request_prevents_repeat_owner_rebind() {
    let theme = compact_theme();
    let mut state = UiState::default();
    let mut framebuffer = Framebuffer::new(200, 60);
    let mut old_value = 0.5;
    let idle = Input::default();

    {
        let mut ui = Ui::new(
            &mut framebuffer,
            &idle,
            Duration::from_millis(16),
            &mut state,
            theme,
        );
        ui.tabs(0, &TWO_TABS);
        ui.slider_f32("OLD SLIDER", &mut old_value, 0.0..=1.0, 0.1);
    }

    let mut input = Input::default();
    input.press_key(Key::Down);
    input.press_key(Key::Right);
    {
        let mut ui = Ui::new(&mut framebuffer, &input, Duration::ZERO, &mut state, theme);
        assert_eq!(ui.tabs(0, &TWO_TABS), None);
        let slider = ui.slider_f32("OLD SLIDER", &mut old_value, 0.0..=1.0, 0.1);
        assert!(slider.focused);
        assert!(slider.changed);
    }
    assert_eq!(state.focused_index(), Some(1));

    input.advance_frame();
    let request = {
        let mut ui = Ui::new(
            &mut framebuffer,
            &input,
            Duration::from_millis(299),
            &mut state,
            theme,
        );
        let request = ui.tabs(usize::MAX, &TWO_TABS);
        let slider = ui.slider_f32("OLD SLIDER", &mut old_value, 0.0..=1.0, 0.1);
        assert!(slider.focused);
        assert!(!slider.changed);
        request
    };
    assert_eq!(request, Some(1));

    state.reset_interaction();
    let selected_tab = request.expect("normalization should request the last tab");
    input.advance_frame();

    let mut new_value = 0.5;
    let new_slider = {
        let mut ui = Ui::new(
            &mut framebuffer,
            &input,
            Duration::from_millis(1),
            &mut state,
            theme,
        );
        assert_eq!(ui.tabs(selected_tab, &TWO_TABS), None);
        ui.slider_f32("NEW SLIDER", &mut new_value, 0.0..=1.0, 0.1)
    };

    assert_eq!(state.focused_index(), Some(0));
    assert!(!new_slider.focused);
    assert!(!new_slider.changed);
    assert!((new_value - 0.5).abs() < f32::EPSILON);
}
