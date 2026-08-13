use gotoo_pixel_engine::{
    Framebuffer, GamepadButton, GamepadId, Input, Key, Pixel, Rect,
    ui::{MenuState, draw_menu_item, draw_panel, draw_text_centered},
};

use super::game::FRAMEBUFFER_WIDTH;

const BACKGROUND: Pixel = Pixel::rgb(4, 8, 8);
const FOREGROUND: Pixel = Pixel::rgb(220, 240, 220);
const ACCENT: Pixel = Pixel::rgb(120, 255, 120);
const BORDER: Pixel = Pixel::rgb(80, 180, 255);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Play,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Main,
    Controls,
    GamepadSetup,
}

#[derive(Debug, Clone)]
struct GamepadSnapshot {
    name: String,
    connected: bool,
    dpad_up: bool,
    dpad_down: bool,
    dpad_left: bool,
    dpad_right: bool,
    south: bool,
    east: bool,
    north: bool,
    west: bool,
    left_shoulder: bool,
    right_shoulder: bool,
    start: bool,
    stick_left: bool,
    stick_right: bool,
}

impl GamepadSnapshot {
    fn disconnected() -> Self {
        Self {
            name: "NO GAMEPAD".to_owned(),
            connected: false,
            dpad_up: false,
            dpad_down: false,
            dpad_left: false,
            dpad_right: false,
            south: false,
            east: false,
            north: false,
            west: false,
            left_shoulder: false,
            right_shoulder: false,
            start: false,
            stick_left: false,
            stick_right: false,
        }
    }

    fn capture(input: &Input) -> Self {
        let Some(id) = first_gamepad(input) else {
            return Self::disconnected();
        };

        Self {
            name: gamepad_display_name(input, id),
            connected: true,
            dpad_up: held(input, id, GamepadButton::DPadUp),
            dpad_down: held(input, id, GamepadButton::DPadDown),
            dpad_left: held(input, id, GamepadButton::DPadLeft),
            dpad_right: held(input, id, GamepadButton::DPadRight),
            south: held(input, id, GamepadButton::South),
            east: held(input, id, GamepadButton::East),
            north: held(input, id, GamepadButton::North),
            west: held(input, id, GamepadButton::West),
            left_shoulder: held(input, id, GamepadButton::LeftShoulder),
            right_shoulder: held(input, id, GamepadButton::RightShoulder),
            start: held(input, id, GamepadButton::Start),
            stick_left: held(input, id, GamepadButton::LeftStickLeft),
            stick_right: held(input, id, GamepadButton::LeftStickRight),
        }
    }
}

#[derive(Debug)]
pub struct SpaceInvadersMenu {
    page: Page,
    main_state: MenuState,
    controls_state: MenuState,
    gamepad: GamepadSnapshot,
}

impl SpaceInvadersMenu {
    pub fn new() -> Self {
        Self {
            page: Page::Main,
            main_state: MenuState::new(3),
            controls_state: MenuState::new(2),
            gamepad: GamepadSnapshot::disconnected(),
        }
    }

    pub fn update(&mut self, input: &Input) -> Option<MenuAction> {
        self.gamepad = GamepadSnapshot::capture(input);

        match self.page {
            Page::Main => self.update_main(input),
            Page::Controls => self.update_controls(input),
            Page::GamepadSetup => {
                if input.gamepad_button_any(GamepadButton::Select).pressed() {
                    self.page = Page::Controls;
                }
                None
            }
        }
    }

    fn update_main(&mut self, input: &Input) -> Option<MenuAction> {
        if menu_up_pressed(input) {
            self.main_state.select_previous();
        }
        if menu_down_pressed(input) {
            self.main_state.select_next();
        }
        if !confirm_pressed(input) {
            return None;
        }

        match self.main_state.selected() {
            Some(0) => Some(MenuAction::Play),
            Some(1) => {
                self.page = Page::Controls;
                None
            }
            Some(2) => Some(MenuAction::Quit),
            _ => None,
        }
    }

    fn update_controls(&mut self, input: &Input) -> Option<MenuAction> {
        if input.gamepad_button_any(GamepadButton::East).pressed() {
            self.page = Page::Main;
            return None;
        }
        if menu_up_pressed(input) {
            self.controls_state.select_previous();
        }
        if menu_down_pressed(input) {
            self.controls_state.select_next();
        }
        if !confirm_pressed(input) {
            return None;
        }

        match self.controls_state.selected() {
            Some(0) => self.page = Page::GamepadSetup,
            Some(1) => self.page = Page::Main,
            _ => {}
        }
        None
    }

