#[allow(dead_code)]
#[path = "../breakout/game.rs"]
mod breakout;
#[allow(dead_code)]
#[path = "../pong/game.rs"]
mod pong;
#[allow(dead_code)]
#[path = "../smart_boy_hero/game.rs"]
mod smart_boy_hero;
#[allow(dead_code)]
#[path = "../snake/game.rs"]
mod snake;
#[allow(dead_code)]
#[path = "../space_invaders/enhanced.rs"]
mod space_invaders;
#[allow(dead_code)]
#[path = "../tetris/game.rs"]
mod tetris;

use breakout::BreakoutGame;
#[cfg(feature = "outline-fonts")]
use gotoo_pixel_engine::outline_text::OutlineFont;
use gotoo_pixel_engine::{
    ActionId, ControlBinding, ControlMap, Frame, Framebuffer, Game, GameResult, GamepadButton, Key,
    MouseButton, Pixel, Rect, Size,
    ui::{
        PauseConfig, PauseGame, UiComponentStyle, UiStyleOverride, UiStyleSheet, UiTheme,
        VirtualButton, VirtualPad, draw_panel, draw_text_centered,
        experimental::{self, UiId, UiNavInput, UiStateStore},
        experimental_spatial::{
            GridSpec, PointerInput, SpatialCard, SpatialInput, SpatialOutput, SpatialState,
            run_default_card_grid_styled,
        },
    },
};
use pong::PongGame;
use smart_boy_hero::SmartBoyHeroGame;
use snake::{SnakeGame, SnakeInteractionMode};
use space_invaders::EnhancedSpaceInvadersGame;
use tetris::TetrisGame;

const CATALOG_UP: ActionId = ActionId::new("arcade.catalog.up");
const CATALOG_DOWN: ActionId = ActionId::new("arcade.catalog.down");
const CATALOG_LEFT: ActionId = ActionId::new("arcade.catalog.left");
const CATALOG_RIGHT: ActionId = ActionId::new("arcade.catalog.right");
const CATALOG_SELECT: ActionId = ActionId::new("arcade.catalog.select");

const GAME_LABELS: [&str; 6] = [
    "SNAKE",
    "TETRIS",
    "SPACE INVADERS",
    "SMART BOY HERO",
    "PONG",
    "BREAKOUT",
];
const GAME_KEYS: [&str; 6] = [
    "snake",
    "tetris",
    "space-invaders",
    "smart-boy-hero",
    "pong",
    "breakout",
];

const BG: Pixel = Pixel::rgb(7, 10, 14);
const PANEL: Pixel = Pixel::rgb(12, 18, 24);
const CARD: Pixel = Pixel::rgb(15, 24, 31);
const CARD_FOCUSED: Pixel = Pixel::rgb(18, 39, 43);
const CARD_HOVERED: Pixel = Pixel::rgb(20, 31, 40);
const CARD_ACTIVE: Pixel = Pixel::rgb(27, 49, 47);
const FG: Pixel = Pixel::rgb(224, 234, 220);
const MUTED: Pixel = Pixel::rgb(135, 154, 164);
const BORDER: Pixel = Pixel::rgb(48, 77, 96);
const ACCENT: Pixel = Pixel::rgb(120, 235, 180);
const TOUCH_ACCENT: Pixel = Pixel::rgb(245, 190, 90);

const TOUCH_UP: Rect = Rect {
    x: 392,
    y: 44,
    width: 36,
    height: 30,
};
const TOUCH_LEFT: Rect = Rect {
    x: 370,
    y: 76,
    width: 36,
    height: 30,
};
const TOUCH_RIGHT: Rect = Rect {
    x: 414,
    y: 76,
    width: 36,
    height: 30,
};
const TOUCH_DOWN: Rect = Rect {
    x: 392,
    y: 108,
    width: 36,
    height: 30,
};
const TOUCH_SELECT: Rect = Rect {
    x: 376,
    y: 154,
    width: 68,
    height: 40,
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
    eyebrow: Rect,
    title: Rect,
    game_list: Rect,
    footer: Rect,
    touch_panel: Option<Rect>,
}

