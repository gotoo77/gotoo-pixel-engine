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

#[derive(Debug)]
pub struct SpaceInvadersMenu {
    page: Page,
    main_state: MenuState,
    controls_state: MenuState,
}

impl SpaceInvadersMenu {
    pub const fn new() -> Self {
        Self {
            page: Page::Main,
            main_state: MenuState::new(3),
            controls_state: MenuState::new(2),
        }
    }

    pub fn update(&mut self, input: &Input) -> Option<MenuAction> {
        match self.page {
            Page::Main => self.update_main(input),
            Page::Controls => self.update_controls(input),
            Page::GamepadSetup => {
                if input.key(Key::Escape).pressed() {
                    self.page = Page::Controls;
                }
                None
            }
        }
    }

    fn update_main(&mut self, input: &Input) -> Option<MenuAction> {
        if back_pressed(input) {
            return Some(MenuAction::Quit);
        }
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
        if back_pressed(input) {
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

    pub fn render(&self, framebuffer: &mut Framebuffer, input: &Input) {
        framebuffer.clear(BACKGROUND);
        match self.page {
            Page::Main => self.render_main(framebuffer),
            Page::Controls => self.render_controls(framebuffer, input),
            Page::GamepadSetup => self.render_gamepad_setup(framebuffer, input),
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

    fn render_controls(&self, framebuffer: &mut Framebuffer, input: &Input) {
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
        draw_control_row(framebuffer, 86, "BACK", "ESC");

        framebuffer.draw_text(18, 104, "GAMEPAD", ACCENT);
        let gamepad_id = first_gamepad(input);
        let gamepad_name = gamepad_display_name(input, gamepad_id);
        draw_control_row(framebuffer, 118, "DEVICE", &gamepad_name);
        draw_control_row(
            framebuffer,
            130,
            "STATUS",
            if gamepad_id.is_some() {
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

    fn render_gamepad_setup(&self, framebuffer: &mut Framebuffer, input: &Input) {
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

        let gamepad_id = first_gamepad(input);
        let gamepad_name = gamepad_display_name(input, gamepad_id);
        draw_text_centered(
            framebuffer,
            Rect {
                x: 16,
                y: 34,
                width: 224,
                height: 12,
            },
            &gamepad_name,
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
            if gamepad_id.is_some() {
                "STATUS CONNECTED"
            } else {
                "STATUS NOT DETECTED"
            },
            1,
            if gamepad_id.is_some() {
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
            if gamepad_id.is_some() {
                "DPAD CENTER AUTO"
            } else {
                "DPAD CENTER WAITING"
            },
            1,
            FOREGROUND,
        );

        draw_gamepad_status(framebuffer, input, gamepad_id, 18, 84, "DPAD UP", GamepadButton::DPadUp);
        draw_gamepad_status(framebuffer, input, gamepad_id, 132, 84, "SOUTH", GamepadButton::South);
        draw_gamepad_status(framebuffer, input, gamepad_id, 18, 98, "DPAD DOWN", GamepadButton::DPadDown);
        draw_gamepad_status(framebuffer, input, gamepad_id, 132, 98, "EAST", GamepadButton::East);
        draw_gamepad_status(framebuffer, input, gamepad_id, 18, 112, "DPAD LEFT", GamepadButton::DPadLeft);
        draw_gamepad_status(framebuffer, input, gamepad_id, 132, 112, "NORTH", GamepadButton::North);
        draw_gamepad_status(framebuffer, input, gamepad_id, 18, 126, "DPAD RIGHT", GamepadButton::DPadRight);
        draw_gamepad_status(framebuffer, input, gamepad_id, 132, 126, "WEST", GamepadButton::West);
        draw_gamepad_status(framebuffer, input, gamepad_id, 18, 146, "L SHOULDER", GamepadButton::LeftShoulder);
        draw_gamepad_status(framebuffer, input, gamepad_id, 132, 146, "R SHOULDER", GamepadButton::RightShoulder);
        draw_gamepad_status(framebuffer, input, gamepad_id, 18, 160, "START", GamepadButton::Start);
        draw_gamepad_status(framebuffer, input, gamepad_id, 132, 160, "SELECT", GamepadButton::Select);
        draw_gamepad_status(framebuffer, input, gamepad_id, 18, 174, "STICK LEFT", GamepadButton::LeftStickLeft);
        draw_gamepad_status(framebuffer, input, gamepad_id, 132, 174, "STICK RIGHT", GamepadButton::LeftStickRight);

        draw_text_centered(
            framebuffer,
            Rect {
                x: 16,
                y: 196,
                width: 224,
                height: 12,
            },
            "ESC TO GO BACK",
            1,
            FOREGROUND,
        );
    }
}

fn draw_control_row(framebuffer: &mut Framebuffer, y: i32, label: &str, value: &str) {
    framebuffer.draw_text(18, y, label, FOREGROUND);
    framebuffer.draw_text(104, y, value, FOREGROUND);
}

fn draw_gamepad_status(
    framebuffer: &mut Framebuffer,
    input: &Input,
    gamepad_id: Option<GamepadId>,
    x: i32,
    y: i32,
    label: &str,
    button: GamepadButton,
) {
    let held = gamepad_id
        .map(|id| input.gamepad_button(id, button).held())
        .unwrap_or(false);
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

fn gamepad_display_name(input: &Input, gamepad_id: Option<GamepadId>) -> String {
    gamepad_id
        .and_then(|id| input.gamepad_info(id))
        .map(|info| info.name.to_ascii_uppercase().chars().take(23).collect())
        .unwrap_or_else(|| "NO GAMEPAD".to_owned())
}

fn menu_up_pressed(input: &Input) -> bool {
    input.key(Key::Up).pressed()
        || input.key(Key::W).pressed()
        || input.gamepad_button_any(GamepadButton::DPadUp).pressed()
        || input.gamepad_button_any(GamepadButton::LeftStickUp).pressed()
}

fn menu_down_pressed(input: &Input) -> bool {
    input.key(Key::Down).pressed()
        || input.key(Key::S).pressed()
        || input.gamepad_button_any(GamepadButton::DPadDown).pressed()
        || input.gamepad_button_any(GamepadButton::LeftStickDown).pressed()
}

fn confirm_pressed(input: &Input) -> bool {
    input.key(Key::Space).pressed() || input.gamepad_button_any(GamepadButton::South).pressed()
}

fn back_pressed(input: &Input) -> bool {
    input.key(Key::Escape).pressed() || input.gamepad_button_any(GamepadButton::East).pressed()
}