    pub fn render(&self, framebuffer: &mut Framebuffer) {
        framebuffer.clear(BACKGROUND);
        match self.page {
            Page::Main => self.render_main(framebuffer),
            Page::Controls => self.render_controls(framebuffer),
            Page::GamepadSetup => self.render_gamepad_setup(framebuffer),
        }
    }

    fn render_main(&self, framebuffer: &mut Framebuffer) {
        draw_panel(
            framebuffer,
            Rect {
                x: 42,
                y: 30,
                width: 172,
                height: 158,
            },
            BACKGROUND,
            BORDER,
        );
        draw_text_centered(
            framebuffer,
            Rect {
                x: 48,
                y: 46,
                width: 160,
                height: 24,
            },
            "SPACE INVADERS",
            2,
            ACCENT,
        );

        for (index, (label, y)) in [("PLAY", 92), ("CONTROLS", 116), ("QUIT", 140)]
            .into_iter()
            .enumerate()
        {
            draw_menu_item(
                framebuffer,
                Rect {
                    x: 62,
                    y,
                    width: 132,
                    height: 16,
                },
                label,
                self.main_state.selected() == Some(index),
                1,
                FOREGROUND,
                ACCENT,
            );
        }

        draw_text_centered(
            framebuffer,
            Rect {
                x: 8,
                y: 204,
                width: FRAMEBUFFER_WIDTH - 16,
                height: 12,
            },
            "UP/DOWN + FIRE TO SELECT",
            1,
            FOREGROUND,
        );
    }

    fn render_controls(&self, framebuffer: &mut Framebuffer) {
        draw_panel(
            framebuffer,
            Rect {
                x: 8,
                y: 6,
                width: 240,
                height: 212,
            },
            BACKGROUND,
            BORDER,
        );
        draw_text_centered(
            framebuffer,
            Rect {
                x: 16,
                y: 14,
                width: 224,
                height: 16,
            },
            "CONTROLS",
            1,
            ACCENT,
        );

        framebuffer.draw_text(18, 36, "KEYBOARD", ACCENT);
        draw_control_row(framebuffer, 50, "MOVE LEFT", "LEFT / A");
        draw_control_row(framebuffer, 62, "MOVE RIGHT", "RIGHT / D");
        draw_control_row(framebuffer, 74, "FIRE", "SPACE");
        draw_control_row(framebuffer, 86, "QUIT", "ESC");

        framebuffer.draw_text(18, 104, "GAMEPAD", ACCENT);
        draw_control_row(framebuffer, 118, "DEVICE", &self.gamepad.name);
        draw_control_row(
            framebuffer,
            130,
            "STATUS",
            if self.gamepad.connected {
                "CONNECTED"
            } else {
                "NOT DETECTED"
            },
        );
        draw_control_row(framebuffer, 142, "MOVE", "DPAD / LEFT STICK");
        draw_control_row(framebuffer, 154, "FIRE", "SOUTH");
        draw_control_row(framebuffer, 166, "BACK", "EAST");

        for (index, (label, y)) in [("GAMEPAD SETUP", 182), ("BACK", 198)]
            .into_iter()
            .enumerate()
        {
            draw_menu_item(
                framebuffer,
                Rect {
                    x: 54,
                    y,
                    width: 148,
                    height: 12,
                },
                label,
                self.controls_state.selected() == Some(index),
                1,
                FOREGROUND,
                ACCENT,
            );
        }
    }

