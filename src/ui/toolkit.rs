use std::ops::RangeInclusive;
use std::time::Duration;

use crate::{ButtonState, Font, Framebuffer, Input, Key, MouseButton, Pixel, Rect, TextRenderer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RepeatConfig {
    pub initial_delay: Duration,
    pub interval: Duration,
}

impl RepeatConfig {
    pub const fn new(initial_delay: Duration, interval: Duration) -> Self {
        Self {
            initial_delay,
            interval,
        }
    }
}

impl Default for RepeatConfig {
    fn default() -> Self {
        Self::new(Duration::from_millis(300), Duration::from_millis(60))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RepeatState {
    elapsed: Duration,
    next_repeat: Duration,
}

impl RepeatState {
    pub fn reset(&mut self, config: RepeatConfig) {
        self.elapsed = Duration::ZERO;
        self.next_repeat = config.initial_delay;
    }

    /// Returns the number of logical input pulses for this frame.
    ///
    /// A fresh press always produces one pulse immediately. Held input then
    /// repeats after `initial_delay` using elapsed time rather than frame count.
    /// Returning a count preserves repeats crossed by a long frame instead of
    /// silently collapsing them into a single action.
    pub fn update(
        &mut self,
        button: ButtonState,
        delta_time: Duration,
        config: RepeatConfig,
    ) -> u32 {
        if button.pressed() {
            self.reset(config);
            return 1;
        }
        if !button.held() {
            self.reset(config);
            return 0;
        }

        self.elapsed = self.elapsed.saturating_add(delta_time);
        if self.elapsed < self.next_repeat || config.interval.is_zero() {
            return 0;
        }

        let overdue = self.elapsed.saturating_sub(self.next_repeat);
        let extra = overdue.as_nanos() / config.interval.as_nanos();
        let pulses = extra.saturating_add(1).min(u128::from(u32::MAX)) as u32;
        self.next_repeat = self
            .next_repeat
            .saturating_add(config.interval.saturating_mul(pulses));
        pulses
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiTheme {
    pub font: Font,
    pub text_scale: u32,
    pub padding: u32,
    pub row_height: u32,
    pub row_spacing: u32,
    pub text: Pixel,
    pub muted_text: Pixel,
    pub control_background: Pixel,
    pub border: Pixel,
    pub accent: Pixel,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            font: Font::default(),
            text_scale: 1,
            padding: 8,
            row_height: 24,
            row_spacing: 4,
            text: Pixel::rgb(220, 225, 235),
            muted_text: Pixel::rgb(145, 155, 175),
            control_background: Pixel::rgb(13, 17, 28),
            border: Pixel::rgb(70, 82, 105),
            accent: Pixel::rgb(105, 205, 235),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UiResponse {
    pub focused: bool,
    pub hovered: bool,
    pub active: bool,
    pub clicked: bool,
    pub changed: bool,
}

#[derive(Debug, Default)]
pub struct UiState {
    focused: usize,
    interactive_count: usize,
    pointer_active: Option<usize>,
    horizontal_repeat_owner: Option<usize>,
    left_repeat: RepeatState,
    right_repeat: RepeatState,
}

impl UiState {
    pub fn focused_index(&self) -> Option<usize> {
        (self.interactive_count > 0).then_some(self.focused)
    }
}

pub struct Ui<'a> {
    framebuffer: &'a mut Framebuffer,
    input: &'a Input,
    delta_time: Duration,
    state: &'a mut UiState,
    theme: UiTheme,
    text_renderer: TextRenderer,
    repeat_config: RepeatConfig,
    cursor_y: i32,
    interactive_count: usize,
}

impl<'a> Ui<'a> {
    pub fn new(
        framebuffer: &'a mut Framebuffer,
        input: &'a Input,
        delta_time: Duration,
        state: &'a mut UiState,
        theme: UiTheme,
    ) -> Self {
        if state.interactive_count > 0 {
            if input.key(Key::Up).pressed() {
                state.focused = if state.focused == 0 {
                    state.interactive_count - 1
                } else {
                    state.focused - 1
                };
            }
            if input.key(Key::Down).pressed() {
                state.focused = (state.focused + 1) % state.interactive_count;
            }
        }

        if input.mouse_position().is_none() {
            state.pointer_active = None;
        }

        let cursor_y = u32_to_i32(theme.padding);
        let text_renderer = TextRenderer::new(theme.font);
        Self {
            framebuffer,
            input,
            delta_time,
            state,
            theme,
            text_renderer,
            repeat_config: RepeatConfig::default(),
            cursor_y,
            interactive_count: 0,
        }
    }

    pub fn with_repeat_config(mut self, repeat_config: RepeatConfig) -> Self {
        self.repeat_config = repeat_config;
        self
    }

    pub fn label(&mut self, text: &str) {
        let rect = self.next_row();
        self.draw_text_left(rect, text, self.theme.muted_text);
    }

    pub fn button(&mut self, label: &str) -> UiResponse {
        let rect = self.next_row();
        let ordinal = self.next_interactive();
        let response = self.click_response(rect, ordinal);
        self.draw_control(rect, label, response);
        response
    }

    pub fn toggle(&mut self, label: &str, value: &mut bool) -> UiResponse {
        let rect = self.next_row();
        let ordinal = self.next_interactive();
        let mut response = self.click_response(rect, ordinal);
        if response.clicked {
            *value = !*value;
            response.changed = true;
        }
        let value_label = if *value { "ON" } else { "OFF" };
        let text = format!("{label}: {value_label}");
        self.draw_control(rect, &text, response);
        response
    }

    pub fn slider_f32(
        &mut self,
        label: &str,
        value: &mut f32,
        range: RangeInclusive<f32>,
        step: f32,
    ) -> UiResponse {
        let rect = self.next_row();
        let ordinal = self.next_interactive();
        let hovered = self.pointer_over(rect);
        let focused = self.state.focused == ordinal;
        let left_button = self.input.mouse_button(MouseButton::Left);
        let track = slider_track_rect(rect);

        if left_button.pressed() && hovered {
            self.state.focused = ordinal;
        }
        if left_button.pressed()
            && self
                .input
                .mouse_position()
                .is_some_and(|position| track.contains(position))
        {
            self.state.pointer_active = Some(ordinal);
        }

        let mut changed = false;
        if self.state.pointer_active == Some(ordinal) {
            if let Some((mouse_x, _)) = self.input.mouse_position() {
                if left_button.pressed() || left_button.held() || left_button.released() {
                    let before = value.to_bits();
                    *value = slider_value_from_pointer(mouse_x, track, range.clone(), step);
                    changed |= before != value.to_bits();
                }
                if left_button.released() || (!left_button.pressed() && !left_button.held()) {
                    self.state.pointer_active = None;
                }
            } else {
                self.state.pointer_active = None;
            }
        }

        let focused = self.state.focused == ordinal;
        if focused {
            if self.state.horizontal_repeat_owner != Some(ordinal) {
                self.state.left_repeat.reset(self.repeat_config);
                self.state.right_repeat.reset(self.repeat_config);
                self.state.horizontal_repeat_owner = Some(ordinal);
            }
            let left_pulses = self.state.left_repeat.update(
                self.input.key(Key::Left),
                self.delta_time,
                self.repeat_config,
            );
            let right_pulses = self.state.right_repeat.update(
                self.input.key(Key::Right),
                self.delta_time,
                self.repeat_config,
            );
            let direction = i64::from(right_pulses) - i64::from(left_pulses);
            if direction != 0 {
                let before = value.to_bits();
                *value = stepped_slider_value(*value, range.clone(), step, direction);
                changed |= before != value.to_bits();
            }
        } else if self.state.horizontal_repeat_owner == Some(ordinal) {
            self.state.left_repeat.reset(self.repeat_config);
            self.state.right_repeat.reset(self.repeat_config);
            self.state.horizontal_repeat_owner = None;
        }

        let active = self.state.pointer_active == Some(ordinal);
        let response = UiResponse {
            focused,
            hovered,
            active,
            changed,
            ..UiResponse::default()
        };
        self.draw_slider(rect, track, label, *value, range, response);
        response
    }

    fn next_row(&mut self) -> Rect {
        let padding = self.theme.padding;
        let width = self
            .framebuffer
            .width()
            .saturating_sub(padding.saturating_mul(2));
        let rect = Rect {
            x: u32_to_i32(padding),
            y: self.cursor_y,
            width,
            height: self.theme.row_height,
        };
        let advance = self
            .theme
            .row_height
            .saturating_add(self.theme.row_spacing);
        self.cursor_y = self.cursor_y.saturating_add(u32_to_i32(advance));
        rect
    }

    fn next_interactive(&mut self) -> usize {
        let ordinal = self.interactive_count;
        self.interactive_count = self.interactive_count.saturating_add(1);
        ordinal
    }

    fn pointer_over(&self, rect: Rect) -> bool {
        self.input
            .mouse_position()
            .is_some_and(|position| rect.contains(position))
    }

    fn click_response(&mut self, rect: Rect, ordinal: usize) -> UiResponse {
        let hovered = self.pointer_over(rect);
        let left_button = self.input.mouse_button(MouseButton::Left);
        if left_button.pressed() && hovered {
            self.state.focused = ordinal;
            self.state.pointer_active = Some(ordinal);
        }

        let mut clicked = self.state.focused == ordinal && self.input.key(Key::Space).pressed();
        if self.state.pointer_active == Some(ordinal) {
            if self.input.mouse_position().is_none() {
                self.state.pointer_active = None;
            } else if left_button.released() {
                clicked |= hovered;
                self.state.pointer_active = None;
            } else if !left_button.pressed() && !left_button.held() {
                self.state.pointer_active = None;
            }
        }

        UiResponse {
            focused: self.state.focused == ordinal,
            hovered,
            active: self.state.pointer_active == Some(ordinal),
            clicked,
            changed: false,
        }
    }

    fn draw_control(&mut self, rect: Rect, label: &str, response: UiResponse) {
        self.framebuffer.fill_rect(
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            self.theme.control_background,
        );
        let border = if response.focused || response.hovered {
            self.theme.accent
        } else {
            self.theme.border
        };
        self.framebuffer
            .draw_rect(rect.x, rect.y, rect.width, rect.height, border);
        self.draw_text_centered(rect, label, self.theme.text);
    }

    fn draw_slider(
        &mut self,
        rect: Rect,
        track: Rect,
        label: &str,
        value: f32,
        range: RangeInclusive<f32>,
        response: UiResponse,
    ) {
        self.framebuffer.fill_rect(
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            self.theme.control_background,
        );
        let border = if response.focused || response.hovered {
            self.theme.accent
        } else {
            self.theme.border
        };
        self.framebuffer
            .draw_rect(rect.x, rect.y, rect.width, rect.height, border);

        let label_rect = Rect {
            x: rect.x.saturating_add(4),
            y: rect.y,
            width: rect.width / 2,
            height: rect.height,
        };
        self.draw_text_left(label_rect, label, self.theme.text);

        let (min, max) = normalized_range(range);
        let normalized = if max > min {
            ((value.clamp(min, max) - min) / (max - min)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.framebuffer.fill_rect(
            track.x,
            track.y,
            track.width,
            track.height,
            self.theme.border,
        );
        let fill_width = ((track.width as f32) * normalized).round() as u32;
        if fill_width > 0 {
            self.framebuffer.fill_rect(
                track.x,
                track.y,
                fill_width.min(track.width),
                track.height,
                self.theme.accent,
            );
        }

        let value_text = format!("{value:.2}");
        let (value_width, value_height) = self.text_size(&value_text);
        let value_x = rect
            .x
            .saturating_add(u32_to_i32(rect.width.saturating_sub(value_width).saturating_sub(4)));
        let value_y = centered_coordinate(rect.y, rect.height, value_height);
        self.text_renderer.draw_scaled(
            self.framebuffer,
            value_x,
            value_y,
            &value_text,
            self.theme.text_scale.max(1),
            self.theme.text,
        );
    }

    fn text_size(&self, text: &str) -> (u32, u32) {
        self.text_renderer
            .text_size(text, self.theme.text_scale.max(1))
    }

    fn draw_text_centered(&mut self, rect: Rect, text: &str, color: Pixel) {
        let (width, height) = self.text_size(text);
        let x = centered_coordinate(rect.x, rect.width, width);
        let y = centered_coordinate(rect.y, rect.height, height);
        self.text_renderer.draw_scaled(
            self.framebuffer,
            x,
            y,
            text,
            self.theme.text_scale.max(1),
            color,
        );
    }

    fn draw_text_left(&mut self, rect: Rect, text: &str, color: Pixel) {
        let (_, height) = self.text_size(text);
        let y = centered_coordinate(rect.y, rect.height, height);
        self.text_renderer.draw_scaled(
            self.framebuffer,
            rect.x,
            y,
            text,
            self.theme.text_scale.max(1),
            color,
        );
    }
}

impl Drop for Ui<'_> {
    fn drop(&mut self) {
        self.state.interactive_count = self.interactive_count;
        if self.interactive_count == 0 {
            self.state.focused = 0;
            self.state.pointer_active = None;
            self.state.horizontal_repeat_owner = None;
            self.state.left_repeat.reset(self.repeat_config);
            self.state.right_repeat.reset(self.repeat_config);
            return;
        }

        if self.state.focused >= self.interactive_count {
            self.state.focused = self.interactive_count - 1;
        }
        if self
            .state
            .pointer_active
            .is_some_and(|ordinal| ordinal >= self.interactive_count)
        {
            self.state.pointer_active = None;
        }
        if self
            .state
            .horizontal_repeat_owner
            .is_some_and(|ordinal| ordinal >= self.interactive_count)
        {
            self.state.horizontal_repeat_owner = None;
            self.state.left_repeat.reset(self.repeat_config);
            self.state.right_repeat.reset(self.repeat_config);
        }
    }
}

fn normalized_range(range: RangeInclusive<f32>) -> (f32, f32) {
    let start = *range.start();
    let end = *range.end();
    if start <= end {
        (start, end)
    } else {
        (end, start)
    }
}

fn normalize_step(step: f32) -> f32 {
    if step.is_finite() && step != 0.0 {
        step.abs()
    } else {
        0.0
    }
}

fn snap_slider_value(value: f32, min: f32, max: f32, step: f32) -> f32 {
    if !value.is_finite() {
        return min;
    }
    let clamped = value.clamp(min, max);
    let step = normalize_step(step);
    if step == 0.0 || max <= min {
        return clamped;
    }
    let steps = ((clamped - min) / step).round();
    (min + steps * step).clamp(min, max)
}

fn stepped_slider_value(
    value: f32,
    range: RangeInclusive<f32>,
    step: f32,
    direction: i64,
) -> f32 {
    let (min, max) = normalized_range(range);
    let step = normalize_step(step);
    if step == 0.0 || direction == 0 {
        return snap_slider_value(value, min, max, step);
    }
    let base = snap_slider_value(value, min, max, step);
    snap_slider_value(base + step * direction as f32, min, max, step)
}

fn slider_value_from_pointer(
    mouse_x: i32,
    track: Rect,
    range: RangeInclusive<f32>,
    step: f32,
) -> f32 {
    let (min, max) = normalized_range(range);
    if max <= min || track.width <= 1 {
        return min;
    }
    let start = i64::from(track.x);
    let end = start.saturating_add(i64::from(track.width.saturating_sub(1)));
    let x = i64::from(mouse_x).clamp(start, end);
    let normalized = (x - start) as f32 / (end - start) as f32;
    snap_slider_value(min + (max - min) * normalized, min, max, step)
}

fn slider_track_rect(rect: Rect) -> Rect {
    let x_offset = rect.width / 2;
    let right_padding = 44_u32.min(rect.width.saturating_sub(x_offset));
    let width = rect
        .width
        .saturating_sub(x_offset)
        .saturating_sub(right_padding)
        .max(1);
    let height = 5_u32.min(rect.height.max(1));
    Rect {
        x: rect.x.saturating_add(u32_to_i32(x_offset)),
        y: centered_coordinate(rect.y, rect.height, height),
        width,
        height,
    }
}

fn centered_coordinate(origin: i32, extent: u32, content_extent: u32) -> i32 {
    let coordinate = i64::from(origin) + (i64::from(extent) - i64::from(content_extent)) / 2;
    coordinate.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn u32_to_i32(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_three_buttons(input: &Input, state: &mut UiState) -> [UiResponse; 3] {
        let mut framebuffer = Framebuffer::new(120, 90);
        let mut ui = Ui::new(
            &mut framebuffer,
            input,
            Duration::from_millis(16),
            state,
            UiTheme::default(),
        );
        [ui.button("ONE"), ui.button("TWO"), ui.button("THREE")]
    }

    #[test]
    fn repeat_is_time_based_and_preserves_catch_up_pulses() {
        let config = RepeatConfig::default();
        let mut repeat = RepeatState::default();
        let pressed = ButtonState::from_transition(false, true);
        let held = ButtonState::from_transition(true, true);
        let released = ButtonState::from_transition(true, false);

        assert_eq!(repeat.update(pressed, Duration::ZERO, config), 1);
        assert_eq!(repeat.update(held, Duration::from_millis(299), config), 0);
        assert_eq!(repeat.update(held, Duration::from_millis(1), config), 1);
        assert_eq!(repeat.update(held, Duration::from_millis(180), config), 3);
        assert_eq!(repeat.update(released, Duration::ZERO, config), 0);
        assert_eq!(repeat.update(pressed, Duration::ZERO, config), 1);
    }

    #[test]
    fn keyboard_focus_moves_and_wraps_using_previous_frame_shape() {
        let mut state = UiState::default();
        let idle = Input::default();
        let first = render_three_buttons(&idle, &mut state);
        assert!(first[0].focused);
        assert_eq!(state.focused_index(), Some(0));

        let mut down = Input::default();
        down.press_key(Key::Down);
        let second = render_three_buttons(&down, &mut state);
        assert!(second[1].focused);
        assert_eq!(state.focused_index(), Some(1));

        let mut up = Input::default();
        up.press_key(Key::Up);
        let third = render_three_buttons(&up, &mut state);
        assert!(third[0].focused);

        let fourth = render_three_buttons(&up, &mut state);
        assert!(fourth[2].focused);
        assert_eq!(state.focused_index(), Some(2));
    }

    #[test]
    fn slider_keyboard_step_clamps_to_range() {
        let mut state = UiState::default();
        let mut framebuffer = Framebuffer::new(160, 40);
        let idle = Input::default();
        let mut value = 0.8;
        {
            let mut ui = Ui::new(
                &mut framebuffer,
                &idle,
                Duration::from_millis(16),
                &mut state,
                UiTheme::default(),
            );
            ui.slider_f32("VALUE", &mut value, 0.0..=1.0, 0.2);
        }

        let mut right = Input::default();
        right.press_key(Key::Right);
        let response = {
            let mut ui = Ui::new(
                &mut framebuffer,
                &right,
                Duration::from_millis(16),
                &mut state,
                UiTheme::default(),
            );
            ui.slider_f32("VALUE", &mut value, 0.0..=1.0, 0.2)
        };
        assert!(response.changed);
        assert!((value - 1.0).abs() < f32::EPSILON);

        let response = {
            let mut ui = Ui::new(
                &mut framebuffer,
                &right,
                Duration::from_millis(16),
                &mut state,
                UiTheme::default(),
            );
            ui.slider_f32("VALUE", &mut value, 0.0..=1.0, 0.2)
        };
        assert!(!response.changed);
        assert!((value - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn slider_drag_cancels_when_pointer_leaves_and_does_not_resume_without_press() {
        let theme = UiTheme {
            padding: 0,
            row_height: 20,
            row_spacing: 0,
            ..UiTheme::default()
        };
        let mut state = UiState::default();
        let mut framebuffer = Framebuffer::new(160, 20);
        let mut value = 0.0;

        let mut input = Input::default();
        input.set_mouse_position(Some((100, 10)));
        input.press_mouse_button(MouseButton::Left);
        {
            let mut ui = Ui::new(
                &mut framebuffer,
                &input,
                Duration::from_millis(16),
                &mut state,
                theme,
            );
            ui.slider_f32("VALUE", &mut value, 0.0..=1.0, 0.1);
        }
        assert_eq!(state.pointer_active, Some(0));
        let pressed_value = value;
        assert!(pressed_value > 0.0);

        input.advance_frame();
        input.set_mouse_position(Some((112, 10)));
        {
            let mut ui = Ui::new(
                &mut framebuffer,
                &input,
                Duration::from_millis(16),
                &mut state,
                theme,
            );
            ui.slider_f32("VALUE", &mut value, 0.0..=1.0, 0.1);
        }
        assert!(value >= pressed_value);

        input.set_mouse_position(None);
        {
            let mut ui = Ui::new(
                &mut framebuffer,
                &input,
                Duration::from_millis(16),
                &mut state,
                theme,
            );
            ui.slider_f32("VALUE", &mut value, 0.0..=1.0, 0.1);
        }
        assert_eq!(state.pointer_active, None);
        let cancelled_value = value;

        input.set_mouse_position(Some((118, 10)));
        {
            let mut ui = Ui::new(
                &mut framebuffer,
                &input,
                Duration::from_millis(16),
                &mut state,
                theme,
            );
            ui.slider_f32("VALUE", &mut value, 0.0..=1.0, 0.1);
        }
        assert_eq!(value.to_bits(), cancelled_value.to_bits());
    }
}
