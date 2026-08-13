use crate::{GamepadButton, GamepadId, Input};

#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashSet;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) struct GamepadInputBackend {
    gilrs: Option<gilrs::Gilrs>,
    centered_button_axes: HashSet<(GamepadId, GamepadButton)>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for GamepadInputBackend {
    fn default() -> Self {
        match gilrs::Gilrs::new() {
            Ok(gilrs) => Self {
                gilrs: Some(gilrs),
                centered_button_axes: HashSet::new(),
            },
            Err(err) => {
                eprintln!("[gpe] gamepad backend unavailable: {err}");
                Self {
                    gilrs: None,
                    centered_button_axes: HashSet::new(),
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl GamepadInputBackend {
    pub(crate) fn poll(&mut self, input: &mut Input) {
        let (gilrs, centered_button_axes) = (&mut self.gilrs, &mut self.centered_button_axes);
        let Some(gilrs) = gilrs.as_mut() else {
            return;
        };

        while let Some(event) = gilrs.next_event() {
            let id = GamepadId::new(usize::from(event.id));
            match event.event {
                gilrs::EventType::ButtonPressed(button, _) => {
                    if let Some(button) = button_from_gilrs(button) {
                        input.set_gamepad_button(id, button, true);
                    }
                }
                gilrs::EventType::ButtonReleased(button, _) => {
                    if let Some(button) = button_from_gilrs(button) {
                        input.set_gamepad_button(id, button, false);
                    }
                }
                gilrs::EventType::ButtonChanged(button, value, _) => {
                    if let Some(button) = button_from_gilrs(button) {
                        update_button_value(input, centered_button_axes, id, button, value);
                    }
                }
                gilrs::EventType::AxisChanged(axis, value, _) => {
                    update_axis(input, id, axis, value);
                }
                gilrs::EventType::Disconnected => {
                    input.disconnect_gamepad(id);
                    centered_button_axes.retain(|(gamepad_id, _)| *gamepad_id != id);
                }
                _ => {}
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn button_from_gilrs(button: gilrs::Button) -> Option<GamepadButton> {
    match button {
        gilrs::Button::South => Some(GamepadButton::South),
        gilrs::Button::East => Some(GamepadButton::East),
        gilrs::Button::North => Some(GamepadButton::North),
        gilrs::Button::West => Some(GamepadButton::West),
        gilrs::Button::Start => Some(GamepadButton::Start),
        gilrs::Button::Select => Some(GamepadButton::Select),
        gilrs::Button::DPadUp => Some(GamepadButton::DPadUp),
        gilrs::Button::DPadDown => Some(GamepadButton::DPadDown),
        gilrs::Button::DPadLeft => Some(GamepadButton::DPadLeft),
        gilrs::Button::DPadRight => Some(GamepadButton::DPadRight),
        _ => None,
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn update_button_value(
    input: &mut Input,
    centered_button_axes: &mut HashSet<(GamepadId, GamepadButton)>,
    id: GamepadId,
    button: GamepadButton,
    value: f32,
) {
    const LOW: f32 = 0.2;
    const HIGH: f32 = 0.8;

    let Some(opposite) = opposite_dpad_button(button) else {
        input.set_gamepad_button(id, button, value >= 0.5);
        return;
    };

    let key = (id, button);
    if value > LOW && value < HIGH {
        centered_button_axes.insert(key);
        input.set_gamepad_button(id, button, false);
        input.set_gamepad_button(id, opposite, false);
        return;
    }

    if centered_button_axes.contains(&key) {
        input.set_gamepad_button(id, button, value >= HIGH);
        input.set_gamepad_button(id, opposite, value <= LOW);
    } else {
        input.set_gamepad_button(id, button, value >= 0.5);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn opposite_dpad_button(button: GamepadButton) -> Option<GamepadButton> {
    match button {
        GamepadButton::DPadUp => Some(GamepadButton::DPadDown),
        GamepadButton::DPadDown => Some(GamepadButton::DPadUp),
        GamepadButton::DPadLeft => Some(GamepadButton::DPadRight),
        GamepadButton::DPadRight => Some(GamepadButton::DPadLeft),
        _ => None,
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn update_axis(input: &mut Input, id: GamepadId, axis: gilrs::Axis, value: f32) {
    const THRESHOLD: f32 = 0.5;

    match axis {
        gilrs::Axis::LeftStickX => {
            input.set_gamepad_button(id, GamepadButton::LeftStickLeft, value <= -THRESHOLD);
            input.set_gamepad_button(id, GamepadButton::LeftStickRight, value >= THRESHOLD);
        }
        gilrs::Axis::LeftStickY => {
            input.set_gamepad_button(id, GamepadButton::LeftStickUp, value >= THRESHOLD);
            input.set_gamepad_button(id, GamepadButton::LeftStickDown, value <= -THRESHOLD);
        }
        gilrs::Axis::DPadX => {
            input.set_gamepad_button(id, GamepadButton::DPadLeft, value <= -THRESHOLD);
            input.set_gamepad_button(id, GamepadButton::DPadRight, value >= THRESHOLD);
        }
        gilrs::Axis::DPadY => {
            input.set_gamepad_button(id, GamepadButton::DPadUp, value >= THRESHOLD);
            input.set_gamepad_button(id, GamepadButton::DPadDown, value <= -THRESHOLD);
        }
        _ => {}
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::collections::HashSet;

    use super::update_button_value;
    use crate::{GamepadButton, GamepadId, Input};

    #[test]
    fn digital_dpad_release_does_not_invent_opposite_direction() {
        let id = GamepadId::new(0);
        let mut input = Input::default();
        let mut centered = HashSet::new();

        update_button_value(
            &mut input,
            &mut centered,
            id,
            GamepadButton::DPadRight,
            1.0,
        );
        assert!(input.gamepad_button(id, GamepadButton::DPadRight).held());

        update_button_value(
            &mut input,
            &mut centered,
            id,
            GamepadButton::DPadRight,
            0.0,
        );
        assert!(!input.gamepad_button(id, GamepadButton::DPadRight).held());
        assert!(!input.gamepad_button(id, GamepadButton::DPadLeft).held());
    }

    #[test]
    fn centered_horizontal_button_axis_synthesizes_both_dpad_directions() {
        let id = GamepadId::new(0);
        let mut input = Input::default();
        let mut centered = HashSet::new();

        update_button_value(
            &mut input,
            &mut centered,
            id,
            GamepadButton::DPadRight,
            0.431,
        );
        update_button_value(
            &mut input,
            &mut centered,
            id,
            GamepadButton::DPadRight,
            0.0,
        );
        assert!(input.gamepad_button(id, GamepadButton::DPadLeft).held());
        assert!(!input.gamepad_button(id, GamepadButton::DPadRight).held());

        update_button_value(
            &mut input,
            &mut centered,
            id,
            GamepadButton::DPadRight,
            0.431,
        );
        assert!(!input.gamepad_button(id, GamepadButton::DPadLeft).held());
        assert!(!input.gamepad_button(id, GamepadButton::DPadRight).held());

        update_button_value(
            &mut input,
            &mut centered,
            id,
            GamepadButton::DPadRight,
            1.0,
        );
        assert!(!input.gamepad_button(id, GamepadButton::DPadLeft).held());
        assert!(input.gamepad_button(id, GamepadButton::DPadRight).held());
    }

    #[test]
    fn centered_vertical_button_axis_synthesizes_both_dpad_directions() {
        let id = GamepadId::new(0);
        let mut input = Input::default();
        let mut centered = HashSet::new();

        update_button_value(
            &mut input,
            &mut centered,
            id,
            GamepadButton::DPadUp,
            0.431,
        );
        update_button_value(
            &mut input,
            &mut centered,
            id,
            GamepadButton::DPadUp,
            0.0,
        );
        assert!(input.gamepad_button(id, GamepadButton::DPadDown).held());
        assert!(!input.gamepad_button(id, GamepadButton::DPadUp).held());

        update_button_value(
            &mut input,
            &mut centered,
            id,
            GamepadButton::DPadUp,
            1.0,
        );
        assert!(!input.gamepad_button(id, GamepadButton::DPadDown).held());
        assert!(input.gamepad_button(id, GamepadButton::DPadUp).held());
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
pub(crate) struct GamepadInputBackend;

#[cfg(target_arch = "wasm32")]
impl GamepadInputBackend {
    pub(crate) fn poll(&mut self, _input: &mut Input) {}
}
