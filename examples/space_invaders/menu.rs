use gotoo_pixel_engine::{
    Framebuffer, GamepadButton, Input, Key, Pixel, Rect,
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
}

#[derive(Debug)]
pub struct SpaceInvadersMenu {
    page: Page,
    state: MenuState,
}

impl SpaceInvadersMenu {
    pub const fn new() -> Self {
        Self {
            page: Page::Main,
            state: MenuState::new(3),
        }
    }

    pub fn update(&mut self, input: &Input) -> Option<MenuAction> {
        match self.page {
            Page::Main => self.update_main(input),
            Page::Controls => {
                if back_pressed(input) || confirm_pressed(input) {
                    self.page = Page::Main;
                }
                None
            }
        }
    }

    fn update_main(&mut self, input: &Input) -> Option<MenuAction> {
        if menu_up_pressed(input) {
            self.state.select_previous();
        }
        if menu_down_pressed(input) {
            self.state.select_next();
        }
        if !confirm_pressed(input) {
            return None;
        }

        match self.state.selected() {
            Some(0) => Some(MenuAction::Play),
            Some(1) => {
                self.page = Page::Controls;
                None
            }
            Some(2) => Some(MenuAction::Quit),
            _ => None,
        }
    }

    pub fn render(&self, framebuffer: &mut Framebuffer) {
        framebuffer.clear(BACKGROUND);
        match self.page {
            Page::Main => self.render_main(framebuffer),
            Page::Controls => self.render_controls(framebuffer),
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
                self.state.selected() == Some(index),
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
                x: 30,
                y: 34,
                width: 196,
                height: 154,
            },
            BACKGROUND,
            BORDER,
        );
        draw_text_centered(
            framebuffer,
            Rect {
                x: 40,
                y: 48,
                width: 176,
                height: 16,
            },
            "CONTROLS",
            1,
            ACCENT,
        );

        for (text, y) in [
            ("MOVE  ARROWS / DPAD", 82),
            ("FIRE  SPACE / PAD SOUTH", 104),
            ("ESC   BACK", 126),
            ("FIRE OR ESC TO GO BACK", 164),
        ] {
            draw_text_centered(
                framebuffer,
                Rect {
                    x: 36,
                    y,
                    width: 184,
                    height: 12,
                },
                text,
                1,
                FOREGROUND,
            );
        }
    }
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
