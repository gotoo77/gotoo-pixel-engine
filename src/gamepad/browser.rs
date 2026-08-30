use std::collections::HashSet;

use wasm_bindgen::JsCast;
use web_sys::{Gamepad as WebGamepad, GamepadButton as WebGamepadButton, GamepadMappingType};

use super::{
    AxisCalibration, GamepadAxis, GamepadButton, GamepadCapabilities, GamepadDeviceInfo, GamepadId,
    GamepadMappingSource, GamepadProfile, Input,
};

const STANDARD_BUTTONS: [(u32, GamepadButton); 17] = [
    (0, GamepadButton::South),
    (1, GamepadButton::East),
    (2, GamepadButton::West),
    (3, GamepadButton::North),
    (4, GamepadButton::LeftShoulder),
    (5, GamepadButton::RightShoulder),
    (6, GamepadButton::LeftTrigger),
    (7, GamepadButton::RightTrigger),
    (8, GamepadButton::Select),
    (9, GamepadButton::Start),
    (10, GamepadButton::LeftStickPress),
    (11, GamepadButton::RightStickPress),
    (12, GamepadButton::DPadUp),
    (13, GamepadButton::DPadDown),
    (14, GamepadButton::DPadLeft),
    (15, GamepadButton::DPadRight),
    (16, GamepadButton::Guide),
];

const ALL_BUTTONS: [GamepadButton; 25] = [
    GamepadButton::South,
    GamepadButton::East,
    GamepadButton::North,
    GamepadButton::West,
    GamepadButton::LeftShoulder,
    GamepadButton::RightShoulder,
    GamepadButton::Start,
    GamepadButton::Select,
    GamepadButton::DPadUp,
    GamepadButton::DPadDown,
    GamepadButton::DPadLeft,
    GamepadButton::DPadRight,
    GamepadButton::LeftStickUp,
    GamepadButton::LeftStickDown,
    GamepadButton::LeftStickLeft,
    GamepadButton::LeftStickRight,
    GamepadButton::LeftTrigger,
    GamepadButton::RightTrigger,
    GamepadButton::LeftStickPress,
    GamepadButton::RightStickPress,
    GamepadButton::Guide,
    GamepadButton::RightStickUp,
    GamepadButton::RightStickDown,
    GamepadButton::RightStickLeft,
    GamepadButton::RightStickRight,
];

#[derive(Default)]
pub(crate) struct GamepadInputBackend {
    connected: HashSet<GamepadId>,
}

impl GamepadInputBackend {
    pub(crate) fn poll(&mut self, input: &mut Input) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Ok(gamepads) = window.navigator().get_gamepads() else {
            return;
        };

        let mut seen = HashSet::new();
        for value in gamepads.iter() {
            if value.is_null_or_undefined() {
                continue;
            }
            let Ok(gamepad) = value.dyn_into::<WebGamepad>() else {
                continue;
            };
            if !gamepad.connected() {
                continue;
            }

            let id = GamepadId::new(gamepad.index() as usize);
            seen.insert(id);
            let standard = gamepad.mapping() == GamepadMappingType::Standard;
            input.connect_gamepad_info(GamepadDeviceInfo {
                id,
                name: gamepad.id(),
                os_name: None,
                mapping_name: standard.then(|| "standard".to_owned()),
                mapping_source: if standard {
                    GamepadMappingSource::BrowserStandard
                } else {
                    GamepadMappingSource::Unknown
                },
                guid: None,
                vendor_id: None,
                product_id: None,
                capabilities: if standard {
                    GamepadCapabilities::standard()
                } else {
                    GamepadCapabilities::unknown()
                },
            });

            if standard {
                let profile = input.gamepad_profile(id);
                update_standard_gamepad(input, id, &gamepad, profile);
            } else {
                clear_gamepad(input, id);
            }
        }

        let disconnected = self
            .connected
            .difference(&seen)
            .copied()
            .collect::<Vec<_>>();
        for id in disconnected {
            input.disconnect_gamepad(id);
            input.remove_gamepad_profile(id);
        }
        self.connected = seen;
    }
}

