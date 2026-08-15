#[path = "pong/game.rs"]
mod game;

use game::{FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH, PongGame};
use gotoo_pixel_engine::{
    EngineConfig, EngineError, Frame, Framebuffer, Game, GameResult, GamepadButton, Key, Pixel,
    Rect, run,
    ui::{
        MenuState, draw_menu_item, draw_panel, draw_text_centered, menu_confirm_pressed,
        menu_down_pressed, menu_up_pressed,
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
    playing: bool,
    game: PongGame,
}

impl PongApp {
    fn new() -> Self {
        Self {
            page: Page::Main,
            main_menu: MenuState::new(3),
            playing: false,
            game: PongGame::new(),
        }
    }

    fn update_menu(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.game.sync_gamepads(frame.input);

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
                        Some(0) => {
                            self.game.reset_match();
                            self.playing = true;
                        }
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

    fn render_controls(&self, framebuffer: &mut Framebuffer) {
        framebuffer.clear(BG);
        draw_panel(
            framebuffer,
            Rect {
                x: 36,
                y: 22,
                width: 248,
                height: 136,
            },
            BG,
            BORDER,
        );
        draw_text_centered(
            framebuffer,
            Rect {
                x: 48,
                y: 34,
                width: 224,
                height: 16,
            },
            "CONTROLS",
            1,
            ACCENT,
        );
        framebuffer.draw_text(58, 64, "P1  W/S", FG);
        framebuffer.draw_text(58, 78, "P2  UP/DOWN", FG);
        framebuffer.draw_text(
            58,
            100,
            if self.game.gamepad_connected(0) {
                "P1 PAD CONNECTED"
            } else {
                "P1 PAD NONE"
            },
            FG,
        );
        framebuffer.draw_text(
            58,
            114,
            if self.game.gamepad_connected(1) {
                "P2 PAD CONNECTED"
            } else {
                "P2 PAD NONE"
            },
            FG,
        );
        draw_text_centered(
            framebuffer,
            Rect {
                x: 48,
                y: 136,
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
        if self.playing {
            self.game.update(frame)
        } else {
            self.update_menu(frame)
        }
    }
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
