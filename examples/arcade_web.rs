#[allow(dead_code)]
#[path = "breakout.rs"]
mod breakout;
#[allow(dead_code)]
#[path = "pong.rs"]
mod pong;
#[allow(dead_code)]
#[path = "snake/game.rs"]
mod snake;
#[allow(dead_code)]
#[path = "space_invaders/enhanced.rs"]
mod space_invaders;
#[allow(dead_code)]
#[path = "tetris/game.rs"]
mod tetris;

use breakout::BreakoutApp;
use gotoo_pixel_engine::{
    ActionId, ControlMap, EngineConfig, Frame, Game, GameResult, GamepadButton, GamepadId,
    GamepadProfile, Key, Pixel, Rect, run,
    ui::{MenuState, VirtualButton, VirtualPad, draw_menu_item, draw_panel, draw_text_centered},
};
use pong::PongApp;
use snake::{SnakeGame, SnakeInteractionMode};
use space_invaders::EnhancedSpaceInvadersGame;
use tetris::TetrisGame;
use wasm_bindgen::prelude::*;

const FRAMEBUFFER_WIDTH: u32 = 480;
const FRAMEBUFFER_HEIGHT: u32 = 260;

const CATALOG_UP: ActionId = ActionId::new("arcade.catalog.up");
const CATALOG_DOWN: ActionId = ActionId::new("arcade.catalog.down");
const CATALOG_SELECT: ActionId = ActionId::new("arcade.catalog.select");
const RETURN_TO_CATALOG: ActionId = ActionId::new("arcade.return_to_catalog");

const GAME_LABELS: [&str; 5] = ["SNAKE", "TETRIS", "SPACE INVADERS", "PONG", "BREAKOUT"];

const BG: Pixel = Pixel::rgb(7, 10, 14);
const PANEL: Pixel = Pixel::rgb(12, 18, 24);
const FG: Pixel = Pixel::rgb(224, 234, 220);
const BORDER: Pixel = Pixel::rgb(80, 150, 220);
const ACCENT: Pixel = Pixel::rgb(120, 235, 180);
const TOUCH_ACCENT: Pixel = Pixel::rgb(245, 190, 90);

const CATALOG_PANEL: Rect = Rect {
    x: 18,
    y: 16,
    width: 444,
    height: 228,
};
const GAME_LIST: Rect = Rect {
    x: 34,
    y: 66,
    width: 316,
    height: 152,
};
const TOUCH_PANEL: Rect = Rect {
    x: 364,
    y: 34,
    width: 92,
    height: 184,
};
const TOUCH_UP: Rect = Rect {
    x: 378,
    y: 48,
    width: 64,
    height: 42,
};
const TOUCH_SELECT: Rect = Rect {
    x: 378,
    y: 106,
    width: 64,
    height: 42,
};
const TOUCH_DOWN: Rect = Rect {
    x: 378,
    y: 164,
    width: 64,
    height: 42,
};

// This rectangle lies outside every current game's touch framebuffer extent
// when they are hosted inside the common 480x260 Arcade surface.
const RETURN_BUTTON: Rect = Rect {
    x: 400,
    y: 228,
    width: 72,
    height: 24,
};

struct ArcadeApp {
    catalog_menu: MenuState,
    catalog_controls: ControlMap,
    catalog_pad: VirtualPad,
    return_controls: ControlMap,
    return_pad: VirtualPad,
    active_game: Option<Box<dyn Game>>,
    waiting_for_launch_release: bool,
}

impl Default for ArcadeApp {
    fn default() -> Self {
        Self::new()
    }
}

impl ArcadeApp {
    fn new() -> Self {
        Self {
            catalog_menu: MenuState::new(GAME_LABELS.len()),
            catalog_controls: catalog_controls(),
            catalog_pad: VirtualPad::new([
                VirtualButton::new(CATALOG_UP, TOUCH_UP),
                VirtualButton::new(CATALOG_SELECT, TOUCH_SELECT),
                VirtualButton::new(CATALOG_DOWN, TOUCH_DOWN),
            ]),
            return_controls: return_controls(),
            return_pad: VirtualPad::new([VirtualButton::new(RETURN_TO_CATALOG, RETURN_BUTTON)]),
            active_game: None,
            waiting_for_launch_release: false,
        }
    }

