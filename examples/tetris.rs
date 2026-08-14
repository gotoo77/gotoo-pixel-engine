#[path = "tetris/game.rs"]
mod game;

use game::{FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH, TetrisGame};
use gotoo_pixel_engine::{
    EngineConfig, EngineError, Frame, Game, GameResult, GamepadButton, Key, Pixel, Rect,
    ui::{MenuState, draw_menu_item, draw_panel, draw_text_centered},
    run,
};

const BACKGROUND: Pixel = Pixel::rgb(9, 12, 16);
const FOREGROUND: Pixel = Pixel::rgb(226, 234, 218);
const ACCENT: Pixel = Pixel::rgb(80, 220, 230);
const BORDER: Pixel = Pixel::rgb(112, 126, 138);

struct TetrisApp {
    game: TetrisGame,
    menu: MenuState,
    playing: bool,
}

impl TetrisApp {
    fn new() -> Self {
        Self {
            game: TetrisGame::new(),
            menu: MenuState::new(2),
            playing: false,
        }
    }

    fn update_menu(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if frame.input.key(Key::Escape).pressed() {
            return GameResult::Exit;
        }
        if menu_up_pressed(frame) {
            self.menu.select_previous();
        }
        if menu_down_pressed(frame) {
            self.menu.select_next();
        }
        if confirm_pressed(frame) {
            match self.menu.selected() {
                Some(0) => self.playing = true,
                Some(1) => return GameResult::Exit,
                _ => {}
            }
        }

        render_menu(frame, self.menu);
        GameResult::Continue
    }
}

impl Game for TetrisApp {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if self.playing {
            self.game.update(frame)
        } else {
            self.update_menu(frame)
        }
    }
}

fn menu_up_pressed(frame: &Frame<'_>) -> bool {
    frame.input.key(Key::Up).pressed()
        || frame.input.key(Key::W).pressed()
        || frame
            .input
            .gamepad_button_any(GamepadButton::DPadUp)
            .pressed()
        || frame
            .input
            .gamepad_button_any(GamepadButton::LeftStickUp)
            .pressed()
}

fn menu_down_pressed(frame: &Frame<'_>) -> bool {
    frame.input.key(Key::Down).pressed()
        || frame.input.key(Key::S).pressed()
        || frame
            .input
            .gamepad_button_any(GamepadButton::DPadDown)
            .pressed()
        || frame
            .input
            .gamepad_button_any(GamepadButton::LeftStickDown)
            .pressed()
}

fn confirm_pressed(frame: &Frame<'_>) -> bool {
    frame.input.key(Key::Space).pressed()
        || frame
            .input
            .gamepad_button_any(GamepadButton::South)
            .pressed()
}

fn render_menu(frame: &mut Frame<'_>, menu: MenuState) {
    let framebuffer = &mut frame.framebuffer;
    framebuffer.clear(BACKGROUND);

    draw_panel(
        framebuffer,
        Rect {
            x: 24,
            y: 38,
            width: 172,
            height: 142,
        },
        BACKGROUND,
        BORDER,
    );
    draw_text_centered(
        framebuffer,
        Rect {
            x: 36,
            y: 54,
            width: 148,
            height: 24,
        },
        "TETRIS",
        2,
        ACCENT,
    );

    for (index, (label, y)) in [("PLAY", 106), ("QUIT", 132)].into_iter().enumerate() {
        draw_menu_item(
            framebuffer,
            Rect {
                x: 44,
                y,
                width: 132,
                height: 16,
            },
            label,
            menu.selected() == Some(index),
            1,
            FOREGROUND,
            ACCENT,
        );
    }

    draw_text_centered(
        framebuffer,
        Rect {
            x: 8,
            y: 194,
            width: FRAMEBUFFER_WIDTH - 16,
            height: 12,
        },
        "UP/DOWN + DROP TO SELECT",
        1,
        FOREGROUND,
    );
}

fn main() -> Result<(), EngineError> {
    run(
        EngineConfig {
            title: "Tetris - Gotoo Pixel Engine".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            // Temporary WSLg workaround: this surface size is known to be stable.
            // Some larger/taller sizes currently stall during presentation.
            window_width: 960,
            window_height: 612,
        },
        TetrisApp::new(),
    )
}
