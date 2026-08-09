use std::collections::VecDeque;
use std::time::Duration;

use gotoo_pixel_engine::{
    Frame, Framebuffer, Game, GameResult, Key, MouseButton, Pixel, Touch, TouchPhase,
};

pub const FRAMEBUFFER_WIDTH: u32 = 320;
pub const FRAMEBUFFER_HEIGHT: u32 = 180;

const GRID_WIDTH: i32 = 32;
const GRID_HEIGHT: i32 = 18;
const CELL_SIZE: i32 = 10;
const INITIAL_SEED: u32 = 0x5EED_1234;
const TURN_QUEUE_CAPACITY: usize = 2;
const MAX_CATCH_UP: usize = 5;
const TICK_PERIOD: Duration = Duration::from_millis(120);
const EXIT_KEY: Key = Key::Escape;
const RESTART_KEY: Key = Key::Space;
const HUD_TEXT_SCALE: u32 = 1;
const GAME_OVER_TEXT_SCALE: u32 = 2;
const D_PAD_UP: Rect = Rect {
    x: 232,
    y: 56,
    width: 40,
    height: 40,
};
const D_PAD_LEFT: Rect = Rect {
    x: 192,
    y: 96,
    width: 40,
    height: 40,
};
const D_PAD_CENTER: Rect = Rect {
    x: 232,
    y: 96,
    width: 40,
    height: 40,
};
const D_PAD_RIGHT: Rect = Rect {
    x: 272,
    y: 96,
    width: 40,
    height: 40,
};
const D_PAD_DOWN: Rect = Rect {
    x: 232,
    y: 136,
    width: 40,
    height: 40,
};
const D_PAD_ZONES: [DirectionZone; 4] = [
    DirectionZone {
        rect: D_PAD_UP,
        direction: Direction::Up,
    },
    DirectionZone {
        rect: D_PAD_LEFT,
        direction: Direction::Left,
    },
    DirectionZone {
        rect: D_PAD_RIGHT,
        direction: Direction::Right,
    },
    DirectionZone {
        rect: D_PAD_DOWN,
        direction: Direction::Down,
    },
];
const REPLAY_BUTTON: Rect = Rect {
    x: 104,
    y: 110,
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

#[derive(Debug)]
pub struct SnakeGame {
    world: SnakeWorld,
    accumulator: Duration,
    touch_controls: TouchControls,
}

impl SnakeGame {
    pub fn new() -> Self {
        Self {
            world: SnakeWorld::new(INITIAL_SEED),
            accumulator: Duration::ZERO,
            touch_controls: TouchControls::default(),
        }
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

    fn draw(&self, frame: &mut Frame<'_>) {
        let framebuffer = &mut frame.framebuffer;

        framebuffer.clear(BACKGROUND);
        draw_grid(framebuffer);

        if let Some(food) = self.world.food() {
            let center_x = food.x * CELL_SIZE + CELL_SIZE / 2;
            let center_y = food.y * CELL_SIZE + CELL_SIZE / 2;
            framebuffer.fill_circle(center_x, center_y, 4, FOOD);
        }

        for (index, cell) in self.world.snake().iter().enumerate().rev() {
            let color = if index == 0 { SNAKE_HEAD } else { SNAKE_BODY };
            framebuffer.fill_rect(
                cell.x * CELL_SIZE + 1,
                cell.y * CELL_SIZE + 1,
                (CELL_SIZE - 2) as u32,
                (CELL_SIZE - 2) as u32,
                color,
            );
        }

        framebuffer.draw_rect(0, 0, FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT, BORDER);
        draw_score_hud(framebuffer, self.world.score());

        if self.world.phase() == Phase::GameOver {
            draw_game_over(framebuffer, self.world.score());
        } else if self.touch_controls.visible() {
            draw_d_pad(framebuffer);
        }
    }
}

impl Game for SnakeGame {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if frame.input.key(EXIT_KEY).pressed() {
            return GameResult::Exit;
        }

        let controls =
            SnakeControls::from_frame(frame, self.world.phase(), &mut self.touch_controls);
        self.update_logic(frame.delta_time, controls);
        self.draw(frame);

        GameResult::Continue
    }
}

fn draw_grid(framebuffer: &mut Framebuffer) {
    for x in 1..GRID_WIDTH {
        let pixel_x = x * CELL_SIZE;
        framebuffer.draw_line(
            pixel_x,
            0,
            pixel_x,
            FRAMEBUFFER_HEIGHT as i32 - 1,
            GRID_LINE,
        );
    }
    for y in 1..GRID_HEIGHT {
        let pixel_y = y * CELL_SIZE;
        framebuffer.draw_line(0, pixel_y, FRAMEBUFFER_WIDTH as i32 - 1, pixel_y, GRID_LINE);
    }
}

fn draw_score_hud(framebuffer: &mut Framebuffer, score: u32) {
    let text = format!("SCORE {score}");
    let (width, height) = Framebuffer::text_size(&text, HUD_TEXT_SCALE);

    framebuffer.fill_rect(2, 2, width + 4, height + 4, HUD_BACKDROP);
    framebuffer.draw_rect(2, 2, width + 4, height + 4, BORDER);
    framebuffer.draw_text_scaled(4, 4, &text, HUD_TEXT_SCALE, HUD_TEXT);
}

fn draw_game_over(framebuffer: &mut Framebuffer, score: u32) {
    framebuffer.fill_rect(64, 38, 192, 108, PANEL_FILL);
    framebuffer.draw_rect(64, 38, 192, 108, GAME_OVER);

    draw_centered_text(
        framebuffer,
        52,
        "GAME OVER",
        GAME_OVER_TEXT_SCALE,
        GAME_OVER,
    );
    draw_centered_text(
        framebuffer,
        78,
        &format!("SCORE {score}"),
        GAME_OVER_TEXT_SCALE,
        HUD_TEXT,
    );
    draw_replay_button(framebuffer);
}

fn draw_replay_button(framebuffer: &mut Framebuffer) {
    framebuffer.fill_rect(
        REPLAY_BUTTON.x,
        REPLAY_BUTTON.y,
        REPLAY_BUTTON.width,
        REPLAY_BUTTON.height,
        BUTTON_FILL,
    );
    framebuffer.draw_rect(
        REPLAY_BUTTON.x,
        REPLAY_BUTTON.y,
        REPLAY_BUTTON.width,
        REPLAY_BUTTON.height,
        BUTTON_BORDER,
    );

    let text = "REJOUER";
    let (text_width, text_height) = Framebuffer::text_size(text, GAME_OVER_TEXT_SCALE);
    let text_x = REPLAY_BUTTON.x + centered_offset(REPLAY_BUTTON.width, text_width);
    let text_y = REPLAY_BUTTON.y + centered_offset(REPLAY_BUTTON.height, text_height);
    framebuffer.draw_text_scaled(text_x, text_y, text, GAME_OVER_TEXT_SCALE, BUTTON_TEXT);
}

fn draw_d_pad(framebuffer: &mut Framebuffer) {
    for zone in D_PAD_ZONES {
        draw_d_pad_button(framebuffer, zone.rect, zone.direction);
    }

    framebuffer.fill_rect(
        D_PAD_CENTER.x,
        D_PAD_CENTER.y,
        D_PAD_CENTER.width,
        D_PAD_CENTER.height,
        D_PAD_CENTER_FILL,
    );
    framebuffer.draw_rect(
        D_PAD_CENTER.x,
        D_PAD_CENTER.y,
        D_PAD_CENTER.width,
        D_PAD_CENTER.height,
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

fn draw_centered_text(framebuffer: &mut Framebuffer, y: i32, text: &str, scale: u32, pixel: Pixel) {
    let (width, _) = Framebuffer::text_size(text, scale);
    let x = centered_offset(FRAMEBUFFER_WIDTH, width);
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

    fn from_frame(frame: &Frame<'_>, phase: Phase, touch_controls: &mut TouchControls) -> Self {
        touch_controls.observe_touches(frame.input.touches());

        match phase {
            Phase::Running => Self::from_running_frame(frame, touch_controls),
            Phase::GameOver => Self::from_game_over_inputs(
                frame.input.key(RESTART_KEY).pressed(),
                frame.input.mouse_button(MouseButton::Left).pressed(),
                frame.input.mouse_position(),
                frame.input.touches(),
                touch_controls,
            ),
        }
    }

    fn from_running_frame(frame: &Frame<'_>, touch_controls: &mut TouchControls) -> Self {
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
            .extend(touch_controls.directions_from_touches(frame.input.touches()));
        controls.restart = frame.input.key(RESTART_KEY).pressed();
        controls
    }

    fn from_game_over_inputs(
        keyboard_restart_pressed: bool,
        mouse_left_pressed: bool,
        mouse_position: Option<(i32, i32)>,
        touches: &[Touch],
        touch_controls: &mut TouchControls,
    ) -> Self {
        touch_controls.reset_contact();

        Self {
            directions: Vec::new(),
            restart: replay_requested(
                keyboard_restart_pressed,
                mouse_left_pressed,
                mouse_position,
                touches,
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

    fn directions_from_touches(&mut self, touches: &[Touch]) -> Vec<Direction> {
        self.d_pad.directions_from_touches(touches)
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

    fn directions_from_touches(&mut self, touches: &[Touch]) -> Vec<Direction> {
        let mut directions = Vec::new();

        for touch in touches {
            match touch.phase {
                TouchPhase::Started => {
                    if let Some(direction) = self.start(touch) {
                        directions.push(direction);
                    }
                }
                TouchPhase::Moved => {
                    if let Some(direction) = self.move_active(touch) {
                        directions.push(direction);
                    }
                }
                TouchPhase::Ended => self.end_active(touch),
                TouchPhase::Cancelled => self.cancel_active(touch),
            }
        }

        directions
    }

    fn start(&mut self, touch: &Touch) -> Option<Direction> {
        if self.active.is_some() {
            return None;
        }

        let position = touch.position?;
        let zone = d_pad_zone_at(position)?;

        self.active = Some(DPadContact {
            id: touch.id,
            last_direction: zone.direction,
        });

        zone.direction
    }

    fn move_active(&mut self, touch: &Touch) -> Option<Direction> {
        let contact = self.active.as_mut()?;
        if contact.id != touch.id {
            return None;
        }

        let position = touch.position?;
        let zone = d_pad_zone_at(position)?;
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
struct Rect {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl Rect {
    fn contains(self, position: (i32, i32)) -> bool {
        if self.width == 0 || self.height == 0 {
            return false;
        }

        let min_x = i64::from(self.x);
        let min_y = i64::from(self.y);
        let max_x = min_x + i64::from(self.width) - 1;
        let max_y = min_y + i64::from(self.height) - 1;
        let x = i64::from(position.0);
        let y = i64::from(position.1);

        x >= min_x && x <= max_x && y >= min_y && y <= max_y
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

fn d_pad_zone_at(position: (i32, i32)) -> Option<DPadZone> {
    for zone in D_PAD_ZONES {
        if zone.rect.contains(position) {
            return Some(DPadZone {
                direction: Some(zone.direction),
            });
        }
    }

    if D_PAD_CENTER.contains(position) {
        return Some(DPadZone { direction: None });
    }

    None
}

fn replay_requested(
    keyboard_restart_pressed: bool,
    mouse_left_pressed: bool,
    mouse_position: Option<(i32, i32)>,
    touches: &[Touch],
) -> bool {
    keyboard_restart_pressed
        || (mouse_left_pressed
            && mouse_position.is_some_and(|position| REPLAY_BUTTON.contains(position)))
        || touches.iter().any(|touch| {
            touch.phase == TouchPhase::Started
                && touch
                    .position
                    .is_some_and(|position| REPLAY_BUTTON.contains(position))
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
    use std::collections::VecDeque;
    use std::time::Duration;

    use super::{
        BUTTON_BORDER, BUTTON_TEXT, Cell, D_PAD_ARROW, D_PAD_CENTER, D_PAD_DOWN, D_PAD_LEFT,
        D_PAD_RIGHT, D_PAD_UP, DPadTracker, Direction, EXIT_KEY, GAME_OVER, HUD_TEXT, Phase,
        REPLAY_BUTTON, RESTART_KEY, SnakeControls, SnakeGame, SnakeWorld, TICK_PERIOD,
        TURN_QUEUE_CAPACITY, TouchControls, d_pad_zone_at, direction_for_key, draw_d_pad,
        draw_game_over, draw_score_hud,
    };
    use gotoo_pixel_engine::{Framebuffer, Key, Pixel, Touch, TouchPhase};

    fn head(game: &SnakeGame) -> Cell {
        game.world.snake[0]
    }

    fn tick(game: &mut SnakeGame) {
        game.update_logic(TICK_PERIOD, SnakeControls::none());
    }

    fn replay_button_center() -> (i32, i32) {
        (
            REPLAY_BUTTON.x + REPLAY_BUTTON.width as i32 / 2,
            REPLAY_BUTTON.y + REPLAY_BUTTON.height as i32 / 2,
        )
    }

    fn touch(id: u64, phase: TouchPhase, position: Option<(i32, i32)>) -> Touch {
        Touch {
            id,
            phase,
            position,
        }
    }

    fn rect_center(rect: super::Rect) -> (i32, i32) {
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

    fn d_pad_direction_at(position: (i32, i32)) -> Option<Direction> {
        d_pad_zone_at(position).and_then(|zone| zone.direction)
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
        let mut framebuffer = Framebuffer::new(super::FRAMEBUFFER_WIDTH, super::FRAMEBUFFER_HEIGHT);

        draw_score_hud(&mut framebuffer, 12);

        assert!(count_pixels(&framebuffer, HUD_TEXT) > 0);
    }

    #[test]
    fn game_over_panel_draws_score_and_replay_button() {
        let mut framebuffer = Framebuffer::new(super::FRAMEBUFFER_WIDTH, super::FRAMEBUFFER_HEIGHT);

        draw_game_over(&mut framebuffer, 7);

        assert!(count_pixels(&framebuffer, GAME_OVER) > 0);
        assert!(count_pixels(&framebuffer, HUD_TEXT) > 0);
        assert!(count_pixels(&framebuffer, BUTTON_TEXT) > 0);
        assert_eq!(
            framebuffer.pixel(REPLAY_BUTTON.x, REPLAY_BUTTON.y),
            Some(BUTTON_BORDER)
        );
    }

    #[test]
    fn d_pad_draws_direction_arrows() {
        let mut framebuffer = Framebuffer::new(super::FRAMEBUFFER_WIDTH, super::FRAMEBUFFER_HEIGHT);

        draw_d_pad(&mut framebuffer);

        assert!(count_pixels(&framebuffer, D_PAD_ARROW) > 0);
    }

    #[test]
    fn d_pad_hit_tests_up() {
        assert_eq!(
            d_pad_direction_at(rect_center(D_PAD_UP)),
            Some(Direction::Up)
        );
    }

    #[test]
    fn d_pad_hit_tests_down() {
        assert_eq!(
            d_pad_direction_at(rect_center(D_PAD_DOWN)),
            Some(Direction::Down)
        );
    }

    #[test]
    fn d_pad_hit_tests_left() {
        assert_eq!(
            d_pad_direction_at(rect_center(D_PAD_LEFT)),
            Some(Direction::Left)
        );
    }

    #[test]
    fn d_pad_hit_tests_right() {
        assert_eq!(
            d_pad_direction_at(rect_center(D_PAD_RIGHT)),
            Some(Direction::Right)
        );
    }

    #[test]
    fn d_pad_center_is_neutral() {
        let zone = d_pad_zone_at(rect_center(D_PAD_CENTER));

        assert_eq!(zone.and_then(|zone| zone.direction), None);
        assert!(d_pad_zone_at(rect_center(D_PAD_CENTER)).is_some());
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
        assert_eq!(d_pad_direction_at((232, 56)), Some(Direction::Up));
        assert_eq!(d_pad_direction_at((271, 95)), Some(Direction::Up));
        assert_eq!(d_pad_direction_at((231, 95)), None);
        assert_eq!(d_pad_direction_at((232, 96)), None);

        assert_eq!(d_pad_direction_at((192, 96)), Some(Direction::Left));
        assert_eq!(d_pad_direction_at((231, 135)), Some(Direction::Left));
        assert_eq!(d_pad_direction_at((232, 135)), None);

        assert_eq!(d_pad_direction_at((272, 96)), Some(Direction::Right));
        assert_eq!(d_pad_direction_at((311, 135)), Some(Direction::Right));
        assert_eq!(d_pad_direction_at((312, 135)), None);

        assert_eq!(d_pad_direction_at((232, 136)), Some(Direction::Down));
        assert_eq!(d_pad_direction_at((271, 175)), Some(Direction::Down));
        assert_eq!(d_pad_direction_at((272, 175)), None);
    }

    #[test]
    fn d_pad_started_produces_direction_immediately() {
        let mut tracker = DPadTracker::default();

        let directions = tracker.directions_from_touches(&[touch(
            1,
            TouchPhase::Started,
            Some(rect_center(D_PAD_RIGHT)),
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
            Some(rect_center(D_PAD_CENTER)),
        )]);

        assert!(directions.is_empty());
        assert_eq!(tracker.active.as_ref().map(|contact| contact.id), Some(1));
    }

    #[test]
    fn d_pad_moved_in_same_zone_does_not_repeat() {
        let mut tracker = DPadTracker::default();

        let directions = tracker.directions_from_touches(&[
            touch(1, TouchPhase::Started, Some(rect_center(D_PAD_RIGHT))),
            touch(1, TouchPhase::Moved, Some((300, 110))),
        ]);

        assert_eq!(directions, [Direction::Right]);
    }

    #[test]
    fn d_pad_moved_to_another_zone_produces_new_direction() {
        let mut tracker = DPadTracker::default();

        let directions = tracker.directions_from_touches(&[
            touch(1, TouchPhase::Started, Some(rect_center(D_PAD_RIGHT))),
            touch(1, TouchPhase::Moved, Some(rect_center(D_PAD_UP))),
        ]);

        assert_eq!(directions, [Direction::Right, Direction::Up]);
    }

    #[test]
    fn d_pad_supports_fast_direction_sequences() {
        let mut tracker = DPadTracker::default();

        let directions = tracker.directions_from_touches(&[
            touch(1, TouchPhase::Started, Some(rect_center(D_PAD_RIGHT))),
            touch(1, TouchPhase::Moved, Some(rect_center(D_PAD_UP))),
            touch(1, TouchPhase::Moved, Some(rect_center(D_PAD_LEFT))),
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
            touch(1, TouchPhase::Started, Some(rect_center(D_PAD_RIGHT))),
            touch(2, TouchPhase::Started, Some(rect_center(D_PAD_UP))),
            touch(2, TouchPhase::Moved, Some(rect_center(D_PAD_LEFT))),
            touch(1, TouchPhase::Moved, Some(rect_center(D_PAD_DOWN))),
        ]);

        assert_eq!(directions, [Direction::Right, Direction::Down]);
        assert_eq!(tracker.active.as_ref().map(|contact| contact.id), Some(1));
    }

    #[test]
    fn d_pad_ended_touch_releases_contact() {
        let mut tracker = DPadTracker::default();

        let directions = tracker.directions_from_touches(&[
            touch(1, TouchPhase::Started, Some(rect_center(D_PAD_RIGHT))),
            touch(1, TouchPhase::Ended, Some(rect_center(D_PAD_RIGHT))),
        ]);

        assert_eq!(directions, [Direction::Right]);
        assert!(tracker.active.is_none());
    }

    #[test]
    fn d_pad_cancelled_touch_releases_contact() {
        let mut tracker = DPadTracker::default();

        let directions = tracker.directions_from_touches(&[
            touch(1, TouchPhase::Started, Some(rect_center(D_PAD_RIGHT))),
            touch(1, TouchPhase::Cancelled, Some(rect_center(D_PAD_RIGHT))),
        ]);

        assert_eq!(directions, [Direction::Right]);
        assert!(tracker.active.is_none());
    }

    #[test]
    fn d_pad_accepts_new_contact_after_previous_contact_ends() {
        let mut tracker = DPadTracker::default();

        let directions = tracker.directions_from_touches(&[
            touch(1, TouchPhase::Started, Some(rect_center(D_PAD_RIGHT))),
            touch(1, TouchPhase::Ended, Some(rect_center(D_PAD_RIGHT))),
            touch(2, TouchPhase::Started, Some(rect_center(D_PAD_UP))),
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
        let game = SnakeGame::new();

        assert!(!game.touch_controls.visible());
    }

    #[test]
    fn snake_does_not_move_before_a_complete_tick() {
        let mut game = SnakeGame::new();
        let initial_head = head(&game);

        game.update_logic(
            TICK_PERIOD - Duration::from_millis(1),
            SnakeControls::none(),
        );

        assert_eq!(head(&game), initial_head);
    }

    #[test]
    fn snake_moves_after_one_tick() {
        let mut game = SnakeGame::new();

        tick(&mut game);

        assert_eq!(head(&game), Cell { x: 17, y: 9 });
    }

    #[test]
    fn multiple_small_delta_times_accumulate_into_a_tick() {
        let mut game = SnakeGame::new();

        game.update_logic(Duration::from_millis(40), SnakeControls::none());
        game.update_logic(Duration::from_millis(40), SnakeControls::none());
        game.update_logic(Duration::from_millis(40), SnakeControls::none());

        assert_eq!(head(&game), Cell { x: 17, y: 9 });
    }

    #[test]
    fn eating_food_grows_the_snake() {
        let mut game = SnakeGame::new();
        game.world.food = Some(Cell { x: 17, y: 9 });
        let initial_len = game.world.snake.len();

        tick(&mut game);

        assert_eq!(game.world.snake.len(), initial_len + 1);
        assert_eq!(head(&game), Cell { x: 17, y: 9 });
        assert_eq!(game.world.score(), 1);
    }

    #[test]
    fn new_game_starts_with_zero_score() {
        let game = SnakeGame::new();

        assert_eq!(game.world.score(), 0);
    }

    #[test]
    fn score_increments_once_per_food_eaten() {
        let mut game = SnakeGame::new();

        game.world.food = Some(Cell { x: 17, y: 9 });
        tick(&mut game);
        game.world.food = Some(Cell { x: 18, y: 9 });
        tick(&mut game);

        assert_eq!(game.world.score(), 2);
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
        let mut game = SnakeGame::new();
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
        let mut game = SnakeGame::new();

        game.update_logic(
            TICK_PERIOD,
            SnakeControls::with_directions([Direction::Left]),
        );

        assert_eq!(head(&game), Cell { x: 17, y: 9 });
        assert_eq!(game.world.direction, Direction::Right);
    }

    #[test]
    fn two_valid_turns_are_applied_on_successive_ticks() {
        let mut game = SnakeGame::new();

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
        let mut game = SnakeGame::new();
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
                Some(rect_center(D_PAD_RIGHT)),
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
        let mut game = SnakeGame::new();
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