    fn update_catalog(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.catalog_pad
            .update(frame.input, &mut self.catalog_controls);
        self.catalog_controls.update(frame.input);

        if self.catalog_controls.action(CATALOG_UP).pressed() {
            self.catalog_menu.select_previous();
        }
        if self.catalog_controls.action(CATALOG_DOWN).pressed() {
            self.catalog_menu.select_next();
        }
        if self.catalog_controls.action(CATALOG_SELECT).pressed()
            && let Some(index) = self.catalog_menu.selected()
        {
            self.launch(index);
            self.render_catalog(frame.framebuffer);
            return GameResult::Continue;
        }

        self.render_catalog(frame.framebuffer);
        GameResult::Continue
    }

    fn update_active_game(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if self.waiting_for_launch_release {
            self.catalog_pad
                .update(frame.input, &mut self.catalog_controls);
            self.catalog_controls.update(frame.input);
            if self.catalog_controls.action(CATALOG_SELECT).held() {
                self.render_catalog(frame.framebuffer);
                return GameResult::Continue;
            }
            self.waiting_for_launch_release = false;
            self.catalog_pad.reset(&mut self.catalog_controls);
        }

        self.return_pad
            .update(frame.input, &mut self.return_controls);
        self.return_controls.update(frame.input);

        if self.return_controls.action(RETURN_TO_CATALOG).pressed() {
            self.return_to_catalog();
            self.render_catalog(frame.framebuffer);
            return GameResult::Continue;
        }

        let result = self
            .active_game
            .as_mut()
            .expect("active game update requires a game")
            .update(frame);

        if result == GameResult::Exit {
            self.return_to_catalog();
            self.render_catalog(frame.framebuffer);
        } else {
            draw_return_button(frame.framebuffer);
        }

        GameResult::Continue
    }

    fn launch(&mut self, index: usize) {
        let Some(game) = build_game(index) else {
            return;
        };

        self.return_pad.reset(&mut self.return_controls);
        self.active_game = Some(game);
        self.waiting_for_launch_release = true;
    }

    fn return_to_catalog(&mut self) {
        self.return_pad.reset(&mut self.return_controls);
        self.catalog_pad.reset(&mut self.catalog_controls);
        self.active_game = None;
        self.waiting_for_launch_release = false;
    }

    fn render_catalog(&self, framebuffer: &mut gotoo_pixel_engine::Framebuffer) {
        framebuffer.clear(BG);
        draw_panel(framebuffer, CATALOG_PANEL, PANEL, BORDER);
        draw_text_centered(
            framebuffer,
            Rect {
                x: 34,
                y: 28,
                width: 316,
                height: 28,
            },
            "GPE ARCADE",
            2,
            ACCENT,
        );

        for (index, label) in GAME_LABELS.iter().enumerate() {
            draw_menu_item(
                framebuffer,
                Rect {
                    x: GAME_LIST.x,
                    y: GAME_LIST.y + index as i32 * 29,
                    width: GAME_LIST.width,
                    height: 20,
                },
                label,
                self.catalog_menu.selected() == Some(index),
                1,
                FG,
                ACCENT,
            );
        }

        draw_panel(framebuffer, TOUCH_PANEL, BG, BORDER);
        for (rect, label) in [
            (TOUCH_UP, "UP"),
            (TOUCH_SELECT, "PLAY"),
            (TOUCH_DOWN, "DOWN"),
        ] {
            framebuffer.draw_rect(rect.x, rect.y, rect.width, rect.height, TOUCH_ACCENT);
            draw_text_centered(framebuffer, rect, label, 1, TOUCH_ACCENT);
        }

        draw_text_centered(
            framebuffer,
            Rect {
                x: 34,
                y: 224,
                width: 316,
                height: 12,
            },
            "ARROWS/PAD + SPACE/SOUTH",
            1,
            FG,
        );
    }
}

impl Game for ArcadeApp {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if self.active_game.is_some() {
            self.update_active_game(frame)
        } else {
            self.update_catalog(frame)
        }
    }

    fn gamepad_profile(&self, id: GamepadId) -> Option<GamepadProfile> {
        self.active_game
            .as_ref()
            .and_then(|game| game.gamepad_profile(id))
    }
}

fn catalog_controls() -> ControlMap {
    let mut controls = ControlMap::new();
    controls
        .bind_key(CATALOG_UP, Key::Up)
        .bind_key(CATALOG_UP, Key::W)
        .bind_gamepad(CATALOG_UP, GamepadButton::DPadUp)
        .bind_gamepad(CATALOG_UP, GamepadButton::LeftStickUp)
        .bind_key(CATALOG_DOWN, Key::Down)
        .bind_key(CATALOG_DOWN, Key::S)
        .bind_gamepad(CATALOG_DOWN, GamepadButton::DPadDown)
        .bind_gamepad(CATALOG_DOWN, GamepadButton::LeftStickDown)
        .bind_key(CATALOG_SELECT, Key::Space)
        .bind_gamepad(CATALOG_SELECT, GamepadButton::South);
    controls
}

