use std::collections::VecDeque;
use std::time::Duration;

use gotoo_pixel_engine::{Frame, Game, GameResult, Key, Pixel};

pub const FRAMEBUFFER_WIDTH: u32 = 320;
pub const FRAMEBUFFER_HEIGHT: u32 = 180;

const GRID_WIDTH: i32 = 32;
const GRID_HEIGHT: i32 = 18;
const CELL_SIZE: i32 = 10;
const INITIAL_SEED: u32 = 0x5EED_1234;
const TURN_QUEUE_CAPACITY: usize = 2;
const MAX_CATCH_UP: usize = 5;
const TICK_PERIOD: Duration = Duration::from_millis(120);

const BACKGROUND: Pixel = Pixel::rgb(10, 14, 18);
const GRID_LINE: Pixel = Pixel::rgb(20, 28, 34);
const BORDER: Pixel = Pixel::rgb(88, 102, 112);
const SNAKE_HEAD: Pixel = Pixel::rgb(214, 246, 128);
const SNAKE_BODY: Pixel = Pixel::rgb(82, 190, 118);
const FOOD: Pixel = Pixel::rgb(235, 74, 74);
const GAME_OVER: Pixel = Pixel::rgb(245, 66, 66);

#[derive(Debug)]
pub struct SnakeGame {
    world: SnakeWorld,
    accumulator: Duration,
}

impl SnakeGame {
    pub fn new() -> Self {
        Self {
            world: SnakeWorld::new(INITIAL_SEED),
            accumulator: Duration::ZERO,
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
                break;
            }
        }
    }

    fn restart(&mut self) {
        self.world.restart();
        self.accumulator = Duration::ZERO;
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

        if self.world.phase() == Phase::GameOver {
            framebuffer.draw_line(96, 42, 224, 138, GAME_OVER);
            framebuffer.draw_line(224, 42, 96, 138, GAME_OVER);
            framebuffer.draw_rect(
                2,
                2,
                FRAMEBUFFER_WIDTH - 4,
                FRAMEBUFFER_HEIGHT - 4,
                GAME_OVER,
            );
        }
    }
}

impl Game for SnakeGame {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if frame.input.key(Key::Escape).pressed() {
            return GameResult::Exit;
        }

        self.update_logic(frame.delta_time, SnakeControls::from_frame(frame));
        self.draw(frame);

        GameResult::Continue
    }
}

fn draw_grid(framebuffer: &mut gotoo_pixel_engine::Framebuffer) {
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

    fn from_frame(frame: &Frame<'_>) -> Self {
        let mut controls = Self::none();

        for (key, direction) in [
            (Key::Up, Direction::Up),
            (Key::W, Direction::Up),
            (Key::Right, Direction::Right),
            (Key::D, Direction::Right),
            (Key::Down, Direction::Down),
            (Key::S, Direction::Down),
            (Key::Left, Direction::Left),
            (Key::A, Direction::Left),
        ] {
            if frame.input.key(key).pressed() {
                controls.directions.push(direction);
            }
        }

        controls.restart = frame.input.key(Key::Space).pressed();
        controls
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct SnakeWorld {
    phase: Phase,
    snake: VecDeque<Cell>,
    direction: Direction,
    turn_queue: VecDeque<Direction>,
    food: Option<Cell>,
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
        Cell, Direction, Phase, SnakeControls, SnakeGame, SnakeWorld, TICK_PERIOD,
        TURN_QUEUE_CAPACITY,
    };

    fn head(game: &SnakeGame) -> Cell {
        game.world.snake[0]
    }

    fn tick(game: &mut SnakeGame) {
        game.update_logic(TICK_PERIOD, SnakeControls::none());
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
        game.accumulator = TICK_PERIOD;

        game.update_logic(Duration::ZERO, SnakeControls::restart());

        assert_eq!(game.world.phase, Phase::Running);
        assert_eq!(game.world.snake, super::initial_snake());
        assert_eq!(game.world.direction, Direction::Right);
        assert!(game.world.turn_queue.is_empty());
        assert_eq!(game.accumulator, Duration::ZERO);
        assert!(game.world.food.is_some());
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
