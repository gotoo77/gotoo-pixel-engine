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
    MouseButton, Pixel, Rect, Size, TextInputEvent, TouchPhase,
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
const CATALOG_FILTER_PREV: ActionId = ActionId::new("arcade.catalog.filter-prev");
const CATALOG_FILTER_NEXT: ActionId = ActionId::new("arcade.catalog.filter-next");

const FILTER_ARCADE: u8 = 1;
const FILTER_PUZZLE: u8 = 1 << 1;
const FILTER_ACTION: u8 = 1 << 2;
const FILTER_SHMUP: u8 = 1 << 3;

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
const GAME_TAGS: [&str; 6] = [
    "ARCADE / ACTION",
    "PUZZLE",
    "ARCADE / SHMUP",
    "PUZZLE / ACTION",
    "ARCADE",
    "ARCADE / ACTION",
];
const GAME_MASKS: [u8; 6] = [
    FILTER_ARCADE | FILTER_ACTION,
    FILTER_PUZZLE,
    FILTER_ARCADE | FILTER_SHMUP,
    FILTER_PUZZLE | FILTER_ACTION,
    FILTER_ARCADE,
    FILTER_ARCADE | FILTER_ACTION,
];
const GAME_ACCENTS: [Pixel; 6] = [
    Pixel::rgb(105, 230, 155),
    Pixel::rgb(157, 132, 255),
    Pixel::rgb(255, 137, 95),
    Pixel::rgb(255, 205, 92),
    Pixel::rgb(91, 189, 255),
    Pixel::rgb(240, 103, 184),
];

const BG: Pixel = Pixel::rgb(5, 8, 13);
const PANEL: Pixel = Pixel::rgb(10, 16, 24);
const HEADER: Pixel = Pixel::rgb(13, 22, 33);
const CARD: Pixel = Pixel::rgb(16, 25, 37);
const CARD_FOCUSED: Pixel = Pixel::rgb(22, 42, 50);
const CARD_HOVERED: Pixel = Pixel::rgb(20, 33, 47);
const CARD_ACTIVE: Pixel = Pixel::rgb(29, 52, 55);
const FG: Pixel = Pixel::rgb(230, 238, 235);
const MUTED: Pixel = Pixel::rgb(132, 151, 164);
const BORDER: Pixel = Pixel::rgb(48, 76, 98);
const ACCENT: Pixel = Pixel::rgb(111, 238, 184);
const SEARCH_BG: Pixel = Pixel::rgb(7, 12, 19);
const CHIP_BG: Pixel = Pixel::rgb(18, 29, 42);
const TOUCH_ACCENT: Pixel = Pixel::rgb(250, 190, 85);

const TOUCH_UP: Rect = Rect {
    x: 608,
    y: 92,
    width: 42,
    height: 36,
};
const TOUCH_LEFT: Rect = Rect {
    x: 582,
    y: 134,
    width: 42,
    height: 36,
};
const TOUCH_RIGHT: Rect = Rect {
    x: 634,
    y: 134,
    width: 42,
    height: 36,
};
const TOUCH_DOWN: Rect = Rect {
    x: 608,
    y: 176,
    width: 42,
    height: 36,
};
const TOUCH_SELECT: Rect = Rect {
    x: 588,
    y: 232,
    width: 82,
    height: 44,
};
const PAUSE_BUTTON: Rect = Rect {
    x: 624,
    y: 358,
    width: 78,
    height: 26,
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
                width: 560,
                height: 320,
            },
            Self::Touch => Size {
                width: 720,
                height: 400,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CatalogFilter {
    All,
    Arcade,
    Puzzle,
    Action,
    Shmup,
}

impl CatalogFilter {
    const ALL: [Self; 5] = [
        Self::All,
        Self::Arcade,
        Self::Puzzle,
        Self::Action,
        Self::Shmup,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::All => "ALL",
            Self::Arcade => "ARCADE",
            Self::Puzzle => "PUZZLE",
            Self::Action => "ACTION",
            Self::Shmup => "SHMUP",
        }
    }

    const fn mask(self) -> u8 {
        match self {
            Self::All => 0,
            Self::Arcade => FILTER_ARCADE,
            Self::Puzzle => FILTER_PUZZLE,
            Self::Action => FILTER_ACTION,
            Self::Shmup => FILTER_SHMUP,
        }
    }

    const fn color(self) -> Pixel {
        match self {
            Self::All => ACCENT,
            Self::Arcade => Pixel::rgb(91, 189, 255),
            Self::Puzzle => Pixel::rgb(157, 132, 255),
            Self::Action => Pixel::rgb(240, 103, 184),
            Self::Shmup => Pixel::rgb(255, 137, 95),
        }
    }
}

