use std::cell::RefCell;
use std::collections::HashMap;

use crate::gamepad_profile::{GamepadProfile, GamepadProfiles};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    Escape,
    Space,
    Up,
    Down,
    Left,
    Right,
    A,
    D,
    E,
    F,
    R,
    S,
    W,
    X,
    LeftShift,
    RightShift,
    LeftControl,
    RightControl,
    C,
    L,
    M,
    H,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GamepadId(usize);

impl GamepadId {
    pub const fn new(id: usize) -> Self {
        Self(id)
    }

    pub const fn as_usize(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GamepadDeviceInfo {
    pub id: GamepadId,
    pub name: String,
    pub os_name: Option<String>,
    pub mapping_name: Option<String>,
    pub mapping_source: GamepadMappingSource,
    pub guid: Option<String>,
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,
    pub capabilities: GamepadCapabilities,
}

impl GamepadDeviceInfo {
    pub fn new(id: GamepadId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            os_name: None,
            mapping_name: None,
            mapping_source: GamepadMappingSource::Unknown,
            guid: None,
            vendor_id: None,
            product_id: None,
            capabilities: GamepadCapabilities::unknown(),
        }
    }

    fn unknown(id: GamepadId) -> Self {
        Self::new(id, format!("Gamepad {}", id.as_usize()))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum GamepadMappingSource {
    SdlMappings,
    Driver,
    BrowserStandard,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum GamepadCapability {
    Available,
    Unavailable,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GamepadCapabilities {
    buttons: [GamepadCapability; GAMEPAD_BUTTON_COUNT],
    axes: [GamepadCapability; GAMEPAD_AXIS_COUNT],
}

impl GamepadCapabilities {
    pub const fn unknown() -> Self {
        Self {
            buttons: [GamepadCapability::Unknown; GAMEPAD_BUTTON_COUNT],
            axes: [GamepadCapability::Unknown; GAMEPAD_AXIS_COUNT],
        }
    }

    pub const fn standard() -> Self {
        Self {
            buttons: [GamepadCapability::Available; GAMEPAD_BUTTON_COUNT],
            axes: [GamepadCapability::Available; GAMEPAD_AXIS_COUNT],
        }
    }

    pub fn button(&self, button: GamepadButton) -> GamepadCapability {
        self.buttons[gamepad_button_index(button)]
    }

    pub fn axis(&self, axis: GamepadAxis) -> GamepadCapability {
        self.axes[gamepad_axis_index(axis)]
    }

    #[cfg(any(not(target_arch = "wasm32"), test))]
    pub(crate) fn set_button(&mut self, button: GamepadButton, capability: GamepadCapability) {
        self.buttons[gamepad_button_index(button)] = capability;
    }

    #[cfg(any(not(target_arch = "wasm32"), test))]
    pub(crate) fn set_axis(&mut self, axis: GamepadAxis, capability: GamepadCapability) {
        self.axes[gamepad_axis_index(axis)] = capability;
    }
}

impl Default for GamepadCapabilities {
    fn default() -> Self {
        Self::unknown()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GamepadConnectionEvent {
    Connected(GamepadDeviceInfo),
    Disconnected(GamepadDeviceInfo),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    South,
    East,
    North,
    West,
    LeftShoulder,
    RightShoulder,
    Start,
    Select,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    LeftStickUp,
    LeftStickDown,
    LeftStickLeft,
    LeftStickRight,
    LeftTrigger,
    RightTrigger,
    LeftStickPress,
    RightStickPress,
    Guide,
    RightStickUp,
    RightStickDown,
    RightStickLeft,
    RightStickRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftTrigger,
    RightTrigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TouchPhase {
    Started,
    Moved,
    Ended,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Touch {
    pub id: u64,
    pub phase: TouchPhase,
    pub position: Option<(i32, i32)>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ButtonState {
    bits: u8,
}

impl ButtonState {
    const PRESSED: u8 = 1;
    const HELD: u8 = 1 << 1;
    const RELEASED: u8 = 1 << 2;

    pub fn pressed(self) -> bool {
        self.bits & Self::PRESSED != 0
    }

    pub fn held(self) -> bool {
        self.bits & Self::HELD != 0
    }

    pub fn released(self) -> bool {
        self.bits & Self::RELEASED != 0
    }

    pub(crate) fn from_transition(was_held: bool, held: bool) -> Self {
        let mut bits = 0;
        if held {
            bits |= Self::HELD;
        }
        if held && !was_held {
            bits |= Self::PRESSED;
        }
        if !held && was_held {
            bits |= Self::RELEASED;
        }
        Self { bits }
    }

    fn set_pressed(&mut self) {
        if !self.held() {
            self.bits |= Self::PRESSED;
        }
        self.bits |= Self::HELD;
        self.bits &= !Self::RELEASED;
    }

    fn set_released(&mut self) {
        if self.held() {
            self.bits |= Self::RELEASED;
        }
        self.bits &= !Self::HELD;
    }

    fn set_held(&mut self, held: bool) {
        if held {
            self.set_pressed();
        } else {
            self.set_released();
        }
    }

    fn advance_frame(&mut self) {
        self.bits &= Self::HELD;
    }
}

#[derive(Debug, Clone)]
struct GamepadState {
    info: GamepadDeviceInfo,
    buttons: [ButtonState; GAMEPAD_BUTTON_COUNT],
    axes: [f32; GAMEPAD_AXIS_COUNT],
}

impl GamepadState {
    fn new(info: GamepadDeviceInfo) -> Self {
        Self {
            info,
            buttons: [ButtonState::default(); GAMEPAD_BUTTON_COUNT],
            axes: [0.0; GAMEPAD_AXIS_COUNT],
        }
    }
}

impl PartialEq for GamepadState {
    fn eq(&self, other: &Self) -> bool {
        self.info == other.info
            && self.buttons == other.buttons
            && self
                .axes
                .iter()
                .zip(other.axes.iter())
                .all(|(left, right)| left.to_bits() == right.to_bits())
    }
}

impl Eq for GamepadState {}

#[derive(Debug, Clone)]
pub struct Input {
    keys: [ButtonState; KEY_COUNT],
    mouse_buttons: [ButtonState; MOUSE_BUTTON_COUNT],
    mouse_position: Option<(i32, i32)>,
    touches: Vec<Touch>,
    gamepads: HashMap<GamepadId, GamepadState>,
    gamepad_connection_events: Vec<GamepadConnectionEvent>,
    gamepad_profiles: RefCell<GamepadProfiles>,
}

impl PartialEq for Input {
    fn eq(&self, other: &Self) -> bool {
        self.keys == other.keys
            && self.mouse_buttons == other.mouse_buttons
            && self.mouse_position == other.mouse_position
            && self.touches == other.touches
            && self.gamepads == other.gamepads
            && self.gamepad_connection_events == other.gamepad_connection_events
    }
}

impl Eq for Input {}

impl Input {
    pub fn key(&self, key: Key) -> ButtonState {
        self.keys[key_index(key)]
    }

    pub fn mouse_button(&self, button: MouseButton) -> ButtonState {
        self.mouse_buttons[mouse_button_index(button)]
    }

    pub fn mouse_position(&self) -> Option<(i32, i32)> {
        self.mouse_position
    }

    pub fn touches(&self) -> &[Touch] {
        &self.touches
    }

    pub fn gamepad_button(&self, id: GamepadId, button: GamepadButton) -> ButtonState {
        self.gamepads
            .get(&id)
            .map(|gamepad| gamepad.buttons[gamepad_button_index(button)])
            .unwrap_or_default()
    }

    pub fn gamepad_button_any(&self, button: GamepadButton) -> ButtonState {
        let bits = self
            .gamepads
            .values()
            .map(|gamepad| gamepad.buttons[gamepad_button_index(button)].bits)
            .fold(0, |acc, bits| acc | bits);
        ButtonState { bits }
    }

    pub fn gamepad_axis(&self, id: GamepadId, axis: GamepadAxis) -> f32 {
        self.gamepads
            .get(&id)
            .map(|gamepad| gamepad.axes[gamepad_axis_index(axis)])
            .unwrap_or(0.0)
    }

    pub fn gamepad_button_capability(
        &self,
        id: GamepadId,
        button: GamepadButton,
    ) -> GamepadCapability {
        self.gamepad_info(id)
            .map(|info| info.capabilities.button(button))
            .unwrap_or(GamepadCapability::Unknown)
    }

    pub fn gamepad_axis_capability(&self, id: GamepadId, axis: GamepadAxis) -> GamepadCapability {
        self.gamepad_info(id)
            .map(|info| info.capabilities.axis(axis))
            .unwrap_or(GamepadCapability::Unknown)
    }

    pub fn gamepad_ids(&self) -> impl Iterator<Item = GamepadId> + '_ {
        self.gamepads.keys().copied()
    }

    pub fn gamepad_connected(&self, id: GamepadId) -> bool {
        self.gamepads.contains_key(&id)
    }

    pub fn gamepad_info(&self, id: GamepadId) -> Option<&GamepadDeviceInfo> {
        self.gamepads.get(&id).map(|gamepad| &gamepad.info)
    }

    pub fn gamepad_connection_events(&self) -> &[GamepadConnectionEvent] {
        &self.gamepad_connection_events
    }

    pub(crate) fn gamepad_profile(&self, id: GamepadId) -> GamepadProfile {
        self.gamepad_profiles.borrow().profile(id)
    }

    pub(crate) fn set_gamepad_profile(&self, id: GamepadId, profile: GamepadProfile) {
        self.gamepad_profiles.borrow_mut().set_profile(id, profile);
    }

    pub(crate) fn remove_gamepad_profile(&self, id: GamepadId) {
        self.gamepad_profiles.borrow_mut().remove_profile(id);
    }

    pub(crate) fn press_key(&mut self, key: Key) {
        self.keys[key_index(key)].set_pressed();
    }

    pub(crate) fn release_key(&mut self, key: Key) {
        self.keys[key_index(key)].set_released();
    }

    pub(crate) fn press_mouse_button(&mut self, button: MouseButton) {
        self.mouse_buttons[mouse_button_index(button)].set_pressed();
    }

    pub(crate) fn release_mouse_button(&mut self, button: MouseButton) {
        self.mouse_buttons[mouse_button_index(button)].set_released();
    }

    pub(crate) fn set_mouse_position(&mut self, position: Option<(i32, i32)>) {
        self.mouse_position = position;
    }

    pub(crate) fn push_touch(&mut self, touch: Touch) {
        self.touches.push(touch);
    }

    pub(crate) fn connect_gamepad(&mut self, id: GamepadId, name: impl Into<String>) {
        self.connect_gamepad_info(GamepadDeviceInfo::new(id, name));
    }

    pub(crate) fn connect_gamepad_info(&mut self, info: GamepadDeviceInfo) {
        let id = info.id;
        if let Some(gamepad) = self.gamepads.get_mut(&id) {
            gamepad.info = info;
            return;
        }

        self.gamepads.insert(id, GamepadState::new(info.clone()));
        self.gamepad_connection_events
            .push(GamepadConnectionEvent::Connected(info));
    }

    pub(crate) fn set_gamepad_axis(&mut self, id: GamepadId, axis: GamepadAxis, value: f32) {
        if !self.gamepads.contains_key(&id) {
            self.connect_gamepad(id, format!("Gamepad {}", id.as_usize()));
        }
        let value = if value.is_finite() { value } else { 0.0 };
        let range = match axis {
            GamepadAxis::LeftTrigger | GamepadAxis::RightTrigger => 0.0..=1.0,
            _ => -1.0..=1.0,
        };
        self.gamepads
            .get_mut(&id)
            .expect("gamepad state should exist after insertion")
            .axes[gamepad_axis_index(axis)] = value.clamp(*range.start(), *range.end());
    }

    pub(crate) fn set_gamepad_button(&mut self, id: GamepadId, button: GamepadButton, held: bool) {
        if let std::collections::hash_map::Entry::Vacant(entry) = self.gamepads.entry(id) {
            let info = GamepadDeviceInfo::unknown(id);
            entry.insert(GamepadState::new(info.clone()));
            self.gamepad_connection_events
                .push(GamepadConnectionEvent::Connected(info));
        }

        let state = self
            .gamepads
            .get_mut(&id)
            .expect("gamepad state should exist after insertion");
        state.buttons[gamepad_button_index(button)].set_held(held);
    }

    pub(crate) fn disconnect_gamepad(&mut self, id: GamepadId) {
        if let Some(gamepad) = self.gamepads.remove(&id) {
            self.gamepad_connection_events
                .push(GamepadConnectionEvent::Disconnected(gamepad.info));
        }
    }

    pub(crate) fn reset_window_devices(&mut self) {
        self.keys = [ButtonState::default(); KEY_COUNT];
        self.mouse_buttons = [ButtonState::default(); MOUSE_BUTTON_COUNT];
        self.mouse_position = None;
        self.touches.clear();
    }

    pub(crate) fn advance_frame(&mut self) {
        for key in &mut self.keys {
            key.advance_frame();
        }
        for button in &mut self.mouse_buttons {
            button.advance_frame();
        }
        for gamepad in self.gamepads.values_mut() {
            for button in &mut gamepad.buttons {
                button.advance_frame();
            }
        }
        self.touches.clear();
        self.gamepad_connection_events.clear();
    }
}

impl Default for Input {
    fn default() -> Self {
        Self {
            keys: [ButtonState::default(); KEY_COUNT],
            mouse_buttons: [ButtonState::default(); MOUSE_BUTTON_COUNT],
            mouse_position: None,
            touches: Vec::new(),
            gamepads: HashMap::new(),
            gamepad_connection_events: Vec::new(),
            gamepad_profiles: RefCell::new(GamepadProfiles::default()),
        }
    }
}

const KEY_COUNT: usize = 22;
const MOUSE_BUTTON_COUNT: usize = 3;
const GAMEPAD_BUTTON_COUNT: usize = 25;
const GAMEPAD_AXIS_COUNT: usize = 6;

fn key_index(key: Key) -> usize {
    match key {
        Key::Escape => 0,
        Key::Space => 1,
        Key::Up => 2,
        Key::Down => 3,
        Key::Left => 4,
        Key::Right => 5,
        Key::A => 6,
        Key::D => 7,
        Key::E => 8,
        Key::F => 9,
        Key::R => 10,
        Key::S => 11,
        Key::W => 12,
        Key::X => 13,
        Key::LeftShift => 14,
        Key::C => 15,
        Key::RightShift => 16,
        Key::LeftControl => 17,
        Key::RightControl => 18,
        Key::L => 19,
        Key::M => 20,
        Key::H => 21,
    }
}

fn mouse_button_index(button: MouseButton) -> usize {
    match button {
        MouseButton::Left => 0,
        MouseButton::Right => 1,
        MouseButton::Middle => 2,
    }
}

fn gamepad_button_index(button: GamepadButton) -> usize {
    match button {
        GamepadButton::South => 0,
        GamepadButton::East => 1,
        GamepadButton::North => 2,
        GamepadButton::West => 3,
        GamepadButton::LeftShoulder => 4,
        GamepadButton::RightShoulder => 5,
        GamepadButton::Start => 6,
        GamepadButton::Select => 7,
        GamepadButton::DPadUp => 8,
        GamepadButton::DPadDown => 9,
        GamepadButton::DPadLeft => 10,
        GamepadButton::DPadRight => 11,
        GamepadButton::LeftStickUp => 12,
        GamepadButton::LeftStickDown => 13,
        GamepadButton::LeftStickLeft => 14,
        GamepadButton::LeftStickRight => 15,
        GamepadButton::LeftTrigger => 16,
        GamepadButton::RightTrigger => 17,
        GamepadButton::LeftStickPress => 18,
        GamepadButton::RightStickPress => 19,
        GamepadButton::Guide => 20,
        GamepadButton::RightStickUp => 21,
        GamepadButton::RightStickDown => 22,
        GamepadButton::RightStickLeft => 23,
        GamepadButton::RightStickRight => 24,
    }
}

fn gamepad_axis_index(axis: GamepadAxis) -> usize {
    match axis {
        GamepadAxis::LeftStickX => 0,
        GamepadAxis::LeftStickY => 1,
        GamepadAxis::RightStickX => 2,
        GamepadAxis::RightStickY => 3,
        GamepadAxis::LeftTrigger => 4,
        GamepadAxis::RightTrigger => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        GamepadAxis, GamepadButton, GamepadCapabilities, GamepadCapability, GamepadConnectionEvent,
        GamepadDeviceInfo, GamepadId, GamepadMappingSource, Input, Key, MouseButton, Touch,
        TouchPhase,
    };
    use crate::GamepadProfile;

    #[test]
    fn key_transitions_pressed_held_released() {
        let mut input = Input::default();

        input.press_key(Key::Space);
        let state = input.key(Key::Space);
        assert!(state.pressed());
        assert!(state.held());
        assert!(!state.released());

        input.advance_frame();
        let state = input.key(Key::Space);
        assert!(!state.pressed());
        assert!(state.held());
        assert!(!state.released());

        input.release_key(Key::Space);
        let state = input.key(Key::Space);
        assert!(!state.pressed());
        assert!(!state.held());
        assert!(state.released());

        input.advance_frame();
        let state = input.key(Key::Space);
        assert!(!state.pressed());
        assert!(!state.held());
        assert!(!state.released());
    }

    #[test]
    fn held_key_is_not_pressed_again_on_repeat_press() {
        let mut input = Input::default();

        input.press_key(Key::A);
        input.advance_frame();
        input.press_key(Key::A);

        let state = input.key(Key::A);
        assert!(!state.pressed());
        assert!(state.held());
        assert!(!state.released());
    }

    #[test]
    fn modifiers_and_gameplay_keys_use_independent_key_slots() {
        let mut input = Input::default();

        input.press_key(Key::LeftShift);
        input.press_key(Key::RightShift);
        input.press_key(Key::LeftControl);
        input.press_key(Key::RightControl);

        assert!(input.key(Key::LeftShift).held());
        assert!(input.key(Key::RightShift).held());
        assert!(input.key(Key::LeftControl).held());
        assert!(input.key(Key::RightControl).held());
        assert!(!input.key(Key::X).held());
        assert!(!input.key(Key::C).held());

        input.press_key(Key::X);
        input.press_key(Key::C);
        assert!(input.key(Key::LeftShift).held());
        assert!(input.key(Key::RightShift).held());
        assert!(input.key(Key::LeftControl).held());
        assert!(input.key(Key::RightControl).held());
        assert!(input.key(Key::X).pressed());
        assert!(input.key(Key::C).pressed());
    }

    #[test]
    fn tool_shortcut_keys_use_independent_key_slots() {
        let mut input = Input::default();

        input.press_key(Key::L);
        input.press_key(Key::M);
        input.press_key(Key::H);

        assert!(input.key(Key::L).pressed());
        assert!(input.key(Key::M).pressed());
        assert!(input.key(Key::H).pressed());
        assert!(!input.key(Key::F).held());
        assert!(!input.key(Key::R).held());
    }

    #[test]
    fn mouse_button_uses_same_transition_model() {
        let mut input = Input::default();

        input.press_mouse_button(MouseButton::Left);
        assert!(input.mouse_button(MouseButton::Left).pressed());
        assert!(input.mouse_button(MouseButton::Left).held());

        input.advance_frame();
        input.release_mouse_button(MouseButton::Left);
        assert!(input.mouse_button(MouseButton::Left).released());
        assert!(!input.mouse_button(MouseButton::Left).held());
    }

    #[test]
    fn mouse_position_is_explicitly_optional() {
        let mut input = Input::default();
        assert_eq!(input.mouse_position(), None);

        input.set_mouse_position(Some((12, 34)));
        assert_eq!(input.mouse_position(), Some((12, 34)));

        input.set_mouse_position(None);
        assert_eq!(input.mouse_position(), None);
    }

    #[test]
    fn touch_events_preserve_phase_order_and_ids() {
        let mut input = Input::default();

        input.push_touch(Touch {
            id: 7,
            phase: TouchPhase::Started,
            position: Some((10, 20)),
        });
        input.push_touch(Touch {
            id: 7,
            phase: TouchPhase::Moved,
            position: Some((11, 21)),
        });

        assert_eq!(input.touches().len(), 2);
        assert_eq!(input.touches()[0].phase, TouchPhase::Started);
        assert_eq!(input.touches()[1].phase, TouchPhase::Moved);

        input.advance_frame();
        assert!(input.touches().is_empty());
    }

    #[test]
    fn explicit_connection_is_visible_before_any_button_event() {
        let mut input = Input::default();
        let id = GamepadId::new(7);

        input.connect_gamepad(id, "Test Pad");

        assert!(input.gamepad_connected(id));
        assert_eq!(
            input.gamepad_info(id).map(|info| info.name.as_str()),
            Some("Test Pad")
        );
        assert_eq!(
            input.gamepad_connection_events(),
            &[GamepadConnectionEvent::Connected(
                super::GamepadDeviceInfo::new(id, "Test Pad")
            )]
        );
    }

    #[test]
    fn gamepad_buttons_follow_same_transition_model() {
        let mut input = Input::default();
        let id = GamepadId::new(2);

        input.set_gamepad_button(id, GamepadButton::South, true);
        assert!(input.gamepad_button(id, GamepadButton::South).pressed());
        assert!(input.gamepad_button(id, GamepadButton::South).held());

        input.advance_frame();
        input.set_gamepad_button(id, GamepadButton::South, false);
        assert!(input.gamepad_button(id, GamepadButton::South).released());
        assert!(!input.gamepad_button(id, GamepadButton::South).held());
    }

    #[test]
    fn any_gamepad_combines_connected_devices() {
        let mut input = Input::default();
        input.set_gamepad_button(GamepadId::new(1), GamepadButton::DPadLeft, true);
        input.set_gamepad_button(GamepadId::new(2), GamepadButton::South, true);

        assert!(input.gamepad_button_any(GamepadButton::DPadLeft).held());
        assert!(input.gamepad_button_any(GamepadButton::South).held());
    }

    #[test]
    fn runtime_profile_state_is_separate_from_input_snapshot_equality() {
        let first = Input::default();
        let second = Input::default();
        first.set_gamepad_profile(
            GamepadId::new(1),
            GamepadProfile::standard().with_digital_threshold(0.70),
        );

        assert_eq!(first, second);
    }

    #[test]
    fn disconnect_removes_gamepad_state_and_emits_lifecycle_event() {
        let mut input = Input::default();
        let id = GamepadId::new(3);
        input.connect_gamepad(id, "Disposable Pad");
        input.advance_frame();

        input.disconnect_gamepad(id);

        assert!(!input.gamepad_button(id, GamepadButton::South).held());
        assert!(input.gamepad_ids().all(|gamepad_id| gamepad_id != id));
        assert_eq!(
            input.gamepad_connection_events(),
            &[GamepadConnectionEvent::Disconnected(
                super::GamepadDeviceInfo::new(id, "Disposable Pad")
            )]
        );
    }

    #[test]
    fn gamepad_axes_are_device_scoped_clamped_and_neutral_when_unknown() {
        let mut input = Input::default();
        let first = GamepadId::new(1);
        let second = GamepadId::new(2);

        input.set_gamepad_axis(first, GamepadAxis::RightStickX, 1.5);
        input.set_gamepad_axis(second, GamepadAxis::RightStickX, -0.25);
        input.set_gamepad_axis(first, GamepadAxis::LeftTrigger, -0.5);
        input.set_gamepad_axis(first, GamepadAxis::RightTrigger, f32::NAN);

        assert_eq!(input.gamepad_axis(first, GamepadAxis::RightStickX), 1.0);
        assert_eq!(input.gamepad_axis(second, GamepadAxis::RightStickX), -0.25);
        assert_eq!(input.gamepad_axis(first, GamepadAxis::LeftTrigger), 0.0);
        assert_eq!(input.gamepad_axis(first, GamepadAxis::RightTrigger), 0.0);
        assert_eq!(
            input.gamepad_axis(GamepadId::new(99), GamepadAxis::LeftStickY),
            0.0
        );
    }

    #[test]
    fn disconnect_removes_axes_metadata_and_capabilities() {
        let mut input = Input::default();
        let id = GamepadId::new(4);
        let mut capabilities = GamepadCapabilities::unknown();
        capabilities.set_axis(GamepadAxis::RightStickX, GamepadCapability::Available);
        input.connect_gamepad_info(GamepadDeviceInfo {
            id,
            name: "Mapped Pad".to_owned(),
            os_name: Some("OS Pad".to_owned()),
            mapping_name: Some("Test Mapping".to_owned()),
            mapping_source: GamepadMappingSource::SdlMappings,
            guid: Some("001122".to_owned()),
            vendor_id: Some(0x1234),
            product_id: Some(0xabcd),
            capabilities,
        });
        input.set_gamepad_axis(id, GamepadAxis::RightStickX, 0.75);
        assert_eq!(
            input.gamepad_axis_capability(id, GamepadAxis::RightStickX),
            GamepadCapability::Available
        );

        input.disconnect_gamepad(id);

        assert_eq!(input.gamepad_axis(id, GamepadAxis::RightStickX), 0.0);
        assert!(input.gamepad_info(id).is_none());
        assert_eq!(
            input.gamepad_axis_capability(id, GamepadAxis::RightStickX),
            GamepadCapability::Unknown
        );
    }

    #[test]
    fn unknown_mapping_does_not_fabricate_capabilities() {
        let mut input = Input::default();
        let id = GamepadId::new(8);
        input.connect_gamepad(id, "Unknown Pad");

        assert_eq!(
            input.gamepad_button_capability(id, GamepadButton::Guide),
            GamepadCapability::Unknown
        );
        assert_eq!(
            input.gamepad_axis_capability(id, GamepadAxis::LeftTrigger),
            GamepadCapability::Unknown
        );
    }

    #[test]
    fn device_metadata_is_scoped_by_gamepad_id() {
        let mut input = Input::default();
        let first = GamepadId::new(1);
        let second = GamepadId::new(2);
        let mut first_info = GamepadDeviceInfo::new(first, "First");
        first_info.vendor_id = Some(0x1111);
        let mut second_info = GamepadDeviceInfo::new(second, "Second");
        second_info.vendor_id = Some(0x2222);

        input.connect_gamepad_info(first_info);
        input.connect_gamepad_info(second_info);

        assert_eq!(
            input.gamepad_info(first).and_then(|info| info.vendor_id),
            Some(0x1111)
        );
        assert_eq!(
            input.gamepad_info(second).and_then(|info| info.vendor_id),
            Some(0x2222)
        );
    }

    #[test]
    fn lifecycle_events_are_frame_scoped() {
        let mut input = Input::default();
        input.connect_gamepad(GamepadId::new(9), "Transient Pad");
        assert!(!input.gamepad_connection_events().is_empty());

        input.advance_frame();

        assert!(input.gamepad_connection_events().is_empty());
    }

    #[test]
    fn focus_reset_keeps_gamepad_state() {
        let mut input = Input::default();
        let id = GamepadId::new(4);
        input.press_key(Key::Left);
        input.set_gamepad_button(id, GamepadButton::DPadLeft, true);

        input.reset_window_devices();

        assert!(!input.key(Key::Left).held());
        assert!(input.gamepad_button(id, GamepadButton::DPadLeft).held());
    }
}
