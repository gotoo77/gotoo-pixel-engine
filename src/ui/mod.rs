pub mod experimental;
pub mod experimental_spatial;

mod kernel;
mod pause;
mod toolkit;
mod virtual_pad;

#[cfg(test)]
mod ordinal_identity_tests;
#[cfg(test)]
mod tabs_contract_tests;

pub use pause::{PauseConfig, PauseGame};
pub use toolkit::{RepeatConfig, RepeatState, Ui, UiResponse, UiState, UiTheme};
pub use virtual_pad::{VirtualButton, VirtualPad, VirtualPadUpdate};

use crate::{
    ActionId, ControlBinding, ControlMap, Framebuffer, GamepadButton, Input, Key, Pixel, Rect,
};

const STANDARD_MENU_UP_BINDINGS: [ControlBinding; 4] = [
    ControlBinding::Key(Key::Up),
    ControlBinding::Key(Key::W),
    ControlBinding::Gamepad(GamepadButton::DPadUp),
    ControlBinding::Gamepad(GamepadButton::LeftStickUp),
];
const STANDARD_MENU_DOWN_BINDINGS: [ControlBinding; 4] = [
    ControlBinding::Key(Key::Down),
    ControlBinding::Key(Key::S),
    ControlBinding::Gamepad(GamepadButton::DPadDown),
    ControlBinding::Gamepad(GamepadButton::LeftStickDown),
];
const STANDARD_MENU_CONFIRM_BINDINGS: [ControlBinding; 2] = [
    ControlBinding::Key(Key::Space),
    ControlBinding::Gamepad(GamepadButton::South),
];

pub fn draw_panel(framebuffer: &mut Framebuffer, rect: Rect, background: Pixel, border: Pixel) {
    framebuffer.fill_rect(rect.x, rect.y, rect.width, rect.height, background);
    framebuffer.draw_rect(rect.x, rect.y, rect.width, rect.height, border);
}

pub fn draw_text_centered(
    framebuffer: &mut Framebuffer,
    rect: Rect,
    text: &str,
    scale: u32,
    color: Pixel,
) {
    let scale = scale.max(1);
    let (text_width, text_height) = Framebuffer::text_size(text, scale);
    let x = centered_coordinate(rect.x, rect.width, text_width);
    let y = centered_coordinate(rect.y, rect.height, text_height);
    framebuffer.draw_text_scaled(x, y, text, scale, color);
}

pub fn draw_menu_item(
    framebuffer: &mut Framebuffer,
    rect: Rect,
    label: &str,
    selected: bool,
    scale: u32,
    foreground: Pixel,
    accent: Pixel,
) {
    let color = if selected { accent } else { foreground };
    draw_text_centered(framebuffer, rect, label, scale, color);

    if selected {
        let marker_width = 12_u32.saturating_mul(scale.max(1));
        let marker_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: marker_width.min(rect.width),
            height: rect.height,
        };
        draw_text_centered(framebuffer, marker_rect, ">", scale, accent);
    }
}

pub fn standard_menu_controls(up: ActionId, down: ActionId, confirm: ActionId) -> ControlMap {
    let mut controls = ControlMap::new();
    bind_menu_action(&mut controls, up, &STANDARD_MENU_UP_BINDINGS);
    bind_menu_action(&mut controls, down, &STANDARD_MENU_DOWN_BINDINGS);
    bind_menu_action(&mut controls, confirm, &STANDARD_MENU_CONFIRM_BINDINGS);
    controls
}

pub fn menu_up_pressed(input: &Input) -> bool {
    menu_bindings_pressed(input, &STANDARD_MENU_UP_BINDINGS)
}

pub fn menu_down_pressed(input: &Input) -> bool {
    menu_bindings_pressed(input, &STANDARD_MENU_DOWN_BINDINGS)
}

pub fn menu_confirm_pressed(input: &Input) -> bool {
    menu_bindings_pressed(input, &STANDARD_MENU_CONFIRM_BINDINGS)
}

fn bind_menu_action(controls: &mut ControlMap, action: ActionId, bindings: &[ControlBinding]) {
    for binding in bindings {
        controls.bind(action, *binding);
    }
}

fn menu_bindings_pressed(input: &Input, bindings: &[ControlBinding]) -> bool {
    bindings
        .iter()
        .copied()
        .any(|binding| binding_pressed(input, binding))
}

