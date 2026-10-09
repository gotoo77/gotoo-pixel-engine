use std::collections::{HashMap, HashSet};

use crate::{ActionId, ControlMap, Input, MouseButton, Rect, Touch, TouchPhase};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtualButton {
    pub action: ActionId,
    pub rect: Rect,
}

impl VirtualButton {
    pub const fn new(action: ActionId, rect: Rect) -> Self {
        Self { action, rect }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct VirtualPadUpdate {
    pressed_actions: Vec<ActionId>,
}

impl VirtualPadUpdate {
    /// Ordered action entries observed while processing this frame's touch events.
    ///
    /// Unlike the final `ControlMap` state, this preserves intermediate zone
    /// changes such as RIGHT -> UP -> LEFT occurring inside one frame.
    pub fn pressed_actions(&self) -> &[ActionId] {
        &self.pressed_actions
    }
}

#[derive(Debug, Default, Clone)]
pub struct VirtualPad {
    buttons: Vec<VirtualButton>,
    contacts: HashMap<u64, ActionId>,
    mouse_contact: Option<ActionId>,
    mouse_suppression_frames: u8,
    visible: bool,
}

impl VirtualPad {
    pub fn new(buttons: impl IntoIterator<Item = VirtualButton>) -> Self {
        Self {
            buttons: buttons.into_iter().collect(),
            contacts: HashMap::new(),
            mouse_contact: None,
            mouse_suppression_frames: 0,
            visible: false,
        }
    }

    pub fn buttons(&self) -> &[VirtualButton] {
        &self.buttons
    }

    pub const fn visible(&self) -> bool {
        self.visible
    }

    /// Maps the current frame's raw touch events to virtual actions.
    ///
    /// Call this before `ControlMap::update` so keyboard, gamepad and touch
    /// sources are folded into the same `ButtonState` transition.
    ///
    /// The returned update also preserves ordered action-entry events inside
    /// the frame. Consumers such as Snake can use those events when several
    /// touch moves must not be collapsed into the final held state.
    pub fn update(&mut self, input: &Input, controls: &mut ControlMap) -> VirtualPadUpdate {
        let mut pressed = self.process_touches(input.touches());
        // Browsers may emit compatibility mouse events after a real touch.
        // Keep one input owner per interaction rather than counting it twice.
        if !input.touches().is_empty() || !self.contacts.is_empty() {
            self.mouse_suppression_frames = 3;
        } else {
            self.mouse_suppression_frames = self.mouse_suppression_frames.saturating_sub(1);
        }
        let mouse = input.mouse_button(MouseButton::Left);
        if self.mouse_suppression_frames > 0 || !mouse.held() {
            self.mouse_contact = None;
        } else {
            let next = input.mouse_position().and_then(|pos| self.action_at(pos));
            if let Some(action) = next
                && self.mouse_contact != Some(action)
            {
                pressed.push(action);
                self.visible = true;
            }
            self.mouse_contact = next;
        }
        self.apply_virtual_states(controls);
        VirtualPadUpdate { pressed_actions: pressed }
    }

    pub fn reset(&mut self, controls: &mut ControlMap) {
        self.contacts.clear();
        self.mouse_contact = None;
        self.mouse_suppression_frames = 0;
        self.apply_virtual_states(controls);
    }

    fn update_touches(&mut self, touches: &[Touch], controls: &mut ControlMap) -> VirtualPadUpdate {
        let pressed_actions = self.process_touches(touches);
        self.apply_virtual_states(controls);
        VirtualPadUpdate { pressed_actions }
    }

    fn process_touches(&mut self, touches: &[Touch]) -> Vec<ActionId> {
        if !touches.is_empty() {
            self.visible = true;
        }

        let mut pressed_actions = Vec::new();
        for touch in touches {
            match touch.phase {
                TouchPhase::Started | TouchPhase::Moved => {
                    if let Some(action) = self.update_contact(*touch) {
                        pressed_actions.push(action);
                    }
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    self.contacts.remove(&touch.id);
                }
            }
        }

        pressed_actions
    }

    fn update_contact(&mut self, touch: Touch) -> Option<ActionId> {
        let previous = self.contacts.get(&touch.id).copied();
        let action = touch.position.and_then(|position| self.action_at(position));

        match action {
            Some(action) => {
                self.contacts.insert(touch.id, action);
                (previous != Some(action)).then_some(action)
            }
            None => {
                self.contacts.remove(&touch.id);
                None
            }
        }
    }

    fn action_at(&self, position: (i32, i32)) -> Option<ActionId> {
        self.buttons
            .iter()
            .find(|button| button.rect.contains(position))
            .map(|button| button.action)
    }

    fn apply_virtual_states(&self, controls: &mut ControlMap) {
        let mut held = self.contacts.values().copied().collect::<HashSet<_>>();
        if let Some(action) = self.mouse_contact {
            held.insert(action);
        }
        for action in self
            .buttons
            .iter()
            .map(|button| button.action)
            .collect::<HashSet<_>>()
        {
            controls.set_virtual(action, held.contains(&action));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ButtonState;

    const LEFT: ActionId = ActionId::new("virtual.left");
    const RIGHT: ActionId = ActionId::new("virtual.right");

    fn pad() -> VirtualPad {
        VirtualPad::new([
            VirtualButton::new(
                LEFT,
                Rect {
                    x: 0,
                    y: 0,
                    width: 20,
                    height: 20,
                },
            ),
            VirtualButton::new(
                RIGHT,
                Rect {
                    x: 20,
                    y: 0,
                    width: 20,
                    height: 20,
                },
            ),
        ])
    }

    fn state(map: &mut ControlMap, action: ActionId) -> ButtonState {
        map.update(&Input::default());
        map.action(action)
    }

    #[test]
    fn mouse_click_hold_drag_and_release_drive_virtual_actions() {
        let mut pad = pad();
        let mut controls = ControlMap::new();
        let mut input = Input::default();
        input.set_mouse_position(Some((5, 5)));
        input.press_mouse_button(MouseButton::Left);
        let update = pad.update(&input, &mut controls);
        assert_eq!(update.pressed_actions(), &[LEFT]);
        assert!(state(&mut controls, LEFT).held());

        input.advance_frame();
        let update = pad.update(&input, &mut controls);
        assert!(update.pressed_actions().is_empty());
        assert!(state(&mut controls, LEFT).held());

        input.set_mouse_position(Some((25, 5)));
        let update = pad.update(&input, &mut controls);
        assert_eq!(update.pressed_actions(), &[RIGHT]);
        controls.update(&input);
        assert!(controls.action(LEFT).released());
        assert!(controls.action(RIGHT).pressed());

        input.advance_frame();
        input.release_mouse_button(MouseButton::Left);
        pad.update(&input, &mut controls);
        controls.update(&input);
        assert!(controls.action(RIGHT).released());
    }

    #[test]
    fn real_touch_does_not_double_trigger_compatibility_mouse() {
        let mut pad = pad();
        let mut controls = ControlMap::new();
        let mut input = Input::default();
        input.set_mouse_position(Some((5, 5)));
        input.press_mouse_button(MouseButton::Left);
        input.push_touch(Touch { id: 7, phase: TouchPhase::Started, position: Some((5, 5)) });
        let update = pad.update(&input, &mut controls);
        assert_eq!(update.pressed_actions(), &[LEFT]);

        input.advance_frame();
        input.push_touch(Touch { id: 7, phase: TouchPhase::Ended, position: None });
        pad.update(&input, &mut controls);
        input.advance_frame();
        let update = pad.update(&input, &mut controls);
        assert!(update.pressed_actions().is_empty());
    }

    #[test]
    fn touch_press_and_release_follow_button_transitions() {
        let mut pad = pad();
        let mut controls = ControlMap::new();

        pad.update_touches(
            &[Touch {
                id: 1,
                phase: TouchPhase::Started,
                position: Some((5, 5)),
            }],
            &mut controls,
        );
        let pressed = state(&mut controls, LEFT);
        assert!(pressed.pressed());
        assert!(pressed.held());

        pad.update_touches(&[], &mut controls);
        let held = state(&mut controls, LEFT);
        assert!(!held.pressed());
        assert!(held.held());

        pad.update_touches(
            &[Touch {
                id: 1,
                phase: TouchPhase::Ended,
                position: None,
            }],
            &mut controls,
        );
        let released = state(&mut controls, LEFT);
        assert!(released.released());
        assert!(!released.held());
    }

    #[test]
    fn moving_contact_between_buttons_retargets_action() {
        let mut pad = pad();
        let mut controls = ControlMap::new();

        pad.update_touches(
            &[Touch {
                id: 7,
                phase: TouchPhase::Started,
                position: Some((5, 5)),
            }],
            &mut controls,
        );
        controls.update(&Input::default());

        pad.update_touches(
            &[Touch {
                id: 7,
                phase: TouchPhase::Moved,
                position: Some((25, 5)),
            }],
            &mut controls,
        );
        controls.update(&Input::default());

        assert!(controls.action(LEFT).released());
        assert!(controls.action(RIGHT).pressed());
        assert!(controls.action(RIGHT).held());
    }

    #[test]
    fn ordered_pressed_actions_preserve_multiple_moves_in_one_frame() {
        let mut pad = pad();
        let mut controls = ControlMap::new();

        let update = pad.update_touches(
            &[
                Touch {
                    id: 7,
                    phase: TouchPhase::Started,
                    position: Some((5, 5)),
                },
                Touch {
                    id: 7,
                    phase: TouchPhase::Moved,
                    position: Some((25, 5)),
                },
                Touch {
                    id: 7,
                    phase: TouchPhase::Moved,
                    position: Some((5, 5)),
                },
            ],
            &mut controls,
        );

        assert_eq!(update.pressed_actions(), &[LEFT, RIGHT, LEFT]);
        controls.update(&Input::default());
        assert!(controls.action(LEFT).pressed());
        assert!(controls.action(LEFT).held());
        assert!(!controls.action(RIGHT).held());
    }

    #[test]
    fn moving_inside_same_button_does_not_repeat_press_event() {
        let mut pad = pad();
        let mut controls = ControlMap::new();

        let update = pad.update_touches(
            &[
                Touch {
                    id: 3,
                    phase: TouchPhase::Started,
                    position: Some((5, 5)),
                },
                Touch {
                    id: 3,
                    phase: TouchPhase::Moved,
                    position: Some((10, 10)),
                },
            ],
            &mut controls,
        );

        assert_eq!(update.pressed_actions(), &[LEFT]);
    }

    #[test]
    fn separate_contacts_can_hold_separate_actions() {
        let mut pad = pad();
        let mut controls = ControlMap::new();

        pad.update_touches(
            &[
                Touch {
                    id: 1,
                    phase: TouchPhase::Started,
                    position: Some((5, 5)),
                },
                Touch {
                    id: 2,
                    phase: TouchPhase::Started,
                    position: Some((25, 5)),
                },
            ],
            &mut controls,
        );
        controls.update(&Input::default());

        assert!(controls.action(LEFT).held());
        assert!(controls.action(RIGHT).held());
    }

    #[test]
    fn first_touch_makes_pad_visible() {
        let mut pad = pad();
        let mut controls = ControlMap::new();
        assert!(!pad.visible());

        pad.update_touches(
            &[Touch {
                id: 1,
                phase: TouchPhase::Started,
                position: Some((100, 100)),
            }],
            &mut controls,
        );

        assert!(pad.visible());
    }

    #[test]
    fn reset_releases_active_virtual_actions() {
        let mut pad = pad();
        let mut controls = ControlMap::new();

        pad.update_touches(
            &[Touch {
                id: 1,
                phase: TouchPhase::Started,
                position: Some((5, 5)),
            }],
            &mut controls,
        );
        controls.update(&Input::default());
        assert!(controls.action(LEFT).held());

        pad.reset(&mut controls);
        controls.update(&Input::default());

        assert!(controls.action(LEFT).released());
        assert!(!controls.action(LEFT).held());
    }
}
