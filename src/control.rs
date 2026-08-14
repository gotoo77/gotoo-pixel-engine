use std::collections::{HashMap, HashSet};

use crate::{ButtonState, GamepadButton, GamepadId, Input, Key};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActionId(&'static str);

impl ActionId {
    pub const fn new(id: &'static str) -> Self {
        Self(id)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlBinding {
    Key(Key),
    Gamepad(GamepadButton),
    GamepadDevice(GamepadId, GamepadButton),
}

#[derive(Debug, Default, Clone)]
pub struct ControlMap {
    bindings: HashMap<ActionId, Vec<ControlBinding>>,
    states: HashMap<ActionId, ButtonState>,
    virtual_held: HashMap<ActionId, bool>,
}

impl ControlMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind(&mut self, action: ActionId, binding: ControlBinding) -> &mut Self {
        let bindings = self.bindings.entry(action).or_default();
        if !bindings.contains(&binding) {
            bindings.push(binding);
        }
        self
    }

    pub fn bind_key(&mut self, action: ActionId, key: Key) -> &mut Self {
        self.bind(action, ControlBinding::Key(key))
    }

    pub fn bind_gamepad(&mut self, action: ActionId, button: GamepadButton) -> &mut Self {
        self.bind(action, ControlBinding::Gamepad(button))
    }

    pub fn bind_gamepad_device(
        &mut self,
        action: ActionId,
        gamepad_id: GamepadId,
        button: GamepadButton,
    ) -> &mut Self {
        self.bind(action, ControlBinding::GamepadDevice(gamepad_id, button))
    }

    pub fn clear_bindings(&mut self, action: ActionId) -> &mut Self {
        self.bindings.remove(&action);
        self
    }

    pub fn set_virtual(&mut self, action: ActionId, held: bool) {
        self.virtual_held.insert(action, held);
    }

    pub fn clear_virtual(&mut self) {
        for held in self.virtual_held.values_mut() {
            *held = false;
        }
    }

    pub fn update(&mut self, input: &Input) {
        let actions = self
            .bindings
            .keys()
            .chain(self.states.keys())
            .chain(self.virtual_held.keys())
            .copied()
            .collect::<HashSet<_>>();

        for action in actions {
            let was_held = self.states.get(&action).is_some_and(|state| state.held());
            let virtual_held = self.virtual_held.get(&action).copied().unwrap_or(false);
            let physical_held = self
                .bindings
                .get(&action)
                .into_iter()
                .flatten()
                .any(|binding| match binding {
                    ControlBinding::Key(key) => input.key(*key).held(),
                    ControlBinding::Gamepad(button) => input.gamepad_button_any(*button).held(),
                    ControlBinding::GamepadDevice(gamepad_id, button) => {
                        input.gamepad_button(*gamepad_id, *button).held()
                    }
                });

            self.states.insert(
                action,
                ButtonState::from_transition(was_held, virtual_held || physical_held),
            );
        }
    }

    pub fn action(&self, action: ActionId) -> ButtonState {
        self.states.get(&action).copied().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOVE_LEFT: ActionId = ActionId::new("test.left");
    const FIRE: ActionId = ActionId::new("test.fire");

    #[test]
    fn keyboard_and_gamepad_feed_the_same_action() {
        let mut map = ControlMap::new();
        map.bind_key(MOVE_LEFT, Key::Left)
            .bind_gamepad(MOVE_LEFT, GamepadButton::DPadLeft);
        let mut input = Input::default();

        input.press_key(Key::Left);
        map.update(&input);
        assert!(map.action(MOVE_LEFT).pressed());
        assert!(map.action(MOVE_LEFT).held());

        input.release_key(Key::Left);
        input.set_gamepad_button(GamepadId::new(0), GamepadButton::DPadLeft, true);
        map.update(&input);
        assert!(map.action(MOVE_LEFT).held());
        assert!(!map.action(MOVE_LEFT).released());
    }

    #[test]
    fn gamepad_device_binding_ignores_other_gamepads() {
        let assigned = GamepadId::new(3);
        let other = GamepadId::new(7);
        let mut map = ControlMap::new();
        map.bind_gamepad_device(MOVE_LEFT, assigned, GamepadButton::DPadLeft);
        let mut input = Input::default();

        input.set_gamepad_button(other, GamepadButton::DPadLeft, true);
        map.update(&input);
        assert!(!map.action(MOVE_LEFT).held());

        input.set_gamepad_button(assigned, GamepadButton::DPadLeft, true);
        map.update(&input);
        assert!(map.action(MOVE_LEFT).pressed());
        assert!(map.action(MOVE_LEFT).held());
    }

    #[test]
    fn virtual_source_uses_pressed_held_released_transitions() {
        let mut map = ControlMap::new();
        map.set_virtual(FIRE, true);
        map.update(&Input::default());
        assert!(map.action(FIRE).pressed());
        assert!(map.action(FIRE).held());

        map.update(&Input::default());
        assert!(!map.action(FIRE).pressed());
        assert!(map.action(FIRE).held());

        map.set_virtual(FIRE, false);
        map.update(&Input::default());
        assert!(map.action(FIRE).released());
        assert!(!map.action(FIRE).held());
    }
}
