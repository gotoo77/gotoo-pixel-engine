#[allow(dead_code)]
#[path = "../breakout.rs"]
mod breakout;
#[allow(dead_code)]
#[path = "../pong.rs"]
mod pong;
#[allow(dead_code)]
#[path = "../snake/game.rs"]
mod snake;
#[allow(dead_code)]
#[path = "../space_invaders/enhanced.rs"]
mod space_invaders;
#[allow(dead_code)]
#[path = "../tetris/game.rs"]
mod tetris;

use breakout::BreakoutApp;
use gotoo_pixel_engine::{
    ActionId, ControlMap, Frame, Framebuffer, Game, GameResult, GamepadButton, GamepadId,
    GamepadProfile, Key, Pixel, Rect, Size,
    ui::{
        MenuState, PauseConfig, PauseGame, VirtualButton, VirtualPad, draw_menu_item, draw_panel,
        draw_text_centered,
    },
};
use pong::PongApp;
use snake::{SnakeGame, SnakeInteractionMode};
use space_invaders::EnhancedSpaceInvadersGame;
use tetris::TetrisGame;

const CATALOG_UP: ActionId = ActionId::new("arcade.catalog.up");
const CATALOG_DOWN: ActionId = ActionId::new("arcade.catalog.down");
const CATALOG_SELECT: ActionId = ActionId::new("arcade.catalog.select");

const GAME_LABELS: [&str; 5] = ["SNAKE", "TETRIS", "SPACE INVADERS", "PONG", "BREAKOUT"];

