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

    /// Returns the logical pulse count for this frame.
    ///
    /// The initial press produces one pulse immediately. Held input repeats from
    /// elapsed time, so a long frame can produce more than one repeat pulse.
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
        if self.next_repeat.is_zero() && self.elapsed.is_zero() {
            self.next_repeat = config.initial_delay;
        }

        self.elapsed = self.elapsed.saturating_add(delta_time);
        if config.interval.is_zero() || self.elapsed < self.next_repeat {
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
    previous_interactive_count: usize,
    pointer_active: Option<usize>,
    horizontal_repeat_owner: Option<usize>,
    left_repeat: RepeatState,
    right_repeat: RepeatState,
}

impl UiState {
    pub fn focused_index(&self) -> Option<usize> {
        (self.previous_interactive_count > 0).then_some(self.focused)
    }

    /// Clears transient interaction state before an intentional structural UI change.
    ///
    /// Widget identity is ordinal in T1: declaration order must remain stable inside
    /// a page. Consumers should call this before switching to a page or structure
    /// whose interactive widget order differs.
    pub fn reset_interaction(&mut self) {
        *self = Self::default();
    }
}

pub struct Ui<'a> {
    framebuffer: &'a mut Framebuffer,
    input: &'a Input,
    delta_time: Duration,
    state: &'a mut UiState,
    theme: UiTheme,
    text_renderer: TextRenderer,
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
        if state.previous_interactive_count > 0 {
            if input.key(Key::Up).pressed() {
                state.focused = if state.focused == 0 {
                    state.previous_interactive_count - 1
                } else {
                    state.focused - 1
                };
            }
            if input.key(Key::Down).pressed() {
                state.focused = (state.focused + 1) % state.previous_interactive_count;
            }
        }
        if input.mouse_position().is_none() {
            state.pointer_active = None;
        }

        Self {
            framebuffer,
            input,
            delta_time,
            state,
            theme,
            text_renderer: TextRenderer::new(theme.font),
            cursor_y: u32_to_i32(theme.padding),
            interactive_count: 0,
        }
    }

    pub fn label(&mut self, text: &str) {
        let rect = self.next_row();
        self.draw_text_left(rect, text, self.theme.muted_text);
    }

    /// Draws one ordinal tab-bar widget and returns a requested selection change.
    ///
    /// The consumer owns `selected`. A returned request must be committed only after
    /// this `Ui` has been dropped, with `UiState::reset_interaction()` called before
    /// changing the page structure. Mouse selection happens on left-button press and
    /// deliberately uses no tab-specific pointer-capture state.
    pub fn tabs(&mut self, selected: usize, labels: &[&str]) -> Option<usize> {
        if labels.is_empty() {
            return None;
        }

        let rect = self.next_row();
        let ordinal = self.next_interactive();
        let normalized = selected.min(labels.len() - 1);
        let mut requested = (normalized != selected).then_some(normalized);

        if requested.is_none() && self.state.focused == ordinal && labels.len() > 1 {
            let left = self.input.key(Key::Left).pressed();
            let right = self.input.key(Key::Right).pressed();
            requested = match (left, right) {
                (true, false) => Some(if normalized == 0 {
                    labels.len() - 1
                } else {
                    normalized - 1
                }),
                (false, true) => Some((normalized + 1) % labels.len()),
                _ => None,
            };
        }

        let hovered_tab = self
            .input
            .mouse_position()
            .and_then(|position| tab_index_at_position(rect, labels.len(), position));
        if self.input.mouse_button(MouseButton::Left).pressed()
            && let Some(index) = hovered_tab
        {
            self.state.focused = ordinal;
            if requested.is_none() && index != normalized {
                requested = Some(index);
            }
        }

        self.draw_tabs(rect, labels, normalized, ordinal, hovered_tab);
        requested
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
        let suffix = if *value { "ON" } else { "OFF" };
        self.draw_control(rect, &format!("{label}: {suffix}"), response);
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
        let track = slider_track_rect(rect);
        let mouse = self.input.mouse_position();
        let left_button = self.input.mouse_button(MouseButton::Left);

        if left_button.pressed() && hovered {
            self.state.focused = ordinal;
        }
        if left_button.pressed() && mouse.is_some_and(|position| track.contains(position)) {
            self.state.pointer_active = Some(ordinal);
        }

        let mut changed = false;
        if self.state.pointer_active == Some(ordinal) {
            if let Some((mouse_x, _)) = mouse {
                let before = *value;
                *value = slider_value_from_pointer(mouse_x, track, &range, step);
                changed |= before != *value;
                if left_button.released() || (!left_button.pressed() && !left_button.held()) {
                    self.state.pointer_active = None;
                }
            } else {
                self.state.pointer_active = None;
            }
        }

        let focused = self.state.focused == ordinal;
        if focused {
            let config = RepeatConfig::default();
            if self.state.horizontal_repeat_owner != Some(ordinal) {
                self.state.left_repeat.reset(config);
                self.state.right_repeat.reset(config);
                self.state.horizontal_repeat_owner = Some(ordinal);
            }
            let left =
                self.state
                    .left_repeat
                    .update(self.input.key(Key::Left), self.delta_time, config);
            let right =
                self.state
                    .right_repeat
                    .update(self.input.key(Key::Right), self.delta_time, config);
            let direction = i64::from(right) - i64::from(left);
            if direction != 0 {
                let before = *value;
                *value = stepped_slider_value(*value, &range, step, direction);
                changed |= before != *value;
            }
        } else if self.state.horizontal_repeat_owner == Some(ordinal) {
            let config = RepeatConfig::default();
            self.state.left_repeat.reset(config);
            self.state.right_repeat.reset(config);
            self.state.horizontal_repeat_owner = None;
        }

        let response = UiResponse {
            focused,
            hovered,
            active: self.state.pointer_active == Some(ordinal),
            changed,
            ..UiResponse::default()
        };
        self.draw_slider(rect, track, label, *value, &range, response);
        response
    }

    fn next_row(&mut self) -> Rect {
        let width = self
            .framebuffer
            .width()
            .saturating_sub(self.theme.padding.saturating_mul(2));
        let rect = Rect {
            x: u32_to_i32(self.theme.padding),
            y: self.cursor_y,
            width,
            height: self.theme.row_height,
        };
        let advance = self.theme.row_height.saturating_add(self.theme.row_spacing);
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
        self.draw_control_frame(rect, response);
        self.draw_text_centered(rect, label, self.theme.text);
    }

    fn draw_tabs(
        &mut self,
        rect: Rect,
        labels: &[&str],
        selected: usize,
        ordinal: usize,
        hovered_tab: Option<usize>,
    ) {
        let focused = self.state.focused == ordinal;
        for (index, label) in labels.iter().enumerate() {
            let tab = tab_rect(rect, index, labels.len());
            let is_selected = index == selected;
            let background = if is_selected {
                self.theme.accent
            } else {
                self.theme.control_background
            };
            let foreground = if is_selected {
                self.theme.control_background
            } else {
                self.theme.text
            };
            let border = if hovered_tab == Some(index) {
                self.theme.accent
            } else {
                self.theme.border
            };
            self.framebuffer
                .fill_rect(tab.x, tab.y, tab.width, tab.height, background);
            self.framebuffer
                .draw_rect(tab.x, tab.y, tab.width, tab.height, border);
            self.draw_text_centered(tab, label, foreground);
        }

        let outer_border = if focused {
            self.theme.accent
        } else {
            self.theme.border
        };
        self.framebuffer
            .draw_rect(rect.x, rect.y, rect.width, rect.height, outer_border);
    }

    fn draw_slider(
        &mut self,
        rect: Rect,
        track: Rect,
        label: &str,
        value: f32,
        range: &RangeInclusive<f32>,
        response: UiResponse,
    ) {
        self.draw_control_frame(rect, response);
        let label_rect = Rect {
            x: rect.x.saturating_add(4),
            y: rect.y,
            width: rect.width / 2,
            height: rect.height,
        };
        self.draw_text_left(label_rect, label, self.theme.text);

        let (min, max) = ordered_range(range);
        let ratio = if max > min {
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
        let fill_width = ((track.width as f32) * ratio).round() as u32;
        if fill_width > 0 {
            self.framebuffer.fill_rect(
                track.x,
                track.y,
                fill_width.min(track.width),
                track.height,
                self.theme.accent,
            );
        }
    }

    fn draw_control_frame(&mut self, rect: Rect, response: UiResponse) {
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
    }

    fn text_size(&self, text: &str) -> (u32, u32) {
        self.text_renderer
            .text_size(text, self.theme.text_scale.max(1))
    }

    fn draw_text_centered(&mut self, rect: Rect, text: &str, color: Pixel) {
        let (width, height) = self.text_size(text);
        self.text_renderer.draw_scaled(
            self.framebuffer,
            centered_coordinate(rect.x, rect.width, width),
            centered_coordinate(rect.y, rect.height, height),
            text,
            self.theme.text_scale.max(1),
            color,
        );
    }

    fn draw_text_left(&mut self, rect: Rect, text: &str, color: Pixel) {
        let (_, height) = self.text_size(text);
        self.text_renderer.draw_scaled(
            self.framebuffer,
            rect.x,
            centered_coordinate(rect.y, rect.height, height),
            text,
            self.theme.text_scale.max(1),
            color,
        );
    }
}

