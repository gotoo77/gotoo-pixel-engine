use std::collections::VecDeque;

pub(super) const GRID_WIDTH: i32 = 32;
pub(super) const GRID_HEIGHT: i32 = 18;
const TURN_QUEUE_CAPACITY: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SnakeWorld {
    pub(super) phase: Phase,
    pub(super) snake: VecDeque<Cell>,
    pub(super) direction: Direction,
    pub(super) turn_queue: VecDeque<Direction>,
    pub(super) food: Option<Cell>,
    pub(super) score: u32,
    rng: FoodRng,
}

impl SnakeWorld {
    pub(super) fn new(seed: u32) -> Self {
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

    pub(super) fn restart(&mut self) {
        self.phase = Phase::Running;
        self.snake = initial_snake();
        self.direction = Direction::Right;
        self.turn_queue.clear();
        self.score = 0;
        self.food = self.place_food();
    }

    pub(super) fn phase(&self) -> Phase {
        self.phase
    }

    pub(super) fn snake(&self) -> &VecDeque<Cell> {
        &self.snake
    }

    pub(super) fn food(&self) -> Option<Cell> {
        self.food
    }

    pub(super) fn score(&self) -> u32 {
        self.score
    }

    pub(super) fn queue_direction(&mut self, direction: Direction) {
        let last_direction = self.turn_queue.back().copied().unwrap_or(self.direction);
        if direction == last_direction || direction == last_direction.opposite() {
            return;
        }
        if self.turn_queue.len() >= TURN_QUEUE_CAPACITY {
            return;
        }

        self.turn_queue.push_back(direction);
    }

    pub(super) fn tick(&mut self) -> TickResult {
        if self.phase != Phase::Running {
            return TickResult::default();
        }

        let turned = if let Some(direction) = self.turn_queue.pop_front() {
            self.direction = direction;
            true
        } else {
            false
        };

        let next_head = self.snake[0].next(self.direction);
        let will_grow = self.food == Some(next_head);

        if !next_head.is_inside_grid() || self.collides_with_body(next_head, will_grow) {
            self.phase = Phase::GameOver;
            return TickResult {
                turned,
                ate_food: false,
                game_over: true,
            };
        }

        self.snake.push_front(next_head);

        if will_grow {
            self.score += 1;
            self.food = self.place_food();
        } else {
            self.snake.pop_back();
        }

        TickResult {
            turned,
            ate_food: will_grow,
            game_over: false,
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct TickResult {
    pub(super) turned: bool,
    pub(super) ate_food: bool,
    pub(super) game_over: bool,
}

pub(super) fn initial_snake() -> VecDeque<Cell> {
    VecDeque::from([
        Cell { x: 16, y: 9 },
        Cell { x: 15, y: 9 },
        Cell { x: 14, y: 9 },
    ])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Cell {
    pub(super) x: i32,
    pub(super) y: i32,
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
pub(super) enum Direction {
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
pub(super) enum Phase {
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

    use super::{
        Cell, Direction, GRID_HEIGHT, GRID_WIDTH, Phase, SnakeWorld, TURN_QUEUE_CAPACITY,
    };

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

    #[test]
    fn wall_collision_enters_game_over() {
        let mut world = SnakeWorld::new(123);
        world.snake = VecDeque::from([
            Cell { x: 31, y: 9 },
            Cell { x: 30, y: 9 },
            Cell { x: 29, y: 9 },
        ]);
        world.direction = Direction::Right;

        assert!(world.tick().game_over);
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
    fn food_is_always_placed_on_free_cell() {
        let mut world = SnakeWorld::new(123);
        for _ in 0..128 {
            world.food = world.place_food();
            let food = world.food.expect("free cells should remain");
            assert!(!world.snake.iter().any(|cell| *cell == food));
        }
    }

    #[test]
    fn no_food_when_grid_is_full() {
        let mut world = SnakeWorld::new(123);
        world.snake.clear();
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                world.snake.push_back(Cell { x, y });
            }
        }
        assert_eq!(world.place_food(), None);
    }
}
