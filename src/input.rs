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

    fn advance_frame(&mut self) {
        self.bits &= Self::HELD;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    keys: [ButtonState; KEY_COUNT],
    mouse_buttons: [ButtonState; MOUSE_BUTTON_COUNT],
    mouse_position: Option<(i32, i32)>,
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

    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn advance_frame(&mut self) {
        for key in &mut self.keys {
            key.advance_frame();
        }
        for button in &mut self.mouse_buttons {
            button.advance_frame();
        }
    }
}

impl Default for Input {
    fn default() -> Self {
        Self {
            keys: [ButtonState::default(); KEY_COUNT],
            mouse_buttons: [ButtonState::default(); MOUSE_BUTTON_COUNT],
            mouse_position: None,
        }
    }
}

const KEY_COUNT: usize = 10;
const MOUSE_BUTTON_COUNT: usize = 3;

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

#[cfg(test)]
mod tests {
    use super::{Input, Key, MouseButton};

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
    fn reset_clears_buttons_and_mouse_position() {
        let mut input = Input::default();

        input.press_key(Key::Space);
        input.press_mouse_button(MouseButton::Left);
        input.set_mouse_position(Some((12, 34)));
        input.reset();

        for key in [
            Key::Escape,
            Key::Space,
            Key::Up,
            Key::Down,
            Key::Left,
            Key::Right,
            Key::A,
            Key::D,
            Key::S,
            Key::W,
        ] {
            let state = input.key(key);
            assert!(!state.pressed());
            assert!(!state.held());
            assert!(!state.released());
        }

        for button in [MouseButton::Left, MouseButton::Right, MouseButton::Middle] {
            let state = input.mouse_button(button);
            assert!(!state.pressed());
            assert!(!state.held());
            assert!(!state.released());
        }

        assert_eq!(input.mouse_position(), None);
    }
}