const BG: Pixel = Pixel::rgb(7, 10, 14);
const PANEL: Pixel = Pixel::rgb(12, 18, 24);
const FG: Pixel = Pixel::rgb(224, 234, 220);
const BORDER: Pixel = Pixel::rgb(80, 150, 220);
const ACCENT: Pixel = Pixel::rgb(120, 235, 180);
const TOUCH_ACCENT: Pixel = Pixel::rgb(245, 190, 90);

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
const PAUSE_BUTTON: Rect = Rect {
    x: 400,
    y: 228,
    width: 72,
    height: 24,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ArcadeInteractionMode {
    Native,
    Touch,
}

impl ArcadeInteractionMode {
    pub const fn framebuffer_size(self) -> Size {
        match self {
            Self::Native => Size {
                width: 320,
                height: 224,
            },
            Self::Touch => Size {
                width: 480,
                height: 260,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ArcadeLayout {
    catalog_panel: Rect,
    title: Rect,
    game_list: Rect,
    item_step: i32,
    footer: Rect,
    touch_panel: Option<Rect>,
}

impl ArcadeLayout {
    const fn for_mode(mode: ArcadeInteractionMode) -> Self {
        match mode {
            ArcadeInteractionMode::Native => Self {
                catalog_panel: Rect {
                    x: 12,
                    y: 12,
                    width: 296,
                    height: 200,
                },
                title: Rect {
                    x: 24,
                    y: 24,
                    width: 272,
                    height: 28,
                },
                game_list: Rect {
                    x: 28,
                    y: 60,
                    width: 264,
                    height: 124,
                },
                item_step: 24,
                footer: Rect {
                    x: 28,
                    y: 194,
                    width: 264,
                    height: 10,
                },
                touch_panel: None,
            },
            ArcadeInteractionMode::Touch => Self {
                catalog_panel: Rect {
                    x: 18,
                    y: 16,
                    width: 444,
                    height: 228,
                },
                title: Rect {
                    x: 34,
                    y: 28,
                    width: 316,
                    height: 28,
                },
                game_list: Rect {
                    x: 34,
                    y: 66,
                    width: 316,
                    height: 152,
                },
                item_step: 29,
                footer: Rect {
                    x: 34,
                    y: 224,
                    width: 316,
                    height: 12,
                },
                touch_panel: Some(Rect {
                    x: 364,
                    y: 34,
                    width: 92,
                    height: 184,
                }),
            },
        }
    }
}

pub struct ArcadeApp {
    mode: ArcadeInteractionMode,
    layout: ArcadeLayout,
    catalog_menu: MenuState,
    catalog_controls: ControlMap,
    catalog_pad: Option<VirtualPad>,
    active_game: Option<Box<dyn Game>>,
    waiting_for_launch_release: bool,
    waiting_for_catalog_release: bool,
}

impl ArcadeApp {
    pub fn new(mode: ArcadeInteractionMode) -> Self {
        let touch = mode == ArcadeInteractionMode::Touch;
        Self {
            mode,
            layout: ArcadeLayout::for_mode(mode),
            catalog_menu: MenuState::new(GAME_LABELS.len()),
            catalog_controls: catalog_controls(),
            catalog_pad: touch.then(|| {
                VirtualPad::new([
                    VirtualButton::new(CATALOG_UP, TOUCH_UP),
                    VirtualButton::new(CATALOG_SELECT, TOUCH_SELECT),
                    VirtualButton::new(CATALOG_DOWN, TOUCH_DOWN),
                ])
            }),
            active_game: None,
            waiting_for_launch_release: false,
            waiting_for_catalog_release: false,
        }
    }

    fn update_catalog(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if let Some(catalog_pad) = &mut self.catalog_pad {
            catalog_pad.update(frame.input, &mut self.catalog_controls);
        }
        self.catalog_controls.update(frame.input);

        if self.waiting_for_catalog_release {
            if self.catalog_controls.action(CATALOG_SELECT).held() {
                self.render_catalog(frame.framebuffer);
                return GameResult::Continue;
            }
            self.waiting_for_catalog_release = false;
            if let Some(catalog_pad) = &mut self.catalog_pad {
                catalog_pad.reset(&mut self.catalog_controls);
            }
        }

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
            if let Some(catalog_pad) = &mut self.catalog_pad {
                catalog_pad.update(frame.input, &mut self.catalog_controls);
            }
            self.catalog_controls.update(frame.input);
            if self.catalog_controls.action(CATALOG_SELECT).held() {
                self.render_catalog(frame.framebuffer);
                return GameResult::Continue;
            }
            self.waiting_for_launch_release = false;
            if let Some(catalog_pad) = &mut self.catalog_pad {
                catalog_pad.reset(&mut self.catalog_controls);
            }
        }

        let result = self
            .active_game
            .as_mut()
            .expect("active game update requires a game")
            .update(frame);

        if result == GameResult::Exit {
            self.return_to_catalog();
            self.render_catalog(frame.framebuffer);
        }

        GameResult::Continue
    }

    fn launch(&mut self, index: usize) {
        let Some(game) = build_game(self.mode, index) else {
            return;
        };

        self.active_game = Some(game);
        self.waiting_for_launch_release = true;
    }

    fn return_to_catalog(&mut self) {
        if let Some(catalog_pad) = &mut self.catalog_pad {
            catalog_pad.reset(&mut self.catalog_controls);
        }
        self.active_game = None;
        self.waiting_for_launch_release = false;
        self.waiting_for_catalog_release = true;
    }

    fn render_catalog(&self, framebuffer: &mut Framebuffer) {
        framebuffer.clear(BG);
        draw_panel(framebuffer, self.layout.catalog_panel, PANEL, BORDER);
        draw_text_centered(framebuffer, self.layout.title, "GPE ARCADE", 2, ACCENT);

        for (index, label) in GAME_LABELS.iter().enumerate() {
            draw_menu_item(
                framebuffer,
                Rect {
                    x: self.layout.game_list.x,
                    y: self.layout.game_list.y + index as i32 * self.layout.item_step,
                    width: self.layout.game_list.width,
                    height: 20,
                },
                label,
                self.catalog_menu.selected() == Some(index),
                1,
                FG,
                ACCENT,
            );
        }

        if let Some(touch_panel) = self.layout.touch_panel {
            draw_panel(framebuffer, touch_panel, BG, BORDER);
            for (rect, label) in [
                (TOUCH_UP, "UP"),
                (TOUCH_SELECT, "PLAY"),
                (TOUCH_DOWN, "DOWN"),
            ] {
                framebuffer.draw_rect(rect.x, rect.y, rect.width, rect.height, TOUCH_ACCENT);
                draw_text_centered(framebuffer, rect, label, 1, TOUCH_ACCENT);
            }
        }

        draw_text_centered(
            framebuffer,
            self.layout.footer,
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

fn build_game(mode: ArcadeInteractionMode, index: usize) -> Option<Box<dyn Game>> {
    let game = match (mode, index) {
        (ArcadeInteractionMode::Native, 0) => {
            pause_game(SnakeGame::new(SnakeInteractionMode::Keyboard), mode)
        }
        (ArcadeInteractionMode::Touch, 0) => {
            pause_game(SnakeGame::new(SnakeInteractionMode::Touch), mode)
        }
        (ArcadeInteractionMode::Native, 1) => pause_game(TetrisGame::new(), mode),
        (ArcadeInteractionMode::Touch, 1) => pause_game(TetrisGame::new_touch(), mode),
        (ArcadeInteractionMode::Native, 2) => pause_game(EnhancedSpaceInvadersGame::new(), mode),
        (ArcadeInteractionMode::Touch, 2) => {
            pause_game(EnhancedSpaceInvadersGame::new_touch(), mode)
        }
        (ArcadeInteractionMode::Native, 3) => pause_game(PongApp::new(), mode),
        (ArcadeInteractionMode::Touch, 3) => pause_game(PongApp::new_touch(), mode),
        (ArcadeInteractionMode::Native, 4) => pause_game(BreakoutApp::new(), mode),
        (ArcadeInteractionMode::Touch, 4) => pause_game(BreakoutApp::new_touch(), mode),
        (_, _) => return None,
    };
    Some(game)
}

fn pause_game<G: Game + 'static>(game: G, mode: ArcadeInteractionMode) -> Box<dyn Game> {
    let mut config = PauseConfig::new(mode.framebuffer_size());
    if mode == ArcadeInteractionMode::Touch {
        config = config.with_touch_button(PAUSE_BUTTON);
    }
    Box::new(PauseGame::new(game, config))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outside_extent(rect: Rect, width: u32, height: u32) -> bool {
        rect.x >= width as i32 || rect.y >= height as i32
    }

    #[test]
    fn returning_to_catalog_arms_catalog_select_release_gate() {
        let mut app = ArcadeApp::new(ArcadeInteractionMode::Native);
        app.active_game = Some(Box::new(BreakoutApp::new()));

        app.return_to_catalog();

        assert!(app.active_game.is_none());
        assert!(app.waiting_for_catalog_release);
    }

    #[test]
    fn interaction_modes_use_smallest_current_common_surfaces() {
        assert_eq!(
            ArcadeInteractionMode::Native.framebuffer_size(),
            Size {
                width: 320,
                height: 224
            }
        );
        assert_eq!(
            ArcadeInteractionMode::Touch.framebuffer_size(),
            Size {
                width: 480,
                height: 260
            }
        );
    }

    #[test]
    fn touch_surface_contains_every_touch_game() {
        let size = ArcadeInteractionMode::Touch.framebuffer_size();
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
            assert!(width <= size.width);
            assert!(height <= size.height);
        }
    }

    #[test]
    fn native_surface_contains_every_native_game() {
        let size = ArcadeInteractionMode::Native.framebuffer_size();
        let snake_size = SnakeInteractionMode::Keyboard.framebuffer_size();
        let extents = [
            (snake_size.width, snake_size.height),
            (breakout::FRAMEBUFFER_WIDTH, breakout::FRAMEBUFFER_HEIGHT),
            (tetris::FRAMEBUFFER_WIDTH, tetris::FRAMEBUFFER_HEIGHT),
            (
                space_invaders::FRAMEBUFFER_WIDTH,
                space_invaders::FRAMEBUFFER_HEIGHT,
            ),
            (pong::FRAMEBUFFER_WIDTH, pong::FRAMEBUFFER_HEIGHT),
        ];

        for (width, height) in extents {
            assert!(width <= size.width);
            assert!(height <= size.height);
        }
    }

    #[test]
    fn pause_button_is_outside_every_touch_game_extent() {
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
            assert!(outside_extent(PAUSE_BUTTON, width, height));
        }
    }

    #[test]
    fn catalog_builds_all_five_games_in_both_modes() {
        for mode in [ArcadeInteractionMode::Native, ArcadeInteractionMode::Touch] {
            for index in 0..GAME_LABELS.len() {
                assert!(build_game(mode, index).is_some());
            }
            assert!(build_game(mode, GAME_LABELS.len()).is_none());
        }
    }

    #[test]
    fn native_mode_has_no_virtual_pads() {
        let arcade = ArcadeApp::new(ArcadeInteractionMode::Native);
        assert!(arcade.catalog_pad.is_none());
    }

    #[test]
    fn touch_mode_has_catalog_virtual_pad() {
        let arcade = ArcadeApp::new(ArcadeInteractionMode::Touch);
        assert!(arcade.catalog_pad.is_some());
    }

    #[test]
    fn launch_and_return_switch_between_catalog_and_game() {
        let mut arcade = ArcadeApp::new(ArcadeInteractionMode::Native);
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
