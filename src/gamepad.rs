use crate::{GamepadButton, GamepadId, Input};

#[cfg(not(target_arch = "wasm32"))]
pub(crate) struct GamepadInputBackend {
    gilrs: Option<gilrs::Gilrs>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for GamepadInputBackend {
    fn default() -> Self {
        Self {
            gilrs: gilrs::Gilrs::new().ok(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl GamepadInputBackend {
    pub(crate) fn poll(&mut self, input: &mut Input) {
        let Some(gilrs) = self.gilrs.as_mut() else {
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
                gilrs::EventType::AxisChanged(axis, value, _) => {
                    update_axis(input, id, axis, value);
                }
                gilrs::EventType::Disconnected => input.disconnect_gamepad(id),
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

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
pub(crate) struct GamepadInputBackend;

#[cfg(target_arch = "wasm32")]
impl GamepadInputBackend {
    pub(crate) fn poll(&mut self, _input: &mut Input) {}
}