fn return_controls() -> ControlMap {
    let mut controls = ControlMap::new();
    controls
        .bind_key(RETURN_TO_CATALOG, Key::Escape)
        .bind_gamepad(RETURN_TO_CATALOG, GamepadButton::Start);
    controls
}

fn build_game(index: usize) -> Option<Box<dyn Game>> {
    match index {
        0 => Some(Box::new(SnakeGame::new(SnakeInteractionMode::Touch))),
        1 => Some(Box::new(TetrisGame::new_touch())),
        2 => Some(Box::new(EnhancedSpaceInvadersGame::new_touch())),
        3 => Some(Box::new(PongApp::new_touch())),
        4 => Some(Box::new(BreakoutApp::new_touch())),
        _ => None,
    }
}

fn draw_return_button(framebuffer: &mut gotoo_pixel_engine::Framebuffer) {
    draw_panel(framebuffer, RETURN_BUTTON, BG, TOUCH_ACCENT);
    draw_text_centered(framebuffer, RETURN_BUTTON, "MENU", 1, TOUCH_ACCENT);
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    run(
        EngineConfig {
            title: "GPE Arcade".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width: FRAMEBUFFER_WIDTH * 2,
            window_height: FRAMEBUFFER_HEIGHT * 2,
        },
        ArcadeApp::new(),
    )
    .map_err(|err| JsValue::from_str(&err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outside_extent(rect: Rect, width: u32, height: u32) -> bool {
        rect.x >= width as i32 || rect.y >= height as i32
    }

    #[test]
    fn common_surface_contains_every_touch_game() {
        let snake_size = SnakeInteractionMode::Touch.framebuffer_size();
        let extents = [
            (snake_size.width, snake_size.height),
            (
                breakout::TOUCH_FRAMEBUFFER_WIDTH,
                breakout::FRAMEBUFFER_HEIGHT,
            ),
            (tetris::TOUCH_FRAMEBUFFER_WIDTH, tetris::FRAMEBUFFER_HEIGHT),
            (
                space_invaders::TOUCH_FRAMEBUFFER_WIDTH,
                space_invaders::FRAMEBUFFER_HEIGHT,
            ),
            (pong::FRAMEBUFFER_WIDTH, pong::TOUCH_FRAMEBUFFER_HEIGHT),
        ];

        for (width, height) in extents {
            assert!(width <= FRAMEBUFFER_WIDTH);
            assert!(height <= FRAMEBUFFER_HEIGHT);
        }
    }

    #[test]
    fn return_button_is_outside_every_game_extent() {
        let snake_size = SnakeInteractionMode::Touch.framebuffer_size();
        let extents = [
            (snake_size.width, snake_size.height),
            (
                breakout::TOUCH_FRAMEBUFFER_WIDTH,
                breakout::FRAMEBUFFER_HEIGHT,
            ),
            (tetris::TOUCH_FRAMEBUFFER_WIDTH, tetris::FRAMEBUFFER_HEIGHT),
            (
                space_invaders::TOUCH_FRAMEBUFFER_WIDTH,
                space_invaders::FRAMEBUFFER_HEIGHT,
            ),
            (pong::FRAMEBUFFER_WIDTH, pong::TOUCH_FRAMEBUFFER_HEIGHT),
        ];

        for (width, height) in extents {
            assert!(outside_extent(RETURN_BUTTON, width, height));
        }
    }

    #[test]
    fn catalog_builds_all_five_games() {
        for index in 0..GAME_LABELS.len() {
            assert!(build_game(index).is_some());
        }
        assert!(build_game(GAME_LABELS.len()).is_none());
    }

    #[test]
    fn launch_and_return_switch_between_catalog_and_game() {
        let mut arcade = ArcadeApp::new();
        assert!(arcade.active_game.is_none());
        assert!(!arcade.waiting_for_launch_release);

        arcade.launch(0);
        assert!(arcade.active_game.is_some());
        assert!(arcade.waiting_for_launch_release);

        arcade.return_to_catalog();
        assert!(arcade.active_game.is_none());
        assert!(!arcade.waiting_for_launch_release);
    }
}
