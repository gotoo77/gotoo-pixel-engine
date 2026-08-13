use std::collections::HashMap;

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
    S,
    W,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    South,
    East,
    North,
    West,
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct GamepadState {
    buttons: [ButtonState; GAMEPAD_BUTTON_COUNT],
}

impl Default for GamepadState {
    fn default() -> Self {
        Self {
            buttons: [ButtonState::default(); GAMEPAD_BUTTON_COUNT],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    keys: [ButtonState; KEY_COUNT],
    mouse_buttons: [ButtonState; MOUSE_BUTTON_COUNT],
    mouse_position: Option<(i32, i32)>,
    touches: Vec<Touch>,
    gamepads: HashMap<GamepadId, GamepadState>,
}

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

    pub fn gamepad_ids(&self) -> impl Iterator<Item = GamepadId> + '_ {
        self.gamepads.keys().copied()
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

    pub(crate) fn set_gamepad_button(&mut self, id: GamepadId, button: GamepadButton, held: bool) {
        let state = self.gamepads.entry(id).or_default();
        state.buttons[gamepad_button_index(button)].set_held(held);
    }

    pub(crate) fn disconnect_gamepad(&mut self, id: GamepadId) {
        self.gamepads.remove(&id);
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
        }
    }
}

const KEY_COUNT: usize = 10;
const MOUSE_BUTTON_COUNT: usize = 3;
const GAMEPAD_BUTTON_COUNT: usize = 14;

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
        Key::S => 8,
        Key::W => 9,
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
        GamepadButton::Start => 4,
        GamepadButton::Select => 5,
        GamepadButton::DPadUp => 6,
        GamepadButton::DPadDown => 7,
        GamepadButton::DPadLeft => 8,
        GamepadButton::DPadRight => 9,
        GamepadButton::LeftStickUp => 10,
        GamepadButton::LeftStickDown => 11,
        GamepadButton::LeftStickLeft => 12,
        GamepadButton::LeftStickRight => 13,
    }
}

#[cfg(test)]
mod tests {
    use super::{GamepadButton, GamepadId, Input, Key, MouseButton, Touch, TouchPhase};

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
    fn disconnect_removes_gamepad_state() {
        let mut input = Input::default();
        let id = GamepadId::new(3);
        input.set_gamepad_button(id, GamepadButton::South, true);
        input.disconnect_gamepad(id);

        assert!(!input.gamepad_button(id, GamepadButton::South).held());
        assert!(input.gamepad_ids().all(|gamepad_id| gamepad_id != id));
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
