use crate::{AxisCalibration, GamepadButton, GamepadId, GamepadProfile, Input};

#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DPadAxis {
    Horizontal,
    Vertical,
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) struct GamepadInputBackend {
    gilrs: Option<gilrs::Gilrs>,
    profiles: HashMap<GamepadId, GamepadProfile>,
    centered_dpad_axes: HashMap<(GamepadId, DPadAxis), f32>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for GamepadInputBackend {
    fn default() -> Self {
        match gilrs::Gilrs::new() {
            Ok(gilrs) => Self {
                gilrs: Some(gilrs),
                profiles: HashMap::new(),
                centered_dpad_axes: HashMap::new(),
            },
            Err(err) => {
                eprintln!("[gpe] gamepad backend unavailable: {err}");
                Self {
                    gilrs: None,
                    profiles: HashMap::new(),
                    centered_dpad_axes: HashMap::new(),
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl GamepadInputBackend {
    pub(crate) fn poll(&mut self, input: &mut Input) {
        let Some(gilrs) = self.gilrs.as_mut() else {
            return;
        };

        let connected = gilrs
            .gamepads()
            .map(|(id, gamepad)| (GamepadId::new(usize::from(id)), gamepad.name().to_owned()))
            .collect::<Vec<_>>();
        for (id, name) in connected {
            input.connect_gamepad(id, name);
            self.profiles.entry(id).or_default();
        }

        while let Some(event) = gilrs.next_event() {
            let id = GamepadId::new(usize::from(event.id));
            match event.event {
                gilrs::EventType::Connected => {
                    let name = gilrs.gamepad(event.id).name().to_owned();
                    input.connect_gamepad(id, name);
                    self.profiles.entry(id).or_default();
                }
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
                        let profile = self.profiles.get(&id).copied().unwrap_or_default();
                        update_button_value(
                            input,
                            &mut self.centered_dpad_axes,
                            id,
                            button,
                            value,
                            profile,
                        );
                    }
                }
                gilrs::EventType::AxisChanged(axis, value, _) => {
                    let profile = self.profiles.get(&id).copied().unwrap_or_default();
                    update_axis(input, id, axis, value, profile);
                }
                gilrs::EventType::Disconnected => {
                    input.disconnect_gamepad(id);
                    self.profiles.remove(&id);
                    self.centered_dpad_axes
                        .retain(|(gamepad_id, _), _| *gamepad_id != id);
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
        gilrs::Button::LeftTrigger => Some(GamepadButton::LeftShoulder),
        gilrs::Button::RightTrigger => Some(GamepadButton::RightShoulder),
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
    centered_dpad_axes: &mut HashMap<(GamepadId, DPadAxis), f32>,
    id: GamepadId,
    button: GamepadButton,
    value: f32,
    profile: GamepadProfile,
) {
    const CENTER_LOW: f32 = 0.2;
    const CENTER_HIGH: f32 = 0.8;

    let Some(axis) = dpad_axis(button) else {
        input.set_gamepad_button(id, button, value >= profile.digital_threshold);
        return;
    };

    let key = (id, axis);
    if value > CENTER_LOW && value < CENTER_HIGH {
        centered_dpad_axes.insert(key, value);
        set_dpad_axis(input, id, axis, 0.0, profile.digital_threshold);
        return;
    }

    let Some(center) = centered_dpad_axes.get(&key).copied() else {
        input.set_gamepad_button(id, button, value >= profile.digital_threshold);
        return;
    };

    let dead_zone = match axis {
        DPadAxis::Horizontal => profile.dpad_x.dead_zone(),
        DPadAxis::Vertical => profile.dpad_y.dead_zone(),
    };
    let calibration = centered_dpad_calibration(axis, center, dead_zone);
    set_dpad_axis(
        input,
        id,
        axis,
        calibration.normalize(value),
        profile.digital_threshold,
    );
}

#[cfg(not(target_arch = "wasm32"))]
fn dpad_axis(button: GamepadButton) -> Option<DPadAxis> {
    match button {
        GamepadButton::DPadLeft | GamepadButton::DPadRight => Some(DPadAxis::Horizontal),
        GamepadButton::DPadUp | GamepadButton::DPadDown => Some(DPadAxis::Vertical),
        _ => None,
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn centered_dpad_calibration(axis: DPadAxis, center: f32, dead_zone: f32) -> AxisCalibration {
    match axis {
        DPadAxis::Horizontal => AxisCalibration::new(0.0, center, 1.0, dead_zone),
        // Linux hats conventionally use the low endpoint for physical up and the high
        // endpoint for physical down. Canonical GPE Y keeps positive as up.
        DPadAxis::Vertical => AxisCalibration::new(1.0, center, 0.0, dead_zone),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn set_dpad_axis(input: &mut Input, id: GamepadId, axis: DPadAxis, value: f32, threshold: f32) {
    match axis {
        DPadAxis::Horizontal => set_axis_buttons(
            input,
            id,
            GamepadButton::DPadLeft,
            GamepadButton::DPadRight,
            value,
            threshold,
        ),
        DPadAxis::Vertical => set_axis_buttons(
            input,
            id,
            GamepadButton::DPadDown,
            GamepadButton::DPadUp,
            value,
            threshold,
        ),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn update_axis(
    input: &mut Input,
    id: GamepadId,
    axis: gilrs::Axis,
    value: f32,
    profile: GamepadProfile,
) {
    match axis {
        gilrs::Axis::LeftStickX => set_axis_buttons(
            input,
            id,
            GamepadButton::LeftStickLeft,
            GamepadButton::LeftStickRight,
            profile.left_stick_x.normalize(value),
            profile.digital_threshold,
        ),
        gilrs::Axis::LeftStickY => set_axis_buttons(
            input,
            id,
            GamepadButton::LeftStickDown,
            GamepadButton::LeftStickUp,
            profile.left_stick_y.normalize(value),
            profile.digital_threshold,
        ),
        gilrs::Axis::DPadX => set_axis_buttons(
            input,
            id,
            GamepadButton::DPadLeft,
            GamepadButton::DPadRight,
            profile.dpad_x.normalize(value),
            profile.digital_threshold,
        ),
        gilrs::Axis::DPadY => set_axis_buttons(
            input,
            id,
            GamepadButton::DPadDown,
            GamepadButton::DPadUp,
            profile.dpad_y.normalize(value),
            profile.digital_threshold,
        ),
        _ => {}
    }
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::collections::HashMap;

    use super::{
        DPadAxis, button_from_gilrs, centered_dpad_calibration, set_axis_buttons,
        update_button_value,
    };
    use crate::{AxisCalibration, GamepadButton, GamepadId, GamepadProfile, Input};

    #[test]
    fn trigger_buttons_map_to_shoulders() {
        assert_eq!(
            button_from_gilrs(gilrs::Button::LeftTrigger),
            Some(GamepadButton::LeftShoulder)
        );
        assert_eq!(
            button_from_gilrs(gilrs::Button::RightTrigger),
            Some(GamepadButton::RightShoulder)
        );
    }

    #[test]
    fn digital_dpad_release_does_not_invent_opposite_direction() {
        let id = GamepadId::new(0);
        let mut input = Input::default();
        let mut centered = HashMap::new();
        let profile = GamepadProfile::default();

        update_button_value(
            &mut input,
            &mut centered,
            id,
            GamepadButton::DPadRight,
            1.0,
            profile,
        );
        assert!(input.gamepad_button(id, GamepadButton::DPadRight).held());

        update_button_value(
            &mut input,
            &mut centered,
            id,
            GamepadButton::DPadRight,
            0.0,
            profile,
        );
        assert!(!input.gamepad_button(id, GamepadButton::DPadRight).held());
        assert!(!input.gamepad_button(id, GamepadButton::DPadLeft).held());
    }

    #[test]
    fn centered_horizontal_button_axis_synthesizes_both_dpad_directions() {
        let id = GamepadId::new(0);
        let mut input = Input::default();
        let mut centered = HashMap::new();
        let profile = GamepadProfile::default();

        update_button_value(
            &mut input,
            &mut centered,
            id,
            GamepadButton::DPadRight,
            0.431,
            profile,
        );
        update_button_value(
            &mut input,
            &mut centered,
            id,
            GamepadButton::DPadRight,
            0.0,
            profile,
        );
        assert!(input.gamepad_button(id, GamepadButton::DPadLeft).held());
        assert!(!input.gamepad_button(id, GamepadButton::DPadRight).held());

        update_button_value(
            &mut input,
            &mut centered,
            id,
            GamepadButton::DPadRight,
            0.431,
            profile,
        );
        assert!(!input.gamepad_button(id, GamepadButton::DPadLeft).held());
        assert!(!input.gamepad_button(id, GamepadButton::DPadRight).held());

        update_button_value(
            &mut input,
            &mut centered,
            id,
            GamepadButton::DPadRight,
            1.0,
            profile,
        );
        assert!(!input.gamepad_button(id, GamepadButton::DPadLeft).held());
        assert!(input.gamepad_button(id, GamepadButton::DPadRight).held());
    }

    #[test]
    fn centered_vertical_button_axis_uses_linux_hat_polarity() {
        let id = GamepadId::new(0);
        let mut input = Input::default();
        let mut centered = HashMap::new();
        let profile = GamepadProfile::default();

        update_button_value(
            &mut input,
            &mut centered,
            id,
            GamepadButton::DPadUp,
            0.431,
            profile,
        );
        update_button_value(
            &mut input,
            &mut centered,
            id,
            GamepadButton::DPadUp,
            0.0,
            profile,
        );
        assert!(input.gamepad_button(id, GamepadButton::DPadUp).held());
        assert!(!input.gamepad_button(id, GamepadButton::DPadDown).held());

        update_button_value(
            &mut input,
            &mut centered,
            id,
            GamepadButton::DPadUp,
            1.0,
            profile,
        );
        assert!(!input.gamepad_button(id, GamepadButton::DPadUp).held());
        assert!(input.gamepad_button(id, GamepadButton::DPadDown).held());
    }

    #[test]
    fn centered_axis_calibration_keeps_observed_center_neutral() {
        let horizontal = centered_dpad_calibration(DPadAxis::Horizontal, 0.431, 0.10);
        let vertical = centered_dpad_calibration(DPadAxis::Vertical, 0.431, 0.10);

        assert_eq!(horizontal.normalize(0.431), 0.0);
        assert_eq!(vertical.normalize(0.431), 0.0);
        assert_eq!(horizontal.normalize(0.0), -1.0);
        assert_eq!(horizontal.normalize(1.0), 1.0);
        assert_eq!(vertical.normalize(0.0), 1.0);
        assert_eq!(vertical.normalize(1.0), -1.0);
    }

    #[test]
    fn axis_profile_can_invert_a_stick_without_changing_game_actions() {
        let id = GamepadId::new(1);
        let mut input = Input::default();
        let calibration = AxisCalibration::standard(0.0).inverted();

        set_axis_buttons(
            &mut input,
            id,
            GamepadButton::LeftStickDown,
            GamepadButton::LeftStickUp,
            calibration.normalize(1.0),
            0.5,
        );

        assert!(
            input
                .gamepad_button(id, GamepadButton::LeftStickDown)
                .held()
        );
        assert!(!input.gamepad_button(id, GamepadButton::LeftStickUp).held());
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
pub(crate) struct GamepadInputBackend;

#[cfg(target_arch = "wasm32")]
impl GamepadInputBackend {
    pub(crate) fn poll(&mut self, _input: &mut Input) {}
}