fn update_standard_gamepad(
    input: &mut Input,
    id: GamepadId,
    gamepad: &WebGamepad,
    profile: GamepadProfile,
) {
    for (index, button) in STANDARD_BUTTONS {
        input.set_gamepad_button(
            id,
            button,
            web_button_held(gamepad, index, profile.digital_threshold),
        );
    }

    let left_x = normalized_axis(gamepad, 0, profile.left_stick_x);
    // Browser stick Y is negative upward; canonical GPE stick Y is positive upward.
    let left_y = -normalized_axis(gamepad, 1, profile.left_stick_y);
    let right_x = normalized_axis(gamepad, 2, profile.right_stick_x);
    let right_y = -normalized_axis(gamepad, 3, profile.right_stick_y);
    set_stick_axis(
        input,
        id,
        GamepadAxis::LeftStickX,
        GamepadButton::LeftStickLeft,
        GamepadButton::LeftStickRight,
        left_x,
        profile.digital_threshold,
    );
    set_stick_axis(
        input,
        id,
        GamepadAxis::LeftStickY,
        GamepadButton::LeftStickDown,
        GamepadButton::LeftStickUp,
        left_y,
        profile.digital_threshold,
    );
    set_stick_axis(
        input,
        id,
        GamepadAxis::RightStickX,
        GamepadButton::RightStickLeft,
        GamepadButton::RightStickRight,
        right_x,
        profile.digital_threshold,
    );
    set_stick_axis(
        input,
        id,
        GamepadAxis::RightStickY,
        GamepadButton::RightStickDown,
        GamepadButton::RightStickUp,
        right_y,
        profile.digital_threshold,
    );

    let left_trigger = profile.left_trigger.normalize(web_button_value(gamepad, 6));
    let right_trigger = profile
        .right_trigger
        .normalize(web_button_value(gamepad, 7));
    input.set_gamepad_axis(id, GamepadAxis::LeftTrigger, left_trigger);
    input.set_gamepad_axis(id, GamepadAxis::RightTrigger, right_trigger);
    input.set_gamepad_button(
        id,
        GamepadButton::LeftTrigger,
        left_trigger >= profile.digital_threshold,
    );
    input.set_gamepad_button(
        id,
        GamepadButton::RightTrigger,
        right_trigger >= profile.digital_threshold,
    );
}

fn web_button_held(gamepad: &WebGamepad, index: u32, threshold: f32) -> bool {
    let value = gamepad.buttons().get(index);
    let Ok(button) = value.dyn_into::<WebGamepadButton>() else {
        return false;
    };
    button.pressed() || button.value() >= f64::from(threshold.clamp(0.0, 1.0))
}

fn web_button_value(gamepad: &WebGamepad, index: u32) -> f32 {
    let value = gamepad.buttons().get(index);
    let Ok(button) = value.dyn_into::<WebGamepadButton>() else {
        return 0.0;
    };
    button.value() as f32
}

fn normalized_axis(gamepad: &WebGamepad, index: u32, calibration: AxisCalibration) -> f32 {
    calibration.normalize(web_axis(gamepad, index))
}

fn web_axis(gamepad: &WebGamepad, index: u32) -> f32 {
    gamepad.axes().get(index).as_f64().unwrap_or(0.0) as f32
}

fn set_axis_buttons(
    input: &mut Input,
    id: GamepadId,
    negative: GamepadButton,
    positive: GamepadButton,
    value: f32,
    threshold: f32,
) {
    input.set_gamepad_button(id, negative, value <= -threshold);
    input.set_gamepad_button(id, positive, value >= threshold);
}

fn set_stick_axis(
    input: &mut Input,
    id: GamepadId,
    axis: GamepadAxis,
    negative: GamepadButton,
    positive: GamepadButton,
    value: f32,
    threshold: f32,
) {
    input.set_gamepad_axis(id, axis, value);
    set_axis_buttons(input, id, negative, positive, value, threshold);
}

fn clear_gamepad(input: &mut Input, id: GamepadId) {
    for button in ALL_BUTTONS {
        input.set_gamepad_button(id, button, false);
    }
    for axis in [
        GamepadAxis::LeftStickX,
        GamepadAxis::LeftStickY,
        GamepadAxis::RightStickX,
        GamepadAxis::RightStickY,
        GamepadAxis::LeftTrigger,
        GamepadAxis::RightTrigger,
    ] {
        input.set_gamepad_axis(id, axis, 0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::STANDARD_BUTTONS;
    use crate::GamepadButton;

    #[test]
    fn standard_browser_button_indices_match_the_w3c_layout() {
        assert_eq!(
            STANDARD_BUTTONS,
            [
                (0, GamepadButton::South),
                (1, GamepadButton::East),
                (2, GamepadButton::West),
                (3, GamepadButton::North),
                (4, GamepadButton::LeftShoulder),
                (5, GamepadButton::RightShoulder),
                (6, GamepadButton::LeftTrigger),
                (7, GamepadButton::RightTrigger),
                (8, GamepadButton::Select),
                (9, GamepadButton::Start),
                (10, GamepadButton::LeftStickPress),
                (11, GamepadButton::RightStickPress),
                (12, GamepadButton::DPadUp),
                (13, GamepadButton::DPadDown),
                (14, GamepadButton::DPadLeft),
                (15, GamepadButton::DPadRight),
                (16, GamepadButton::Guide),
            ]
        );
    }
}