    fn render_gamepad_setup(&self, framebuffer: &mut Framebuffer) {
        draw_panel(
            framebuffer,
            Rect {
                x: 8,
                y: 6,
                width: 240,
                height: 212,
            },
            BACKGROUND,
            BORDER,
        );
        draw_text_centered(
            framebuffer,
            Rect {
                x: 16,
                y: 14,
                width: 224,
                height: 16,
            },
            "GAMEPAD SETUP",
            1,
            ACCENT,
        );
        draw_text_centered(
            framebuffer,
            Rect {
                x: 16,
                y: 34,
                width: 224,
                height: 12,
            },
            &self.gamepad.name,
            1,
            FOREGROUND,
        );
        draw_text_centered(
            framebuffer,
            Rect {
                x: 16,
                y: 48,
                width: 224,
                height: 12,
            },
            if self.gamepad.connected {
                "STATUS CONNECTED"
            } else {
                "STATUS NOT DETECTED"
            },
            1,
            if self.gamepad.connected {
                ACCENT
            } else {
                FOREGROUND
            },
        );
        draw_text_centered(
            framebuffer,
            Rect {
                x: 16,
                y: 62,
                width: 224,
                height: 12,
            },
            if self.gamepad.connected {
                "DPAD CENTER AUTO"
            } else {
                "DPAD CENTER WAITING"
            },
            1,
            FOREGROUND,
        );

        draw_gamepad_status(framebuffer, 18, 84, "DPAD UP", self.gamepad.dpad_up);
        draw_gamepad_status(framebuffer, 132, 84, "SOUTH", self.gamepad.south);
        draw_gamepad_status(framebuffer, 18, 98, "DPAD DOWN", self.gamepad.dpad_down);
        draw_gamepad_status(framebuffer, 132, 98, "EAST", self.gamepad.east);
        draw_gamepad_status(framebuffer, 18, 112, "DPAD LEFT", self.gamepad.dpad_left);
        draw_gamepad_status(framebuffer, 132, 112, "NORTH", self.gamepad.north);
        draw_gamepad_status(framebuffer, 18, 126, "DPAD RIGHT", self.gamepad.dpad_right);
        draw_gamepad_status(framebuffer, 132, 126, "WEST", self.gamepad.west);
        draw_gamepad_status(
            framebuffer,
            18,
            146,
            "L SHOULDER",
            self.gamepad.left_shoulder,
        );
        draw_gamepad_status(
            framebuffer,
            132,
            146,
            "R SHOULDER",
            self.gamepad.right_shoulder,
        );
        draw_gamepad_status(framebuffer, 18, 160, "START", self.gamepad.start);
        draw_gamepad_status(framebuffer, 18, 174, "STICK LEFT", self.gamepad.stick_left);
        draw_gamepad_status(
            framebuffer,
            132,
            174,
            "STICK RIGHT",
            self.gamepad.stick_right,
        );

        draw_text_centered(
            framebuffer,
            Rect {
                x: 16,
                y: 196,
                width: 224,
                height: 12,
            },
            "SELECT TO GO BACK",
            1,
            FOREGROUND,
        );
    }
}

fn draw_control_row(framebuffer: &mut Framebuffer, y: i32, label: &str, value: &str) {
    framebuffer.draw_text(18, y, label, FOREGROUND);
    framebuffer.draw_text(104, y, value, FOREGROUND);
}

fn draw_gamepad_status(framebuffer: &mut Framebuffer, x: i32, y: i32, label: &str, held: bool) {
    framebuffer.draw_text(x, y, label, FOREGROUND);
    framebuffer.draw_text(
        x + 72,
        y,
        if held { "ON" } else { "OFF" },
        if held { ACCENT } else { FOREGROUND },
    );
}

fn first_gamepad(input: &Input) -> Option<GamepadId> {
    input.gamepad_ids().min_by_key(|id| id.as_usize())
}

fn gamepad_display_name(input: &Input, gamepad_id: GamepadId) -> String {
    input
        .gamepad_info(gamepad_id)
        .map(|info| info.name.to_ascii_uppercase().chars().take(23).collect())
        .unwrap_or_else(|| "GAMEPAD".to_owned())
}

fn held(input: &Input, gamepad_id: GamepadId, button: GamepadButton) -> bool {
    input.gamepad_button(gamepad_id, button).held()
}

fn menu_up_pressed(input: &Input) -> bool {
    input.key(Key::Up).pressed()
        || input.key(Key::W).pressed()
        || input.gamepad_button_any(GamepadButton::DPadUp).pressed()
        || input
            .gamepad_button_any(GamepadButton::LeftStickUp)
            .pressed()
}

fn menu_down_pressed(input: &Input) -> bool {
    input.key(Key::Down).pressed()
        || input.key(Key::S).pressed()
        || input.gamepad_button_any(GamepadButton::DPadDown).pressed()
        || input
            .gamepad_button_any(GamepadButton::LeftStickDown)
            .pressed()
}

fn confirm_pressed(input: &Input) -> bool {
    input.key(Key::Space).pressed() || input.gamepad_button_any(GamepadButton::South).pressed()
}
