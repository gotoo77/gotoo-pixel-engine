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
    scroll_y: u32,
    previous_content_height: u32,
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
    logical_cursor_y: u32,
    logical_content_bottom: u32,
    applied_scroll_y: u32,
    requested_scroll_y: Option<u32>,
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

        let max_scroll = state
            .previous_content_height
            .saturating_sub(framebuffer.height());
        state.scroll_y = state.scroll_y.min(max_scroll);
        let applied_scroll_y = state.scroll_y;

        Self {
            framebuffer,
            input,
            delta_time,
            state,
            theme,
            text_renderer: TextRenderer::new(theme.font),
            logical_cursor_y: theme.padding,
            logical_content_bottom: 0,
            applied_scroll_y,
            requested_scroll_y: None,
            interactive_count: 0,
        }
    }

    pub fn label(&mut self, text: &str) {
        let rect = self.next_row();
        self.draw_text_left(rect, text, self.theme.muted_text);
    }

    pub fn section(&mut self, title: &str) {
        let rect = self.next_row();
        let (title_width, _) = self.text_size(title);
        self.draw_text_left(rect, title, self.theme.text);

        let gap = 8;
        let used_width = title_width.saturating_add(gap);
        if used_width < rect.width {
            let line_x = rect.x.saturating_add(u32_to_i32(used_width));
            let line_y = centered_coordinate(rect.y, rect.height, 1);
            self.framebuffer.fill_rect(
                line_x,
                line_y,
                rect.width.saturating_sub(used_width),
                1,
                self.theme.border,
            );
        }
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
        let ordinal = self.next_interactive(rect);
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
        let ordinal = self.next_interactive(rect);
        let response = self.click_response(rect, ordinal);
        self.draw_control(rect, label, response);
        response
    }

    pub fn toggle(&mut self, label: &str, value: &mut bool) -> UiResponse {
        let rect = self.next_row();
        let ordinal = self.next_interactive(rect);
        let mut response = self.click_response(rect, ordinal);
        if response.clicked {
            *value = !*value;
            response.changed = true;
        }
        let suffix = if *value { "ON" } else { "OFF" };
        self.draw_control(rect, &format!("{label}: {suffix}"), response);
        response
    }

    pub fn select(&mut self, label: &str, selected: &mut usize, options: &[&str]) -> UiResponse {
        let rect = self.next_row();
        let ordinal = self.next_interactive(rect);
        let hovered = self.pointer_over(rect);
        let left_button = self.input.mouse_button(MouseButton::Left);
        let mut changed = false;
        let mut normalized = false;

        if left_button.pressed() && hovered {
            self.state.focused = ordinal;
        }

        if !options.is_empty() && *selected >= options.len() {
            *selected = options.len() - 1;
            changed = true;
            normalized = true;
        }

        let delta = self.horizontal_repeat_delta(ordinal);
        if !normalized && options.len() > 1 && delta != 0 {
            let before = *selected;
            *selected = wrapped_index(*selected, options.len(), delta);
            changed |= before != *selected;
        }

        let value_rect = Rect {
            x: rect.x.saturating_add(u32_to_i32(rect.width / 2)),
            y: rect.y,
            width: rect.width.saturating_sub(rect.width / 2),
            height: rect.height,
        };
        if !normalized
            && options.len() > 1
            && left_button.pressed()
            && let Some(position) = self.input.mouse_position()
            && value_rect.contains(position)
        {
            let midpoint = i64::from(value_rect.x) + i64::from(value_rect.width) / 2;
            let mouse_delta = if i64::from(position.0) < midpoint {
                -1
            } else {
                1
            };
            let before = *selected;
            *selected = wrapped_index(*selected, options.len(), mouse_delta);
            changed |= before != *selected;
        }

        let response = UiResponse {
            focused: self.state.focused == ordinal,
            hovered,
            changed,
            ..UiResponse::default()
        };
        self.draw_select(rect, value_rect, label, *selected, options, response);
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
        let ordinal = self.next_interactive(rect);
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
        let delta = self.horizontal_repeat_delta(ordinal);
        if delta != 0 {
            let before = *value;
            *value = stepped_slider_value(*value, &range, step, delta);
            changed |= before != *value;
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

    fn horizontal_repeat_delta(&mut self, ordinal: usize) -> i64 {
        let config = RepeatConfig::default();
        if self.state.focused != ordinal {
            if self.state.horizontal_repeat_owner == Some(ordinal) {
                self.state.left_repeat.reset(config);
                self.state.right_repeat.reset(config);
                self.state.horizontal_repeat_owner = None;
            }
            return 0;
        }

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
        i64::from(right) - i64::from(left)
    }

    fn next_row(&mut self) -> Rect {
        let width = self
            .framebuffer
            .width()
            .saturating_sub(self.theme.padding.saturating_mul(2));
        let logical_y = self.logical_cursor_y;
        let physical_y = i64::from(logical_y) - i64::from(self.applied_scroll_y);
        let rect = Rect {
            x: u32_to_i32(self.theme.padding),
            y: clamp_i64_to_i32(physical_y),
            width,
            height: self.theme.row_height,
        };
        self.logical_content_bottom = logical_y.saturating_add(self.theme.row_height);
        let advance = self.theme.row_height.saturating_add(self.theme.row_spacing);
        self.logical_cursor_y = self.logical_cursor_y.saturating_add(advance);
        rect
    }

    fn next_interactive(&mut self, rect: Rect) -> usize {
        let ordinal = self.interactive_count;
        self.interactive_count = self.interactive_count.saturating_add(1);
        if self.state.focused == ordinal {
            self.request_focused_visibility(rect);
        }
        ordinal
    }

    fn request_focused_visibility(&mut self, rect: Rect) {
        let framebuffer_height = self.framebuffer.height();
        let vertical_padding = self.theme.padding.min(framebuffer_height / 2);
        let viewport_top = i64::from(vertical_padding);
        let viewport_bottom = i64::from(framebuffer_height.saturating_sub(vertical_padding));
        let viewport_height = viewport_bottom.saturating_sub(viewport_top);
        if i64::from(rect.height) > viewport_height {
            return;
        }

        let rect_top = i64::from(rect.y);
        let rect_bottom = rect_top.saturating_add(i64::from(rect.height));
        let current_scroll = i64::from(self.applied_scroll_y);
        let requested = if rect_top < viewport_top {
            current_scroll.saturating_sub(viewport_top - rect_top)
        } else if rect_bottom > viewport_bottom {
            current_scroll.saturating_add(rect_bottom - viewport_bottom)
        } else {
            return;
        };
        self.requested_scroll_y = Some(requested.clamp(0, i64::from(u32::MAX)) as u32);
    }

    fn content_height(&self) -> u32 {
        if self.logical_content_bottom == 0 {
            0
        } else {
            self.logical_content_bottom
                .saturating_add(self.theme.padding)
        }
    }

    fn draw_root_scrollbar(&mut self, content_height: u32, scroll_y: u32) {
        let viewport_height = self.framebuffer.height();
        if content_height <= viewport_height || viewport_height == 0 || self.theme.padding < 2 {
            return;
        }

        let horizontal_padding = self.theme.padding.min(self.framebuffer.width());
        if horizontal_padding < 2 {
            return;
        }
        let track_width = 2;
        let track_x = self
            .framebuffer
            .width()
            .saturating_sub(horizontal_padding)
            .saturating_add((horizontal_padding - track_width) / 2);
        let vertical_padding = self.theme.padding.min(viewport_height / 2);
        let track_height = viewport_height.saturating_sub(vertical_padding.saturating_mul(2));
        if track_height == 0 {
            return;
        }

        let proportional_thumb = (u64::from(track_height) * u64::from(viewport_height)
            / u64::from(content_height)) as u32;
        let minimum_thumb = 6_u32.min(track_height);
        let thumb_height = proportional_thumb.max(minimum_thumb).min(track_height);
        let max_scroll = content_height.saturating_sub(viewport_height);
        let travel = track_height.saturating_sub(thumb_height);
        let thumb_offset = if max_scroll == 0 {
            0
        } else {
            (u64::from(travel) * u64::from(scroll_y.min(max_scroll)) / u64::from(max_scroll)) as u32
        };
        let track_y = u32_to_i32(vertical_padding);
        self.framebuffer.fill_rect(
            u32_to_i32(track_x),
            track_y,
            track_width,
            track_height,
            self.theme.border,
        );
        self.framebuffer.fill_rect(
            u32_to_i32(track_x),
            track_y.saturating_add(u32_to_i32(thumb_offset)),
            track_width,
            thumb_height,
            self.theme.accent,
        );
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

    fn draw_select(
        &mut self,
        rect: Rect,
        value_rect: Rect,
        label: &str,
        selected: usize,
        options: &[&str],
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

        let value = options.get(selected).copied().unwrap_or("NONE");
        let color = if options.is_empty() {
            self.theme.muted_text
        } else {
            self.theme.text
        };
        self.draw_text_centered(value_rect, &format!("- {value} +"), color);
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
        let content_height = self.content_height();
        let max_scroll = content_height.saturating_sub(self.framebuffer.height());
        let current_scroll = self.applied_scroll_y.min(max_scroll);
        self.draw_root_scrollbar(content_height, current_scroll);
        self.state.previous_content_height = content_height;
        self.state.scroll_y = self
            .requested_scroll_y
            .unwrap_or(current_scroll)
            .min(max_scroll);

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

fn wrapped_index(index: usize, len: usize, delta: i64) -> usize {
    debug_assert!(len > 0);
    let len = len as i128;
    ((index as i128 + i128::from(delta)).rem_euclid(len)) as usize
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

fn clamp_i64_to_i32(value: i64) -> i32 {
    value.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn u32_to_i32(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compact_theme() -> UiTheme {
        UiTheme {
            padding: 0,
            row_height: 20,
            row_spacing: 0,
            ..UiTheme::default()
        }
    }

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

    fn button_list(input: &Input, state: &mut UiState, count: usize, height: u32) {
        let mut framebuffer = Framebuffer::new(120, height);
        let mut ui = Ui::new(
            &mut framebuffer,
            input,
            Duration::from_millis(16),
            state,
            compact_theme(),
        );
        for _ in 0..count {
            ui.button("ROW");
        }
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
        let theme = compact_theme();
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

    #[test]
    fn select_keyboard_wraps_and_reports_changes() {
        let theme = compact_theme();
        let mut state = UiState::default();
        let mut framebuffer = Framebuffer::new(200, 20);
        let mut selected = 0;

        let mut left = Input::default();
        left.press_key(Key::Left);
        let response = {
            let mut ui = Ui::new(&mut framebuffer, &left, Duration::ZERO, &mut state, theme);
            ui.select("DIRECTION", &mut selected, &["A", "B", "C"])
        };
        assert_eq!(selected, 2);
        assert!(response.changed);

        state.reset_interaction();
        let mut right = Input::default();
        right.press_key(Key::Right);
        let response = {
            let mut ui = Ui::new(&mut framebuffer, &right, Duration::ZERO, &mut state, theme);
            ui.select("DIRECTION", &mut selected, &["A", "B", "C"])
        };
        assert_eq!(selected, 0);
        assert!(response.changed);
    }

    #[test]
    fn select_repeat_preserves_long_frame_pulse_count() {
        let theme = compact_theme();
        let mut state = UiState::default();
        let mut framebuffer = Framebuffer::new(200, 20);
        let options = ["0", "1", "2", "3", "4", "5", "6"];
        let mut selected = 0;
        let mut input = Input::default();

        input.press_key(Key::Right);
        {
            let mut ui = Ui::new(&mut framebuffer, &input, Duration::ZERO, &mut state, theme);
            let response = ui.select("VALUE", &mut selected, &options);
            assert!(response.changed);
        }
        assert_eq!(selected, 1);

        input.advance_frame();
        {
            let mut ui = Ui::new(
                &mut framebuffer,
                &input,
                Duration::from_millis(299),
                &mut state,
                theme,
            );
            assert!(!ui.select("VALUE", &mut selected, &options).changed);
        }
        assert_eq!(selected, 1);

        input.advance_frame();
        {
            let mut ui = Ui::new(
                &mut framebuffer,
                &input,
                Duration::from_millis(181),
                &mut state,
                theme,
            );
            assert!(ui.select("VALUE", &mut selected, &options).changed);
        }
        assert_eq!(selected, 5);
    }

    #[test]
    fn select_edges_normalize_without_losing_ordinal() {
        let theme = compact_theme();
        let idle = Input::default();
        let mut framebuffer = Framebuffer::new(200, 40);
        let mut state = UiState::default();
        let mut selected = 9;

        let (empty, button) = {
            let mut ui = Ui::new(
                &mut framebuffer,
                &idle,
                Duration::from_millis(16),
                &mut state,
                theme,
            );
            let empty = ui.select("EMPTY", &mut selected, &[]);
            let button = ui.button("AFTER");
            (empty, button)
        };
        assert!(!empty.changed);
        assert_eq!(selected, 9);
        assert!(empty.focused);
        assert!(!button.focused);
        assert_eq!(state.focused_index(), Some(0));

        let mut down = Input::default();
        down.press_key(Key::Down);
        let button = {
            let mut ui = Ui::new(
                &mut framebuffer,
                &down,
                Duration::from_millis(16),
                &mut state,
                theme,
            );
            ui.select("EMPTY", &mut selected, &[]);
            ui.button("AFTER")
        };
        assert!(button.focused);
        assert_eq!(state.focused_index(), Some(1));

        state.reset_interaction();
        selected = usize::MAX;
        let normalized = {
            let mut ui = Ui::new(
                &mut framebuffer,
                &idle,
                Duration::from_millis(16),
                &mut state,
                theme,
            );
            ui.select("ONE", &mut selected, &["ONLY"])
        };
        assert_eq!(selected, 0);
        assert!(normalized.changed);
    }

    #[test]
    fn select_mouse_label_focuses_without_change_and_value_changes_without_capture() {
        let theme = compact_theme();
        let mut state = UiState::default();
        let mut framebuffer = Framebuffer::new(200, 20);
        let options = ["A", "B", "C"];
        let mut selected = 1;

        let mut label_click = Input::default();
        label_click.set_mouse_position(Some((20, 10)));
        label_click.press_mouse_button(MouseButton::Left);
        let response = {
            let mut ui = Ui::new(
                &mut framebuffer,
                &label_click,
                Duration::from_millis(16),
                &mut state,
                theme,
            );
            ui.select("VALUE", &mut selected, &options)
        };
        assert!(response.focused);
        assert!(!response.changed);
        assert_eq!(selected, 1);
        assert_eq!(state.pointer_active, None);

        state.reset_interaction();
        let mut left_value_click = Input::default();
        left_value_click.set_mouse_position(Some((120, 10)));
        left_value_click.press_mouse_button(MouseButton::Left);
        let response = {
            let mut ui = Ui::new(
                &mut framebuffer,
                &left_value_click,
                Duration::from_millis(16),
                &mut state,
                theme,
            );
            ui.select("VALUE", &mut selected, &options)
        };
        assert!(response.changed);
        assert_eq!(selected, 0);
        assert_eq!(state.pointer_active, None);

        state.reset_interaction();
        let mut right_value_click = Input::default();
        right_value_click.set_mouse_position(Some((180, 10)));
        right_value_click.press_mouse_button(MouseButton::Left);
        let response = {
            let mut ui = Ui::new(
                &mut framebuffer,
                &right_value_click,
                Duration::from_millis(16),
                &mut state,
                theme,
            );
            ui.select("VALUE", &mut selected, &options)
        };
        assert!(response.changed);
        assert_eq!(selected, 1);
        assert_eq!(state.pointer_active, None);
    }

    #[test]
    fn section_does_not_consume_an_ordinal() {
        let theme = compact_theme();
        let idle = Input::default();
        let mut state = UiState::default();
        let mut framebuffer = Framebuffer::new(200, 40);

        let button = {
            let mut ui = Ui::new(
                &mut framebuffer,
                &idle,
                Duration::from_millis(16),
                &mut state,
                theme,
            );
            ui.section("INPUT");
            ui.button("FIRST")
        };
        assert!(button.focused);
        assert_eq!(state.focused_index(), Some(0));
    }

    #[test]
    fn slider_repeat_is_unchanged_after_horizontal_helper_extraction() {
        let theme = compact_theme();
        let mut state = UiState::default();
        let mut framebuffer = Framebuffer::new(200, 20);
        let mut value = 0.0;
        let mut input = Input::default();

        input.press_key(Key::Right);
        {
            let mut ui = Ui::new(&mut framebuffer, &input, Duration::ZERO, &mut state, theme);
            assert!(ui.slider_f32("VALUE", &mut value, 0.0..=1.0, 0.1).changed);
        }
        assert!((value - 0.1).abs() < f32::EPSILON);

        input.advance_frame();
        {
            let mut ui = Ui::new(
                &mut framebuffer,
                &input,
                Duration::from_millis(480),
                &mut state,
                theme,
            );
            assert!(ui.slider_f32("VALUE", &mut value, 0.0..=1.0, 0.1).changed);
        }
        assert!((value - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn content_height_is_logical_and_scroll_does_not_drift() {
        let mut state = UiState::default();
        let idle = Input::default();
        button_list(&idle, &mut state, 3, 40);
        assert_eq!(state.previous_content_height, 60);
        assert_eq!(state.scroll_y, 0);

        let mut up = Input::default();
        up.press_key(Key::Up);
        button_list(&up, &mut state, 3, 40);
        assert_eq!(state.focused_index(), Some(2));
        assert_eq!(state.previous_content_height, 60);
        assert_eq!(state.scroll_y, 20);

        button_list(&idle, &mut state, 3, 40);
        assert_eq!(state.previous_content_height, 60);
        assert_eq!(state.scroll_y, 20);
        button_list(&idle, &mut state, 3, 40);
        assert_eq!(state.previous_content_height, 60);
        assert_eq!(state.scroll_y, 20);
    }

    #[test]
    fn focus_wrap_requests_root_scroll_in_both_directions() {
        let mut state = UiState::default();
        let idle = Input::default();
        button_list(&idle, &mut state, 3, 40);

        let mut up = Input::default();
        up.press_key(Key::Up);
        button_list(&up, &mut state, 3, 40);
        assert_eq!(state.focused_index(), Some(2));
        assert_eq!(state.scroll_y, 20);

        let mut down = Input::default();
        down.press_key(Key::Down);
        button_list(&down, &mut state, 3, 40);
        assert_eq!(state.focused_index(), Some(0));
        assert_eq!(state.scroll_y, 0);
    }

    #[test]
    fn shorter_content_clamps_scroll_at_end_of_frame() {
        let mut state = UiState::default();
        let idle = Input::default();
        button_list(&idle, &mut state, 3, 40);

        let mut up = Input::default();
        up.press_key(Key::Up);
        button_list(&up, &mut state, 3, 40);
        assert_eq!(state.scroll_y, 20);

        button_list(&idle, &mut state, 1, 40);
        assert_eq!(state.previous_content_height, 20);
        assert_eq!(state.scroll_y, 0);
        assert_eq!(state.focused_index(), Some(0));
    }

    #[test]
    fn structural_reset_clears_scroll_and_cached_content_height() {
        let mut state = UiState::default();
        let idle = Input::default();
        button_list(&idle, &mut state, 3, 40);

        let mut up = Input::default();
        up.press_key(Key::Up);
        button_list(&up, &mut state, 3, 40);
        assert_eq!(state.scroll_y, 20);
        assert_eq!(state.previous_content_height, 60);

        state.reset_interaction();
        assert_eq!(state.scroll_y, 0);
        assert_eq!(state.previous_content_height, 0);
        assert_eq!(state.focused_index(), None);
    }
}
