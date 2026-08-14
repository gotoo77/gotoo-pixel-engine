#[path = "snake/game.rs"]
mod game;

use game::{SnakeGame, SnakeInteractionMode};
use gotoo_pixel_engine::{
    EngineConfig, EngineError, Frame, Game, GameResult, GamepadButton, GamepadId, Input, Key,
    Pixel, Rect, run,
    ui::{
        MenuState, draw_menu_item, draw_panel, draw_text_centered, menu_confirm_pressed,
        menu_down_pressed, menu_up_pressed,
    },
};

const BACKGROUND: Pixel = Pixel::rgb(10, 14, 18);
const FOREGROUND: Pixel = Pixel::rgb(224, 232, 210);
const ACCENT: Pixel = Pixel::rgb(82, 190, 118);
const BORDER: Pixel = Pixel::rgb(88, 102, 112);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Main,
    Controls,
}

struct SnakeApp {
    game: SnakeGame,
    page: Page,
    menu: MenuState,
    playing: bool,
}

impl SnakeApp {
    fn new() -> Self {
        Self {
            game: SnakeGame::new(SnakeInteractionMode::Keyboard),
            page: Page::Main,
            menu: MenuState::new(3),
            playing: false,
        }
    }

    fn update_menu(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if frame.input.key(Key::Escape).pressed() {
            return GameResult::Exit;
        }

        match self.page {
            Page::Main => self.update_main(frame.input),
            Page::Controls => self.update_controls(frame.input),
        }
    }

    fn update_main(&mut self, input: &Input) -> GameResult {
        if menu_up_pressed(input) {
            self.menu.select_previous();
        }
        if menu_down_pressed(input) {
            self.menu.select_next();
        }
        if menu_confirm_pressed(input) {
            match self.menu.selected() {
                Some(0) => self.playing = true,
                Some(1) => self.page = Page::Controls,
                Some(2) => return GameResult::Exit,
                _ => {}
            }
        }
        GameResult::Continue
    }

    fn update_controls(&mut self, input: &Input) -> GameResult {
        if input.gamepad_button_any(GamepadButton::East).pressed() || menu_confirm_pressed(input) {
            self.page = Page::Main;
        }
        GameResult::Continue
    }

    fn render_menu(&self, frame: &mut Frame<'_>) {
        frame.framebuffer.clear(BACKGROUND);
        match self.page {
            Page::Main => self.render_main(frame),
            Page::Controls => self.render_controls(frame),
        }
    }

    fn render_main(&self, frame: &mut Frame<'_>) {
        let framebuffer = &mut frame.framebuffer;
        draw_panel(
            framebuffer,
            Rect {
                x: 70,
                y: 24,
                width: 180,
                height: 150,
            },
            BACKGROUND,
            BORDER,
        );
        draw_text_centered(
            framebuffer,
            Rect {
                x: 82,
                y: 40,
                width: 156,
                height: 24,
            },
            "SNAKE",
            2,
            ACCENT,
        );

        for (index, (label, y)) in [("PLAY", 88), ("CONTROLS", 112), ("QUIT", 136)]
            .into_iter()
            .enumerate()
        {
            draw_menu_item(
                framebuffer,
                Rect {
                    x: 92,
                    y,
                    width: 136,
                    height: 16,
                },
                label,
                self.menu.selected() == Some(index),
                1,
                FOREGROUND,
                ACCENT,
            );
        }

        draw_text_centered(
            framebuffer,
            Rect {
                x: 8,
                y: 184,
                width: 304,
                height: 12,
            },
            "UP/DOWN + SPACE/SOUTH TO SELECT",
            1,
            FOREGROUND,
        );
    }

    fn render_controls(&self, frame: &mut Frame<'_>) {
        let framebuffer = &mut frame.framebuffer;
        draw_panel(
            framebuffer,
            Rect {
                x: 26,
                y: 12,
                width: 268,
                height: 180,
            },
            BACKGROUND,
            BORDER,
        );
        draw_text_centered(
            framebuffer,
            Rect {
                x: 38,
                y: 24,
                width: 244,
                height: 16,
            },
            "CONTROLS",
            1,
            ACCENT,
        );

        framebuffer.draw_text(44, 52, "KEYBOARD", ACCENT);
        draw_control_row(framebuffer, 66, "MOVE", "ARROWS / WASD");
        draw_control_row(framebuffer, 80, "REPLAY", "SPACE");
        draw_control_row(framebuffer, 94, "QUIT", "ESC");

        framebuffer.draw_text(44, 116, "GAMEPAD", ACCENT);
        draw_control_row(
            framebuffer,
            130,
            "DEVICE",
            &gamepad_display_name(frame.input),
        );
        draw_control_row(framebuffer, 144, "MOVE", "DPAD / LEFT STICK");
        draw_control_row(framebuffer, 158, "REPLAY", "SOUTH / START");

        draw_text_centered(
            framebuffer,
            Rect {
                x: 38,
                y: 176,
                width: 244,
                height: 12,
            },
            "SPACE/SOUTH OR EAST TO GO BACK",
            1,
            FOREGROUND,
        );
    }
}

impl Game for SnakeApp {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if self.playing {
            return self.game.update(frame);
        }

        let result = self.update_menu(frame);
        if result == GameResult::Exit {
            return result;
        }
        self.render_menu(frame);
        GameResult::Continue
    }
}

fn draw_control_row(
    framebuffer: &mut gotoo_pixel_engine::Framebuffer,
    y: i32,
    label: &str,
    value: &str,
) {
    framebuffer.draw_text(44, y, label, FOREGROUND);
    framebuffer.draw_text(116, y, value, FOREGROUND);
}

fn gamepad_display_name(input: &Input) -> String {
    first_gamepad(input)
        .and_then(|id| input.gamepad_info(id))
        .map(|info| {
            info.name
                .to_ascii_uppercase()
                .chars()
                .filter(|character| character.is_ascii_alphanumeric() || *character == ' ')
                .take(24)
                .collect()
        })
        .filter(|name: &String| !name.is_empty())
        .unwrap_or_else(|| "NOT DETECTED".to_owned())
}

fn first_gamepad(input: &Input) -> Option<GamepadId> {
    input.gamepad_ids().min_by_key(|id| id.as_usize())
}

fn main() -> Result<(), EngineError> {
    let interaction_mode = SnakeInteractionMode::Keyboard;
    let framebuffer_size = interaction_mode.framebuffer_size();

    run(
        EngineConfig {
            title: "Snake".into(),
            framebuffer_width: framebuffer_size.width,
            framebuffer_height: framebuffer_size.height,
            window_width: framebuffer_size.width * 3,
            window_height: framebuffer_size.height * 3,
        },
        SnakeApp::new(),
    )
}