#[derive(Debug, Clone)]
struct CatalogSearch {
    query: String,
    active: bool,
    cursor: usize,
    select_all: bool,
}

impl Default for CatalogSearch {
    fn default() -> Self {
        Self {
            query: String::new(),
            active: false,
            cursor: 0,
            select_all: false,
        }
    }
}

impl CatalogSearch {
    fn clear(&mut self) {
        self.query.clear();
        self.cursor = 0;
        self.select_all = false;
    }

    fn select_all(&mut self) {
        self.select_all = true;
    }

    fn display(&self) -> String {
        if self.query.is_empty() {
            return if self.active {
                "|  SEARCH GAMES".to_owned()
            } else {
                "SEARCH GAMES   CTRL+F".to_owned()
            };
        }
        let before = &self.query[..self.cursor];
        let after = &self.query[self.cursor..];
        if self.active {
            format!("{before}|{after}")
        } else {
            self.query.clone()
        }
    }

    fn edit(&mut self, events: &[TextInputEvent]) {
        for event in events {
            match event {
                TextInputEvent::Insert(text) => {
                    if self.select_all {
                        self.clear();
                    }
                    let text: String = text
                        .chars()
                        .filter(|character| !character.is_control())
                        .take(48_usize.saturating_sub(self.query.chars().count()))
                        .collect();
                    self.query.insert_str(self.cursor, &text);
                    self.cursor += text.len();
                }
                TextInputEvent::Backspace | TextInputEvent::Delete if self.select_all => {
                    self.clear();
                }
                TextInputEvent::Backspace if self.cursor > 0 => {
                    let previous = self.query[..self.cursor].char_indices().last().unwrap().0;
                    self.query.drain(previous..self.cursor);
                    self.cursor = previous;
                }
                TextInputEvent::Delete if self.cursor < self.query.len() => {
                    self.query.remove(self.cursor);
                }
                TextInputEvent::Left if self.cursor > 0 => {
                    self.cursor = self.query[..self.cursor].char_indices().last().unwrap().0;
                }
                TextInputEvent::Right if self.cursor < self.query.len() => {
                    self.cursor += self.query[self.cursor..].chars().next().unwrap().len_utf8();
                }
                TextInputEvent::Home => self.cursor = 0,
                TextInputEvent::End => self.cursor = self.query.len(),
                _ => {}
            }
            self.select_all = false;
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ArcadeLayout {
    catalog_panel: Rect,
    header: Rect,
    eyebrow: Rect,
    title: Rect,
    status: Rect,
    search: Rect,
    filters: Rect,
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
                    width: 536,
                    height: 296,
                },
                header: Rect {
                    x: 13,
                    y: 13,
                    width: 534,
                    height: 72,
                },
                eyebrow: Rect {
                    x: 28,
                    y: 24,
                    width: 260,
                    height: 12,
                },
                title: Rect {
                    x: 28,
                    y: 40,
                    width: 370,
                    height: 38,
                },
                status: Rect {
                    x: 402,
                    y: 28,
                    width: 126,
                    height: 34,
                },
                search: Rect {
                    x: 28,
                    y: 94,
                    width: 244,
                    height: 34,
                },
                filters: Rect {
                    x: 284,
                    y: 94,
                    width: 244,
                    height: 34,
                },
                game_list: Rect {
                    x: 28,
                    y: 140,
                    width: 500,
                    height: 136,
                },
                footer: Rect {
                    x: 28,
                    y: 288,
                    width: 500,
                    height: 10,
                },
                touch_panel: None,
            },
            ArcadeInteractionMode::Touch => Self {
                catalog_panel: Rect {
                    x: 14,
                    y: 14,
                    width: 692,
                    height: 372,
                },
                header: Rect {
                    x: 15,
                    y: 15,
                    width: 690,
                    height: 76,
                },
                eyebrow: Rect {
                    x: 30,
                    y: 26,
                    width: 300,
                    height: 12,
                },
                title: Rect {
                    x: 30,
                    y: 43,
                    width: 390,
                    height: 40,
                },
                status: Rect {
                    x: 432,
                    y: 30,
                    width: 120,
                    height: 34,
                },
                search: Rect {
                    x: 30,
                    y: 101,
                    width: 248,
                    height: 36,
                },
                filters: Rect {
                    x: 290,
                    y: 101,
                    width: 262,
                    height: 36,
                },
                game_list: Rect {
                    x: 30,
                    y: 151,
                    width: 522,
                    height: 190,
                },
                footer: Rect {
                    x: 30,
                    y: 356,
                    width: 522,
                    height: 12,
                },
                touch_panel: Some(Rect {
                    x: 570,
                    y: 78,
                    width: 118,
                    height: 218,
                }),
            },
        }
    }

    fn filter_rects(self) -> [Rect; 5] {
        let gap = 4_u32;
        let total_gap = gap * 4;
        let chip_width = self.filters.width.saturating_sub(total_gap) / 5;
        let mut rects = [self.filters; 5];
        let mut x = self.filters.x;
        for rect in &mut rects {
            *rect = Rect {
                x,
                y: self.filters.y,
                width: chip_width,
                height: self.filters.height,
            };
            x = x.saturating_add(i32::try_from(chip_width + gap).unwrap_or(i32::MAX));
        }
        rects
    }

    fn search_clear_rect(self) -> Rect {
        Rect {
            x: self.search.x.saturating_add(
                i32::try_from(self.search.width.saturating_sub(30)).unwrap_or(i32::MAX),
            ),
            y: self.search.y,
            width: 30,
            height: self.search.height,
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
    filter: CatalogFilter,
    search: CatalogSearch,
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
            filter: CatalogFilter::All,
            search: CatalogSearch::default(),
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

        if self.update_search_and_filters(frame) == GameResult::Exit {
            return GameResult::Exit;
        }

        let nav_enabled = !self.search.active;
        let input = SpatialInput {
            nav: UiNavInput {
                up: nav_enabled && self.catalog_controls.action(CATALOG_UP).pressed(),
                down: nav_enabled && self.catalog_controls.action(CATALOG_DOWN).pressed(),
                left: nav_enabled && self.catalog_controls.action(CATALOG_LEFT).pressed(),
                right: nav_enabled && self.catalog_controls.action(CATALOG_RIGHT).pressed(),
                confirm: nav_enabled && self.catalog_controls.action(CATALOG_SELECT).pressed(),
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

    fn update_search_and_filters(&mut self, frame: &Frame<'_>) -> GameResult {
        let previous_query = self.search.query.clone();
        let previous_filter = self.filter;
        let ctrl =
            frame.input.key(Key::LeftControl).held() || frame.input.key(Key::RightControl).held();
        let shift =
            frame.input.key(Key::LeftShift).held() || frame.input.key(Key::RightShift).held();

        if ctrl && frame.input.key(Key::F).pressed() {
            self.search.active = true;
            self.search.select_all();
        }
        if frame.input.mouse_button(MouseButton::Left).pressed()
            && let Some(position) = frame.input.mouse_position()
        {
            self.handle_catalog_point(position);
        }
        for touch in frame.input.touches() {
            if touch.phase == TouchPhase::Started
                && let Some(position) = touch.position
            {
                self.handle_catalog_point(position);
            }
        }

        if frame.input.key(Key::Escape).pressed() {
            if self.search.active || !self.search.query.is_empty() {
                self.search.clear();
                self.search.active = false;
            } else if self.filter != CatalogFilter::All {
                self.filter = CatalogFilter::All;
            } else {
                return GameResult::Exit;
            }
        }

        if self.search.active {
            if ctrl && frame.input.key(Key::A).pressed() {
                self.search.select_all();
            }
            if !ctrl {
                self.search.edit(frame.input.text_events());
            }
            if frame.input.key(Key::Enter).pressed() || frame.input.key(Key::Tab).pressed() {
                self.search.active = false;
            }
        } else {
            if frame.input.key(Key::Tab).pressed() {
                self.cycle_filter(if shift { -1 } else { 1 });
            }
            if self.catalog_controls.action(CATALOG_FILTER_PREV).pressed() {
                self.cycle_filter(-1);
            }
            if self.catalog_controls.action(CATALOG_FILTER_NEXT).pressed() {
                self.cycle_filter(1);
            }
        }

        if previous_query != self.search.query || previous_filter != self.filter {
            self.catalog_state = SpatialState::default();
        }
        GameResult::Continue
    }

    fn handle_catalog_point(&mut self, position: (i32, i32)) {
        if point_in_rect(position, self.layout.search) {
            self.search.active = true;
            if point_in_rect(position, self.layout.search_clear_rect()) {
                self.search.clear();
            }
            return;
        }
        for (index, rect) in self.layout.filter_rects().iter().copied().enumerate() {
            if point_in_rect(position, rect) {
                self.filter = CatalogFilter::ALL[index];
                self.catalog_state = SpatialState::default();
                return;
            }
        }
    }

    fn cycle_filter(&mut self, delta: isize) {
        let position = CatalogFilter::ALL
            .iter()
            .position(|filter| *filter == self.filter)
            .unwrap_or(0);
        let next = (position as isize + delta).rem_euclid(CatalogFilter::ALL.len() as isize);
        self.filter = CatalogFilter::ALL[next as usize];
        self.catalog_state = SpatialState::default();
    }

    fn visible_game_indices(&self) -> Vec<usize> {
        let filter_mask = self.filter.mask();
        let query = self.search.query.trim();
        let mut matches = GAME_LABELS
            .iter()
            .enumerate()
            .filter(|(index, _)| filter_mask == 0 || GAME_MASKS[*index] & filter_mask != 0)
            .filter_map(|(index, label)| {
                if query.is_empty() {
                    Some((index, 0))
                } else {
                    let searchable = format!("{} {}", label, GAME_TAGS[index]);
                    fuzzy_score(query, &searchable).map(|score| (index, score))
                }
            })
            .collect::<Vec<_>>();
        if !query.is_empty() {
            matches.sort_by(|(left_index, left_score), (right_index, right_score)| {
                right_score
                    .cmp(left_score)
                    .then_with(|| left_index.cmp(right_index))
            });
        }
        matches.into_iter().map(|(index, _)| index).collect()
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
        let visible = self.visible_game_indices();
        framebuffer.clear(BG);
        draw_panel(framebuffer, self.layout.catalog_panel, PANEL, BORDER);
        framebuffer.fill_rect(
            self.layout.header.x,
            self.layout.header.y,
            self.layout.header.width,
            self.layout.header.height,
            HEADER,
        );
        framebuffer.fill_rect(
            self.layout.header.x,
            self.layout.header.y,
            6,
            self.layout.header.height,
            ACCENT,
        );
        draw_text_centered(
            framebuffer,
            self.layout.eyebrow,
            "GPE.UI / P6.1 LAUNCHER",
            1,
            MUTED,
        );
        self.paint_search_and_filters(framebuffer, visible.len());

        let cards = catalog_cards(&self.catalog_ids, &visible);
        let output = run_default_card_grid_styled(
            framebuffer,
            self.layout.game_list,
            &mut self.catalog_state,
            input,
            catalog_grid_spec(self.mode),
            catalog_theme(),
            catalog_stylesheet(),
            &cards,
        );
        self.paint_catalog_typography(framebuffer, &output, &visible);
        self.paint_touch_controls(framebuffer);
        draw_text_centered(
            framebuffer,
            self.layout.footer,
            "CTRL+F SEARCH  |  TAB FILTER  |  ARROWS/PAD  |  SPACE/SOUTH  |  MOUSE/TOUCH",
            1,
            MUTED,
        );
        cards
            .iter()
            .position(|card| output.activated(card.id))
            .and_then(|position| visible.get(position).copied())
    }

    fn paint_search_and_filters(&mut self, framebuffer: &mut Framebuffer, result_count: usize) {
        framebuffer.fill_rect(
            self.layout.search.x,
            self.layout.search.y,
            self.layout.search.width,
            self.layout.search.height,
            SEARCH_BG,
        );
        framebuffer.draw_rect(
            self.layout.search.x,
            self.layout.search.y,
            self.layout.search.width,
            self.layout.search.height,
            if self.search.active { ACCENT } else { BORDER },
        );
        draw_text_centered(framebuffer, self.layout.search_clear_rect(), "X", 1, MUTED);
        for (filter, rect) in CatalogFilter::ALL
            .iter()
            .copied()
            .zip(self.layout.filter_rects())
        {
            let selected = filter == self.filter;
            let color = filter.color();
            framebuffer.fill_rect(
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                if selected { color } else { CHIP_BG },
            );
            framebuffer.draw_rect(rect.x, rect.y, rect.width, rect.height, color);
            draw_text_centered(
                framebuffer,
                rect,
                filter.label(),
                1,
                if selected { BG } else { color },
            );
        }
        let status = if self.search.query.is_empty() {
            format!("{} / {} GAMES", result_count, GAME_LABELS.len())
        } else {
            format!("{} FUZZY MATCH", result_count)
        };
        draw_text_centered(framebuffer, self.layout.status, &status, 1, ACCENT);
    }

    fn paint_catalog_typography(
        &mut self,
        framebuffer: &mut Framebuffer,
        output: &SpatialOutput,
        visible: &[usize],
    ) {
        #[cfg(feature = "outline-fonts")]
        if let Some(font) = &mut self.outline_font {
            let title_px = if self.mode == ArcadeInteractionMode::Touch {
                30.0
            } else {
                28.0
            };
            let _ = font.draw(
                framebuffer,
                "GPE ARCADE",
                title_px,
                self.layout.title,
                ACCENT,
            );
            let search_label = self.search.display();
            let _ = font.draw(
                framebuffer,
                &search_label,
                13.5,
                Rect {
                    x: self.layout.search.x.saturating_add(10),
                    y: self.layout.search.y.saturating_add(6),
                    width: self.layout.search.width.saturating_sub(44),
                    height: self.layout.search.height.saturating_sub(8),
                },
                if self.search.query.is_empty() && !self.search.active {
                    MUTED
                } else {
                    FG
                },
            );
            if visible.is_empty() {
                let _ = font.draw(
                    framebuffer,
                    "NO GAMES MATCH",
                    25.0,
                    self.layout.game_list,
                    MUTED,
                );
                return;
            }
            for (layout, game_index) in output.layouts().iter().zip(visible.iter().copied()) {
                let accent = GAME_ACCENTS[game_index];
                framebuffer.fill_rect(
                    layout.rect.x,
                    layout.rect.y,
                    layout.rect.width,
                    5_u32.min(layout.rect.height),
                    accent,
                );
                let focused = output.focused_id() == Some(layout.id);
                let title_color = if focused { ACCENT } else { FG };
                let title_px = if output.columns() >= 3 { 15.0 } else { 18.0 };
                let _ = font.draw(
                    framebuffer,
                    GAME_LABELS[game_index],
                    title_px,
                    Rect {
                        x: layout.text_rect.x.saturating_add(8),
                        y: layout.text_rect.y.saturating_add(8),
                        width: layout.text_rect.width.saturating_sub(16),
                        height: layout.text_rect.height.saturating_sub(26),
                    },
                    title_color,
                );
                let _ = font.draw(
                    framebuffer,
                    GAME_TAGS[game_index],
                    if output.columns() >= 3 { 9.0 } else { 10.5 },
                    Rect {
                        x: layout.text_rect.x.saturating_add(8),
                        y: layout.text_rect.y.saturating_add(
                            i32::try_from(layout.text_rect.height.saturating_sub(22))
                                .unwrap_or(i32::MAX),
                        ),
                        width: layout.text_rect.width.saturating_sub(16),
                        height: 18,
                    },
                    accent,
                );
            }
            return;
        }

        draw_text_centered(framebuffer, self.layout.title, "GPE ARCADE", 3, ACCENT);
        let search_label = self.search.display();
        draw_text_centered(
            framebuffer,
            Rect {
                x: self.layout.search.x.saturating_add(8),
                y: self.layout.search.y,
                width: self.layout.search.width.saturating_sub(40),
                height: self.layout.search.height,
            },
            &search_label,
            1,
            if self.search.query.is_empty() && !self.search.active {
                MUTED
            } else {
                FG
            },
        );
        if visible.is_empty() {
            draw_text_centered(
                framebuffer,
                self.layout.game_list,
                "NO GAMES MATCH",
                2,
                MUTED,
            );
            return;
        }
        for (layout, game_index) in output.layouts().iter().zip(visible.iter().copied()) {
            let accent = GAME_ACCENTS[game_index];
            framebuffer.fill_rect(
                layout.rect.x,
                layout.rect.y,
                layout.rect.width,
                5_u32.min(layout.rect.height),
                accent,
            );
            draw_text_centered(
                framebuffer,
                Rect {
                    x: layout.text_rect.x,
                    y: layout.text_rect.y,
                    width: layout.text_rect.width,
                    height: layout.text_rect.height.saturating_sub(20),
                },
                GAME_LABELS[game_index],
                1,
                if output.focused_id() == Some(layout.id) {
                    ACCENT
                } else {
                    FG
                },
            );
            draw_text_centered(
                framebuffer,
                Rect {
                    x: layout.text_rect.x,
                    y: layout.text_rect.y.saturating_add(
                        i32::try_from(layout.text_rect.height.saturating_sub(18))
                            .unwrap_or(i32::MAX),
                    ),
                    width: layout.text_rect.width,
                    height: 14,
                },
                GAME_TAGS[game_index],
                1,
                accent,
            );
        }
    }

    fn paint_touch_controls(&self, framebuffer: &mut Framebuffer) {
        let Some(touch_panel) = self.layout.touch_panel else {
            return;
        };
        draw_panel(framebuffer, touch_panel, BG, BORDER);
        draw_text_centered(
            framebuffer,
            Rect {
                x: touch_panel.x,
                y: touch_panel.y.saturating_add(8),
                width: touch_panel.width,
                height: 10,
            },
            "TOUCH / PAD",
            1,
            MUTED,
        );
        for (rect, label) in [
            (TOUCH_UP, "UP"),
            (TOUCH_LEFT, "<"),
            (TOUCH_RIGHT, ">"),
            (TOUCH_DOWN, "DN"),
            (TOUCH_SELECT, "PLAY"),
        ] {
            framebuffer.fill_rect(rect.x, rect.y, rect.width, rect.height, CHIP_BG);
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
            ControlBinding::Key(Key::A),
            ControlBinding::Gamepad(GamepadButton::DPadLeft),
            ControlBinding::Gamepad(GamepadButton::LeftStickLeft),
        ],
    );
    bind_catalog_action(
        &mut controls,
        CATALOG_RIGHT,
        &[
            ControlBinding::Key(Key::Right),
            ControlBinding::Key(Key::D),
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
    bind_catalog_action(
        &mut controls,
        CATALOG_FILTER_PREV,
        &[ControlBinding::Gamepad(GamepadButton::LeftShoulder)],
    );
    bind_catalog_action(
        &mut controls,
        CATALOG_FILTER_NEXT,
        &[ControlBinding::Gamepad(GamepadButton::RightShoulder)],
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
            width: 560,
            height: 320,
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

fn catalog_cards(ids: &[UiId], visible: &[usize]) -> Vec<SpatialCard<'static>> {
    visible
        .iter()
        .map(|index| SpatialCard {
            id: ids[*index],
            title: "",
            subtitle: "",
            image: None,
            action: CATALOG_SELECT,
        })
        .collect()
}

fn catalog_grid_spec(mode: ArcadeInteractionMode) -> GridSpec {
    match mode {
        ArcadeInteractionMode::Native => GridSpec {
            min_cell_width: 160,
            preferred_cell_height: 64,
            gap: 10,
            padding: 0,
        },
        ArcadeInteractionMode::Touch => GridSpec {
            min_cell_width: 160,
            preferred_cell_height: 90,
            gap: 10,
            padding: 0,
        },
    }
}

fn catalog_theme() -> UiTheme {
    UiTheme {
        padding: 8,
        row_height: 22,
        row_spacing: 8,
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
                border: Some(Pixel::rgb(84, 126, 151)),
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

fn point_in_rect((x, y): (i32, i32), rect: Rect) -> bool {
    let right = i64::from(rect.x) + i64::from(rect.width);
    let bottom = i64::from(rect.y) + i64::from(rect.height);
    i64::from(x) >= i64::from(rect.x)
        && i64::from(x) < right
        && i64::from(y) >= i64::from(rect.y)
        && i64::from(y) < bottom
}

fn fuzzy_score(query: &str, candidate: &str) -> Option<i32> {
    let query = query.to_lowercase();
    let candidate = candidate.to_lowercase();
    let mut score = 0_i32;
    let mut last_match = None;
    let mut cursor = 0_usize;

    for needle in query.chars().filter(|character| !character.is_whitespace()) {
        let mut found = None;
        for (offset, character) in candidate[cursor..].char_indices() {
            if character == needle {
                found = Some(cursor + offset);
                break;
            }
        }
        let position = found?;
        score += 10;
        if let Some(previous) = last_match {
            if position == previous + 1 {
                score += 8;
            } else {
                score -= i32::try_from(position.saturating_sub(previous + 1)).unwrap_or(i32::MAX);
            }
        } else {
            score -= i32::try_from(position).unwrap_or(i32::MAX);
        }
        last_match = Some(position);
        cursor = position + candidate[position..].chars().next().unwrap().len_utf8();
    }
    Some(score)
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
    fn catalog_ids_are_stable() {
        assert_eq!(catalog_ids(), catalog_ids());
    }

    #[test]
    fn launcher_surfaces_use_three_columns_with_room_for_card_metadata() {
        let ids = catalog_ids();
        let visible: Vec<_> = (0..GAME_LABELS.len()).collect();
        let cards = catalog_cards(&ids, &visible);
        for mode in [ArcadeInteractionMode::Native, ArcadeInteractionMode::Touch] {
            let bounds = ArcadeLayout::for_mode(mode).game_list;
            let mut state = SpatialState::default();
            let output = run_card_grid_headless(
                bounds,
                &mut state,
                SpatialInput::default(),
                catalog_grid_spec(mode),
                &cards,
            );
            assert_eq!(output.columns(), 3);
            assert_eq!(output.layouts().len(), GAME_LABELS.len());
            assert!(
                output
                    .layouts()
                    .iter()
                    .all(|layout| layout.rect.height >= 60)
            );
        }
    }

    #[test]
    fn responsive_grid_collapses_to_two_columns_on_narrower_native_bounds() {
        let ids = catalog_ids();
        let visible: Vec<_> = (0..GAME_LABELS.len()).collect();
        let cards = catalog_cards(&ids, &visible);
        let mut state = SpatialState::default();
        let output = run_card_grid_headless(
            Rect {
                x: 0,
                y: 0,
                width: 340,
                height: 210,
            },
            &mut state,
            SpatialInput::default(),
            catalog_grid_spec(ArcadeInteractionMode::Native),
            &cards,
        );
        assert_eq!(output.columns(), 2);
    }

    #[test]
    fn category_filters_are_real_consumer_metadata() {
        let mut app = ArcadeApp::new(ArcadeInteractionMode::Native);
        app.filter = CatalogFilter::Puzzle;
        assert_eq!(app.visible_game_indices(), vec![1, 3]);
        app.filter = CatalogFilter::Shmup;
        assert_eq!(app.visible_game_indices(), vec![2]);
        app.filter = CatalogFilter::Action;
        assert_eq!(app.visible_game_indices(), vec![0, 3, 5]);
    }

    #[test]
    fn fuzzy_search_matches_labels_and_categories() {
        let mut app = ArcadeApp::new(ArcadeInteractionMode::Native);
        app.search.query = "spinv".into();
        assert_eq!(app.visible_game_indices().first().copied(), Some(2));
        app.search.query = "shmup".into();
        assert_eq!(app.visible_game_indices(), vec![2]);
        app.search.query = "zzzz".into();
        assert!(app.visible_game_indices().is_empty());
    }

    #[test]
    fn search_and_category_filter_compose() {
        let mut app = ArcadeApp::new(ArcadeInteractionMode::Native);
        app.filter = CatalogFilter::Puzzle;
        app.search.query = "hero".into();
        assert_eq!(app.visible_game_indices(), vec![3]);
        app.search.query = "tetris".into();
        assert_eq!(app.visible_game_indices(), vec![1]);
    }

    #[test]
    fn catalog_spatial_navigation_moves_right_and_down() {
        let ids = catalog_ids();
        let visible: Vec<_> = (0..GAME_LABELS.len()).collect();
        let cards = catalog_cards(&ids, &visible);
        let bounds = ArcadeLayout::for_mode(ArcadeInteractionMode::Native).game_list;
        let spec = catalog_grid_spec(ArcadeInteractionMode::Native);
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
        assert_eq!(down.focused_id(), Some(ids[4]));
    }

    #[test]
    fn catalog_pointer_click_activates_the_hit_card() {
        let ids = catalog_ids();
        let visible: Vec<_> = (0..GAME_LABELS.len()).collect();
        let cards = catalog_cards(&ids, &visible);
        let bounds = ArcadeLayout::for_mode(ArcadeInteractionMode::Native).game_list;
        let spec = catalog_grid_spec(ArcadeInteractionMode::Native);
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
        let visible: Vec<_> = (0..GAME_LABELS.len()).collect();
        let cards = catalog_cards(&ids, &visible);
        let bounds = ArcadeLayout::for_mode(ArcadeInteractionMode::Touch).game_list;
        let spec = catalog_grid_spec(ArcadeInteractionMode::Touch);
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
    fn launcher_surfaces_are_larger_and_pause_remains_outside_touch_games() {
        assert_eq!(
            ArcadeInteractionMode::Native.framebuffer_size(),
            Size {
                width: 560,
                height: 320
            }
        );
        assert_eq!(
            ArcadeInteractionMode::Touch.framebuffer_size(),
            Size {
                width: 720,
                height: 400
            }
        );
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
    fn native_mode_has_no_virtual_pad_and_touch_mode_has_one() {
        assert!(
            ArcadeApp::new(ArcadeInteractionMode::Native)
                .catalog_pad
                .is_none()
        );
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
