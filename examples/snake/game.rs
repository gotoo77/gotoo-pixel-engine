use std::collections::VecDeque;
use std::time::Duration;

use gotoo_pixel_engine::{
    Frame, Framebuffer, Game, GameResult, Key, LocalStorage, MouseButton, Pixel, Rect, Size, Touch,
    TouchPhase,
};

pub const KEYBOARD_FRAMEBUFFER_WIDTH: u32 = 320;
pub const TOUCH_FRAMEBUFFER_WIDTH: u32 = 480;
pub const FRAMEBUFFER_HEIGHT: u32 = 204;

const GRID_WIDTH: i32 = 32;
const GRID_HEIGHT: i32 = 18;
const CELL_SIZE: i32 = 10;
const PLAYFIELD_WIDTH: u32 = GRID_WIDTH as u32 * CELL_SIZE as u32;
const PLAYFIELD_HEIGHT: u32 = GRID_HEIGHT as u32 * CELL_SIZE as u32;
const INITIAL_SEED: u32 = 0x5EED_1234;
const TURN_QUEUE_CAPACITY: usize = 2;
const MAX_CATCH_UP: usize = 5;
const TICK_PERIOD: Duration = Duration::from_millis(120);
const EXIT_KEY: Key = Key::Escape;
const RESTART_KEY: Key = Key::Space;
const BEST_SCORE_KEY: &str = "gotoo-pixel-engine.snake.best_score.v1";
const HUD_TEXT_SCALE: u32 = 1;
const GAME_OVER_TEXT_SCALE: u32 = 2;
const HUD_HEIGHT: u32 = 24;
const CONTROLS_WIDTH: u32 = 160;
const D_PAD_BUTTON_SIZE: u32 = 52;
const GAME_OVER_PANEL_SIZE: Size = Size {
    width: 192,
    height: 108,
};
const REPLAY_BUTTON_SIZE: Size = Size {
    width: 112,
    height: 28,
};

const BACKGROUND: Pixel = Pixel::rgb(10, 14, 18);
const GRID_LINE: Pixel = Pixel::rgb(20, 28, 34);
const BORDER: Pixel = Pixel::rgb(88, 102, 112);
const SNAKE_HEAD: Pixel = Pixel::rgb(214, 246, 128);
const SNAKE_BODY: Pixel = Pixel::rgb(82, 190, 118);
const FOOD: Pixel = Pixel::rgb(235, 74, 74);
const GAME_OVER: Pixel = Pixel::rgb(245, 66, 66);
const HUD_BACKDROP: Pixel = Pixel::rgb(7, 10, 13);
const HUD_TEXT: Pixel = Pixel::rgb(224, 232, 210);
const PANEL_FILL: Pixel = Pixel::rgb(12, 16, 20);
const BUTTON_FILL: Pixel = Pixel::rgb(28, 38, 46);
const BUTTON_BORDER: Pixel = Pixel::rgb(154, 174, 186);
const BUTTON_TEXT: Pixel = Pixel::rgb(240, 244, 230);
const D_PAD_FILL: Pixel = Pixel::rgb(13, 20, 24);
const D_PAD_CENTER_FILL: Pixel = Pixel::rgb(9, 13, 16);
const D_PAD_BORDER: Pixel = Pixel::rgb(93, 116, 128);
const D_PAD_ARROW: Pixel = Pixel::rgb(196, 216, 204);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SnakeInteractionMode {
    Keyboard,
    Touch,
}

