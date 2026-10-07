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

        let left_center_x = bounds.x
            + i32::try_from(button_size.saturating_add(step)).unwrap_or(i32::MAX);
        let right_center_x = bounds.x
            + i32::try_from(bounds.width.saturating_sub(button_size.saturating_add(step)))
                .unwrap_or(i32::MAX);
        let center_y = bounds.y
            + i32::try_from(bounds.height.saturating_sub(button_size.saturating_add(step)))
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
        let utility_total = utility_width
            .saturating_mul(2)
            .saturating_add(utility_gap);
        let utility_x = bounds.x
            + i32::try_from(bounds.width.saturating_sub(utility_total) / 2).unwrap_or(0);
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
            assert!(layout.buttons().iter().any(|button| button.button == expected));
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

        assert!(augmented.gamepad_button_any(GamepadButton::DPadLeft).pressed());
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
        assert!(augmented.gamepad_button_any(GamepadButton::Start).released());
    }
}