impl Drop for Ui<'_> {
    fn drop(&mut self) {
        self.state.previous_interactive_count = self.interactive_count;
        if self.interactive_count == 0 {
            self.state.focused = 0;
            self.state.pointer_active = None;
            self.state.horizontal_repeat_owner = None;
            return;
        }
        self.state.focused = self.state.focused.min(self.interactive_count - 1);
        if self
            .state
            .pointer_active
            .is_some_and(|ordinal| ordinal >= self.interactive_count)
        {
            self.state.pointer_active = None;
        }
    }
}

fn ordered_range(range: &RangeInclusive<f32>) -> (f32, f32) {
    let start = *range.start();
    let end = *range.end();
    if start <= end {
        (start, end)
    } else {
        (end, start)
    }
}

fn snap_to_step(value: f32, min: f32, max: f32, step: f32) -> f32 {
    let value = value.clamp(min, max);
    let step = step.abs();
    if step == 0.0 || max <= min {
        return value;
    }
    (min + ((value - min) / step).round() * step).clamp(min, max)
}

fn stepped_slider_value(value: f32, range: &RangeInclusive<f32>, step: f32, direction: i64) -> f32 {
    let (min, max) = ordered_range(range);
    let step = step.abs();
    if step == 0.0 {
        return value.clamp(min, max);
    }
    let value = snap_to_step(value, min, max, step);
    snap_to_step(value + step * direction as f32, min, max, step)
}

