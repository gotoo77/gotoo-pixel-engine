#[allow(dead_code)]
#[path = "breakout/game.rs"]
mod game;

use game::{BreakoutGame, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};
use gotoo_pixel_engine::{
    EngineConfig, EngineError, Frame, Framebuffer, Game, GameResult, GamepadButton, Key, Pixel,
    Rect, Size, run,
    ui::{
        MenuState, PauseConfig, PauseGame, draw_menu_item, draw_panel, draw_text_centered,
        menu_confirm_pressed, menu_down_pressed, menu_up_pressed,
    },
};

const BG: Pixel = Pixel::rgb(7, 10, 14);
const FG: Pixel = Pixel::rgb(230, 238, 228);
const ACCENT: Pixel = Pixel::rgb(120, 235, 180);
const BORDER: Pixel = Pixel::rgb(80, 150, 220);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Main,
    Controls,
}

struct BreakoutApp {
    page: Page,
    menu: MenuState,
    game: Option<PauseGame<BreakoutGame>>,
}

impl BreakoutApp {
    fn new() -> Self {
        Self {
            page: Page::Main,
            menu: MenuState::new(3),
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
                    self.menu.select_previous();
                }
                if menu_down_pressed(frame.input) {
                    self.menu.select_next();
                }
                if menu_confirm_pressed(frame.input) {
                    match self.menu.selected() {
                        Some(0) => self.game = Some(paused_breakout()),
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
                self.render_controls(frame.framebuffer);
            }
        }

        GameResult::Continue
    }

    fn render_main_menu(&self, framebuffer: &mut Framebuffer) {
        framebuffer.clear(BG);
        draw_panel(
            framebuffer,
            Rect {
                x: 62,
                y: 24,
                width: 196,
                height: 132,
            },
            BG,
            BORDER,
        );
        draw_text_centered(
            framebuffer,
            Rect {
                x: 76,
                y: 38,
                width: 168,
                height: 24,
            },
            "BREAKOUT",
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
                self.menu.selected() == Some(index),
                1,
                FG,
                ACCENT,
            );
        }
    }

    fn render_controls(&self, framebuffer: &mut Framebuffer) {
        framebuffer.clear(BG);
        draw_panel(
            framebuffer,
            Rect {
                x: 34,
                y: 20,
                width: 252,
                height: 140,
            },
            BG,
            BORDER,
        );
        draw_text_centered(
            framebuffer,
            Rect {
                x: 48,
                y: 32,
                width: 224,
                height: 16,
            },
            "CONTROLS",
            1,
            ACCENT,
        );
        framebuffer.draw_text(58, 60, "KEYBOARD  LEFT/RIGHT OR A/D", FG);
        framebuffer.draw_text(58, 78, "GAMEPAD   DPAD / LEFT STICK", FG);
        framebuffer.draw_text(58, 96, "ACTION    SPACE / SOUTH", FG);
        framebuffer.draw_text(58, 114, "PAUSE     ESC / START", FG);
        draw_text_centered(
            framebuffer,
            Rect {
                x: 48,
                y: 140,
                width: 224,
                height: 12,
            },
            "FIRE OR EAST TO BACK",
            1,
            FG,
        );
    }
}

impl Game for BreakoutApp {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if let Some(game) = &mut self.game {
            game.update(frame)
        } else {
            self.update_menu(frame)
        }
    }
}

fn paused_breakout() -> PauseGame<BreakoutGame> {
    PauseGame::new(
        BreakoutGame::new(),
        PauseConfig::new(Size {
            width: FRAMEBUFFER_WIDTH,
            height: FRAMEBUFFER_HEIGHT,
        }),
    )
}

fn main() -> Result<(), EngineError> {
    run(
        EngineConfig {
            title: "Breakout - Gotoo Pixel Engine".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width: FRAMEBUFFER_WIDTH * 3,
            window_height: FRAMEBUFFER_HEIGHT * 3,
        },
        BreakoutApp::new(),
    )
}