impl ArcadeLayout {
    const fn for_mode(mode: ArcadeInteractionMode) -> Self {
        match mode {
            ArcadeInteractionMode::Native => Self {
                catalog_panel: Rect {
                    x: 10,
                    y: 10,
                    width: 300,
                    height: 204,
                },
                eyebrow: Rect {
                    x: 24,
                    y: 20,
                    width: 272,
                    height: 10,
                },
                title: Rect {
                    x: 24,
                    y: 31,
                    width: 272,
                    height: 31,
                },
                game_list: Rect {
                    x: 22,
                    y: 70,
                    width: 276,
                    height: 112,
                },
                footer: Rect {
                    x: 24,
                    y: 194,
                    width: 272,
                    height: 10,
                },
                touch_panel: None,
            },
            ArcadeInteractionMode::Touch => Self {
                catalog_panel: Rect {
                    x: 12,
                    y: 12,
                    width: 456,
                    height: 236,
                },
                eyebrow: Rect {
                    x: 28,
                    y: 22,
                    width: 326,
                    height: 10,
                },
                title: Rect {
                    x: 28,
                    y: 33,
                    width: 326,
                    height: 31,
                },
                game_list: Rect {
                    x: 26,
                    y: 72,
                    width: 326,
                    height: 140,
                },
                footer: Rect {
                    x: 28,
                    y: 226,
                    width: 326,
                    height: 10,
                },
                touch_panel: Some(Rect {
                    x: 362,
                    y: 28,
                    width: 96,
                    height: 184,
                }),
            },
        }
    }
}

pub struct ArcadeApp {
    mode: ArcadeInteractionMode,
    layout: ArcadeLayout,
    catalog_ids: Vec<UiId>,
    catalog_state: SpatialState,
    catalog_controls: ControlMap,
    catalog_pad: Option<VirtualPad>,
    active_game: Option<Box<dyn Game>>,
    waiting_for_launch_release: bool,
    waiting_for_catalog_release: bool,
    #[cfg(feature = "outline-fonts")]
    outline_font: Option<OutlineFont>,
}

impl ArcadeApp {
    pub fn new(mode: ArcadeInteractionMode) -> Self {
        let touch = mode == ArcadeInteractionMode::Touch;
        Self {
            mode,
            layout: ArcadeLayout::for_mode(mode),
            catalog_ids: catalog_ids(),
            catalog_state: SpatialState::default(),
            catalog_controls: catalog_controls(),
            catalog_pad: touch.then(|| {
                VirtualPad::new([
                    VirtualButton::new(CATALOG_UP, TOUCH_UP),
                    VirtualButton::new(CATALOG_LEFT, TOUCH_LEFT),
                    VirtualButton::new(CATALOG_RIGHT, TOUCH_RIGHT),
                    VirtualButton::new(CATALOG_DOWN, TOUCH_DOWN),
                    VirtualButton::new(CATALOG_SELECT, TOUCH_SELECT),
                ])
            }),
            active_game: None,
            waiting_for_launch_release: false,
            waiting_for_catalog_release: false,
            #[cfg(feature = "outline-fonts")]
            outline_font: OutlineFont::from_bytes(include_bytes!(
                "../../assets/fonts/p4/unbounded/font.ttf"
            ))
            .ok(),
        }
    }

    fn update_catalog(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if let Some(catalog_pad) = &mut self.catalog_pad {
            catalog_pad.update(frame.input, &mut self.catalog_controls);
        }
        self.catalog_controls.update(frame.input);

        if self.waiting_for_catalog_release {
            if self.catalog_controls.action(CATALOG_SELECT).held() {
                self.render_catalog(frame.framebuffer, SpatialInput::default());
                return GameResult::Continue;
            }
            self.waiting_for_catalog_release = false;
            if let Some(catalog_pad) = &mut self.catalog_pad {
                catalog_pad.reset(&mut self.catalog_controls);
            }
        }

        let input = SpatialInput {
            nav: UiNavInput {
                up: self.catalog_controls.action(CATALOG_UP).pressed(),
                down: self.catalog_controls.action(CATALOG_DOWN).pressed(),
                left: self.catalog_controls.action(CATALOG_LEFT).pressed(),
                right: self.catalog_controls.action(CATALOG_RIGHT).pressed(),
                confirm: self.catalog_controls.action(CATALOG_SELECT).pressed(),
                ..UiNavInput::default()
            },
            pointer: PointerInput {
                position: frame.input.mouse_position(),
                pressed: frame.input.mouse_button(MouseButton::Left).pressed(),
                released: frame.input.mouse_button(MouseButton::Left).released(),
            },
            touches: frame.input.touches(),
        };

        if let Some(index) = self.render_catalog(frame.framebuffer, input) {
            self.launch(index);
            self.render_catalog(frame.framebuffer, SpatialInput::default());
        }
        GameResult::Continue
    }