fn slider_value_from_pointer(
    mouse_x: i32,
    track: Rect,
    range: &RangeInclusive<f32>,
    step: f32,
) -> f32 {
    let (min, max) = ordered_range(range);
    if max <= min || track.width <= 1 {
        return min;
    }
    let start = i64::from(track.x);
    let end = start + i64::from(track.width - 1);
    let x = i64::from(mouse_x).clamp(start, end);
    let ratio = (x - start) as f32 / (end - start) as f32;
    snap_to_step(min + (max - min) * ratio, min, max, step)
}

fn slider_track_rect(rect: Rect) -> Rect {
    let x_offset = rect.width / 2;
    let width = rect.width.saturating_sub(x_offset).saturating_sub(8).max(1);
    let height = 5_u32.min(rect.height.max(1));
    Rect {
        x: rect.x.saturating_add(u32_to_i32(x_offset)),
        y: centered_coordinate(rect.y, rect.height, height),
        width,
        height,
    }
}

fn tab_rect(rect: Rect, index: usize, count: usize) -> Rect {
    let count = count as u64;
    let start = (u64::from(rect.width) * index as u64 / count) as u32;
    let end = (u64::from(rect.width) * (index + 1) as u64 / count) as u32;
    Rect {
        x: rect.x.saturating_add(u32_to_i32(start)),
        y: rect.y,
        width: end.saturating_sub(start),
        height: rect.height,
    }
}

fn tab_index_at_position(rect: Rect, count: usize, position: (i32, i32)) -> Option<usize> {
    if count == 0 || rect.width == 0 || !rect.contains(position) {
        return None;
    }
    let relative_x = i64::from(position.0) - i64::from(rect.x);
    let index = (relative_x as u64 * count as u64 / u64::from(rect.width)) as usize;
    Some(index.min(count - 1))
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

    fn three_buttons(input: &Input, state: &mut UiState) -> [UiResponse; 3] {
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
    fn keyboard_focus_moves_and_wraps() {
        let mut state = UiState::default();
        let idle = Input::default();
        assert!(three_buttons(&idle, &mut state)[0].focused);

        let mut down = Input::default();
        down.press_key(Key::Down);
        assert!(three_buttons(&down, &mut state)[1].focused);

        let mut up = Input::default();
        up.press_key(Key::Up);
        assert!(three_buttons(&up, &mut state)[0].focused);
        assert!(three_buttons(&up, &mut state)[2].focused);
        assert_eq!(state.focused_index(), Some(2));
    }

    #[test]
    fn slider_keyboard_step_clamps_to_range() {
        let mut state = UiState::default();
        let mut framebuffer = Framebuffer::new(160, 40);
        let mut value = 0.8;
        let idle = Input::default();
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
    }

    #[test]
    fn slider_drag_cancels_when_pointer_leaves() {
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
        assert!(value > 0.0);

        input.advance_frame();
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
        let cancelled = value;

        input.set_mouse_position(Some((120, 10)));
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
        assert_eq!(value.to_bits(), cancelled.to_bits());
    }
}
