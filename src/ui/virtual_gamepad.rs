use std::collections::{HashMap, HashSet};

use crate::{ButtonState, Framebuffer, GamepadButton, GamepadId, Input, Pixel, Rect, TouchPhase};

const VIRTUAL_GAMEPAD_ID: GamepadId = GamepadId::new(usize::MAX);

const STANDARD_BUTTONS: [GamepadButton; 10] = [
    GamepadButton::DPadUp,
    GamepadButton::DPadDown,
    GamepadButton::DPadLeft,
    GamepadButton::DPadRight,
    GamepadButton::South,
    GamepadButton::East,
    GamepadButton::West,
    GamepadButton::North,
    GamepadButton::Start,
    GamepadButton::Select,
];

const TRACKBALL_BUTTONS: [GamepadButton; 4] = [
    GamepadButton::LeftStickUp,
    GamepadButton::LeftStickDown,
    GamepadButton::LeftStickLeft,
    GamepadButton::LeftStickRight,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtualGamepadButton {
    pub button: GamepadButton,
    pub rect: Rect,
    pub label: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualGamepadLayout {
    buttons: Vec<VirtualGamepadButton>,
}

impl VirtualGamepadLayout {
    /// Builds a conventional mobile layout inside the supplied physical bounds.
    ///
    /// The D-pad is placed bottom-left, face buttons bottom-right, and utility
    /// buttons near the bottom centre. The layout operates in the same coordinate
    /// space as the incoming touch events.
    pub fn standard(bounds: Rect) -> Self {
        let min_extent = bounds.width.min(bounds.height);
        let button_size = (min_extent / 9).clamp(40, 72);
        let gap = (button_size / 6).max(6);
        let step = button_size.saturating_add(gap);

        let left_center_x =
            bounds.x + i32::try_from(button_size.saturating_add(step)).unwrap_or(i32::MAX);
        let right_center_x = bounds.x
            + i32::try_from(
                bounds
                    .width
                    .saturating_sub(button_size.saturating_add(step)),
            )
            .unwrap_or(i32::MAX);
        let center_y = bounds.y
            + i32::try_from(
                bounds
                    .height
                    .saturating_sub(button_size.saturating_add(step)),
            )
            .unwrap_or(i32::MAX);

        let square = |center_x: i32, center_y: i32| Rect {
            x: center_x - i32::try_from(button_size / 2).unwrap_or(0),
            y: center_y - i32::try_from(button_size / 2).unwrap_or(0),
            width: button_size,
            height: button_size,
        };

        let step_i = i32::try_from(step).unwrap_or(i32::MAX);
        let mut buttons = vec![
            VirtualGamepadButton {
                button: GamepadButton::DPadUp,
                rect: square(left_center_x, center_y - step_i),
                label: "UP",
            },
            VirtualGamepadButton {
                button: GamepadButton::DPadDown,
                rect: square(left_center_x, center_y + step_i),
                label: "DN",
            },
            VirtualGamepadButton {
                button: GamepadButton::DPadLeft,
                rect: square(left_center_x - step_i, center_y),
                label: "LT",
            },
            VirtualGamepadButton {
                button: GamepadButton::DPadRight,
                rect: square(left_center_x + step_i, center_y),
                label: "RT",
            },
            VirtualGamepadButton {
                button: GamepadButton::North,
                rect: square(right_center_x, center_y - step_i),
                label: "Y",
            },
            VirtualGamepadButton {
                button: GamepadButton::South,
                rect: square(right_center_x, center_y + step_i),
                label: "A",
            },
            VirtualGamepadButton {
                button: GamepadButton::West,
                rect: square(right_center_x - step_i, center_y),
                label: "X",
            },
            VirtualGamepadButton {
                button: GamepadButton::East,
                rect: square(right_center_x + step_i, center_y),
                label: "B",
            },
        ];

        let utility_width = button_size.saturating_add(button_size / 2);
        let utility_height = (button_size / 2).max(28);
        let utility_gap = gap.saturating_mul(2);
        let utility_total = utility_width.saturating_mul(2).saturating_add(utility_gap);
        let utility_x =
            bounds.x + i32::try_from(bounds.width.saturating_sub(utility_total) / 2).unwrap_or(0);
        let utility_y = bounds.y
            + i32::try_from(bounds.height.saturating_sub(utility_height + gap)).unwrap_or(0);

        buttons.push(VirtualGamepadButton {
            button: GamepadButton::Select,
            rect: Rect {
                x: utility_x,
                y: utility_y,
                width: utility_width,
                height: utility_height,
            },
            label: "SELECT",
        });
        buttons.push(VirtualGamepadButton {
            button: GamepadButton::Start,
            rect: Rect {
                x: utility_x
                    + i32::try_from(utility_width.saturating_add(utility_gap)).unwrap_or(i32::MAX),
                y: utility_y,
                width: utility_width,
                height: utility_height,
            },
            label: "START",
        });

        Self { buttons }
    }

    /// Builds only the face + utility buttons, leaving movement to another
    /// virtual control such as VirtualTrackball.
    pub fn actions(bounds: Rect) -> Self {
        let mut layout = Self::standard(bounds);
        layout.buttons.retain(|button| {
            !matches!(
                button.button,
                GamepadButton::DPadUp
                    | GamepadButton::DPadDown
                    | GamepadButton::DPadLeft
                    | GamepadButton::DPadRight
            )
        });
        layout
    }

    pub fn buttons(&self) -> &[VirtualGamepadButton] {
        &self.buttons
    }

    fn button_at(&self, point: (i32, i32)) -> Option<GamepadButton> {
        self.buttons
            .iter()
            .find(|button| button.rect.contains(point))
            .map(|button| button.button)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtualGamepadStyle {
    pub border: Pixel,
    pub active: Pixel,
    pub text: Pixel,
}

impl Default for VirtualGamepadStyle {
    fn default() -> Self {
        Self {
            border: Pixel::rgb(118, 132, 150),
            active: Pixel::rgb(215, 185, 92),
            text: Pixel::rgb(232, 238, 242),
        }
    }
}

/// Standard gamepad-like mobile controller.
///
/// Touch contacts are tracked independently, so holding one direction while
/// pressing an action button works naturally. The resulting synthetic gamepad
/// is merged into a cloned Input snapshot; physical keyboard/gamepad state is
/// preserved unchanged.
#[derive(Debug, Default, Clone)]
pub struct VirtualGamepad {
    contacts: HashMap<u64, GamepadButton>,
    previous_held: HashSet<GamepadButton>,
    visible: bool,
}

impl VirtualGamepad {
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn visible(&self) -> bool {
        self.visible
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.contacts.clear();
        self.previous_held.clear();
    }

    pub fn reset(&mut self) {
        self.contacts.clear();
        self.previous_held.clear();
    }

    /// Returns an Input snapshot augmented with one synthetic standard gamepad.
    pub fn update_input(&mut self, input: &Input, layout: &VirtualGamepadLayout) -> Input {
        if !input.touches().is_empty() {
            self.visible = true;
        }

        for touch in input.touches() {
            match touch.phase {
                TouchPhase::Started | TouchPhase::Moved => {
                    let next = touch.position.and_then(|point| layout.button_at(point));
                    if let Some(button) = next {
                        self.contacts.insert(touch.id, button);
                    } else {
                        self.contacts.remove(&touch.id);
                    }
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    self.contacts.remove(&touch.id);
                }
            }
        }

        let held = self.contacts.values().copied().collect::<HashSet<_>>();
        let mut augmented = input.clone();
        for button in STANDARD_BUTTONS {
            let state = ButtonState::from_transition(
                self.previous_held.contains(&button),
                held.contains(&button),
            );
            augmented.set_gamepad_button_state(VIRTUAL_GAMEPAD_ID, button, state);
        }
        self.previous_held = held;
        augmented
    }

    pub fn held(&self, button: GamepadButton) -> bool {
        self.previous_held.contains(&button)
    }

    pub fn render(
        &self,
        framebuffer: &mut Framebuffer,
        layout: &VirtualGamepadLayout,
        style: VirtualGamepadStyle,
    ) {
        if !self.visible {
            return;
        }

        for button in layout.buttons() {
            let active = self.held(button.button);
            let color = if active { style.active } else { style.border };
            framebuffer.draw_rect(
                button.rect.x,
                button.rect.y,
                button.rect.width,
                button.rect.height,
                color,
            );
            if active {
                let inset = 4_u32.min(button.rect.width / 4).min(button.rect.height / 4);
                framebuffer.draw_rect(
                    button.rect.x + i32::try_from(inset).unwrap_or(0),
                    button.rect.y + i32::try_from(inset).unwrap_or(0),
                    button.rect.width.saturating_sub(inset.saturating_mul(2)),
                    button.rect.height.saturating_sub(inset.saturating_mul(2)),
                    color,
                );
            }

            let (text_width, text_height) = Framebuffer::text_size(button.label, 1);
            let text_x = button.rect.x
                + i32::try_from(button.rect.width.saturating_sub(text_width) / 2).unwrap_or(0);
            let text_y = button.rect.y
                + i32::try_from(button.rect.height.saturating_sub(text_height) / 2).unwrap_or(0);
            framebuffer.draw_text(text_x, text_y, button.label, style.text);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtualTrackballStyle {
    pub border: Pixel,
    pub active: Pixel,
    pub knob: Pixel,
    pub text: Pixel,
}

impl Default for VirtualTrackballStyle {
    fn default() -> Self {
        Self {
            border: Pixel::rgb(82, 105, 128),
            active: Pixel::rgb(96, 194, 224),
            knob: Pixel::rgb(220, 238, 248),
            text: Pixel::rgb(168, 190, 205),
        }
    }
}

/// Floating relative movement control for touch devices.
///
/// The first touch starting inside area becomes the movement contact. The
/// touch origin is the neutral point; dragging away from it produces synthetic
/// left-stick direction buttons, including diagonals. Sparse touch frames keep
/// the current direction held until the captured contact moves or ends.
#[derive(Debug, Clone)]
pub struct VirtualTrackball {
    contact_id: Option<u64>,
    anchor: Option<(i32, i32)>,
    current: Option<(i32, i32)>,
    previous_held: HashSet<GamepadButton>,
    visible: bool,
    deadzone: i32,
    radius: i32,
}

impl Default for VirtualTrackball {
    fn default() -> Self {
        Self {
            contact_id: None,
            anchor: None,
            current: None,
            previous_held: HashSet::new(),
            visible: false,
            deadzone: 18,
            radius: 72,
        }
    }
}

impl VirtualTrackball {
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn visible(&self) -> bool {
        self.visible
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.reset();
    }

    pub fn reset(&mut self) {
        self.contact_id = None;
        self.anchor = None;
        self.current = None;
        self.previous_held.clear();
    }

    pub fn with_deadzone(mut self, deadzone: i32) -> Self {
        self.deadzone = deadzone.max(0);
        self
    }

    pub fn with_radius(mut self, radius: i32) -> Self {
        self.radius = radius.max(1);
        self
    }

    fn held_buttons(&self) -> HashSet<GamepadButton> {
        let Some(anchor) = self.anchor else {
            return HashSet::new();
        };
        let Some(current) = self.current else {
            return HashSet::new();
        };
        let dx = current.0.saturating_sub(anchor.0);
        let dy = current.1.saturating_sub(anchor.1);
        let mut held = HashSet::new();

        if dx < -self.deadzone {
            held.insert(GamepadButton::LeftStickLeft);
        } else if dx > self.deadzone {
            held.insert(GamepadButton::LeftStickRight);
        }
        if dy < -self.deadzone {
            held.insert(GamepadButton::LeftStickUp);
        } else if dy > self.deadzone {
            held.insert(GamepadButton::LeftStickDown);
        }

        held
    }

    /// Returns an Input snapshot augmented with synthetic left-stick directions.
    pub fn update_input(&mut self, input: &Input, area: Rect) -> Input {
        if !input.touches().is_empty() {
            self.visible = true;
        }

        for touch in input.touches() {
            match touch.phase {
                TouchPhase::Started => {
                    if self.contact_id.is_none()
                        && let Some(point) = touch.position
                        && area.contains(point)
                    {
                        self.contact_id = Some(touch.id);
                        self.anchor = Some(point);
                        self.current = Some(point);
                    }
                }
                TouchPhase::Moved => {
                    if self.contact_id == Some(touch.id)
                        && let Some(point) = touch.position
                    {
                        self.current = Some(point);
                    }
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    if self.contact_id == Some(touch.id) {
                        self.contact_id = None;
                        self.anchor = None;
                        self.current = None;
                    }
                }
            }
        }

        let held = self.held_buttons();
        let mut augmented = input.clone();
        for button in TRACKBALL_BUTTONS {
            let state = ButtonState::from_transition(
                self.previous_held.contains(&button),
                held.contains(&button),
            );
            augmented.set_gamepad_button_state(VIRTUAL_GAMEPAD_ID, button, state);
        }
        self.previous_held = held;
        augmented
    }

    pub fn render(
        &self,
        framebuffer: &mut Framebuffer,
        area: Rect,
        style: VirtualTrackballStyle,
    ) {
        if !self.visible {
            return;
        }

        let fallback = (
            area.x + i32::try_from(area.width / 2).unwrap_or(0),
            area.y + i32::try_from(area.height / 2).unwrap_or(0),
        );
        let center = self.anchor.unwrap_or(fallback);
        let raw = self.current.unwrap_or(center);
        let dx = raw.0.saturating_sub(center.0);
        let dy = raw.1.saturating_sub(center.1);
        let distance = ((i64::from(dx) * i64::from(dx) + i64::from(dy) * i64::from(dy)) as f64)
            .sqrt();
        let scale = if distance > f64::from(self.radius) && distance > 0.0 {
            f64::from(self.radius) / distance
        } else {
            1.0
        };
        let knob = (
            center.0.saturating_add((f64::from(dx) * scale).round() as i32),
            center.1.saturating_add((f64::from(dy) * scale).round() as i32),
        );

        let active = self.contact_id.is_some();
        let ring = if active { style.active } else { style.border };
        framebuffer.draw_circle(center.0, center.1, self.radius as u32, ring);
        framebuffer.draw_circle(
            center.0,
            center.1,
            self.deadzone.max(1) as u32,
            style.border,
        );
        framebuffer.fill_circle(knob.0, knob.1, 14, style.knob);

        let label_y = area
            .y
            .saturating_add(i32::try_from(area.height).unwrap_or(0))
            .saturating_sub(22);
        framebuffer.draw_text(area.x + 8, label_y, "MOVE", style.text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Touch;

    fn bounds() -> Rect {
        Rect {
            x: 0,
            y: 0,
            width: 540,
            height: 960,
        }
    }

    #[test]
    fn standard_layout_exposes_dpad_face_and_utility_buttons() {
        let layout = VirtualGamepadLayout::standard(bounds());
        for expected in STANDARD_BUTTONS {
            assert!(
                layout
                    .buttons()
                    .iter()
                    .any(|button| button.button == expected)
            );
        }
    }

    #[test]
    fn standard_layout_stays_inside_portrait_and_landscape_bounds() {
        for bounds in [
            Rect {
                x: 0,
                y: 0,
                width: 540,
                height: 960,
            },
            Rect {
                x: 0,
                y: 0,
                width: 1280,
                height: 960,
            },
        ] {
            let layout = VirtualGamepadLayout::standard(bounds);
            for button in layout.buttons() {
                assert!(button.rect.x >= bounds.x);
                assert!(button.rect.y >= bounds.y);
                assert!(
                    button.rect.x + i32::try_from(button.rect.width).unwrap_or(i32::MAX)
                        <= bounds.x + i32::try_from(bounds.width).unwrap_or(i32::MAX)
                );
                assert!(
                    button.rect.y + i32::try_from(button.rect.height).unwrap_or(i32::MAX)
                        <= bounds.y + i32::try_from(bounds.height).unwrap_or(i32::MAX)
                );
            }
        }
    }

    #[test]
    fn action_layout_keeps_face_and_utility_buttons_without_dpad() {
        let layout = VirtualGamepadLayout::actions(bounds());
        for button in [
            GamepadButton::South,
            GamepadButton::East,
            GamepadButton::West,
            GamepadButton::North,
            GamepadButton::Start,
            GamepadButton::Select,
        ] {
            assert!(layout.buttons().iter().any(|item| item.button == button));
        }
        for button in [
            GamepadButton::DPadUp,
            GamepadButton::DPadDown,
            GamepadButton::DPadLeft,
            GamepadButton::DPadRight,
        ] {
            assert!(!layout.buttons().iter().any(|item| item.button == button));
        }
    }

    #[test]
    fn trackball_supports_diagonal_hold_and_release() {
        let area = Rect {
            x: 0,
            y: 480,
            width: 260,
            height: 480,
        };
        let mut trackball = VirtualTrackball::new().with_deadzone(10);

        let mut started = Input::default();
        started.push_touch(Touch {
            id: 31,
            phase: TouchPhase::Started,
            position: Some((100, 700)),
        });
        let neutral = trackball.update_input(&started, area);
        assert!(!neutral.gamepad_button_any(GamepadButton::LeftStickRight).held());

        let mut moved = Input::default();
        moved.push_touch(Touch {
            id: 31,
            phase: TouchPhase::Moved,
            position: Some((145, 655)),
        });
        let diagonal = trackball.update_input(&moved, area);
        assert!(diagonal.gamepad_button_any(GamepadButton::LeftStickRight).pressed());
        assert!(diagonal.gamepad_button_any(GamepadButton::LeftStickUp).pressed());

        let sparse = trackball.update_input(&Input::default(), area);
        assert!(sparse.gamepad_button_any(GamepadButton::LeftStickRight).held());
        assert!(sparse.gamepad_button_any(GamepadButton::LeftStickUp).held());

        let mut ended = Input::default();
        ended.push_touch(Touch {
            id: 31,
            phase: TouchPhase::Ended,
            position: None,
        });
        let released = trackball.update_input(&ended, area);
        assert!(released.gamepad_button_any(GamepadButton::LeftStickRight).released());
        assert!(released.gamepad_button_any(GamepadButton::LeftStickUp).released());
    }

    #[test]
    fn trackball_ignores_touch_started_outside_its_area() {
        let area = Rect {
            x: 0,
            y: 480,
            width: 260,
            height: 480,
        };
        let mut trackball = VirtualTrackball::new();
        let mut input = Input::default();
        input.push_touch(Touch {
            id: 9,
            phase: TouchPhase::Started,
            position: Some((400, 700)),
        });
        let augmented = trackball.update_input(&input, area);
        assert!(!augmented.gamepad_button_any(GamepadButton::LeftStickLeft).held());
        assert!(!augmented.gamepad_button_any(GamepadButton::LeftStickRight).held());
    }

    #[test]
    fn multitouch_holds_direction_and_action_simultaneously() {
        let layout = VirtualGamepadLayout::standard(bounds());
        let left = layout
            .buttons()
            .iter()
            .find(|button| button.button == GamepadButton::DPadLeft)
            .unwrap()
            .rect;
        let south = layout
            .buttons()
            .iter()
            .find(|button| button.button == GamepadButton::South)
            .unwrap()
            .rect;
        let center = |rect: Rect| {
            (
                rect.x + i32::try_from(rect.width / 2).unwrap_or(0),
                rect.y + i32::try_from(rect.height / 2).unwrap_or(0),
            )
        };

        let mut input = Input::default();
        input.push_touch(Touch {
            id: 1,
            phase: TouchPhase::Started,
            position: Some(center(left)),
        });
        input.push_touch(Touch {
            id: 2,
            phase: TouchPhase::Started,
            position: Some(center(south)),
        });

        let mut pad = VirtualGamepad::new();
        let augmented = pad.update_input(&input, &layout);

        assert!(
            augmented
                .gamepad_button_any(GamepadButton::DPadLeft)
                .pressed()
        );
        assert!(augmented.gamepad_button_any(GamepadButton::DPadLeft).held());
        assert!(augmented.gamepad_button_any(GamepadButton::South).pressed());
        assert!(augmented.gamepad_button_any(GamepadButton::South).held());
    }

    #[test]
    fn held_and_release_transitions_survive_sparse_touch_frames() {
        let layout = VirtualGamepadLayout::standard(bounds());
        let south = layout
            .buttons()
            .iter()
            .find(|button| button.button == GamepadButton::South)
            .unwrap()
            .rect;
        let point = (
            south.x + i32::try_from(south.width / 2).unwrap_or(0),
            south.y + i32::try_from(south.height / 2).unwrap_or(0),
        );
        let mut pad = VirtualGamepad::new();

        let mut started = Input::default();
        started.push_touch(Touch {
            id: 9,
            phase: TouchPhase::Started,
            position: Some(point),
        });
        let first = pad.update_input(&started, &layout);
        assert!(first.gamepad_button_any(GamepadButton::South).pressed());

        let held = pad.update_input(&Input::default(), &layout);
        assert!(!held.gamepad_button_any(GamepadButton::South).pressed());
        assert!(held.gamepad_button_any(GamepadButton::South).held());

        let mut ended = Input::default();
        ended.push_touch(Touch {
            id: 9,
            phase: TouchPhase::Ended,
            position: None,
        });
        let released = pad.update_input(&ended, &layout);
        assert!(released.gamepad_button_any(GamepadButton::South).released());
        assert!(!released.gamepad_button_any(GamepadButton::South).held());
    }

    #[test]
    fn cancelled_touch_releases_virtual_button() {
        let layout = VirtualGamepadLayout::standard(bounds());
        let start = layout
            .buttons()
            .iter()
            .find(|button| button.button == GamepadButton::Start)
            .unwrap()
            .rect;
        let point = (
            start.x + i32::try_from(start.width / 2).unwrap_or(0),
            start.y + i32::try_from(start.height / 2).unwrap_or(0),
        );
        let mut pad = VirtualGamepad::new();

        let mut input = Input::default();
        input.push_touch(Touch {
            id: 4,
            phase: TouchPhase::Started,
            position: Some(point),
        });
        pad.update_input(&input, &layout);

        let mut cancelled = Input::default();
        cancelled.push_touch(Touch {
            id: 4,
            phase: TouchPhase::Cancelled,
            position: None,
        });
        let augmented = pad.update_input(&cancelled, &layout);
        assert!(
            augmented
                .gamepad_button_any(GamepadButton::Start)
                .released()
        );
    }
}
