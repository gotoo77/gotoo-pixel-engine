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
    ActionId, Frame, Framebuffer, Game, GameResult, GamepadButton, Input, Key, MouseButton,
    Pixel, PixelGameHost, Rect, Size, TextInputEvent,
    ui::{
        PauseConfig, PauseGame, UiComponentStyle, UiStyleOverride, UiStyleSheet, UiTheme,
        draw_panel, draw_text_centered,
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

pub const HOST_SIZE: Size = Size {
    width: 1280,
    height: 720,
};
const GAME_SIZE: Size = Size {
    width: 560,
    height: 320,
};
const HOST_BOUNDS: Rect = Rect {
    x: 0,
    y: 0,
    width: HOST_SIZE.width,
    height: HOST_SIZE.height,
};

const SELECT: ActionId = ActionId::new("arcade.high-res.select");
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

const BG: Pixel = Pixel::rgb(4, 8, 13);
const PANEL: Pixel = Pixel::rgb(9, 15, 23);
const HEADER: Pixel = Pixel::rgb(12, 22, 33);
const CARD: Pixel = Pixel::rgb(16, 25, 37);
const CARD_FOCUSED: Pixel = Pixel::rgb(22, 42, 50);
const CARD_HOVERED: Pixel = Pixel::rgb(20, 33, 47);
const CARD_ACTIVE: Pixel = Pixel::rgb(29, 52, 55);
const FG: Pixel = Pixel::rgb(234, 241, 239);
const MUTED: Pixel = Pixel::rgb(135, 154, 168);
const BORDER: Pixel = Pixel::rgb(48, 76, 98);
const ACCENT: Pixel = Pixel::rgb(111, 238, 184);
const SEARCH_BG: Pixel = Pixel::rgb(7, 12, 19);
const CHIP_BG: Pixel = Pixel::rgb(18, 29, 42);

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

#[derive(Debug, Clone, Default)]
struct CatalogSearch {
    query: String,
    active: bool,
    cursor: usize,
    select_all: bool,
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
                "| SEARCH".to_owned()
            } else {
                "SEARCH / CTRL+F".to_owned()
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
                    let previous = self.query[..self.cursor]
                        .char_indices()
                        .last()
                        .expect("cursor > 0 must have a previous character")
                        .0;
                    self.query.drain(previous..self.cursor);
                    self.cursor = previous;
                }
                TextInputEvent::Delete if self.cursor < self.query.len() => {
                    self.query.remove(self.cursor);
                }
                TextInputEvent::Left if self.cursor > 0 => {
                    self.cursor = self.query[..self.cursor]
                        .char_indices()
                        .last()
                        .expect("cursor > 0 must have a previous character")
                        .0;
                }
                TextInputEvent::Right if self.cursor < self.query.len() => {
                    self.cursor += self.query[self.cursor..]
                        .chars()
                        .next()
                        .expect("cursor before end must have a character")
                        .len_utf8();
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
struct Layout {
    panel: Rect,
    header: Rect,
    eyebrow: Rect,
    title: Rect,
    status: Rect,
    search: Rect,
    filters: Rect,
    cards: Rect,
    footer: Rect,
}

impl Layout {
    const fn high_res() -> Self {
        Self {
            panel: Rect {
                x: 28,
                y: 28,
                width: 1224,
                height: 664,
            },
            header: Rect {
                x: 29,
                y: 29,
                width: 1222,
                height: 152,
            },
            eyebrow: Rect {
                x: 72,
                y: 52,
                width: 460,
                height: 28,
            },
            title: Rect {
                x: 72,
                y: 84,
                width: 710,
                height: 72,
            },
            status: Rect {
                x: 1000,
                y: 72,
                width: 180,
                height: 44,
            },
            search: Rect {
                x: 72,
                y: 212,
                width: 490,
                height: 64,
            },
            filters: Rect {
                x: 594,
                y: 212,
                width: 586,
                height: 64,
            },
            cards: Rect {
                x: 72,
                y: 310,
                width: 1108,
                height: 290,
            },
            footer: Rect {
                x: 72,
                y: 632,
                width: 1108,
                height: 34,
            },
        }
    }

    fn filter_rects(self) -> [Rect; 5] {
        let gap = 10_u32;
        let width = self.filters.width.saturating_sub(gap * 4) / 5;
        let mut output = [self.filters; 5];
        let mut x = self.filters.x;
        for rect in &mut output {
            *rect = Rect {
                x,
                y: self.filters.y,
                width,
                height: self.filters.height,
            };
            x = x.saturating_add(i32::try_from(width + gap).unwrap_or(i32::MAX));
        }
        output
    }

    fn search_clear_rect(self) -> Rect {
        Rect {
            x: self.search.x.saturating_add(
                i32::try_from(self.search.width.saturating_sub(56)).unwrap_or(i32::MAX),
            ),
            y: self.search.y,
            width: 56,
            height: self.search.height,
        }
    }
}

pub struct ArcadeHighResApp {
    layout: Layout,
    ids: Vec<UiId>,
    spatial: SpatialState,
    filter: CatalogFilter,
    search: CatalogSearch,
    active_game: Option<PixelGameHost>,
    waiting_for_launch_release: bool,
    waiting_for_catalog_release: bool,
    #[cfg(feature = "outline-fonts")]
    brand_font: Option<OutlineFont>,
    #[cfg(feature = "outline-fonts")]
    ui_font: Option<OutlineFont>,
}

impl ArcadeHighResApp {
    pub fn new() -> Self {
        Self {
            layout: Layout::high_res(),
            ids: catalog_ids(),
            spatial: SpatialState::default(),
            filter: CatalogFilter::All,
            search: CatalogSearch::default(),
            active_game: None,
            waiting_for_launch_release: false,
            waiting_for_catalog_release: false,
            #[cfg(feature = "outline-fonts")]
            brand_font: OutlineFont::from_bytes(include_bytes!(
                "../../assets/fonts/p4/unbounded/font.ttf"
            ))
            .ok(),
            #[cfg(feature = "outline-fonts")]
            ui_font: OutlineFont::from_bytes(include_bytes!("../../assets/fonts/p4/exo2/font.ttf"))
                .ok(),
        }
    }

    fn update_catalog(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if self.waiting_for_catalog_release {
            if select_held(frame.input) {
                self.render_catalog(frame.framebuffer, SpatialInput::default());
                return GameResult::Continue;
            }
            self.waiting_for_catalog_release = false;
        }

        if self.update_search_and_filters(frame) == GameResult::Exit {
            return GameResult::Exit;
        }

        let nav = nav_input(frame.input, self.search.active);
        if nav.up || nav.down || nav.left || nav.right {
            self.search.active = false;
        }
        let spatial_input = SpatialInput {
            nav,
            pointer: PointerInput {
                position: frame.input.mouse_position(),
                pressed: frame.input.mouse_button(MouseButton::Left).pressed(),
                released: frame.input.mouse_button(MouseButton::Left).released(),
            },
            touches: frame.input.touches(),
        };
        if let Some(index) = self.render_catalog(frame.framebuffer, spatial_input) {
            self.launch(index);
        }
        GameResult::Continue
    }

    fn update_active_game(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if self.waiting_for_launch_release {
            if select_held(frame.input) {
                self.present_game(frame.framebuffer);
                return GameResult::Continue;
            }
            self.waiting_for_launch_release = false;
        }

        frame.framebuffer.clear(BG);
        let (result, _) = self
            .active_game
            .as_mut()
            .expect("active game update requires a game")
            .update_and_present(frame, HOST_BOUNDS);

        if result == GameResult::Exit {
            self.return_to_catalog();
            self.render_catalog(frame.framebuffer, SpatialInput::default());
        }
        GameResult::Continue
    }

    fn present_game(&self, host: &mut Framebuffer) {
        host.clear(BG);
        if let Some(game) = &self.active_game {
            let _ = game.present(host, HOST_BOUNDS);
        }
    }

    fn update_search_and_filters(&mut self, frame: &Frame<'_>) -> GameResult {
        let previous_query = self.search.query.clone();
        let previous_filter = self.filter;
        let control =
            frame.input.key(Key::LeftControl).held() || frame.input.key(Key::RightControl).held();
        let shift =
            frame.input.key(Key::LeftShift).held() || frame.input.key(Key::RightShift).held();

        if control && frame.input.key(Key::F).pressed() {
            self.search.active = true;
            self.search.select_all();
        }
        if frame.input.mouse_button(MouseButton::Left).pressed()
            && let Some(point) = frame.input.mouse_position()
        {
            self.handle_point(point);
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
            if control && frame.input.key(Key::A).pressed() {
                self.search.select_all();
            }
            if raw_direction_pressed(frame.input, false) {
                self.search.active = false;
            } else {
                if !control {
                    self.search.edit(frame.input.text_events());
                }
                if frame.input.key(Key::Enter).pressed() || frame.input.key(Key::Tab).pressed() {
                    self.search.active = false;
                }
            }
        } else {
            if frame.input.key(Key::Tab).pressed() {
                self.cycle_filter(if shift { -1 } else { 1 });
            }
            if frame
                .input
                .gamepad_button_any(GamepadButton::LeftShoulder)
                .pressed()
            {
                self.cycle_filter(-1);
            }
            if frame
                .input
                .gamepad_button_any(GamepadButton::RightShoulder)
                .pressed()
            {
                self.cycle_filter(1);
            }
        }

        if previous_query != self.search.query || previous_filter != self.filter {
            self.spatial = SpatialState::default();
        }
        GameResult::Continue
    }

    fn handle_point(&mut self, point: (i32, i32)) {
        if self.layout.search.contains(point) {
            self.search.active = true;
            if self.layout.search_clear_rect().contains(point) {
                self.search.clear();
            }
            return;
        }
        for (index, rect) in self.layout.filter_rects().iter().copied().enumerate() {
            if rect.contains(point) {
                self.filter = CatalogFilter::ALL[index];
                self.spatial = SpatialState::default();
                return;
            }
        }
    }

    fn cycle_filter(&mut self, delta: isize) {
        let current = CatalogFilter::ALL
            .iter()
            .position(|filter| *filter == self.filter)
            .unwrap_or(0);
        let next = (current as isize + delta).rem_euclid(CatalogFilter::ALL.len() as isize);
        self.filter = CatalogFilter::ALL[next as usize];
        self.spatial = SpatialState::default();
    }

    fn visible_games(&self) -> Vec<usize> {
        let mask = self.filter.mask();
        let query = self.search.query.trim();
        let mut matches = GAME_LABELS
            .iter()
            .enumerate()
            .filter(|(index, _)| mask == 0 || GAME_MASKS[*index] & mask != 0)
            .filter_map(|(index, label)| {
                if query.is_empty() {
                    Some((index, 0))
                } else {
                    fuzzy_score(query, &format!("{} {}", label, GAME_TAGS[index]))
                        .map(|score| (index, score))
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

    fn render_catalog(
        &mut self,
        framebuffer: &mut Framebuffer,
        input: SpatialInput<'_>,
    ) -> Option<usize> {
        let visible = self.visible_games();
        framebuffer.clear(BG);
        draw_panel(framebuffer, self.layout.panel, PANEL, BORDER);
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
            10,
            self.layout.header.height,
            ACCENT,
        );

        let cards = cards(&self.ids, &visible);
        let output = run_default_card_grid_styled(
            framebuffer,
            self.layout.cards,
            &mut self.spatial,
            input,
            GridSpec {
                min_cell_width: 330,
                preferred_cell_height: 130,
                gap: 18,
                padding: 0,
            },
            theme(),
            stylesheet(),
            &cards,
        );
        self.paint_chrome(framebuffer, visible.len());
        self.paint_cards(framebuffer, &output, &visible);

        cards
            .iter()
            .position(|card| output.activated(card.id))
            .and_then(|position| visible.get(position).copied())
    }

    fn paint_chrome(&mut self, framebuffer: &mut Framebuffer, count: usize) {
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

        let filter_rects = self.layout.filter_rects();
        for (filter, rect) in CatalogFilter::ALL.iter().copied().zip(filter_rects) {
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
        }

        #[cfg(feature = "outline-fonts")]
        {
            if let Some(font) = &mut self.brand_font {
                draw_outline_fit_centered(
                    font,
                    framebuffer,
                    "GPE ARCADE",
                    58.0,
                    42.0,
                    self.layout.title,
                    ACCENT,
                );
            }
            let status = if self.search.query.is_empty() {
                format!("{} / {} GAMES", count, GAME_LABELS.len())
            } else {
                format!("{} FUZZY MATCH", count)
            };
            if let Some(font) = &mut self.ui_font {
                draw_outline_fit_centered(
                    font,
                    framebuffer,
                    "GPE.UI / P6.1 HIGH-RES LAUNCHER",
                    20.0,
                    16.0,
                    self.layout.eyebrow,
                    MUTED,
                );
                draw_outline_fit_centered(
                    font,
                    framebuffer,
                    &self.search.display(),
                    27.0,
                    20.0,
                    Rect {
                        x: self.layout.search.x + 18,
                        y: self.layout.search.y + 8,
                        width: self.layout.search.width.saturating_sub(78),
                        height: self.layout.search.height.saturating_sub(16),
                    },
                    if self.search.query.is_empty() && !self.search.active {
                        MUTED
                    } else {
                        FG
                    },
                );
                draw_outline_fit_centered(
                    font,
                    framebuffer,
                    "×",
                    28.0,
                    22.0,
                    self.layout.search_clear_rect(),
                    MUTED,
                );
                for (filter, rect) in CatalogFilter::ALL.iter().copied().zip(filter_rects) {
                    draw_outline_fit_centered(
                        font,
                        framebuffer,
                        filter.label(),
                        22.0,
                        17.0,
                        rect,
                        if filter == self.filter { BG } else { filter.color() },
                    );
                }
                draw_outline_fit_centered(
                    font,
                    framebuffer,
                    &status,
                    22.0,
                    17.0,
                    self.layout.status,
                    ACCENT,
                );
                draw_outline_fit_centered(
                    font,
                    framebuffer,
                    "CTRL+F SEARCH  ·  TAB FILTER  ·  ARROWS/WASD/PAD  ·  SPACE/ENTER/A",
                    18.0,
                    14.0,
                    self.layout.footer,
                    MUTED,
                );
                return;
            }
        }

        draw_text_centered(
            framebuffer,
            self.layout.title,
            "GPE ARCADE",
            5,
            ACCENT,
        );
        draw_text_centered(
            framebuffer,
            self.layout.search,
            &self.search.display(),
            2,
            MUTED,
        );
    }

    fn paint_cards(
        &mut self,
        framebuffer: &mut Framebuffer,
        output: &SpatialOutput,
        visible: &[usize],
    ) {
        if visible.is_empty() {
            #[cfg(feature = "outline-fonts")]
            if let Some(font) = &mut self.ui_font {
                draw_outline_fit_centered(
                    font,
                    framebuffer,
                    "NO GAMES MATCH",
                    38.0,
                    28.0,
                    self.layout.cards,
                    MUTED,
                );
                return;
            }
            draw_text_centered(framebuffer, self.layout.cards, "NO GAMES MATCH", 3, MUTED);
            return;
        }

        for (layout, game_index) in output.layouts().iter().zip(visible.iter().copied()) {
            let accent = GAME_ACCENTS[game_index];
            framebuffer.fill_rect(
                layout.rect.x,
                layout.rect.y,
                layout.rect.width,
                8_u32.min(layout.rect.height),
                accent,
            );

            #[cfg(feature = "outline-fonts")]
            if let Some(font) = &mut self.ui_font {
                let focused = output.focused_id() == Some(layout.id);
                draw_outline_fit_centered(
                    font,
                    framebuffer,
                    GAME_LABELS[game_index],
                    31.0,
                    23.0,
                    Rect {
                        x: layout.rect.x + 18,
                        y: layout.rect.y + 34,
                        width: layout.rect.width.saturating_sub(36),
                        height: 52,
                    },
                    if focused { ACCENT } else { FG },
                );
                draw_outline_fit_centered(
                    font,
                    framebuffer,
                    GAME_TAGS[game_index],
                    19.0,
                    15.0,
                    Rect {
                        x: layout.rect.x + 18,
                        y: layout.rect.y
                            + i32::try_from(layout.rect.height.saturating_sub(54))
                                .unwrap_or(i32::MAX),
                        width: layout.rect.width.saturating_sub(36),
                        height: 34,
                    },
                    accent,
                );
                continue;
            }

            draw_text_centered(
                framebuffer,
                layout.rect,
                GAME_LABELS[game_index],
                2,
                FG,
            );
        }
    }

    fn launch(&mut self, index: usize) {
        let Some(game) = build_game(index) else {
            return;
        };
        self.active_game = Some(PixelGameHost::from_boxed(game, GAME_SIZE));
        self.waiting_for_launch_release = true;
    }

    fn return_to_catalog(&mut self) {
        self.active_game = None;
        self.waiting_for_launch_release = false;
        self.waiting_for_catalog_release = true;
    }
}

impl Game for ArcadeHighResApp {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if self.active_game.is_some() {
            self.update_active_game(frame)
        } else {
            self.update_catalog(frame)
        }
    }
}

fn build_game(index: usize) -> Option<Box<dyn Game>> {
    let game: Box<dyn Game> = match index {
        0 => pause_game(SnakeGame::new(SnakeInteractionMode::Keyboard)),
        1 => pause_game(TetrisGame::new()),
        2 => pause_game(EnhancedSpaceInvadersGame::new()),
        3 => Box::new(SmartBoyHeroGame::new()),
        4 => pause_game(PongGame::new()),
        5 => pause_game(BreakoutGame::new()),
        _ => return None,
    };
    Some(game)
}

fn pause_game<G: Game + 'static>(game: G) -> Box<dyn Game> {
    Box::new(PauseGame::new(game, PauseConfig::new(GAME_SIZE)))
}

fn nav_input(input: &Input, search_active: bool) -> UiNavInput {
    let wasd = !search_active;
    UiNavInput {
        up: input.key(Key::Up).pressed()
            || (wasd && input.key(Key::W).pressed())
            || input.gamepad_button_any(GamepadButton::DPadUp).pressed()
            || input
                .gamepad_button_any(GamepadButton::LeftStickUp)
                .pressed(),
        down: input.key(Key::Down).pressed()
            || (wasd && input.key(Key::S).pressed())
            || input.gamepad_button_any(GamepadButton::DPadDown).pressed()
            || input
                .gamepad_button_any(GamepadButton::LeftStickDown)
                .pressed(),
        left: input.key(Key::Left).pressed()
            || (wasd && input.key(Key::A).pressed())
            || input.gamepad_button_any(GamepadButton::DPadLeft).pressed()
            || input
                .gamepad_button_any(GamepadButton::LeftStickLeft)
                .pressed(),
        right: input.key(Key::Right).pressed()
            || (wasd && input.key(Key::D).pressed())
            || input.gamepad_button_any(GamepadButton::DPadRight).pressed()
            || input
                .gamepad_button_any(GamepadButton::LeftStickRight)
                .pressed(),
        confirm: !search_active
            && (input.key(Key::Space).pressed()
                || input.key(Key::Enter).pressed()
                || input.gamepad_button_any(GamepadButton::South).pressed()),
        ..UiNavInput::default()
    }
}

fn raw_direction_pressed(input: &Input, include_wasd: bool) -> bool {
    input.key(Key::Up).pressed()
        || input.key(Key::Down).pressed()
        || input.key(Key::Left).pressed()
        || input.key(Key::Right).pressed()
        || (include_wasd
            && (input.key(Key::W).pressed()
                || input.key(Key::A).pressed()
                || input.key(Key::S).pressed()
                || input.key(Key::D).pressed()))
        || input.gamepad_button_any(GamepadButton::DPadUp).pressed()
        || input.gamepad_button_any(GamepadButton::DPadDown).pressed()
        || input.gamepad_button_any(GamepadButton::DPadLeft).pressed()
        || input.gamepad_button_any(GamepadButton::DPadRight).pressed()
        || input
            .gamepad_button_any(GamepadButton::LeftStickUp)
            .pressed()
        || input
            .gamepad_button_any(GamepadButton::LeftStickDown)
            .pressed()
        || input
            .gamepad_button_any(GamepadButton::LeftStickLeft)
            .pressed()
        || input
            .gamepad_button_any(GamepadButton::LeftStickRight)
            .pressed()
}

fn select_held(input: &Input) -> bool {
    input.key(Key::Space).held()
        || input.key(Key::Enter).held()
        || input.gamepad_button_any(GamepadButton::South).held()
}

fn catalog_ids() -> Vec<UiId> {
    let mut state = UiStateStore::default();
    let (_, ids) = experimental::run_headless(
        HOST_SIZE,
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

fn cards(ids: &[UiId], visible: &[usize]) -> Vec<SpatialCard<'static>> {
    visible
        .iter()
        .map(|index| SpatialCard {
            id: ids[*index],
            title: "",
            subtitle: "",
            image: None,
            action: SELECT,
        })
        .collect()
}

fn theme() -> UiTheme {
    UiTheme {
        padding: 12,
        row_height: 40,
        row_spacing: 12,
        text: FG,
        muted_text: MUTED,
        control_background: CARD,
        border: BORDER,
        accent: ACCENT,
        ..UiTheme::default()
    }
}

fn stylesheet() -> UiStyleSheet {
    UiStyleSheet {
        button: UiComponentStyle {
            base: UiStyleOverride {
                background: Some(CARD),
                border: Some(BORDER),
                border_width: Some(2),
                ..UiStyleOverride::default()
            },
            focused: UiStyleOverride {
                background: Some(CARD_FOCUSED),
                border: Some(ACCENT),
                border_width: Some(4),
                ..UiStyleOverride::default()
            },
            hovered: UiStyleOverride {
                background: Some(CARD_HOVERED),
                border: Some(Pixel::rgb(84, 126, 151)),
                border_width: Some(3),
                ..UiStyleOverride::default()
            },
            active: UiStyleOverride {
                background: Some(CARD_ACTIVE),
                border: Some(ACCENT),
                border_width: Some(4),
                ..UiStyleOverride::default()
            },
        },
        ..UiStyleSheet::default()
    }
}

#[cfg(feature = "outline-fonts")]
fn draw_outline_fit_centered(
    font: &mut OutlineFont,
    framebuffer: &mut Framebuffer,
    text: &str,
    preferred: f32,
    minimum: f32,
    bounds: Rect,
    color: Pixel,
) {
    let mut px = preferred;
    let mut measured = font.measure(text, px, bounds);
    while measured.width > bounds.width && px > minimum {
        px = (px - 0.75).max(minimum);
        measured = font.measure(text, px, bounds);
    }
    let x_offset = bounds.width.saturating_sub(measured.width) / 2;
    let y_offset = bounds.height.saturating_sub(measured.height) / 2;
    let draw_bounds = Rect {
        x: bounds
            .x
            .saturating_add(i32::try_from(x_offset).unwrap_or(i32::MAX)),
        y: bounds
            .y
            .saturating_add(i32::try_from(y_offset).unwrap_or(i32::MAX)),
        width: bounds.width.saturating_sub(x_offset),
        height: bounds.height.saturating_sub(y_offset),
    };
    let _ = font.draw(framebuffer, text, px, draw_bounds, color);
}

fn fuzzy_score(query: &str, candidate: &str) -> Option<i32> {
    let query = query.to_lowercase();
    let candidate = candidate.to_lowercase();
    let mut score = 0_i32;
    let mut last = None;
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
        if let Some(previous) = last {
            if position == previous + 1 {
                score += 8;
            } else {
                score -= i32::try_from(position.saturating_sub(previous + 1)).unwrap_or(i32::MAX);
            }
        } else {
            score -= i32::try_from(position).unwrap_or(i32::MAX);
        }
        last = Some(position);
        cursor = position
            + candidate[position..]
                .chars()
                .next()
                .expect("matched position must contain a character")
                .len_utf8();
    }
    Some(score)
}