impl SnakeInteractionMode {
    pub fn framebuffer_size(self) -> Size {
        SnakeLayout::for_mode(self).framebuffer_size
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SnakeLayout {
    framebuffer_size: Size,
    hud: Rect,
    playfield: Rect,
    controls: Option<Rect>,
    d_pad: Option<DPadLayout>,
    replay: Rect,
    game_over_panel: Rect,
}

impl SnakeLayout {
    fn for_mode(mode: SnakeInteractionMode) -> Self {
        match mode {
            SnakeInteractionMode::Keyboard => Self::keyboard(),
            SnakeInteractionMode::Touch => Self::touch(),
        }
    }

    fn keyboard() -> Self {
        Self::new(KEYBOARD_FRAMEBUFFER_WIDTH, None)
    }

    fn touch() -> Self {
        Self::new(TOUCH_FRAMEBUFFER_WIDTH, Some(CONTROLS_WIDTH))
    }

    fn new(framebuffer_width: u32, controls_width: Option<u32>) -> Self {
        let framebuffer_size = Size {
            width: framebuffer_width,
            height: FRAMEBUFFER_HEIGHT,
        };
        let hud = Rect {
            x: 0,
            y: 0,
            width: framebuffer_width,
            height: HUD_HEIGHT,
        };
        let playfield = Rect {
            x: 0,
            y: hud.height as i32,
            width: PLAYFIELD_WIDTH,
            height: PLAYFIELD_HEIGHT,
        };
        let controls = controls_width.map(|width| Rect {
            x: playfield.width as i32,
            y: playfield.y,
            width,
            height: playfield.height,
        });
        let d_pad = controls.map(DPadLayout::new);
        let game_over_panel = Rect {
            x: playfield.x + centered_offset(playfield.width, GAME_OVER_PANEL_SIZE.width),
            y: playfield.y + centered_offset(playfield.height, GAME_OVER_PANEL_SIZE.height),
            width: GAME_OVER_PANEL_SIZE.width,
            height: GAME_OVER_PANEL_SIZE.height,
        };
        let replay = Rect {
            x: playfield.x + centered_offset(playfield.width, REPLAY_BUTTON_SIZE.width),
            y: game_over_panel.y + 72,
            width: REPLAY_BUTTON_SIZE.width,
            height: REPLAY_BUTTON_SIZE.height,
        };

        Self {
            framebuffer_size,
            hud,
            playfield,
            controls,
            d_pad,
            replay,
            game_over_panel,
        }
    }

    fn cell_origin(self, cell: Cell) -> (i32, i32) {
        (
            self.playfield.x + cell.x * CELL_SIZE,
            self.playfield.y + cell.y * CELL_SIZE,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DPadLayout {
    up: Rect,
    left: Rect,
    center: Rect,
    right: Rect,
    down: Rect,
}

impl DPadLayout {
    fn new(controls: Rect) -> Self {
        let left_x = controls.x + 2;
        let center_x = left_x + D_PAD_BUTTON_SIZE as i32;
        let right_x = center_x + D_PAD_BUTTON_SIZE as i32;
        let center_y = controls.y + 64;
        let up_y = center_y - D_PAD_BUTTON_SIZE as i32;
        let down_y = center_y + D_PAD_BUTTON_SIZE as i32;

        Self {
            up: Rect {
                x: center_x,
                y: up_y,
                width: D_PAD_BUTTON_SIZE,
                height: D_PAD_BUTTON_SIZE,
            },
            left: Rect {
                x: left_x,
                y: center_y,
                width: D_PAD_BUTTON_SIZE,
                height: D_PAD_BUTTON_SIZE,
            },
            center: Rect {
                x: center_x,
                y: center_y,
                width: D_PAD_BUTTON_SIZE,
                height: D_PAD_BUTTON_SIZE,
            },
            right: Rect {
                x: right_x,
                y: center_y,
                width: D_PAD_BUTTON_SIZE,
                height: D_PAD_BUTTON_SIZE,
            },
            down: Rect {
                x: center_x,
                y: down_y,
                width: D_PAD_BUTTON_SIZE,
                height: D_PAD_BUTTON_SIZE,
            },
        }
    }

    fn direction_zones(self) -> [DirectionZone; 4] {
        [
            DirectionZone {
                rect: self.up,
                direction: Direction::Up,
            },
            DirectionZone {
                rect: self.left,
                direction: Direction::Left,
            },
            DirectionZone {
                rect: self.right,
                direction: Direction::Right,
            },
            DirectionZone {
                rect: self.down,
                direction: Direction::Down,
            },
        ]
    }
}

#[derive(Debug)]
pub struct SnakeGame {
    world: SnakeWorld,
    accumulator: Duration,
    interaction_mode: SnakeInteractionMode,
    touch_controls: TouchControls,
    best_score: u32,
    best_score_loaded: bool,
}

impl SnakeGame {
    pub fn new(interaction_mode: SnakeInteractionMode) -> Self {
        Self {
            world: SnakeWorld::new(INITIAL_SEED),
            accumulator: Duration::ZERO,
            interaction_mode,
            touch_controls: TouchControls::default(),
            best_score: 0,
            best_score_loaded: false,
        }
    }

    fn layout(&self) -> SnakeLayout {
        SnakeLayout::for_mode(self.interaction_mode)
    }

    fn update_logic(&mut self, delta_time: Duration, controls: SnakeControls) {
        if controls.restart && self.world.phase() == Phase::GameOver {
            self.restart();
        }

        for direction in controls.directions {
            self.world.queue_direction(direction);
        }

        if self.world.phase() != Phase::Running {
            return;
        }

        self.accumulator += delta_time;

        let mut ticks = 0;
        while self.accumulator >= TICK_PERIOD && ticks < MAX_CATCH_UP {
            self.world.tick();
            self.accumulator -= TICK_PERIOD;
            ticks += 1;

            if self.world.phase() != Phase::Running {
                self.touch_controls.reset_contact();
                break;
            }
        }
    }

    fn restart(&mut self) {
        self.world.restart();
        self.accumulator = Duration::ZERO;
        self.touch_controls.reset_contact();
    }

    fn load_best_score_once(&mut self, storage: &mut dyn LocalStorage) {
        if self.best_score_loaded {
            return;
        }

        self.best_score = read_best_score(storage);
        self.best_score_loaded = true;
    }

    fn persist_best_score_if_needed(&mut self, storage: &mut dyn LocalStorage) {
        let score = self.world.score();
        if score <= self.best_score {
            return;
        }

        self.best_score = score;
        let _ = storage.set(BEST_SCORE_KEY, &score.to_string());
    }

    fn draw(&self, frame: &mut Frame<'_>) {
        let framebuffer = &mut frame.framebuffer;
        let layout = self.layout();

        framebuffer.clear(BACKGROUND);
        draw_grid(framebuffer, layout);

        if let Some(food) = self.world.food() {
            let (cell_x, cell_y) = layout.cell_origin(food);
            let center_x = cell_x + CELL_SIZE / 2;
            let center_y = cell_y + CELL_SIZE / 2;
            framebuffer.fill_circle(center_x, center_y, 4, FOOD);
        }

        for (index, cell) in self.world.snake().iter().enumerate().rev() {
            let color = if index == 0 { SNAKE_HEAD } else { SNAKE_BODY };
            let (cell_x, cell_y) = layout.cell_origin(*cell);
            framebuffer.fill_rect(
                cell_x + 1,
                cell_y + 1,
                (CELL_SIZE - 2) as u32,
                (CELL_SIZE - 2) as u32,
                color,
            );
        }

        framebuffer.draw_rect(
            layout.playfield.x,
            layout.playfield.y,
            layout.playfield.width,
            layout.playfield.height,
            BORDER,
        );
        draw_score_hud(framebuffer, layout, self.world.score(), self.best_score);

        if self.world.phase() == Phase::GameOver {
            draw_game_over(framebuffer, layout, self.world.score());
        } else if self.touch_controls.visible() && layout.d_pad.is_some() {
            draw_d_pad(framebuffer, layout);
        }
    }
}

impl Game for SnakeGame {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.load_best_score_once(frame.storage);

        if frame.input.key(EXIT_KEY).pressed() {
            return GameResult::Exit;
        }

        let layout = self.layout();
        let controls =
            SnakeControls::from_frame(frame, self.world.phase(), &mut self.touch_controls, layout);
        self.update_logic(frame.delta_time, controls);
        self.persist_best_score_if_needed(frame.storage);
        self.draw(frame);

        GameResult::Continue
    }
}

fn draw_grid(framebuffer: &mut Framebuffer, layout: SnakeLayout) {
    for x in 1..GRID_WIDTH {
        let pixel_x = layout.playfield.x + x * CELL_SIZE;
        framebuffer.draw_line(
            pixel_x,
            layout.playfield.y,
            pixel_x,
            layout.playfield.y + layout.playfield.height as i32 - 1,
            GRID_LINE,
        );
    }
    for y in 1..GRID_HEIGHT {
        let pixel_y = layout.playfield.y + y * CELL_SIZE;
        framebuffer.draw_line(
            layout.playfield.x,
            pixel_y,
            layout.playfield.x + layout.playfield.width as i32 - 1,
            pixel_y,
            GRID_LINE,
        );
    }
}

fn read_best_score(storage: &mut dyn LocalStorage) -> u32 {
    storage
        .get(BEST_SCORE_KEY)
        .ok()
        .flatten()
        .and_then(|value| value.trim().parse::<u32>().ok())
        .unwrap_or(0)
}

fn draw_score_hud(framebuffer: &mut Framebuffer, layout: SnakeLayout, score: u32, best_score: u32) {
    let text = format!("SCORE {score}    BEST {best_score}");
    let (_, height) = Framebuffer::text_size(&text, HUD_TEXT_SCALE);
    let text_y = layout.hud.y + centered_offset(layout.hud.height, height);

    framebuffer.fill_rect(
        layout.hud.x,
        layout.hud.y,
        layout.hud.width,
        layout.hud.height,
        HUD_BACKDROP,
    );
    framebuffer.draw_rect(
        layout.hud.x,
        layout.hud.y,
        layout.hud.width,
        layout.hud.height,
        BORDER,
    );
    framebuffer.draw_text_scaled(4, text_y, &text, HUD_TEXT_SCALE, HUD_TEXT);
}

fn draw_game_over(framebuffer: &mut Framebuffer, layout: SnakeLayout, score: u32) {
    framebuffer.fill_rect(
        layout.game_over_panel.x,
        layout.game_over_panel.y,
        layout.game_over_panel.width,
        layout.game_over_panel.height,
        PANEL_FILL,
    );
    framebuffer.draw_rect(
        layout.game_over_panel.x,
        layout.game_over_panel.y,
        layout.game_over_panel.width,
        layout.game_over_panel.height,
        GAME_OVER,
    );

    draw_centered_text_in_rect(
        framebuffer,
        layout.playfield,
        layout.game_over_panel.y + 14,
        "GAME OVER",
        GAME_OVER_TEXT_SCALE,
        GAME_OVER,
    );
    draw_centered_text_in_rect(
        framebuffer,
        layout.playfield,
        layout.game_over_panel.y + 40,
        &format!("SCORE {score}"),
        GAME_OVER_TEXT_SCALE,
        HUD_TEXT,
    );
    draw_replay_button(framebuffer, layout);
}

fn draw_replay_button(framebuffer: &mut Framebuffer, layout: SnakeLayout) {
    framebuffer.fill_rect(
        layout.replay.x,
        layout.replay.y,
        layout.replay.width,
        layout.replay.height,
        BUTTON_FILL,
    );
    framebuffer.draw_rect(
        layout.replay.x,
        layout.replay.y,
        layout.replay.width,
        layout.replay.height,
        BUTTON_BORDER,
    );

    let text = "REJOUER";
    let (text_width, text_height) = Framebuffer::text_size(text, GAME_OVER_TEXT_SCALE);
    let text_x = layout.replay.x + centered_offset(layout.replay.width, text_width);
    let text_y = layout.replay.y + centered_offset(layout.replay.height, text_height);
    framebuffer.draw_text_scaled(text_x, text_y, text, GAME_OVER_TEXT_SCALE, BUTTON_TEXT);
}

fn draw_d_pad(framebuffer: &mut Framebuffer, layout: SnakeLayout) {
    let Some(controls) = layout.controls else {
        return;
    };
    let Some(d_pad) = layout.d_pad else {
        return;
    };

    framebuffer.draw_rect(
        controls.x,
        controls.y,
        controls.width,
        controls.height,
        BORDER,
    );

    for zone in d_pad.direction_zones() {
        draw_d_pad_button(framebuffer, zone.rect, zone.direction);
    }

    framebuffer.fill_rect(
        d_pad.center.x,
        d_pad.center.y,
        d_pad.center.width,
        d_pad.center.height,
        D_PAD_CENTER_FILL,
    );
    framebuffer.draw_rect(
        d_pad.center.x,
        d_pad.center.y,
        d_pad.center.width,
        d_pad.center.height,
        D_PAD_BORDER,
    );
}

fn draw_d_pad_button(framebuffer: &mut Framebuffer, rect: Rect, direction: Direction) {
    framebuffer.fill_rect(rect.x, rect.y, rect.width, rect.height, D_PAD_FILL);
    framebuffer.draw_rect(rect.x, rect.y, rect.width, rect.height, D_PAD_BORDER);
    draw_d_pad_arrow(framebuffer, rect, direction);
}

fn draw_d_pad_arrow(framebuffer: &mut Framebuffer, rect: Rect, direction: Direction) {
    let center_x = rect.x + rect.width as i32 / 2;
    let center_y = rect.y + rect.height as i32 / 2;

    match direction {
        Direction::Up => {
            framebuffer.draw_line(
                center_x,
                center_y - 9,
                center_x - 8,
                center_y + 5,
                D_PAD_ARROW,
            );
            framebuffer.draw_line(
                center_x,
                center_y - 9,
                center_x + 8,
                center_y + 5,
                D_PAD_ARROW,
            );
            framebuffer.draw_line(center_x, center_y - 9, center_x, center_y + 10, D_PAD_ARROW);
        }
        Direction::Down => {
            framebuffer.draw_line(
                center_x,
                center_y + 9,
                center_x - 8,
                center_y - 5,
                D_PAD_ARROW,
            );
            framebuffer.draw_line(
                center_x,
                center_y + 9,
                center_x + 8,
                center_y - 5,
                D_PAD_ARROW,
            );
            framebuffer.draw_line(center_x, center_y + 9, center_x, center_y - 10, D_PAD_ARROW);
        }
        Direction::Left => {
            framebuffer.draw_line(
                center_x - 9,
                center_y,
                center_x + 5,
                center_y - 8,
                D_PAD_ARROW,
            );
            framebuffer.draw_line(
                center_x - 9,
                center_y,
                center_x + 5,
                center_y + 8,
                D_PAD_ARROW,
            );
            framebuffer.draw_line(center_x - 9, center_y, center_x + 10, center_y, D_PAD_ARROW);
        }
        Direction::Right => {
            framebuffer.draw_line(
                center_x + 9,
                center_y,
                center_x - 5,
                center_y - 8,
                D_PAD_ARROW,
            );
            framebuffer.draw_line(
                center_x + 9,
                center_y,
                center_x - 5,
                center_y + 8,
                D_PAD_ARROW,
            );
            framebuffer.draw_line(center_x + 9, center_y, center_x - 10, center_y, D_PAD_ARROW);
        }
    }
}

fn draw_centered_text_in_rect(
    framebuffer: &mut Framebuffer,
    rect: Rect,
    y: i32,
    text: &str,
    scale: u32,
    pixel: Pixel,
) {
    let (width, _) = Framebuffer::text_size(text, scale);
    let x = rect.x + centered_offset(rect.width, width);
    framebuffer.draw_text_scaled(x, y, text, scale, pixel);
}

fn centered_offset(container_width: u32, content_width: u32) -> i32 {
    container_width.saturating_sub(content_width) as i32 / 2
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SnakeControls {
    directions: Vec<Direction>,
    restart: bool,
}

impl SnakeControls {
    fn none() -> Self {
        Self {
            directions: Vec::new(),
            restart: false,
        }
    }

    fn from_frame(
        frame: &Frame<'_>,
        phase: Phase,
        touch_controls: &mut TouchControls,
        layout: SnakeLayout,
    ) -> Self {
        touch_controls.observe_touches(frame.input.touches());

        match phase {
            Phase::Running => Self::from_running_frame(frame, touch_controls, layout),
            Phase::GameOver => Self::from_game_over_inputs_in_layout(
                frame.input.key(RESTART_KEY).pressed(),
                frame.input.mouse_button(MouseButton::Left).pressed(),
                frame.input.mouse_position(),
                frame.input.touches(),
                touch_controls,
                layout,
            ),
        }
    }

    fn from_running_frame(
        frame: &Frame<'_>,
        touch_controls: &mut TouchControls,
        layout: SnakeLayout,
    ) -> Self {
        let mut controls = Self::none();

        for key in DIRECTION_KEYS {
            if frame.input.key(key).pressed()
                && let Some(direction) = direction_for_key(key)
            {
                controls.directions.push(direction);
            }
        }

        controls
            .directions
            .extend(touch_controls.directions_from_touches(frame.input.touches(), layout));
        controls.restart = frame.input.key(RESTART_KEY).pressed();
        controls
    }

    #[cfg(test)]
    fn from_game_over_inputs(
        keyboard_restart_pressed: bool,
        mouse_left_pressed: bool,
        mouse_position: Option<(i32, i32)>,
        touches: &[Touch],
        touch_controls: &mut TouchControls,
    ) -> Self {
        Self::from_game_over_inputs_in_layout(
            keyboard_restart_pressed,
            mouse_left_pressed,
            mouse_position,
            touches,
            touch_controls,
            SnakeLayout::touch(),
        )
    }

    fn from_game_over_inputs_in_layout(
        keyboard_restart_pressed: bool,
        mouse_left_pressed: bool,
        mouse_position: Option<(i32, i32)>,
        touches: &[Touch],
        touch_controls: &mut TouchControls,
        layout: SnakeLayout,
    ) -> Self {
        touch_controls.reset_contact();

        Self {
            directions: Vec::new(),
            restart: replay_requested(
                keyboard_restart_pressed,
                mouse_left_pressed,
                mouse_position,
                touches,
                layout,
            ),
        }
    }

    #[cfg(test)]
    fn with_directions(directions: impl IntoIterator<Item = Direction>) -> Self {
        Self {
            directions: directions.into_iter().collect(),
            restart: false,
        }
    }

    #[cfg(test)]
    fn restart() -> Self {
        Self {
            directions: Vec::new(),
            restart: true,
        }
    }
}

const DIRECTION_KEYS: [Key; 8] = [
    Key::Up,
    Key::W,
    Key::Right,
    Key::D,
    Key::Down,
    Key::S,
    Key::Left,
    Key::A,
];

fn direction_for_key(key: Key) -> Option<Direction> {
    match key {
        Key::Up | Key::W => Some(Direction::Up),
        Key::Right | Key::D => Some(Direction::Right),
        Key::Down | Key::S => Some(Direction::Down),
        Key::Left | Key::A => Some(Direction::Left),
        _ => None,
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct TouchControls {
    visible: bool,
    d_pad: DPadTracker,
}

impl TouchControls {
    fn observe_touches(&mut self, touches: &[Touch]) {
        if !touches.is_empty() {
            self.visible = true;
        }
    }

    fn visible(&self) -> bool {
        self.visible
    }

    fn reset_contact(&mut self) {
        self.d_pad.reset();
    }

    fn directions_from_touches(
        &mut self,
        touches: &[Touch],
        layout: SnakeLayout,
    ) -> Vec<Direction> {
        self.d_pad
            .directions_from_touches_in_layout(touches, layout)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct DPadTracker {
    active: Option<DPadContact>,
}

impl DPadTracker {
    fn reset(&mut self) {
        self.active = None;
    }

    #[cfg(test)]
    fn directions_from_touches(&mut self, touches: &[Touch]) -> Vec<Direction> {
        self.directions_from_touches_in_layout(touches, SnakeLayout::touch())
    }

    fn directions_from_touches_in_layout(
        &mut self,
        touches: &[Touch],
        layout: SnakeLayout,
    ) -> Vec<Direction> {
        let mut directions = Vec::new();

        for touch in touches {
            match touch.phase {
                TouchPhase::Started => {
                    if let Some(direction) = self.start(touch, layout) {
                        directions.push(direction);
                    }
                }
                TouchPhase::Moved => {
                    if let Some(direction) = self.move_active(touch, layout) {
                        directions.push(direction);
                    }
                }
                TouchPhase::Ended => self.end_active(touch),
                TouchPhase::Cancelled => self.cancel_active(touch),
            }
        }

        directions
    }

    fn start(&mut self, touch: &Touch, layout: SnakeLayout) -> Option<Direction> {
        if self.active.is_some() {
            return None;
        }

        let position = touch.position?;
        let zone = d_pad_zone_at_in_layout(position, layout)?;

        self.active = Some(DPadContact {
            id: touch.id,
            last_direction: zone.direction,
        });

        zone.direction
    }

    fn move_active(&mut self, touch: &Touch, layout: SnakeLayout) -> Option<Direction> {
        let contact = self.active.as_mut()?;
        if contact.id != touch.id {
            return None;
        }

        let position = touch.position?;
        let zone = d_pad_zone_at_in_layout(position, layout)?;
        let direction = zone.direction?;

        if contact.last_direction == Some(direction) {
            return None;
        }

        contact.last_direction = Some(direction);
        Some(direction)
    }

    fn end_active(&mut self, touch: &Touch) {
        if self
            .active
            .as_ref()
            .is_some_and(|contact| contact.id == touch.id)
        {
            self.active = None;
        }
    }

    fn cancel_active(&mut self, touch: &Touch) {
        self.end_active(touch);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DirectionZone {
    rect: Rect,
    direction: Direction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DPadZone {
    direction: Option<Direction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DPadContact {
    id: u64,
    last_direction: Option<Direction>,
}

#[cfg(test)]
fn d_pad_zone_at(position: (i32, i32)) -> Option<DPadZone> {
    d_pad_zone_at_in_layout(position, SnakeLayout::touch())
}

fn d_pad_zone_at_in_layout(position: (i32, i32), layout: SnakeLayout) -> Option<DPadZone> {
    let d_pad = layout.d_pad?;

    for zone in d_pad.direction_zones() {
        if zone.rect.contains(position) {
            return Some(DPadZone {
                direction: Some(zone.direction),
            });
        }
    }

    if d_pad.center.contains(position) {
        return Some(DPadZone { direction: None });
    }

    None
}

fn replay_requested(
    keyboard_restart_pressed: bool,
    mouse_left_pressed: bool,
    mouse_position: Option<(i32, i32)>,
    touches: &[Touch],
    layout: SnakeLayout,
) -> bool {
    keyboard_restart_pressed
        || (mouse_left_pressed
            && mouse_position.is_some_and(|position| layout.replay.contains(position)))
        || touches.iter().any(|touch| {
            touch.phase == TouchPhase::Started
                && touch
                    .position
                    .is_some_and(|position| layout.replay.contains(position))
        })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SnakeWorld {
    phase: Phase,
    snake: VecDeque<Cell>,
    direction: Direction,
    turn_queue: VecDeque<Direction>,
    food: Option<Cell>,
    score: u32,
    rng: FoodRng,
}

impl SnakeWorld {
    fn new(seed: u32) -> Self {
        let mut world = Self {
            phase: Phase::Running,
            snake: VecDeque::new(),
            direction: Direction::Right,
            turn_queue: VecDeque::new(),
            food: None,
            score: 0,
            rng: FoodRng::new(seed),
        };
        world.restart();
        world
    }

    fn restart(&mut self) {
        self.phase = Phase::Running;
        self.snake = initial_snake();
        self.direction = Direction::Right;
        self.turn_queue.clear();
        self.score = 0;
        self.food = self.place_food();
    }

    fn phase(&self) -> Phase {
        self.phase
    }

    fn snake(&self) -> &VecDeque<Cell> {
        &self.snake
    }

    fn food(&self) -> Option<Cell> {
        self.food
    }

    fn score(&self) -> u32 {
        self.score
    }

    fn queue_direction(&mut self, direction: Direction) {
        let last_direction = self.turn_queue.back().copied().unwrap_or(self.direction);
        if direction == last_direction || direction == last_direction.opposite() {
            return;
        }
        if self.turn_queue.len() >= TURN_QUEUE_CAPACITY {
            return;
        }

        self.turn_queue.push_back(direction);
    }

    fn tick(&mut self) {
        if self.phase != Phase::Running {
            return;
        }

        if let Some(direction) = self.turn_queue.pop_front() {
            self.direction = direction;
        }

        let next_head = self.snake[0].next(self.direction);
        let will_grow = self.food == Some(next_head);

        if !next_head.is_inside_grid() || self.collides_with_body(next_head, will_grow) {
            self.phase = Phase::GameOver;
            return;
        }

        self.snake.push_front(next_head);

        if will_grow {
            self.score += 1;
            self.food = self.place_food();
        } else {
            self.snake.pop_back();
        }
    }

    fn collides_with_body(&self, cell: Cell, will_grow: bool) -> bool {
        let cells_to_check = if will_grow {
            self.snake.len()
        } else {
            self.snake.len().saturating_sub(1)
        };

        self.snake
            .iter()
            .take(cells_to_check)
            .any(|snake_cell| *snake_cell == cell)
    }

    fn place_food(&mut self) -> Option<Cell> {
        let free_cells = self.free_cells();
        if free_cells.is_empty() {
            return None;
        }

        let index = self.rng.next_index(free_cells.len());
        Some(free_cells[index])
    }

    fn free_cells(&self) -> Vec<Cell> {
        let mut cells = Vec::new();

        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let cell = Cell { x, y };
                if !self.snake.iter().any(|snake_cell| *snake_cell == cell) {
                    cells.push(cell);
                }
            }
        }

        cells
    }
}

fn initial_snake() -> VecDeque<Cell> {
    VecDeque::from([
        Cell { x: 16, y: 9 },
        Cell { x: 15, y: 9 },
        Cell { x: 14, y: 9 },
    ])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cell {
    x: i32,
    y: i32,
}

impl Cell {
    fn next(self, direction: Direction) -> Self {
        let (dx, dy) = direction.offset();
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    fn is_inside_grid(self) -> bool {
        self.x >= 0 && self.y >= 0 && self.x < GRID_WIDTH && self.y < GRID_HEIGHT
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn offset(self) -> (i32, i32) {
        match self {
            Self::Up => (0, -1),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
        }
    }

    fn opposite(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Running,
    GameOver,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FoodRng {
    state: u32,
}

impl FoodRng {
    fn new(seed: u32) -> Self {
        Self { state: seed.max(1) }
    }

    fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x.max(1);
        x
    }

    fn next_index(&mut self, upper_bound: usize) -> usize {
        debug_assert!(upper_bound > 0);
        self.next_u32() as usize % upper_bound
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::collections::VecDeque;
    use std::time::Duration;

    use super::{
        BEST_SCORE_KEY, BUTTON_BORDER, BUTTON_TEXT, CELL_SIZE, Cell, D_PAD_ARROW,
        D_PAD_CENTER_FILL, D_PAD_FILL, DPadTracker, Direction, EXIT_KEY, FOOD, FRAMEBUFFER_HEIGHT,
        GAME_OVER, GRID_LINE, HUD_BACKDROP, HUD_TEXT, KEYBOARD_FRAMEBUFFER_WIDTH, PANEL_FILL,
        Phase, RESTART_KEY, SNAKE_BODY, SNAKE_HEAD, SnakeControls, SnakeGame, SnakeInteractionMode,
        SnakeLayout, SnakeWorld, TICK_PERIOD, TOUCH_FRAMEBUFFER_WIDTH, TURN_QUEUE_CAPACITY,
        TouchControls, d_pad_zone_at, d_pad_zone_at_in_layout, direction_for_key, draw_d_pad,
        draw_game_over, draw_grid, draw_score_hud,
    };
    use gotoo_pixel_engine::{
        Frame, Framebuffer, Game, GameResult, Input, Key, LocalStorage, Pixel, Rect, Size,
        StorageError, Touch, TouchPhase, Viewport,
    };

    #[derive(Debug, Default, Clone, PartialEq, Eq)]
    struct TestStorage {
        entries: HashMap<String, String>,
        fail_reads: bool,
        fail_writes: bool,
    }

    impl TestStorage {
        fn with_entry(key: &str, value: &str) -> Self {
            let mut storage = Self::default();
            storage.entries.insert(key.into(), value.into());
            storage
        }

        fn failing_reads() -> Self {
            Self {
                fail_reads: true,
                ..Self::default()
            }
        }

        fn failing_writes() -> Self {
            Self {
                fail_writes: true,
                ..Self::default()
            }
        }
    }

    impl LocalStorage for TestStorage {
        fn get(&mut self, key: &str) -> Result<Option<String>, StorageError> {
            if self.fail_reads {
                return Err(StorageError::new("read failed"));
            }

            Ok(self.entries.get(key).cloned())
        }

        fn set(&mut self, key: &str, value: &str) -> Result<(), StorageError> {
            if self.fail_writes {
                return Err(StorageError::new("write failed"));
            }

            self.entries.insert(key.into(), value.into());
            Ok(())
        }
    }

    fn head(game: &SnakeGame) -> Cell {
        game.world.snake[0]
    }

    fn tick(game: &mut SnakeGame) {
        game.update_logic(TICK_PERIOD, SnakeControls::none());
    }

    fn keyboard_game() -> SnakeGame {
        SnakeGame::new(SnakeInteractionMode::Keyboard)
    }

    fn touch_game() -> SnakeGame {
        SnakeGame::new(SnakeInteractionMode::Touch)
    }

    fn update_with_storage(
        game: &mut SnakeGame,
        storage: &mut dyn LocalStorage,
        delta_time: Duration,
    ) -> GameResult {
        let framebuffer_size = game.layout().framebuffer_size;
        update_with_storage_and_surface(game, storage, delta_time, framebuffer_size)
    }

    fn update_with_storage_and_surface(
        game: &mut SnakeGame,
        storage: &mut dyn LocalStorage,
        delta_time: Duration,
        surface_size: Size,
    ) -> GameResult {
        let framebuffer_size = game.layout().framebuffer_size;
        let mut framebuffer = Framebuffer::new(framebuffer_size.width, framebuffer_size.height);
        let input = Input::default();
        let mut frame = Frame {
            framebuffer: &mut framebuffer,
            input: &input,
            delta_time,
            surface_size,
            viewport: Viewport::new(surface_size, framebuffer_size),
            storage,
        };

        game.update(&mut frame)
    }

    fn touch_layout() -> SnakeLayout {
        SnakeLayout::touch()
    }

    fn keyboard_layout() -> SnakeLayout {
        SnakeLayout::keyboard()
    }

    fn replay_button_center() -> (i32, i32) {
        let replay = touch_layout().replay;

        (
            replay.x + replay.width as i32 / 2,
            replay.y + replay.height as i32 / 2,
        )
    }

    fn keyboard_replay_button_center() -> (i32, i32) {
        let replay = keyboard_layout().replay;

        (
            replay.x + replay.width as i32 / 2,
            replay.y + replay.height as i32 / 2,
        )
    }

    fn touch(id: u64, phase: TouchPhase, position: Option<(i32, i32)>) -> Touch {
        Touch {
            id,
            phase,
            position,
        }
    }

    fn d_pad_up() -> Rect {
        touch_layout()
            .d_pad
            .expect("touch layout should have a D-pad")
            .up
    }

    fn d_pad_down() -> Rect {
        touch_layout()
            .d_pad
            .expect("touch layout should have a D-pad")
            .down
    }

    fn d_pad_left() -> Rect {
        touch_layout()
            .d_pad
            .expect("touch layout should have a D-pad")
            .left
    }

    fn d_pad_right() -> Rect {
        touch_layout()
            .d_pad
            .expect("touch layout should have a D-pad")
            .right
    }

    fn d_pad_center() -> Rect {
        touch_layout()
            .d_pad
            .expect("touch layout should have a D-pad")
            .center
    }

    fn rect_center(rect: Rect) -> (i32, i32) {
        (
            rect.x + rect.width as i32 / 2,
            rect.y + rect.height as i32 / 2,
        )
    }

    fn active_d_pad_contact() -> super::DPadContact {
        super::DPadContact {
            id: 1,
            last_direction: Some(Direction::Right),
        }
    }

    fn touch_controls_with_active_contact() -> TouchControls {
        TouchControls {
            visible: true,
            d_pad: DPadTracker {
                active: Some(active_d_pad_contact()),
            },
        }
    }

    fn count_pixels(framebuffer: &Framebuffer, pixel: Pixel) -> usize {
        let mut count = 0;

        for y in 0..framebuffer.height() as i32 {
            for x in 0..framebuffer.width() as i32 {
                if framebuffer.pixel(x, y) == Some(pixel) {
                    count += 1;
                }
            }
        }

        count
    }

    fn count_pixels_in_rect(framebuffer: &Framebuffer, pixel: Pixel, rect: Rect) -> usize {
        let mut count = 0;

        for y in rect.y..rect.y + rect.height as i32 {
            for x in rect.x..rect.x + rect.width as i32 {
                if framebuffer.pixel(x, y) == Some(pixel) {
                    count += 1;
                }
            }
        }

        count
    }

    fn count_pixels_outside_rect(framebuffer: &Framebuffer, pixel: Pixel, rect: Rect) -> usize {
        let mut count = 0;

        for y in 0..framebuffer.height() as i32 {
            for x in 0..framebuffer.width() as i32 {
                if !rect.contains((x, y)) && framebuffer.pixel(x, y) == Some(pixel) {
                    count += 1;
                }
            }
        }

        count
    }

    fn assert_color_only_inside(framebuffer: &Framebuffer, pixel: Pixel, rect: Rect) {
        assert!(count_pixels_in_rect(framebuffer, pixel, rect) > 0);
        assert_eq!(count_pixels_outside_rect(framebuffer, pixel, rect), 0);
    }

    fn d_pad_direction_at(position: (i32, i32)) -> Option<Direction> {
        d_pad_zone_at(position).and_then(|zone| zone.direction)
    }

    #[test]
    fn keyboard_layout_uses_expected_framebuffer_and_zones() {
        let layout = keyboard_layout();

        assert_eq!(KEYBOARD_FRAMEBUFFER_WIDTH, 320);
        assert_eq!(FRAMEBUFFER_HEIGHT, 204);
        assert_eq!(
            layout.framebuffer_size,
            Size {
                width: 320,
                height: 204
            }
        );
        assert_eq!(
            layout.hud,
            Rect {
                x: 0,
                y: 0,
                width: 320,
                height: 24
            }
        );
        assert_eq!(
            layout.playfield,
            Rect {
                x: 0,
                y: 24,
                width: 320,
                height: 180
            }
        );
        assert_eq!(layout.controls, None);
        assert_eq!(layout.d_pad, None);
    }

    #[test]
    fn touch_layout_uses_expected_framebuffer_and_zones() {
        let layout = touch_layout();

        assert_eq!(TOUCH_FRAMEBUFFER_WIDTH, 480);
        assert_eq!(FRAMEBUFFER_HEIGHT, 204);
        assert_eq!(
            layout.framebuffer_size,
            Size {
                width: 480,
                height: 204
            }
        );
        assert_eq!(
            layout.hud,
            Rect {
                x: 0,
                y: 0,
                width: 480,
                height: 24
            }
        );
        assert_eq!(
            layout.playfield,
            Rect {
                x: 0,
                y: 24,
                width: 320,
                height: 180
            }
        );
        assert_eq!(
            layout.controls,
            Some(Rect {
                x: 320,
                y: 24,
                width: 160,
                height: 180
            })
        );
        assert!(layout.d_pad.is_some());
    }

    #[test]
    fn interaction_modes_expose_expected_framebuffer_sizes() {
        assert_eq!(
            SnakeInteractionMode::Keyboard.framebuffer_size(),
            Size {
                width: 320,
                height: 204
            }
        );
        assert_eq!(
            SnakeInteractionMode::Touch.framebuffer_size(),
            Size {
                width: 480,
                height: 204
            }
        );
    }

    #[test]
    fn examples_engine_config_dimensions_are_derived_from_interaction_mode() {
        let keyboard_size = SnakeInteractionMode::Keyboard.framebuffer_size();
        let touch_size = SnakeInteractionMode::Touch.framebuffer_size();

        assert_eq!(
            (
                keyboard_size.width,
                keyboard_size.height,
                keyboard_size.width * 3,
                keyboard_size.height * 3,
            ),
            (320, 204, 960, 612)
        );
        assert_eq!(
            (
                touch_size.width,
                touch_size.height,
                touch_size.width * 3,
                touch_size.height * 3,
            ),
            (480, 204, 1440, 612)
        );
    }

    #[test]
    fn playfield_and_world_dimensions_stay_fixed_in_both_modes() {
        for layout in [keyboard_layout(), touch_layout()] {
            assert_eq!(
                layout.playfield,
                Rect {
                    x: 0,
                    y: 24,
                    width: 320,
                    height: 180
                }
            );
            assert_eq!(layout.playfield.width as i32 / CELL_SIZE, super::GRID_WIDTH);
            assert_eq!(
                layout.playfield.height as i32 / CELL_SIZE,
                super::GRID_HEIGHT
            );
        }
    }

    #[test]
    fn layout_zones_do_not_overlap_the_playfield() {
        let layout = touch_layout();
        let controls = layout.controls.expect("touch layout should have controls");
        let d_pad = layout.d_pad.expect("touch layout should have a D-pad");

        assert!(!layout.hud.intersects(layout.playfield));
        assert!(!controls.intersects(layout.playfield));
        for rect in [d_pad.up, d_pad.left, d_pad.center, d_pad.right, d_pad.down] {
            assert!(controls.contains((rect.x, rect.y)));
            assert!(controls.contains((
                rect.x + rect.width as i32 - 1,
                rect.y + rect.height as i32 - 1,
            )));
            assert!(!rect.intersects(layout.playfield));
        }
    }

    #[test]
    fn cell_origin_includes_playfield_offset() {
        let layout = touch_layout();

        assert_eq!(layout.cell_origin(Cell { x: 0, y: 0 }), (0, 24));
        assert_eq!(layout.cell_origin(Cell { x: 1, y: 0 }), (CELL_SIZE, 24));
        assert_eq!(layout.cell_origin(Cell { x: 0, y: 1 }), (0, 24 + CELL_SIZE));
        assert_eq!(
            layout.cell_origin(Cell { x: 31, y: 17 }),
            (
                layout.playfield.x + 31 * CELL_SIZE,
                layout.playfield.y + 17 * CELL_SIZE
            )
        );
    }

    #[test]
    fn grid_is_drawn_only_inside_playfield() {
        let mut framebuffer = Framebuffer::new(TOUCH_FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT);
        let layout = touch_layout();

        draw_grid(&mut framebuffer, layout);

        assert_color_only_inside(&framebuffer, GRID_LINE, layout.playfield);
        assert_eq!(
            framebuffer.pixel(layout.playfield.x + CELL_SIZE, layout.playfield.y),
            Some(GRID_LINE)
        );
        assert_eq!(
            framebuffer.pixel(layout.playfield.x, layout.playfield.y + CELL_SIZE),
            Some(GRID_LINE)
        );
    }

    #[test]
    fn snake_and_food_are_rendered_only_inside_playfield() {
        let mut game = touch_game();
        game.world.food = Some(Cell { x: 4, y: 5 });
        let mut framebuffer = Framebuffer::new(TOUCH_FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT);
        let mut storage = TestStorage::default();
        let surface_size = Size {
            width: TOUCH_FRAMEBUFFER_WIDTH,
            height: FRAMEBUFFER_HEIGHT,
        };
        let layout = touch_layout();

        game.draw(&mut gotoo_pixel_engine::Frame {
            framebuffer: &mut framebuffer,
            input: &gotoo_pixel_engine::Input::default(),
            delta_time: Duration::ZERO,
            surface_size,
            viewport: Viewport::new(surface_size, surface_size),
            storage: &mut storage,
        });

        assert_color_only_inside(&framebuffer, FOOD, layout.playfield);
        assert_color_only_inside(&framebuffer, SNAKE_HEAD, layout.playfield);
        assert_color_only_inside(&framebuffer, SNAKE_BODY, layout.playfield);
    }

    #[test]
    fn hud_is_drawn_only_inside_hud_rect() {
        let mut framebuffer = Framebuffer::new(TOUCH_FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT);
        let layout = touch_layout();

        draw_score_hud(&mut framebuffer, layout, 12, 37);

        assert_color_only_inside(&framebuffer, HUD_BACKDROP, layout.hud);
        assert_color_only_inside(&framebuffer, HUD_TEXT, layout.hud);
        assert_eq!(
            count_pixels_in_rect(&framebuffer, HUD_TEXT, layout.playfield),
            0
        );
    }

    #[test]
    fn hud_has_room_for_future_best_score_text() {
        let (width, height) = Framebuffer::text_size("SCORE 12    BEST 37", super::HUD_TEXT_SCALE);
        let layout = touch_layout();

        assert!(width + 8 <= layout.hud.width);
        assert!(height <= layout.hud.height);
    }

    #[test]
    fn d_pad_is_drawn_only_inside_controls_rect() {
        let mut framebuffer = Framebuffer::new(TOUCH_FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT);
        let layout = touch_layout();
        let controls = layout.controls.expect("touch layout should have controls");

        draw_d_pad(&mut framebuffer, layout);

        assert_color_only_inside(&framebuffer, D_PAD_FILL, controls);
        assert_color_only_inside(&framebuffer, D_PAD_CENTER_FILL, controls);
        assert_color_only_inside(&framebuffer, D_PAD_ARROW, controls);
        assert_eq!(
            count_pixels_in_rect(&framebuffer, D_PAD_ARROW, layout.playfield),
            0
        );
    }

    #[test]
    fn d_pad_is_not_drawn_in_keyboard_mode() {
        let mut game = keyboard_game();
        game.touch_controls.visible = true;
        let mut framebuffer = Framebuffer::new(KEYBOARD_FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT);
        let mut storage = TestStorage::default();
        let surface_size = SnakeInteractionMode::Keyboard.framebuffer_size();

        game.draw(&mut gotoo_pixel_engine::Frame {
            framebuffer: &mut framebuffer,
            input: &gotoo_pixel_engine::Input::default(),
            delta_time: Duration::ZERO,
            surface_size,
            viewport: Viewport::new(surface_size, surface_size),
            storage: &mut storage,
        });

        assert_eq!(count_pixels(&framebuffer, D_PAD_FILL), 0);
        assert_eq!(count_pixels(&framebuffer, D_PAD_CENTER_FILL), 0);
        assert_eq!(count_pixels(&framebuffer, D_PAD_ARROW), 0);
    }

    #[test]
    fn d_pad_hit_test_requires_a_layout_with_d_pad() {
        let position = rect_center(d_pad_right());

        assert_eq!(
            d_pad_zone_at_in_layout(position, touch_layout()).and_then(|zone| zone.direction),
            Some(Direction::Right)
        );
        assert_eq!(d_pad_zone_at_in_layout(position, keyboard_layout()), None);
    }

    #[test]
    fn game_over_panel_and_replay_are_centered_in_playfield() {
        for layout in [keyboard_layout(), touch_layout()] {
            assert_eq!(
                layout.game_over_panel.x,
                layout.playfield.x
                    + (layout.playfield.width - layout.game_over_panel.width) as i32 / 2
            );
            assert_eq!(
                layout.game_over_panel.y,
                layout.playfield.y
                    + (layout.playfield.height - layout.game_over_panel.height) as i32 / 2
            );
            assert!(
                layout
                    .playfield
                    .contains((layout.replay.x, layout.replay.y))
            );
            assert!(layout.playfield.contains((
                layout.replay.x + layout.replay.width as i32 - 1,
                layout.replay.y + layout.replay.height as i32 - 1,
            )));
        }
    }

    #[test]
    fn keyboard_direction_controls_work_in_both_modes() {
        for mut game in [keyboard_game(), touch_game()] {
            game.update_logic(TICK_PERIOD, SnakeControls::with_directions([Direction::Up]));

            assert_eq!(head(&game), Cell { x: 16, y: 8 });
            assert_eq!(game.world.direction, Direction::Up);
        }
    }

    #[test]
    fn keyboard_direction_mapping_stays_unchanged() {
        assert_eq!(direction_for_key(Key::Up), Some(Direction::Up));
        assert_eq!(direction_for_key(Key::W), Some(Direction::Up));
        assert_eq!(direction_for_key(Key::Right), Some(Direction::Right));
        assert_eq!(direction_for_key(Key::D), Some(Direction::Right));
        assert_eq!(direction_for_key(Key::Down), Some(Direction::Down));
        assert_eq!(direction_for_key(Key::S), Some(Direction::Down));
        assert_eq!(direction_for_key(Key::Left), Some(Direction::Left));
        assert_eq!(direction_for_key(Key::A), Some(Direction::Left));
        assert_eq!(direction_for_key(Key::Space), None);
        assert_eq!(RESTART_KEY, Key::Space);
        assert_eq!(EXIT_KEY, Key::Escape);
    }

    #[test]
    fn running_hud_draws_score_text() {
        let mut framebuffer = Framebuffer::new(TOUCH_FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT);

        draw_score_hud(&mut framebuffer, touch_layout(), 12, 37);

        assert!(count_pixels(&framebuffer, HUD_TEXT) > 0);
    }

    #[test]
    fn game_over_panel_draws_score_and_replay_button() {
        let mut framebuffer = Framebuffer::new(TOUCH_FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT);
        let layout = touch_layout();

        draw_game_over(&mut framebuffer, layout, 7);

        assert!(count_pixels(&framebuffer, GAME_OVER) > 0);
        assert!(count_pixels(&framebuffer, HUD_TEXT) > 0);
        assert!(count_pixels(&framebuffer, BUTTON_TEXT) > 0);
        assert_eq!(
            framebuffer.pixel(layout.replay.x, layout.replay.y),
            Some(BUTTON_BORDER)
        );
        assert_color_only_inside(&framebuffer, PANEL_FILL, layout.playfield);
        assert_color_only_inside(&framebuffer, BUTTON_TEXT, layout.playfield);
    }

    #[test]
    fn d_pad_draws_direction_arrows() {
        let mut framebuffer = Framebuffer::new(TOUCH_FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT);

        draw_d_pad(&mut framebuffer, touch_layout());

        assert!(count_pixels(&framebuffer, D_PAD_ARROW) > 0);
    }

    #[test]
    fn d_pad_hit_tests_up() {
        assert_eq!(
            d_pad_direction_at(rect_center(d_pad_up())),
            Some(Direction::Up)
        );
    }

    #[test]
    fn d_pad_hit_tests_down() {
        assert_eq!(
            d_pad_direction_at(rect_center(d_pad_down())),
            Some(Direction::Down)
        );
    }

    #[test]
    fn d_pad_hit_tests_left() {
        assert_eq!(
            d_pad_direction_at(rect_center(d_pad_left())),
            Some(Direction::Left)
        );
    }

    #[test]
    fn d_pad_hit_tests_right() {
        assert_eq!(
            d_pad_direction_at(rect_center(d_pad_right())),
            Some(Direction::Right)
        );
    }

    #[test]
    fn d_pad_center_is_neutral() {
        let zone = d_pad_zone_at(rect_center(d_pad_center()));

        assert_eq!(zone.and_then(|zone| zone.direction), None);
        assert!(d_pad_zone_at(rect_center(d_pad_center())).is_some());
    }

    #[test]
    fn outside_d_pad_is_ignored() {
        assert_eq!(d_pad_zone_at((0, 0)), None);
    }

    #[test]
    fn d_pad_started_outside_control_is_ignored() {
        let mut tracker = DPadTracker::default();

        let directions =
            tracker.directions_from_touches(&[touch(1, TouchPhase::Started, Some((0, 0)))]);

        assert!(directions.is_empty());
        assert!(tracker.active.is_none());
    }

    #[test]
    fn d_pad_rect_boundaries_are_inclusive_and_non_overlapping() {
        assert_eq!(d_pad_direction_at((374, 36)), Some(Direction::Up));
        assert_eq!(d_pad_direction_at((425, 87)), Some(Direction::Up));
        assert_eq!(d_pad_direction_at((373, 87)), None);
        assert_eq!(d_pad_direction_at((374, 88)), None);

        assert_eq!(d_pad_direction_at((322, 88)), Some(Direction::Left));
        assert_eq!(d_pad_direction_at((373, 139)), Some(Direction::Left));
        assert_eq!(d_pad_direction_at((374, 139)), None);

        assert_eq!(d_pad_direction_at((426, 88)), Some(Direction::Right));
        assert_eq!(d_pad_direction_at((477, 139)), Some(Direction::Right));
        assert_eq!(d_pad_direction_at((478, 139)), None);

        assert_eq!(d_pad_direction_at((374, 140)), Some(Direction::Down));
        assert_eq!(d_pad_direction_at((425, 191)), Some(Direction::Down));
        assert_eq!(d_pad_direction_at((426, 191)), None);
    }

    #[test]
    fn d_pad_started_produces_direction_immediately() {
        let mut tracker = DPadTracker::default();

        let directions = tracker.directions_from_touches(&[touch(
            1,
            TouchPhase::Started,
            Some(rect_center(d_pad_right())),
        )]);

        assert_eq!(directions, [Direction::Right]);
        assert_eq!(tracker.active.as_ref().map(|contact| contact.id), Some(1));
    }

    #[test]
    fn d_pad_started_in_center_tracks_without_direction() {
        let mut tracker = DPadTracker::default();

        let directions = tracker.directions_from_touches(&[touch(
            1,
            TouchPhase::Started,
            Some(rect_center(d_pad_center())),
        )]);

        assert!(directions.is_empty());
        assert_eq!(tracker.active.as_ref().map(|contact| contact.id), Some(1));
    }

    #[test]
    fn d_pad_moved_in_same_zone_does_not_repeat() {
        let mut tracker = DPadTracker::default();

        let directions = tracker.directions_from_touches(&[
            touch(1, TouchPhase::Started, Some(rect_center(d_pad_right()))),
            touch(1, TouchPhase::Moved, Some((450, 110))),
        ]);

        assert_eq!(directions, [Direction::Right]);
    }

    #[test]
    fn d_pad_moved_to_another_zone_produces_new_direction() {
        let mut tracker = DPadTracker::default();

        let directions = tracker.directions_from_touches(&[
            touch(1, TouchPhase::Started, Some(rect_center(d_pad_right()))),
            touch(1, TouchPhase::Moved, Some(rect_center(d_pad_up()))),
        ]);

        assert_eq!(directions, [Direction::Right, Direction::Up]);
    }

    #[test]
    fn d_pad_supports_fast_direction_sequences() {
        let mut tracker = DPadTracker::default();

        let directions = tracker.directions_from_touches(&[
            touch(1, TouchPhase::Started, Some(rect_center(d_pad_right()))),
            touch(1, TouchPhase::Moved, Some(rect_center(d_pad_up()))),
            touch(1, TouchPhase::Moved, Some(rect_center(d_pad_left()))),
        ]);

        assert_eq!(
            directions,
            [Direction::Right, Direction::Up, Direction::Left]
        );
    }

    #[test]
    fn d_pad_second_contact_is_ignored_while_first_contact_is_active() {
        let mut tracker = DPadTracker::default();

        let directions = tracker.directions_from_touches(&[
            touch(1, TouchPhase::Started, Some(rect_center(d_pad_right()))),
            touch(2, TouchPhase::Started, Some(rect_center(d_pad_up()))),
            touch(2, TouchPhase::Moved, Some(rect_center(d_pad_left()))),
            touch(1, TouchPhase::Moved, Some(rect_center(d_pad_down()))),
        ]);

        assert_eq!(directions, [Direction::Right, Direction::Down]);
        assert_eq!(tracker.active.as_ref().map(|contact| contact.id), Some(1));
    }

    #[test]
    fn d_pad_ended_touch_releases_contact() {
        let mut tracker = DPadTracker::default();

        let directions = tracker.directions_from_touches(&[
            touch(1, TouchPhase::Started, Some(rect_center(d_pad_right()))),
            touch(1, TouchPhase::Ended, Some(rect_center(d_pad_right()))),
        ]);

        assert_eq!(directions, [Direction::Right]);
        assert!(tracker.active.is_none());
    }

    #[test]
    fn d_pad_cancelled_touch_releases_contact() {
        let mut tracker = DPadTracker::default();

        let directions = tracker.directions_from_touches(&[
            touch(1, TouchPhase::Started, Some(rect_center(d_pad_right()))),
            touch(1, TouchPhase::Cancelled, Some(rect_center(d_pad_right()))),
        ]);

        assert_eq!(directions, [Direction::Right]);
        assert!(tracker.active.is_none());
    }

    #[test]
    fn d_pad_accepts_new_contact_after_previous_contact_ends() {
        let mut tracker = DPadTracker::default();

        let directions = tracker.directions_from_touches(&[
            touch(1, TouchPhase::Started, Some(rect_center(d_pad_right()))),
            touch(1, TouchPhase::Ended, Some(rect_center(d_pad_right()))),
            touch(2, TouchPhase::Started, Some(rect_center(d_pad_up()))),
        ]);

        assert_eq!(directions, [Direction::Right, Direction::Up]);
        assert_eq!(tracker.active.as_ref().map(|contact| contact.id), Some(2));
    }

    #[test]
    fn touch_controls_visibility_is_activated_by_touch_events() {
        let mut touch_controls = TouchControls::default();

        assert!(!touch_controls.visible());
        touch_controls.observe_touches(&[touch(1, TouchPhase::Started, None)]);

        assert!(touch_controls.visible());
    }

    #[test]
    fn new_game_starts_with_hidden_touch_controls() {
        let game = touch_game();

        assert!(!game.touch_controls.visible());
    }

    #[test]
    fn snake_does_not_move_before_a_complete_tick() {
        let mut game = touch_game();
        let initial_head = head(&game);

        game.update_logic(
            TICK_PERIOD - Duration::from_millis(1),
            SnakeControls::none(),
        );

        assert_eq!(head(&game), initial_head);
    }

    #[test]
    fn snake_moves_after_one_tick() {
        let mut game = touch_game();

        tick(&mut game);

        assert_eq!(head(&game), Cell { x: 17, y: 9 });
    }

    #[test]
    fn multiple_small_delta_times_accumulate_into_a_tick() {
        let mut game = touch_game();

        game.update_logic(Duration::from_millis(40), SnakeControls::none());
        game.update_logic(Duration::from_millis(40), SnakeControls::none());
        game.update_logic(Duration::from_millis(40), SnakeControls::none());

        assert_eq!(head(&game), Cell { x: 17, y: 9 });
    }

    #[test]
    fn eating_food_grows_the_snake() {
        let mut game = touch_game();
        game.world.food = Some(Cell { x: 17, y: 9 });
        let initial_len = game.world.snake.len();

        tick(&mut game);

        assert_eq!(game.world.snake.len(), initial_len + 1);
        assert_eq!(head(&game), Cell { x: 17, y: 9 });
        assert_eq!(game.world.score(), 1);
    }

    #[test]
    fn new_game_starts_with_zero_score() {
        let game = touch_game();

        assert_eq!(game.world.score(), 0);
        assert_eq!(game.best_score, 0);
        assert!(!game.best_score_loaded);
    }

    #[test]
    fn score_increments_once_per_food_eaten() {
        let mut game = touch_game();

        game.world.food = Some(Cell { x: 17, y: 9 });
        tick(&mut game);
        game.world.food = Some(Cell { x: 18, y: 9 });
        tick(&mut game);

        assert_eq!(game.world.score(), 2);
    }

    #[test]
    fn best_score_defaults_to_zero_when_storage_is_absent() {
        let mut game = touch_game();
        let mut storage = TestStorage::default();

        assert_eq!(
            update_with_storage(&mut game, &mut storage, Duration::ZERO),
            GameResult::Continue
        );

        assert_eq!(game.best_score, 0);
        assert!(game.best_score_loaded);
    }

    #[test]
    fn best_score_loads_persisted_value_once() {
        let mut game = touch_game();
        let mut storage = TestStorage::with_entry(BEST_SCORE_KEY, "37");

        update_with_storage(&mut game, &mut storage, Duration::ZERO);

        assert_eq!(game.best_score, 37);

        storage.entries.insert(BEST_SCORE_KEY.into(), "99".into());
        update_with_storage(&mut game, &mut storage, Duration::ZERO);

        assert_eq!(game.best_score, 37);
    }

    #[test]
    fn corrupt_best_score_storage_falls_back_to_zero() {
        let mut game = touch_game();
        let mut storage = TestStorage::with_entry(BEST_SCORE_KEY, "not a number");

        update_with_storage(&mut game, &mut storage, Duration::ZERO);

        assert_eq!(game.best_score, 0);
        assert!(game.best_score_loaded);
    }

    #[test]
    fn best_score_read_error_falls_back_to_zero() {
        let mut game = touch_game();
        let mut storage = TestStorage::failing_reads();

        update_with_storage(&mut game, &mut storage, Duration::ZERO);

        assert_eq!(game.best_score, 0);
        assert!(game.best_score_loaded);
    }

    #[test]
    fn new_record_updates_best_score_and_persists_immediately() {
        let mut game = touch_game();
        let mut storage = TestStorage::default();
        game.world.food = Some(Cell { x: 17, y: 9 });

        update_with_storage(&mut game, &mut storage, TICK_PERIOD);

        assert_eq!(game.world.score(), 1);
        assert_eq!(game.best_score, 1);
        assert_eq!(
            storage.entries.get(BEST_SCORE_KEY).map(String::as_str),
            Some("1")
        );
    }

    #[test]
    fn score_below_best_does_not_overwrite_storage() {
        let mut game = touch_game();
        let mut storage = TestStorage::with_entry(BEST_SCORE_KEY, "10");
        game.world.score = 4;

        update_with_storage(&mut game, &mut storage, Duration::ZERO);

        assert_eq!(game.best_score, 10);
        assert_eq!(
            storage.entries.get(BEST_SCORE_KEY).map(String::as_str),
            Some("10")
        );
    }

    #[test]
    fn best_score_write_error_keeps_record_in_memory() {
        let mut game = touch_game();
        let mut storage = TestStorage::failing_writes();
        game.world.score = 5;

        update_with_storage(&mut game, &mut storage, Duration::ZERO);

        assert_eq!(game.best_score, 5);
        assert!(!storage.entries.contains_key(BEST_SCORE_KEY));
    }

    #[test]
    fn restart_keeps_best_score() {
        let mut game = touch_game();
        let mut storage = TestStorage::with_entry(BEST_SCORE_KEY, "8");

        update_with_storage(&mut game, &mut storage, Duration::ZERO);
        game.world.score = 3;
        game.restart();

        assert_eq!(game.world.score(), 0);
        assert_eq!(game.best_score, 8);
        assert!(game.best_score_loaded);
    }

    #[test]
    fn best_score_persists_between_snake_instances_with_memory_storage() {
        let mut storage = TestStorage::default();

        let mut first = touch_game();
        first.world.score = 6;
        update_with_storage(&mut first, &mut storage, Duration::ZERO);

        let mut second = touch_game();
        update_with_storage(&mut second, &mut storage, Duration::ZERO);

        assert_eq!(second.best_score, 6);
        assert_eq!(second.world.score(), 0);
    }

    #[test]
    fn score_and_best_are_identical_between_interaction_modes() {
        let mut keyboard = keyboard_game();
        let mut touch = touch_game();
        let mut keyboard_storage = TestStorage::with_entry(BEST_SCORE_KEY, "9");
        let mut touch_storage = TestStorage::with_entry(BEST_SCORE_KEY, "9");

        keyboard.world.food = Some(Cell { x: 17, y: 9 });
        touch.world.food = Some(Cell { x: 17, y: 9 });
        update_with_storage(&mut keyboard, &mut keyboard_storage, TICK_PERIOD);
        update_with_storage(&mut touch, &mut touch_storage, TICK_PERIOD);

        assert_eq!(keyboard.world.score(), touch.world.score());
        assert_eq!(keyboard.best_score, touch.best_score);
        assert_eq!(keyboard.best_score, 9);
    }

    #[test]
    fn business_world_is_identical_at_same_seed_for_both_modes() {
        let keyboard = keyboard_game();
        let touch = touch_game();

        assert_eq!(keyboard.world, touch.world);
    }

    #[test]
    fn surface_resize_does_not_restart_snake_world() {
        let mut game = touch_game();
        let mut storage = TestStorage::with_entry(BEST_SCORE_KEY, "4");

        update_with_storage(&mut game, &mut storage, Duration::ZERO);
        game.world.food = Some(Cell { x: 17, y: 9 });
        update_with_storage(&mut game, &mut storage, TICK_PERIOD);

        let world_before = game.world.clone();
        let best_before = game.best_score;

        update_with_storage_and_surface(
            &mut game,
            &mut storage,
            Duration::ZERO,
            Size {
                width: 412,
                height: 915,
            },
        );

        assert_eq!(game.world, world_before);
        assert_eq!(game.best_score, best_before);
    }

    #[test]
    fn game_over_preserves_score() {
        let mut world = SnakeWorld::new(123);
        world.score = 3;
        world.snake = VecDeque::from([
            Cell { x: 31, y: 9 },
            Cell { x: 30, y: 9 },
            Cell { x: 29, y: 9 },
        ]);
        world.direction = Direction::Right;

        world.tick();

        assert_eq!(world.phase, Phase::GameOver);
        assert_eq!(world.score(), 3);
    }

    #[test]
    fn food_is_always_placed_on_a_free_cell() {
        let mut world = SnakeWorld::new(123);

        for _ in 0..128 {
            world.food = world.place_food();
            let food = world
                .food
                .expect("food should exist while cells remain free");

            assert!(!world.snake.iter().any(|cell| *cell == food));
        }
    }

    #[test]
    fn wall_collision_enters_game_over() {
        let mut world = SnakeWorld::new(123);
        world.snake = VecDeque::from([
            Cell { x: 31, y: 9 },
            Cell { x: 30, y: 9 },
            Cell { x: 29, y: 9 },
        ]);
        world.direction = Direction::Right;

        world.tick();

        assert_eq!(world.phase, Phase::GameOver);
    }

    #[test]
    fn game_over_transition_cleans_active_d_pad_contact() {
        let mut game = touch_game();
        game.world.snake = VecDeque::from([
            Cell { x: 31, y: 9 },
            Cell { x: 30, y: 9 },
            Cell { x: 29, y: 9 },
        ]);
        game.world.direction = Direction::Right;
        game.touch_controls.d_pad.active = Some(active_d_pad_contact());

        tick(&mut game);

        assert_eq!(game.world.phase, Phase::GameOver);
        assert!(game.touch_controls.d_pad.active.is_none());
    }

    #[test]
    fn body_collision_enters_game_over() {
        let mut world = SnakeWorld::new(123);
        world.snake = VecDeque::from([
            Cell { x: 8, y: 8 },
            Cell { x: 8, y: 9 },
            Cell { x: 7, y: 9 },
            Cell { x: 7, y: 8 },
            Cell { x: 8, y: 8 },
        ]);
        world.direction = Direction::Down;
        world.food = Some(Cell { x: 8, y: 9 });

        world.tick();

        assert_eq!(world.phase, Phase::GameOver);
    }

    #[test]
    fn immediate_u_turn_is_rejected() {
        let mut game = touch_game();

        game.update_logic(
            TICK_PERIOD,
            SnakeControls::with_directions([Direction::Left]),
        );

        assert_eq!(head(&game), Cell { x: 17, y: 9 });
        assert_eq!(game.world.direction, Direction::Right);
    }

    #[test]
    fn two_valid_turns_are_applied_on_successive_ticks() {
        let mut game = touch_game();

        game.update_logic(
            TICK_PERIOD,
            SnakeControls::with_directions([Direction::Up, Direction::Left]),
        );
        assert_eq!(head(&game), Cell { x: 16, y: 8 });
        assert_eq!(game.world.direction, Direction::Up);

        tick(&mut game);
        assert_eq!(head(&game), Cell { x: 15, y: 8 });
        assert_eq!(game.world.direction, Direction::Left);
    }

    #[test]
    fn restart_resets_the_game_after_game_over() {
        let mut game = touch_game();
        game.world.phase = Phase::GameOver;
        game.world.snake = VecDeque::from([Cell { x: 1, y: 1 }]);
        game.world.direction = Direction::Down;
        game.world.turn_queue = VecDeque::from([Direction::Left]);
        game.world.score = 4;
        game.accumulator = TICK_PERIOD;
        game.touch_controls.d_pad.active = Some(active_d_pad_contact());

        game.update_logic(Duration::ZERO, SnakeControls::restart());

        assert_eq!(game.world.phase, Phase::Running);
        assert_eq!(game.world.snake, super::initial_snake());
        assert_eq!(game.world.direction, Direction::Right);
        assert!(game.world.turn_queue.is_empty());
        assert_eq!(game.world.score(), 0);
        assert_eq!(game.accumulator, Duration::ZERO);
        assert!(game.touch_controls.d_pad.active.is_none());
        assert!(game.world.food.is_some());
    }

    #[test]
    fn game_over_keyboard_space_requests_restart_without_direction() {
        let mut touch_controls = TouchControls::default();

        let controls =
            SnakeControls::from_game_over_inputs(true, false, None, &[], &mut touch_controls);

        assert!(controls.restart);
        assert!(controls.directions.is_empty());
    }

    #[test]
    fn game_over_mouse_replay_rect_requests_restart() {
        let mut touch_controls = TouchControls::default();

        let controls = SnakeControls::from_game_over_inputs(
            false,
            true,
            Some(replay_button_center()),
            &[],
            &mut touch_controls,
        );

        assert!(controls.restart);
        assert!(controls.directions.is_empty());
    }

    #[test]
    fn game_over_mouse_replay_rect_requests_restart_in_keyboard_layout() {
        let mut touch_controls = TouchControls::default();

        let controls = SnakeControls::from_game_over_inputs_in_layout(
            false,
            true,
            Some(keyboard_replay_button_center()),
            &[],
            &mut touch_controls,
            keyboard_layout(),
        );

        assert!(controls.restart);
        assert!(controls.directions.is_empty());
    }

    #[test]
    fn game_over_mouse_outside_replay_rect_is_ignored() {
        let mut touch_controls = TouchControls::default();

        let controls = SnakeControls::from_game_over_inputs(
            false,
            true,
            Some((0, 0)),
            &[],
            &mut touch_controls,
        );

        assert!(!controls.restart);
        assert!(controls.directions.is_empty());
    }

    #[test]
    fn game_over_touch_started_in_replay_rect_requests_restart() {
        let mut touch_controls = TouchControls::default();

        let controls = SnakeControls::from_game_over_inputs(
            false,
            false,
            None,
            &[touch(1, TouchPhase::Started, Some(replay_button_center()))],
            &mut touch_controls,
        );

        assert!(controls.restart);
        assert!(controls.directions.is_empty());
    }

    #[test]
    fn game_over_touch_replay_rect_requests_restart_in_keyboard_layout() {
        let mut touch_controls = TouchControls::default();

        let controls = SnakeControls::from_game_over_inputs_in_layout(
            false,
            false,
            None,
            &[touch(
                1,
                TouchPhase::Started,
                Some(keyboard_replay_button_center()),
            )],
            &mut touch_controls,
            keyboard_layout(),
        );

        assert!(controls.restart);
        assert!(controls.directions.is_empty());
    }

    #[test]
    fn game_over_touch_outside_replay_rect_is_ignored() {
        let mut touch_controls = TouchControls::default();

        let controls = SnakeControls::from_game_over_inputs(
            false,
            false,
            None,
            &[touch(1, TouchPhase::Started, Some((0, 0)))],
            &mut touch_controls,
        );

        assert!(!controls.restart);
        assert!(controls.directions.is_empty());
    }

    #[test]
    fn game_over_touch_in_d_pad_is_ignored() {
        let mut touch_controls = TouchControls::default();

        let controls = SnakeControls::from_game_over_inputs(
            false,
            false,
            None,
            &[touch(
                1,
                TouchPhase::Started,
                Some(rect_center(d_pad_right())),
            )],
            &mut touch_controls,
        );

        assert!(!controls.restart);
        assert!(controls.directions.is_empty());
        assert!(touch_controls.d_pad.active.is_none());
    }

    #[test]
    fn game_over_touch_replay_does_not_feed_d_pad_tracker() {
        let mut touch_controls = touch_controls_with_active_contact();

        let controls = SnakeControls::from_game_over_inputs(
            false,
            false,
            None,
            &[touch(1, TouchPhase::Started, Some(replay_button_center()))],
            &mut touch_controls,
        );

        assert!(controls.restart);
        assert!(controls.directions.is_empty());
        assert!(touch_controls.d_pad.active.is_none());
    }

    #[test]
    fn replay_touch_restarts_from_game_over() {
        let mut game = touch_game();
        game.world.phase = Phase::GameOver;
        game.world.score = 2;

        let controls = SnakeControls::from_game_over_inputs(
            false,
            false,
            None,
            &[touch(1, TouchPhase::Started, Some(replay_button_center()))],
            &mut game.touch_controls,
        );
        game.update_logic(Duration::ZERO, controls);

        assert_eq!(game.world.phase, Phase::Running);
        assert_eq!(game.world.score(), 0);
        assert_eq!(game.world.snake, super::initial_snake());
    }

    #[test]
    fn no_food_is_placed_when_no_free_cell_remains() {
        let mut world = SnakeWorld::new(123);
        world.snake.clear();
        for y in 0..super::GRID_HEIGHT {
            for x in 0..super::GRID_WIDTH {
                world.snake.push_back(Cell { x, y });
            }
        }

        assert_eq!(world.place_food(), None);
    }

    #[test]
    fn turn_queue_capacity_is_limited() {
        let mut world = SnakeWorld::new(123);

        world.queue_direction(Direction::Up);
        world.queue_direction(Direction::Left);
        world.queue_direction(Direction::Down);

        assert_eq!(world.turn_queue.len(), TURN_QUEUE_CAPACITY);
        assert_eq!(
            world.turn_queue,
            VecDeque::from([Direction::Up, Direction::Left])
        );
    }
}
