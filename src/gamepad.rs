use crate::{
    AxisCalibration, GamepadAxis, GamepadButton, GamepadCapabilities, GamepadDeviceInfo, GamepadId,
    GamepadMappingSource, GamepadProfile, Input,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::GamepadCapability;

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
    centered_dpad_axes: HashMap<(GamepadId, DPadAxis), f32>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for GamepadInputBackend {
    fn default() -> Self {
        match gilrs::Gilrs::new() {
            Ok(gilrs) => Self {
                gilrs: Some(gilrs),
                centered_dpad_axes: HashMap::new(),
            },
            Err(err) => {
                eprintln!("[gpe] gamepad backend unavailable: {err}");
                Self {
                    gilrs: None,
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
            .map(|(id, gamepad)| device_info(GamepadId::new(usize::from(id)), gamepad))
            .collect::<Vec<_>>();
        for info in connected {
            input.connect_gamepad_info(info);
        }

        while let Some(event) = gilrs.next_event() {
            let id = GamepadId::new(usize::from(event.id));
            match event.event {
                gilrs::EventType::Connected => {
                    input.connect_gamepad_info(device_info(id, gilrs.gamepad(event.id)));
                }
                gilrs::EventType::ButtonPressed(button, _) => {
                    if let Some(button) = button_from_gilrs(button) {
                        update_button_edge(input, &self.centered_dpad_axes, id, button, true);
                    }
                }
                gilrs::EventType::ButtonReleased(button, _) => {
                    if let Some(button) = button_from_gilrs(button) {
                        update_button_edge(input, &self.centered_dpad_axes, id, button, false);
                    }
                }
                gilrs::EventType::ButtonChanged(button, value, _) => {
                    if let Some(button) = button_from_gilrs(button) {
                        let profile = input.gamepad_profile(id);
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
                    let profile = input.gamepad_profile(id);
                    update_axis(input, id, axis, value, profile);
                }
                gilrs::EventType::Disconnected => {
                    input.disconnect_gamepad(id);
                    input.remove_gamepad_profile(id);
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
        gilrs::Button::LeftTrigger2 => Some(GamepadButton::LeftTrigger),
        gilrs::Button::RightTrigger2 => Some(GamepadButton::RightTrigger),
        gilrs::Button::Start => Some(GamepadButton::Start),
        gilrs::Button::Select => Some(GamepadButton::Select),
        gilrs::Button::Mode => Some(GamepadButton::Guide),
        gilrs::Button::LeftThumb => Some(GamepadButton::LeftStickPress),
        gilrs::Button::RightThumb => Some(GamepadButton::RightStickPress),
        gilrs::Button::DPadUp => Some(GamepadButton::DPadUp),
        gilrs::Button::DPadDown => Some(GamepadButton::DPadDown),
        gilrs::Button::DPadLeft => Some(GamepadButton::DPadLeft),
        gilrs::Button::DPadRight => Some(GamepadButton::DPadRight),
        _ => None,
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn device_info(id: GamepadId, gamepad: gilrs::Gamepad<'_>) -> GamepadDeviceInfo {
    let mapping_source = match gamepad.mapping_source() {
        gilrs::MappingSource::SdlMappings => GamepadMappingSource::SdlMappings,
        gilrs::MappingSource::Driver => GamepadMappingSource::Driver,
        gilrs::MappingSource::None => GamepadMappingSource::Unknown,
    };
    let capabilities = if gamepad.mapping_source() == gilrs::MappingSource::None {
        GamepadCapabilities::unknown()
    } else {
        capabilities_from_gilrs(gamepad)
    };
    let guid = gamepad
        .uuid()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    GamepadDeviceInfo {
        id,
        name: gamepad.name().to_owned(),
        os_name: Some(gamepad.os_name().to_owned()),
        mapping_name: gamepad.map_name().map(str::to_owned),
        mapping_source,
        guid: Some(guid),
        vendor_id: gamepad.vendor_id(),
        product_id: gamepad.product_id(),
        capabilities,
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn capabilities_from_gilrs(gamepad: gilrs::Gamepad<'_>) -> GamepadCapabilities {
    let mut capabilities = GamepadCapabilities::unknown();
    for (source, target) in GILRS_BUTTONS {
        capabilities.set_button(target, availability(gamepad.button_code(source).is_some()));
    }
    for (source, target) in GILRS_AXES {
        capabilities.set_axis(target, availability(gamepad.axis_code(source).is_some()));
    }
    capabilities.set_axis(
        GamepadAxis::LeftTrigger,
        capabilities.button(GamepadButton::LeftTrigger),
    );
    capabilities.set_axis(
        GamepadAxis::RightTrigger,
        capabilities.button(GamepadButton::RightTrigger),
    );
    for (axis, negative, positive) in [
        (
            GamepadAxis::LeftStickX,
            GamepadButton::LeftStickLeft,
            GamepadButton::LeftStickRight,
        ),
        (
            GamepadAxis::LeftStickY,
            GamepadButton::LeftStickDown,
            GamepadButton::LeftStickUp,
        ),
        (
            GamepadAxis::RightStickX,
            GamepadButton::RightStickLeft,
            GamepadButton::RightStickRight,
        ),
        (
            GamepadAxis::RightStickY,
            GamepadButton::RightStickDown,
            GamepadButton::RightStickUp,
        ),
    ] {
        let capability = capabilities.axis(axis);
        capabilities.set_button(negative, capability);
        capabilities.set_button(positive, capability);
    }
    capabilities
}

#[cfg(not(target_arch = "wasm32"))]
fn availability(available: bool) -> GamepadCapability {
    if available {
        GamepadCapability::Available
    } else {
        GamepadCapability::Unavailable
    }
}

#[cfg(not(target_arch = "wasm32"))]
const GILRS_BUTTONS: [(gilrs::Button, GamepadButton); 17] = [
    (gilrs::Button::South, GamepadButton::South),
    (gilrs::Button::East, GamepadButton::East),
    (gilrs::Button::North, GamepadButton::North),
    (gilrs::Button::West, GamepadButton::West),
    (gilrs::Button::LeftTrigger, GamepadButton::LeftShoulder),
    (gilrs::Button::RightTrigger, GamepadButton::RightShoulder),
    (gilrs::Button::LeftTrigger2, GamepadButton::LeftTrigger),
    (gilrs::Button::RightTrigger2, GamepadButton::RightTrigger),
    (gilrs::Button::Start, GamepadButton::Start),
    (gilrs::Button::Select, GamepadButton::Select),
    (gilrs::Button::Mode, GamepadButton::Guide),
    (gilrs::Button::LeftThumb, GamepadButton::LeftStickPress),
    (gilrs::Button::RightThumb, GamepadButton::RightStickPress),
    (gilrs::Button::DPadUp, GamepadButton::DPadUp),
    (gilrs::Button::DPadDown, GamepadButton::DPadDown),
    (gilrs::Button::DPadLeft, GamepadButton::DPadLeft),
    (gilrs::Button::DPadRight, GamepadButton::DPadRight),
];

#[cfg(not(target_arch = "wasm32"))]
const GILRS_AXES: [(gilrs::Axis, GamepadAxis); 4] = [
    (gilrs::Axis::LeftStickX, GamepadAxis::LeftStickX),
    (gilrs::Axis::LeftStickY, GamepadAxis::LeftStickY),
    (gilrs::Axis::RightStickX, GamepadAxis::RightStickX),
    (gilrs::Axis::RightStickY, GamepadAxis::RightStickY),
];

#[cfg(not(target_arch = "wasm32"))]
fn update_button_edge(
    input: &mut Input,
    centered_dpad_axes: &HashMap<(GamepadId, DPadAxis), f32>,
    id: GamepadId,
    button: GamepadButton,
    held: bool,
) {
    if let Some(axis) = dpad_axis(button)
        && centered_dpad_axes.contains_key(&(id, axis))
    {
        return;
    }

    input.set_gamepad_button(id, button, held);
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
        if matches!(
            button,
            GamepadButton::LeftTrigger | GamepadButton::RightTrigger
        ) {
            let (axis, value) = match button {
                GamepadButton::LeftTrigger => (
                    GamepadAxis::LeftTrigger,
                    profile.left_trigger.normalize(value),
                ),
                GamepadButton::RightTrigger => (
                    GamepadAxis::RightTrigger,
                    profile.right_trigger.normalize(value),
                ),
                _ => unreachable!(),
            };
            input.set_gamepad_axis(id, axis, value);
            input.set_gamepad_button(id, button, value >= profile.digital_threshold);
            return;
        }
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
        gilrs::Axis::LeftStickX => set_stick_axis(
            input,
            id,
            GamepadAxis::LeftStickX,
            GamepadButton::LeftStickLeft,
            GamepadButton::LeftStickRight,
            profile.left_stick_x.normalize(value),
            profile.digital_threshold,
        ),
        gilrs::Axis::LeftStickY => set_stick_axis(
            input,
            id,
            GamepadAxis::LeftStickY,
            GamepadButton::LeftStickDown,
            GamepadButton::LeftStickUp,
            profile.left_stick_y.normalize(value),
            profile.digital_threshold,
        ),
        gilrs::Axis::RightStickX => set_stick_axis(
            input,
            id,
            GamepadAxis::RightStickX,
            GamepadButton::RightStickLeft,
            GamepadButton::RightStickRight,
            profile.right_stick_x.normalize(value),
            profile.digital_threshold,
        ),
        gilrs::Axis::RightStickY => set_stick_axis(
            input,
            id,
            GamepadAxis::RightStickY,
            GamepadButton::RightStickDown,
            GamepadButton::RightStickUp,
            profile.right_stick_y.normalize(value),
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
        DPadAxis, button_from_gilrs, centered_dpad_calibration, set_axis_buttons, update_axis,
        update_button_edge, update_button_value,
    };
    use crate::{AxisCalibration, GamepadAxis, GamepadButton, GamepadId, GamepadProfile, Input};

    #[test]
    fn gilrs_buttons_map_shoulders_triggers_stick_presses_and_guide() {
        assert_eq!(
            button_from_gilrs(gilrs::Button::LeftTrigger),
            Some(GamepadButton::LeftShoulder)
        );
        assert_eq!(
            button_from_gilrs(gilrs::Button::RightTrigger),
            Some(GamepadButton::RightShoulder)
        );
        assert_eq!(
            button_from_gilrs(gilrs::Button::LeftTrigger2),
            Some(GamepadButton::LeftTrigger)
        );
        assert_eq!(
            button_from_gilrs(gilrs::Button::RightTrigger2),
            Some(GamepadButton::RightTrigger)
        );
        assert_eq!(
            button_from_gilrs(gilrs::Button::LeftThumb),
            Some(GamepadButton::LeftStickPress)
        );
        assert_eq!(
            button_from_gilrs(gilrs::Button::RightThumb),
            Some(GamepadButton::RightStickPress)
        );
        assert_eq!(
            button_from_gilrs(gilrs::Button::Mode),
            Some(GamepadButton::Guide)
        );
    }

    #[test]
    fn trigger_values_update_analog_and_digital_state() {
        let id = GamepadId::new(5);
        let mut input = Input::default();
        let mut centered = HashMap::new();
        let profile = GamepadProfile::default();

        for (value, held) in [(0.0, false), (0.25, false), (0.75, true), (1.0, true)] {
            update_button_value(
                &mut input,
                &mut centered,
                id,
                GamepadButton::LeftTrigger,
                value,
                profile,
            );
            assert_eq!(
                input.gamepad_axis(id, crate::GamepadAxis::LeftTrigger),
                value
            );
            assert_eq!(
                input.gamepad_button(id, GamepadButton::LeftTrigger).held(),
                held
            );
        }
    }

    #[test]
    fn right_stick_axes_update_analog_and_digital_state() {
        let id = GamepadId::new(6);
        let mut input = Input::default();
        let profile = GamepadProfile::default();

        update_axis(&mut input, id, gilrs::Axis::RightStickX, -0.8, profile);
        update_axis(&mut input, id, gilrs::Axis::RightStickY, 0.7, profile);

        assert_eq!(input.gamepad_axis(id, GamepadAxis::RightStickX), -0.8);
        assert_eq!(input.gamepad_axis(id, GamepadAxis::RightStickY), 0.7);
        assert!(
            input
                .gamepad_button(id, GamepadButton::RightStickLeft)
                .held()
        );
        assert!(input.gamepad_button(id, GamepadButton::RightStickUp).held());
        assert!(
            !input
                .gamepad_button(id, GamepadButton::RightStickRight)
                .held()
        );
        assert!(
            !input
                .gamepad_button(id, GamepadButton::RightStickDown)
                .held()
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
    fn centered_dpad_ignores_gilrs_digital_edge_before_axis_value() {
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

        update_button_edge(&mut input, &centered, id, GamepadButton::DPadUp, true);
        update_button_value(
            &mut input,
            &mut centered,
            id,
            GamepadButton::DPadUp,
            1.0,
            profile,
        );

        assert!(!input.gamepad_button(id, GamepadButton::DPadUp).pressed());
        assert!(input.gamepad_button(id, GamepadButton::DPadDown).pressed());
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
mod browser;
#[cfg(target_arch = "wasm32")]
pub(crate) use browser::GamepadInputBackend;
