//! Minimal Standard keyboard/gamepad consumer. This is example policy, not a Core shell.
//! One modal owner; the shared menu only returns intents. The caller alone advances its world.
use super::gpe::{
    ActionId, ControlMap, Frame, GamepadButton, Input, Key, Pixel,
    ui::{
        UiTheme,
        experimental::UiNavInput,
        menu::{PauseSettingsMenu, SettingsIntent},
        standard_menu_controls,
    },
};

const UP: ActionId = ActionId::new("demo.menu.up");
const DOWN: ActionId = ActionId::new("demo.menu.down");
const CONFIRM: ActionId = ActionId::new("demo.menu.confirm");
const BACK: ActionId = ActionId::new("demo.menu.back");
const LEFT: ActionId = ActionId::new("demo.menu.left");
const RIGHT: ActionId = ActionId::new("demo.menu.right");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuFrame {
    Advance,
    Suspended,
    Exit,
}

#[derive(Default)]
enum Phase {
    #[default]
    Running,
    Entering,
    Menu,
    Leaving,
}

pub struct MenuConsumer {
    phase: Phase,
    pub menu: PauseSettingsMenu,
    controls: ControlMap,
}

impl Default for MenuConsumer {
    fn default() -> Self {
        let mut controls = standard_menu_controls(UP, DOWN, CONFIRM);
        controls
            .bind_key(BACK, Key::Escape)
            .bind_gamepad(BACK, GamepadButton::Start)
            .bind_key(LEFT, Key::Left)
            .bind_gamepad(LEFT, GamepadButton::DPadLeft)
            .bind_key(RIGHT, Key::Right)
            .bind_gamepad(RIGHT, GamepadButton::DPadRight);
        Self {
            phase: Phase::Running,
            menu: PauseSettingsMenu::new(),
            controls,
        }
    }
}

impl MenuConsumer {
    pub fn update(&mut self, frame: &mut Frame<'_>) -> MenuFrame {
        self.controls.update(frame.input);
        let nav = match self.phase {
            Phase::Running => {
                if !self.controls.action(BACK).pressed() {
                    return MenuFrame::Advance;
                }
                self.menu.reset_to_pause();
                self.phase = Phase::Entering;
                UiNavInput::default()
            }
            Phase::Entering => {
                if !menu_source_held(frame.input) {
                    self.phase = Phase::Menu;
                }
                UiNavInput::default()
            }
            Phase::Leaving => {
                if !menu_source_held(frame.input) {
                    self.phase = Phase::Running;
                }
                // Even the neutral/release frame is withheld from the world.
                return MenuFrame::Suspended;
            }
            Phase::Menu => UiNavInput {
                up: self.controls.action(UP).pressed(),
                down: self.controls.action(DOWN).pressed(),
                left: self.controls.action(LEFT).pressed(),
                right: self.controls.action(RIGHT).pressed(),
                confirm: self.controls.action(CONFIRM).pressed(),
                cancel: self.controls.action(BACK).pressed(),
            },
        };
        frame.framebuffer.clear(Pixel::rgb(4, 10, 17));
        // Deliberately no apply_audio: this visual-only example owns no sounds.
        // A real consumer applies proposals explicitly and reports backend errors.
        match self.menu.update(
            frame.framebuffer,
            nav,
            UiTheme {
                row_height: 44,
                padding: 24,
                ..Default::default()
            },
        ) {
            SettingsIntent::None => MenuFrame::Suspended,
            SettingsIntent::Resume => {
                self.phase = Phase::Leaving;
                MenuFrame::Suspended
            }
            SettingsIntent::Exit => MenuFrame::Exit,
        }
    }
}

fn menu_source_held(input: &Input) -> bool {
    [
        Key::Escape,
        Key::Space,
        Key::Up,
        Key::W,
        Key::Down,
        Key::S,
        Key::Left,
        Key::Right,
    ]
    .into_iter()
    .any(|key| input.key(key).held())
        || [
            GamepadButton::Start,
            GamepadButton::South,
            GamepadButton::DPadUp,
            GamepadButton::DPadDown,
            GamepadButton::DPadLeft,
            GamepadButton::DPadRight,
            GamepadButton::LeftStickUp,
            GamepadButton::LeftStickDown,
        ]
        .into_iter()
        .any(|button| input.gamepad_button_any(button).held())
}