    fn update_active_game(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if self.waiting_for_launch_release {
            if let Some(catalog_pad) = &mut self.catalog_pad {
                catalog_pad.update(frame.input, &mut self.catalog_controls);
            }
            self.catalog_controls.update(frame.input);
            if self.catalog_controls.action(CATALOG_SELECT).held() {
                self.render_catalog(frame.framebuffer, SpatialInput::default());
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
            self.render_catalog(frame.framebuffer, SpatialInput::default());
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

    fn render_catalog(
        &mut self,
        framebuffer: &mut Framebuffer,
        input: SpatialInput<'_>,
    ) -> Option<usize> {
        framebuffer.clear(BG);
        draw_panel(framebuffer, self.layout.catalog_panel, PANEL, BORDER);
        draw_text_centered(framebuffer, self.layout.eyebrow, "GPE.UI / P6.1", 1, MUTED);

        let cards = catalog_cards(&self.catalog_ids);
        let output = run_default_card_grid_styled(
            framebuffer,
            self.layout.game_list,
            &mut self.catalog_state,
            input,
            catalog_grid_spec(self.layout.game_list),
            catalog_theme(),
            catalog_stylesheet(),
            &cards,
        );

        self.paint_catalog_typography(framebuffer, &output);
        self.paint_touch_controls(framebuffer);
        draw_text_centered(
            framebuffer,
            self.layout.footer,
            "ARROWS/PAD | SPACE/SOUTH | MOUSE/TOUCH",
            1,
            MUTED,
        );

        cards.iter().position(|card| output.activated(card.id))
    }

    fn paint_catalog_typography(&mut self, framebuffer: &mut Framebuffer, output: &SpatialOutput) {
        #[cfg(feature = "outline-fonts")]
        if let Some(font) = &mut self.outline_font {
            let title_px = if self.mode == ArcadeInteractionMode::Touch {
                25.0
            } else {
                23.0
            };
            let _ = font.draw(
                framebuffer,
                "GPE ARCADE",
                title_px,
                self.layout.title,
                ACCENT,
            );
            let card_px = if output.columns() >= 3 { 10.5 } else { 12.5 };
            for (index, layout) in output.layouts().iter().enumerate() {
                let color = if output.focused_id() == Some(layout.id) {
                    ACCENT
                } else {
                    FG
                };
                let label_bounds = Rect {
                    x: layout.text_rect.x.saturating_add(4),
                    y: layout.text_rect.y,
                    width: layout.text_rect.width.saturating_sub(8),
                    height: layout.text_rect.height,
                };
                let _ = font.draw(
                    framebuffer,
                    GAME_LABELS[index],
                    card_px,
                    label_bounds,
                    color,
                );
            }
            return;
        }

        draw_text_centered(framebuffer, self.layout.title, "GPE ARCADE", 2, ACCENT);
        for (index, layout) in output.layouts().iter().enumerate() {
            let color = if output.focused_id() == Some(layout.id) {
                ACCENT
            } else {
                FG
            };
            draw_text_centered(framebuffer, layout.text_rect, GAME_LABELS[index], 1, color);
        }
    }

    fn paint_touch_controls(&self, framebuffer: &mut Framebuffer) {
        let Some(touch_panel) = self.layout.touch_panel else {
            return;
        };
        draw_panel(framebuffer, touch_panel, BG, BORDER);
        for (rect, label) in [
            (TOUCH_UP, "UP"),
            (TOUCH_LEFT, "<"),
            (TOUCH_RIGHT, ">"),
            (TOUCH_DOWN, "DN"),
            (TOUCH_SELECT, "PLAY"),
        ] {
            framebuffer.draw_rect(rect.x, rect.y, rect.width, rect.height, TOUCH_ACCENT);
            draw_text_centered(framebuffer, rect, label, 1, TOUCH_ACCENT);
        }
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
}

fn catalog_controls() -> ControlMap {
    let mut controls = ControlMap::new();
    bind_catalog_action(
        &mut controls,
        CATALOG_UP,
        &[
            ControlBinding::Key(Key::Up),
            ControlBinding::Key(Key::W),
            ControlBinding::Gamepad(GamepadButton::DPadUp),
            ControlBinding::Gamepad(GamepadButton::LeftStickUp),
        ],
    );
    bind_catalog_action(
        &mut controls,
        CATALOG_DOWN,
        &[
            ControlBinding::Key(Key::Down),
            ControlBinding::Key(Key::S),
            ControlBinding::Gamepad(GamepadButton::DPadDown),
            ControlBinding::Gamepad(GamepadButton::LeftStickDown),
        ],
    );
    bind_catalog_action(
        &mut controls,
        CATALOG_LEFT,
        &[
            ControlBinding::Key(Key::Left),
            ControlBinding::Gamepad(GamepadButton::DPadLeft),
            ControlBinding::Gamepad(GamepadButton::LeftStickLeft),
        ],
    );
    bind_catalog_action(
        &mut controls,
        CATALOG_RIGHT,
        &[
            ControlBinding::Key(Key::Right),
            ControlBinding::Gamepad(GamepadButton::DPadRight),
            ControlBinding::Gamepad(GamepadButton::LeftStickRight),
        ],
    );
    bind_catalog_action(
        &mut controls,
        CATALOG_SELECT,
        &[
            ControlBinding::Key(Key::Space),
            ControlBinding::Gamepad(GamepadButton::South),
        ],
    );
    controls
}

fn bind_catalog_action(controls: &mut ControlMap, action: ActionId, bindings: &[ControlBinding]) {
    for binding in bindings {
        controls.bind(action, *binding);
    }
}

fn catalog_ids() -> Vec<UiId> {
    let mut state = UiStateStore::default();
    let (_, ids) = experimental::run_headless(
        Size {
            width: 320,
            height: 224,
        },
        &mut state,
        UiNavInput::default(),
        UiTheme::default(),
        |ui| {
            GAME_LABELS
                .iter()
                .zip(GAME_KEYS.iter())
                .map(|(label, key)| ui.keyed(key, |ui| ui.button(*label).id()))
                .collect::<Vec<_>>()
        },
    );
    ids
}

fn catalog_cards(ids: &[UiId]) -> Vec<SpatialCard<'static>> {
    GAME_LABELS
        .iter()
        .zip(ids.iter().copied())
        .map(|(_, id)| SpatialCard {
            id,
            title: "",
            subtitle: "",
            image: None,
            action: CATALOG_SELECT,
        })
        .collect()
}

fn catalog_grid_spec(bounds: Rect) -> GridSpec {
    let wide = bounds.width >= 300;
    GridSpec {
        min_cell_width: if wide { 96 } else { 126 },
        preferred_cell_height: if wide { 62 } else { 50 },
        gap: 7,
        padding: 0,
    }
}

fn catalog_theme() -> UiTheme {
    UiTheme {
        padding: 6,
        row_height: 20,
        row_spacing: 6,
        text: FG,
        muted_text: MUTED,
        control_background: CARD,
        border: BORDER,
        accent: ACCENT,
        ..UiTheme::default()
    }
}

fn catalog_stylesheet() -> UiStyleSheet {
    UiStyleSheet {
        button: UiComponentStyle {
            base: UiStyleOverride {
                background: Some(CARD),
                border: Some(BORDER),
                border_width: Some(1),
                ..UiStyleOverride::default()
            },
            focused: UiStyleOverride {
                background: Some(CARD_FOCUSED),
                border: Some(ACCENT),
                border_width: Some(2),
                ..UiStyleOverride::default()
            },
            hovered: UiStyleOverride {
                background: Some(CARD_HOVERED),
                border: Some(Pixel::rgb(82, 126, 145)),
                ..UiStyleOverride::default()
            },
            active: UiStyleOverride {
                background: Some(CARD_ACTIVE),
                border: Some(ACCENT),
                ..UiStyleOverride::default()
            },
        },
        ..UiStyleSheet::default()
    }
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
        (ArcadeInteractionMode::Native, 3) => Box::new(SmartBoyHeroGame::new()),
        (ArcadeInteractionMode::Touch, 3) => Box::new(SmartBoyHeroGame::new_touch()),
        (ArcadeInteractionMode::Native, 4) => pause_game(PongGame::new(), mode),
        (ArcadeInteractionMode::Touch, 4) => pause_game(PongGame::new_touch(), mode),
        (ArcadeInteractionMode::Native, 5) => pause_game(BreakoutGame::new(), mode),
        (ArcadeInteractionMode::Touch, 5) => pause_game(BreakoutGame::new_touch(), mode),
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
    use gotoo_pixel_engine::{Touch, TouchPhase, ui::experimental_spatial::run_card_grid_headless};

    use super::*;

    fn outside_extent(rect: Rect, width: u32, height: u32) -> bool {
        rect.x >= width as i32 || rect.y >= height as i32
    }

    fn rect_center(rect: Rect) -> (i32, i32) {
        (
            rect.x + i32::try_from(rect.width / 2).unwrap_or(i32::MAX),
            rect.y + i32::try_from(rect.height / 2).unwrap_or(i32::MAX),
        )
    }

    #[test]
    fn returning_to_catalog_arms_catalog_select_release_gate() {
        let mut app = ArcadeApp::new(ArcadeInteractionMode::Native);
        app.active_game = Some(Box::new(BreakoutGame::new()));
        app.return_to_catalog();
        assert!(app.active_game.is_none());
        assert!(app.waiting_for_catalog_release);
    }

    #[test]
    fn catalog_ids_are_stable() {
        assert_eq!(catalog_ids(), catalog_ids());
    }

    #[test]
    fn responsive_catalog_uses_two_native_and_three_touch_columns() {
        let ids = catalog_ids();
        let cards = catalog_cards(&ids);
        for (mode, expected) in [
            (ArcadeInteractionMode::Native, 2),
            (ArcadeInteractionMode::Touch, 3),
        ] {
            let bounds = ArcadeLayout::for_mode(mode).game_list;
            let mut state = SpatialState::default();
            let output = run_card_grid_headless(
                bounds,
                &mut state,
                SpatialInput::default(),
                catalog_grid_spec(bounds),
                &cards,
            );
            assert_eq!(output.columns(), expected);
            assert_eq!(output.layouts().len(), GAME_LABELS.len());
        }
    }

    #[test]
    fn catalog_spatial_navigation_moves_right_and_down() {
        let ids = catalog_ids();
        let cards = catalog_cards(&ids);
        let bounds = ArcadeLayout::for_mode(ArcadeInteractionMode::Native).game_list;
        let spec = catalog_grid_spec(bounds);
        let mut state = SpatialState::default();
        let initial =
            run_card_grid_headless(bounds, &mut state, SpatialInput::default(), spec, &cards);
        assert_eq!(initial.focused_id(), Some(ids[0]));

        let right = run_card_grid_headless(
            bounds,
            &mut state,
            SpatialInput {
                nav: UiNavInput {
                    right: true,
                    ..UiNavInput::default()
                },
                ..SpatialInput::default()
            },
            spec,
            &cards,
        );
        assert_eq!(right.focused_id(), Some(ids[1]));

        let down = run_card_grid_headless(
            bounds,
            &mut state,
            SpatialInput {
                nav: UiNavInput {
                    down: true,
                    ..UiNavInput::default()
                },
                ..SpatialInput::default()
            },
            spec,
            &cards,
        );
        assert_eq!(down.focused_id(), Some(ids[3]));
    }

    #[test]
    fn catalog_pointer_click_activates_the_hit_card() {
        let ids = catalog_ids();
        let cards = catalog_cards(&ids);
        let bounds = ArcadeLayout::for_mode(ArcadeInteractionMode::Native).game_list;
        let spec = catalog_grid_spec(bounds);
        let mut state = SpatialState::default();
        let initial =
            run_card_grid_headless(bounds, &mut state, SpatialInput::default(), spec, &cards);
        let position = rect_center(initial.layouts()[2].rect);

        run_card_grid_headless(
            bounds,
            &mut state,
            SpatialInput {
                pointer: PointerInput {
                    position: Some(position),
                    pressed: true,
                    released: false,
                },
                ..SpatialInput::default()
            },
            spec,
            &cards,
        );
        let released = run_card_grid_headless(
            bounds,
            &mut state,
            SpatialInput {
                pointer: PointerInput {
                    position: Some(position),
                    pressed: false,
                    released: true,
                },
                ..SpatialInput::default()
            },
            spec,
            &cards,
        );
        assert!(released.activated(ids[2]));
    }

    #[test]
    fn catalog_touch_tap_activates_the_hit_card() {
        let ids = catalog_ids();
        let cards = catalog_cards(&ids);
        let bounds = ArcadeLayout::for_mode(ArcadeInteractionMode::Touch).game_list;
        let spec = catalog_grid_spec(bounds);
        let mut state = SpatialState::default();
        let initial =
            run_card_grid_headless(bounds, &mut state, SpatialInput::default(), spec, &cards);
        let position = rect_center(initial.layouts()[4].rect);
        let started = [Touch {
            id: 7,
            phase: TouchPhase::Started,
            position: Some(position),
        }];
        let ended = [Touch {
            id: 7,
            phase: TouchPhase::Ended,
            position: Some(position),
        }];

        run_card_grid_headless(
            bounds,
            &mut state,
            SpatialInput {
                touches: &started,
                ..SpatialInput::default()
            },
            spec,
            &cards,
        );
        let output = run_card_grid_headless(
            bounds,
            &mut state,
            SpatialInput {
                touches: &ended,
                ..SpatialInput::default()
            },
            spec,
            &cards,
        );
        assert!(output.activated(ids[4]));
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
            (
                smart_boy_hero::TOUCH_FRAMEBUFFER_WIDTH,
                smart_boy_hero::TOUCH_FRAMEBUFFER_HEIGHT,
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
            (
                smart_boy_hero::FRAMEBUFFER_WIDTH,
                smart_boy_hero::FRAMEBUFFER_HEIGHT,
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
            (
                smart_boy_hero::TOUCH_FRAMEBUFFER_WIDTH,
                smart_boy_hero::TOUCH_FRAMEBUFFER_HEIGHT,
            ),
            (pong::FRAMEBUFFER_WIDTH, pong::TOUCH_FRAMEBUFFER_HEIGHT),
        ];
        for (width, height) in extents {
            assert!(outside_extent(PAUSE_BUTTON, width, height));
        }
    }

    #[test]
    fn catalog_builds_all_games_in_both_modes() {
        for mode in [ArcadeInteractionMode::Native, ArcadeInteractionMode::Touch] {
            for index in 0..GAME_LABELS.len() {
                assert!(build_game(mode, index).is_some());
            }
            assert!(build_game(mode, GAME_LABELS.len()).is_none());
        }
    }

    #[test]
    fn native_mode_has_no_virtual_pads() {
        assert!(
            ArcadeApp::new(ArcadeInteractionMode::Native)
                .catalog_pad
                .is_none()
        );
    }

    #[test]
    fn touch_mode_has_catalog_virtual_pad() {
        assert!(
            ArcadeApp::new(ArcadeInteractionMode::Touch)
                .catalog_pad
                .is_some()
        );
    }

    #[test]
    fn launch_and_return_switch_between_catalog_and_game() {
        let mut arcade = ArcadeApp::new(ArcadeInteractionMode::Native);
        assert!(arcade.active_game.is_none());
        arcade.launch(0);
        assert!(arcade.active_game.is_some());
        assert!(arcade.waiting_for_launch_release);
        arcade.return_to_catalog();
        assert!(arcade.active_game.is_none());
        assert!(!arcade.waiting_for_launch_release);
    }
}
