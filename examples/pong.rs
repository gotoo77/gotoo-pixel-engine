#[allow(dead_code)]
#[path = "pong/game.rs"]
mod game;

use game::{FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH, PongGame};
use gotoo_pixel_engine::{
    EngineConfig, EngineError, Frame, Framebuffer, Game, GameResult, GamepadButton, Input, Key,
    Pixel, Rect, Size, run,
    ui::{
        MenuState, PauseConfig, PauseGame, draw_menu_item, draw_panel, draw_text_centered,
        menu_confirm_pressed, menu_down_pressed, menu_up_pressed,
    },
};

const BG: Pixel = Pixel::rgb(6, 10, 14);
const FG: Pixel = Pixel::rgb(230, 238, 230);
const ACCENT: Pixel = Pixel::rgb(110, 235, 180);
const BORDER: Pixel = Pixel::rgb(80, 150, 220);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Main,
    Controls,
}

struct PongApp {
    page: Page,
    main_menu: MenuState,
    game: Option<PauseGame<PongGame>>,
}

impl PongApp {
    fn new() -> Self {
        Self {
            page: Page::Main,
            main_menu: MenuState::new(3),
            game: None,
        }
    }

    fn update_menu(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if frame.input.key(Key::Escape).pressed() {
            return GameResult::Exit;
        }

        match self.page {
            Page::Main => {
                if menu_up_pressed(frame.input) {
                    self.main_menu.select_previous();
                }
                if menu_down_pressed(frame.input) {
                    self.main_menu.select_next();
                }
                if menu_confirm_pressed(frame.input) {
                    match self.main_menu.selected() {
                        Some(0) => self.game = Some(paused_pong()),
                        Some(1) => self.page = Page::Controls,
                        Some(2) => return GameResult::Exit,
                        _ => {}
                    }
                }
                self.render_main_menu(frame.framebuffer);
            }
            Page::Controls => {
                if frame.input.key(Key::Escape).pressed()
                    || frame
                        .input
                        .gamepad_button_any(GamepadButton::East)
                        .pressed()
                    || menu_confirm_pressed(frame.input)
                {
                    self.page = Page::Main;
                }
                self.render_controls(frame.framebuffer, frame.input);
            }
        }

        GameResult::Continue
    }

    fn render_main_menu(&self, framebuffer: &mut Framebuffer) {
        framebuffer.clear(BG);
        draw_panel(
            framebuffer,
            Rect {
                x: 66,
                y: 24,
                width: 188,
                height: 132,
            },
            BG,
            BORDER,
        );
        draw_text_centered(
            framebuffer,
            Rect {
                x: 80,
                y: 38,
                width: 160,
                height: 24,
            },
            "PONG",
            2,
            ACCENT,
        );

        for (index, (label, y)) in [("PLAY", 82), ("CONTROLS", 104), ("QUIT", 126)]
            .into_iter()
            .enumerate()
        {
            draw_menu_item(
                framebuffer,
                Rect {
                    x: 94,
                    y,
                    width: 132,
                    height: 16,
                },
                label,
                self.main_menu.selected() == Some(index),
                1,
                FG,
                ACCENT,
            );
        }
    }

    fn render_controls(&self, framebuffer: &mut Framebuffer, input: &Input) {
        framebuffer.clear(BG);
        draw_panel(
            framebuffer,
            Rect {
                x: 36,
                y: 18,
                width: 248,
                height: 144,
            },
            BG,
            BORDER,
        );
        draw_text_centered(
            framebuffer,
            Rect {
                x: 48,
                y: 30,
                width: 224,
                height: 16,
            },
            "CONTROLS",
            1,
            ACCENT,
        );
        framebuffer.draw_text(58, 56, "P1  W/S", FG);
        framebuffer.draw_text(58, 70, "P2  UP/DOWN", FG);
        framebuffer.draw_text(
            58,
            92,
            if gamepad_connected(input, 0) {
                "P1 PAD CONNECTED"
            } else {
                "P1 PAD NONE"
            },
            FG,
        );
        framebuffer.draw_text(
            58,
            106,
            if gamepad_connected(input, 1) {
                "P2 PAD CONNECTED"
            } else {
                "P2 PAD NONE"
            },
            FG,
        );
        framebuffer.draw_text(58, 124, "PAUSE  ESC / START", FG);
        draw_text_centered(
            framebuffer,
            Rect {
                x: 48,
                y: 144,
                width: 224,
                height: 12,
            },
            "FIRE OR EAST TO BACK",
            1,
            FG,
        );
    }
}

impl Game for PongApp {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if let Some(game) = &mut self.game {
            game.update(frame)
        } else {
            self.update_menu(frame)
        }
    }
}

fn paused_pong() -> PauseGame<PongGame> {
    PauseGame::new(
        PongGame::new(),
        PauseConfig::new(Size {
            width: FRAMEBUFFER_WIDTH,
            height: FRAMEBUFFER_HEIGHT,
        }),
    )
}

fn gamepad_connected(input: &Input, player: usize) -> bool {
    input.gamepad_ids().nth(player).is_some()
}

fn main() -> Result<(), EngineError> {
    run(
        EngineConfig {
            title: "Pong - Gotoo Pixel Engine".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width: FRAMEBUFFER_WIDTH * 3,
            window_height: FRAMEBUFFER_HEIGHT * 3,
        },
        PongApp::new(),
    )
}
