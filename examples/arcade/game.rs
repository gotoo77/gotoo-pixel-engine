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
use gotoo_pixel_engine::{
    ActionId, ControlMap, Frame, Framebuffer, Game, GameResult, MouseButton, Pixel, Rect, Size,
    TextRenderer,
    ui::{
        PauseConfig, PauseGame, UiTheme, VirtualButton, VirtualPad, draw_panel, draw_text_centered,
        experimental::{self, UiId, UiNavInput, UiStateStore},
        experimental_spatial::{
            CardLayout, CardPainter, CardVisualState, GridSpec, PointerInput, SpatialCard,
            SpatialInput, SpatialState, run_card_grid,
        },
        standard_menu_controls,
    },
};
use pong::PongGame;
use smart_boy_hero::SmartBoyHeroGame;
use snake::{SnakeGame, SnakeInteractionMode};
use space_invaders::EnhancedSpaceInvadersGame;
use tetris::TetrisGame;

const CATALOG_UP: ActionId = ActionId::new("arcade.catalog.up");
const CATALOG_DOWN: ActionId = ActionId::new("arcade.catalog.down");
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
const FG: Pixel = Pixel::rgb(224, 234, 220);
const BORDER: Pixel = Pixel::rgb(80, 150, 220);
const ACCENT: Pixel = Pixel::rgb(120, 235, 180);
const TOUCH_ACCENT: Pixel = Pixel::rgb(245, 190, 90);
const CATALOG_ROW_HOVER: Pixel = Pixel::rgb(15, 25, 32);
const CATALOG_ROW_FOCUSED: Pixel = Pixel::rgb(18, 34, 40);
const CATALOG_ROW_ACTIVE: Pixel = Pixel::rgb(24, 46, 48);
const CATALOG_SEPARATOR: Pixel = Pixel::rgb(32, 50, 64);

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

struct ArcadeCatalogPainter;

impl CardPainter for ArcadeCatalogPainter {
    fn paint(
        &self,
        framebuffer: &mut Framebuffer,
        card: &SpatialCard<'_>,
        layout: CardLayout,
        visual: CardVisualState,
        theme: UiTheme,
    ) {
        let background = if visual.active {
            CATALOG_ROW_ACTIVE
        } else if visual.focused {
            CATALOG_ROW_FOCUSED
        } else if visual.hovered {
            CATALOG_ROW_HOVER
        } else {
            PANEL
        };
        framebuffer.fill_rect(
            layout.rect.x,
            layout.rect.y,
            layout.rect.width,
            layout.rect.height,
            background,
        );

        let separator_y = layout.rect.y.saturating_add(
            i32::try_from(layout.rect.height.saturating_sub(1)).unwrap_or(i32::MAX),
        );
        framebuffer.fill_rect(
            layout.rect.x,
            separator_y,
            layout.rect.width,
            1,
            CATALOG_SEPARATOR,
        );

        if visual.focused {
            framebuffer.fill_rect(
                layout.rect.x,
                layout.rect.y,
                4_u32.min(layout.rect.width),
                layout.rect.height,
                theme.accent,
            );
        } else if visual.hovered {
            framebuffer.fill_rect(
                layout.rect.x,
                layout.rect.y,
                2_u32.min(layout.rect.width),
                layout.rect.height,
                theme.border,
            );
        }

        let text = TextRenderer::new(theme.font);
        let scale = theme.text_scale.max(1);
        let (_, text_height) = text.text_size(card.title, scale);
        let text_y = layout.rect.y.saturating_add(
            i32::try_from(layout.rect.height.saturating_sub(text_height) / 2).unwrap_or(i32::MAX),
        );
        let text_x = layout.rect.x.saturating_add(12);
        text.draw_scaled(framebuffer, text_x, text_y, card.title, scale, theme.text);

        if visual.focused {
            let marker = ">";
            let (marker_width, marker_height) = text.text_size(marker, scale);
            let marker_x = layout.rect.x.saturating_add(
                i32::try_from(
                    layout
                        .rect
                        .width
                        .saturating_sub(marker_width)
                        .saturating_sub(10),
                )
                .unwrap_or(i32::MAX),
            );
            let marker_y = layout.rect.y.saturating_add(
                i32::try_from(layout.rect.height.saturating_sub(marker_height) / 2)
                    .unwrap_or(i32::MAX),
            );
            text.draw_scaled(framebuffer, marker_x, marker_y, marker, scale, theme.accent);
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
        draw_text_centered(framebuffer, self.layout.title, "GPE ARCADE", 2, ACCENT);

        let cards = catalog_cards(&self.catalog_ids);
        let output = run_card_grid(
            framebuffer,
            self.layout.game_list,
            &mut self.catalog_state,
            input,
            catalog_grid_spec(self.layout.game_list),
            catalog_theme(),
            &cards,
            &ArcadeCatalogPainter,
        );
        let activated = cards.iter().position(|card| output.activated(card.id));

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
        activated
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
    standard_menu_controls(CATALOG_UP, CATALOG_DOWN, CATALOG_SELECT)
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
        .map(|(label, id)| SpatialCard {
            id,
            title: label,
            subtitle: "",
            image: None,
            action: CATALOG_SELECT,
        })
        .collect()
}

fn catalog_grid_spec(bounds: Rect) -> GridSpec {
    if bounds.height >= 140 {
        GridSpec {
            min_cell_width: bounds.width,
            preferred_cell_height: 22,
            gap: 4,
            padding: 0,
        }
    } else {
        GridSpec {
            min_cell_width: bounds.width,
            preferred_cell_height: 19,
            gap: 2,
            padding: 0,
        }
    }
}

fn catalog_theme() -> UiTheme {
    UiTheme {
        padding: 4,
        row_height: 20,
        row_spacing: 2,
        text: FG,
        muted_text: FG,
        control_background: PANEL,
        border: BORDER,
        accent: ACCENT,
        ..UiTheme::default()
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
    fn catalog_headless_initial_focus_and_down_navigation_are_linear() {
        let ids = catalog_ids();
        let cards = catalog_cards(&ids);
        let bounds = ArcadeLayout::for_mode(ArcadeInteractionMode::Native).game_list;
        let spec = catalog_grid_spec(bounds);
        let mut state = SpatialState::default();

        let initial =
            run_card_grid_headless(bounds, &mut state, SpatialInput::default(), spec, &cards);
        assert_eq!(initial.focused_id(), Some(ids[0]));

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
        assert_eq!(down.focused_id(), Some(ids[1]));
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
    fn catalog_layouts_stay_inside_the_catalog_bounds() {
        for mode in [ArcadeInteractionMode::Native, ArcadeInteractionMode::Touch] {
            let ids = catalog_ids();
            let cards = catalog_cards(&ids);
            let bounds = ArcadeLayout::for_mode(mode).game_list;
            let mut state = SpatialState::default();
            let output = run_card_grid_headless(
                bounds,
                &mut state,
                SpatialInput::default(),
                catalog_grid_spec(bounds),
                &cards,
            );

            assert_eq!(output.layouts().len(), GAME_LABELS.len());
            for layout in output.layouts() {
                assert!(layout.rect.x >= bounds.x);
                assert!(layout.rect.y >= bounds.y);
                assert!(
                    i64::from(layout.rect.x) + i64::from(layout.rect.width)
                        <= i64::from(bounds.x) + i64::from(bounds.width)
                );
                assert!(
                    i64::from(layout.rect.y) + i64::from(layout.rect.height)
                        <= i64::from(bounds.y) + i64::from(bounds.height)
                );
            }
        }
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