fn binding_pressed(input: &Input, binding: ControlBinding) -> bool {
    match binding {
        ControlBinding::Key(key) => input.key(key).pressed(),
        ControlBinding::Gamepad(button) => input.gamepad_button_any(button).pressed(),
        ControlBinding::GamepadDevice(id, button) => input.gamepad_button(id, button).pressed(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuState {
    selected: usize,
    item_count: usize,
}

impl MenuState {
    pub const fn new(item_count: usize) -> Self {
        Self {
            selected: 0,
            item_count,
        }
    }

    pub const fn selected(self) -> Option<usize> {
        if self.item_count == 0 {
            None
        } else {
            Some(self.selected)
        }
    }

    pub fn select_next(&mut self) {
        if self.item_count == 0 {
            return;
        }
        self.selected = (self.selected + 1) % self.item_count;
    }

    pub fn select_previous(&mut self) {
        if self.item_count == 0 {
            return;
        }
        self.selected = if self.selected == 0 {
            self.item_count - 1
        } else {
            self.selected - 1
        };
    }
}

fn centered_coordinate(origin: i32, extent: u32, content_extent: u32) -> i32 {
    let coordinate = i64::from(origin) + (i64::from(extent) - i64::from(content_extent)) / 2;
    coordinate.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    const MENU_UP: ActionId = ActionId::new("test.menu.up");
    const MENU_DOWN: ActionId = ActionId::new("test.menu.down");
    const MENU_CONFIRM: ActionId = ActionId::new("test.menu.confirm");

    #[test]
    fn menu_navigation_wraps_in_both_directions() {
        let mut menu = MenuState::new(3);
        assert_eq!(menu.selected(), Some(0));

        menu.select_previous();
        assert_eq!(menu.selected(), Some(2));

        menu.select_next();
        assert_eq!(menu.selected(), Some(0));
        menu.select_next();
        assert_eq!(menu.selected(), Some(1));
    }

    #[test]
    fn empty_menu_has_no_selection_and_navigation_is_noop() {
        let mut menu = MenuState::new(0);
        assert_eq!(menu.selected(), None);
        menu.select_next();
        menu.select_previous();
        assert_eq!(menu.selected(), None);
    }

    #[test]
    fn menu_input_helpers_are_idle_for_default_input() {
        let input = Input::default();

        assert!(!menu_up_pressed(&input));
        assert!(!menu_down_pressed(&input));
        assert!(!menu_confirm_pressed(&input));
    }

    #[test]
    fn standard_menu_control_map_is_idle_for_default_input() {
        let mut controls = standard_menu_controls(MENU_UP, MENU_DOWN, MENU_CONFIRM);
        controls.update(&Input::default());
        assert!(!controls.action(MENU_UP).held());
        assert!(!controls.action(MENU_DOWN).held());
        assert!(!controls.action(MENU_CONFIRM).held());
    }

    #[test]
    fn menu_helpers_and_control_map_share_keyboard_policy() {
        let mut input = Input::default();
        input.press_key(Key::W);
        let mut controls = standard_menu_controls(MENU_UP, MENU_DOWN, MENU_CONFIRM);
        controls.update(&input);

        assert!(menu_up_pressed(&input));
        assert!(controls.action(MENU_UP).pressed());
    }

    #[test]
    fn menu_helpers_and_control_map_share_gamepad_policy() {
        let mut input = Input::default();
        let id = crate::GamepadId::new(0);
        input.set_gamepad_button(id, GamepadButton::South, true);
        let mut controls = standard_menu_controls(MENU_UP, MENU_DOWN, MENU_CONFIRM);
        controls.update(&input);

        assert!(menu_confirm_pressed(&input));
        assert!(controls.action(MENU_CONFIRM).pressed());
    }

    #[test]
    fn panel_draws_border_and_background() {
        let mut framebuffer = Framebuffer::new(8, 8);
        let background = Pixel::rgb(10, 20, 30);
        let border = Pixel::rgb(200, 210, 220);

        draw_panel(
            &mut framebuffer,
            Rect {
                x: 1,
                y: 1,
                width: 6,
                height: 6,
            },
            background,
            border,
        );

        assert_eq!(framebuffer.pixel(1, 1), Some(border));
        assert_eq!(framebuffer.pixel(2, 2), Some(background));
    }
}
