use std::collections::{HashMap, HashSet};

use crate::{ActionId, ControlMap, Input, Rect, Touch, TouchPhase};

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

#[derive(Debug, Default, Clone)]
pub struct VirtualPad {
    buttons: Vec<VirtualButton>,
    contacts: HashMap<u64, ActionId>,
    visible: bool,
}

impl VirtualPad {
    pub fn new(buttons: impl IntoIterator<Item = VirtualButton>) -> Self {
        Self {
            buttons: buttons.into_iter().collect(),
            contacts: HashMap::new(),
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
    pub fn update(&mut self, input: &Input, controls: &mut ControlMap) {
        self.update_touches(input.touches(), controls);
    }

    pub fn reset(&mut self, controls: &mut ControlMap) {
        self.contacts.clear();
        self.apply_virtual_states(controls);
    }

    fn update_touches(&mut self, touches: &[Touch], controls: &mut ControlMap) {
        if !touches.is_empty() {
            self.visible = true;
        }

        for touch in touches {
            match touch.phase {
                TouchPhase::Started | TouchPhase::Moved => self.update_contact(*touch),
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    self.contacts.remove(&touch.id);
                }
            }
        }

        self.apply_virtual_states(controls);
    }

    fn update_contact(&mut self, touch: Touch) {
        let action = touch.position.and_then(|position| self.action_at(position));

        if let Some(action) = action {
            self.contacts.insert(touch.id, action);
        } else {
            self.contacts.remove(&touch.id);
        }
    }

    fn action_at(&self, position: (i32, i32)) -> Option<ActionId> {
        self.buttons
            .iter()
            .find(|button| button.rect.contains(position))
            .map(|button| button.action)
    }

    fn apply_virtual_states(&self, controls: &mut ControlMap) {
        let held = self.contacts.values().copied().collect::<HashSet<_>>();
        for action in self.buttons.iter().map(|button| button.action).collect::<HashSet<_>>() {
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
}
