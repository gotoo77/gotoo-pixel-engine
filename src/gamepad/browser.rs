use std::collections::HashSet;

use wasm_bindgen::JsCast;
use web_sys::{Gamepad as WebGamepad, GamepadButton as WebGamepadButton, GamepadMappingType};

use crate::{GamepadButton, GamepadId, GamepadProfile, Input};

const STANDARD_BUTTONS: [(u32, GamepadButton); 12] = [
    (0, GamepadButton::South),
    (1, GamepadButton::East),
    (2, GamepadButton::West),
    (3, GamepadButton::North),
    (4, GamepadButton::LeftShoulder),
    (5, GamepadButton::RightShoulder),
    (8, GamepadButton::Select),
    (9, GamepadButton::Start),
    (12, GamepadButton::DPadUp),
    (13, GamepadButton::DPadDown),
    (14, GamepadButton::DPadLeft),
    (15, GamepadButton::DPadRight),
];

const ALL_BUTTONS: [GamepadButton; 16] = [
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
];

#[derive(Default)]
pub(crate) struct GamepadInputBackend {
    connected: HashSet<GamepadId>,
}

impl GamepadInputBackend {
    pub(crate) fn poll<F>(&mut self, input: &mut Input, mut profile_for: F)
    where
        F: FnMut(GamepadId) -> Option<GamepadProfile>,
    {
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
            input.connect_gamepad(id, gamepad.id());

            if gamepad.mapping() == GamepadMappingType::Standard {
                update_standard_gamepad(input, id, &gamepad, profile_for(id).unwrap_or_default());
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

    let left_x = profile.left_stick_x.normalize(web_axis(gamepad, 0));
    let left_y = profile.left_stick_y.normalize(web_axis(gamepad, 1));
    set_axis_buttons(
        input,
        id,
        GamepadButton::LeftStickLeft,
        GamepadButton::LeftStickRight,
        left_x,
        profile.digital_threshold,
    );
    // Standard Gamepad axis 1 is negative upward and positive downward.
    set_axis_buttons(
        input,
        id,
        GamepadButton::LeftStickUp,
        GamepadButton::LeftStickDown,
        left_y,
        profile.digital_threshold,
    );
}

fn web_button_held(gamepad: &WebGamepad, index: u32, threshold: f32) -> bool {
    let value = gamepad.buttons().get(index);
    let Ok(button) = value.dyn_into::<WebGamepadButton>() else {
        return false;
    };
    button.pressed() || button.value() >= f64::from(threshold.clamp(0.0, 1.0))
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

fn clear_gamepad(input: &mut Input, id: GamepadId) {
    for button in ALL_BUTTONS {
        input.set_gamepad_button(id, button, false);
    }
}
