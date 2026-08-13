use std::time::Duration;

use gotoo_pixel_engine::{
    ActionId, ControlMap, Frame, Game, GameResult, GamepadButton, Key, Pixel,
};

pub const FRAMEBUFFER_WIDTH: u32 = 220;
pub const FRAMEBUFFER_HEIGHT: u32 = 224;

const CONTROL_LEFT: ActionId = ActionId::new("tetris.left");
const CONTROL_RIGHT: ActionId = ActionId::new("tetris.right");
const CONTROL_ROTATE: ActionId = ActionId::new("tetris.rotate");
const CONTROL_SOFT_DROP: ActionId = ActionId::new("tetris.soft_drop");
const CONTROL_HARD_DROP: ActionId = ActionId::new("tetris.hard_drop");
const CONTROL_EXIT: ActionId = ActionId::new("tetris.exit");

const BOARD_WIDTH: i32 = 10;
const BOARD_HEIGHT: i32 = 20;
const CELL_SIZE: i32 = 10;
const BOARD_X: i32 = 8;
const BOARD_Y: i32 = 16;
const GRAVITY: Duration = Duration::from_millis(500);
const SOFT_DROP: Duration = Duration::from_millis(45);

const BG: Pixel = Pixel::rgb(9, 12, 16);
const GRID: Pixel = Pixel::rgb(28, 35, 42);
const BORDER: Pixel = Pixel::rgb(112, 126, 138);
const TEXT: Pixel = Pixel::rgb(226, 234, 218);
const GAME_OVER: Pixel = Pixel::rgb(245, 76, 76);
const COLORS: [Pixel; 7] = [
    Pixel::rgb(80, 220, 230),
    Pixel::rgb(80, 110, 230),
    Pixel::rgb(235, 150, 55),
    Pixel::rgb(235, 220, 70),
    Pixel::rgb(80, 205, 105),
    Pixel::rgb(180, 85, 220),
    Pixel::rgb(225, 75, 75),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
}

impl Kind {
    const ALL: [Self; 7] = [
        Self::I,
        Self::J,
        Self::L,
        Self::O,
        Self::S,
        Self::T,
        Self::Z,
    ];
    fn index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Piece {
    kind: Kind,
    rotation: u8,
    x: i32,
    y: i32,
}

impl Piece {
    fn cells(self) -> [(i32, i32); 4] {
        let cells = base_cells(self.kind);
        cells.map(|(mut x, mut y)| {
            if self.kind != Kind::O {
                for _ in 0..self.rotation % 4 {
                    (x, y) = (3 - y, x);
                }
            }
            (self.x + x, self.y + y)
        })
    }
}

#[derive(Debug, Clone)]
struct Bag {
    values: [Kind; 7],
    cursor: usize,
    state: u32,
}

impl Bag {
    fn new(seed: u32) -> Self {
        let mut bag = Self {
            values: Kind::ALL,
            cursor: 7,
            state: seed,
        };
        bag.refill();
        bag
    }

    fn next(&mut self) -> Kind {
        if self.cursor == 7 {
            self.refill();
        }
        let value = self.values[self.cursor];
        self.cursor += 1;
        value
    }

    fn refill(&mut self) {
        self.values = Kind::ALL;
        for i in (1..7).rev() {
            self.state = self
                .state
                .wrapping_mul(1_664_525)
                .wrapping_add(1_013_904_223);
            let j = self.state as usize % (i + 1);
            self.values.swap(i, j);
        }
        self.cursor = 0;
    }
}

#[derive(Debug, Clone)]
struct TetrisWorld {
    board: [[Option<Kind>; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
    active: Piece,
    next: Kind,
    bag: Bag,
    score: u32,
    lines: u32,
    game_over: bool,
}

impl TetrisWorld {
    fn new() -> Self {
        let mut bag = Bag::new(0x007E_7115);
        let first = bag.next();
        let next = bag.next();
        Self {
            board: [[None; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
            active: spawn(first),
            next,
            bag,
            score: 0,
            lines: 0,
            game_over: false,
        }
    }

    fn restart(&mut self) {
        *self = Self::new();
    }

    fn valid(&self, piece: Piece) -> bool {
        piece.cells().into_iter().all(|(x, y)| {
            (0..BOARD_WIDTH).contains(&x)
                && y < BOARD_HEIGHT
                && (y < 0 || self.board[y as usize][x as usize].is_none())
        })
    }

    fn translate(&mut self, dx: i32, dy: i32) -> bool {
        if self.game_over {
            return false;
        }
        let candidate = Piece {
            x: self.active.x + dx,
            y: self.active.y + dy,
            ..self.active
        };
        if self.valid(candidate) {
            self.active = candidate;
            true
        } else {
            false
        }
    }

    fn rotate(&mut self) {
        if self.game_over || self.active.kind == Kind::O {
            return;
        }
        let rotated = Piece {
            rotation: (self.active.rotation + 1) % 4,
            ..self.active
        };
        for kick in [0, -1, 1, -2, 2] {
            let candidate = Piece {
                x: rotated.x + kick,
                ..rotated
            };
            if self.valid(candidate) {
                self.active = candidate;
                return;
            }
        }
    }

    fn hard_drop(&mut self) {
        if self.game_over {
            return;
        }
        let mut distance = 0;
        while self.translate(0, 1) {
            distance += 1;
        }
        self.score = self.score.saturating_add(distance * 2);
        self.lock();
    }

    fn gravity_step(&mut self) {
        if !self.game_over && !self.translate(0, 1) {
            self.lock();
        }
    }

    fn lock(&mut self) {
        let cells = self.active.cells();
        if cells.iter().any(|&(_, y)| y < 0) {
            self.game_over = true;
            return;
        }
        for (x, y) in cells {
            self.board[y as usize][x as usize] = Some(self.active.kind);
        }
        let cleared = self.clear_lines();
        self.lines += cleared;
        self.score = self.score.saturating_add(match cleared {
            1 => 100,
            2 => 300,
            3 => 500,
            4 => 800,
            _ => 0,
        });
        self.active = spawn(self.next);
        self.next = self.bag.next();
        if !self.valid(self.active) {
            self.game_over = true;
        }
    }

    fn clear_lines(&mut self) -> u32 {
        let mut write = BOARD_HEIGHT - 1;
        let mut cleared = 0;
        for read in (0..BOARD_HEIGHT).rev() {
            if self.board[read as usize].iter().all(Option::is_some) {
                cleared += 1;
            } else {
                if write != read {
                    self.board[write as usize] = self.board[read as usize];
                }
                write -= 1;
            }
        }
        while write >= 0 {
            self.board[write as usize] = [None; BOARD_WIDTH as usize];
            write -= 1;
        }
        cleared
    }
}

#[derive(Debug)]
pub struct TetrisGame {
    world: TetrisWorld,
    accumulator: Duration,
    controls: ControlMap,
}

impl TetrisGame {
    pub fn new() -> Self {
        Self {
            world: TetrisWorld::new(),
            accumulator: Duration::ZERO,
            controls: default_controls(),
        }
    }

    fn input(&mut self, frame: &Frame<'_>) -> GameResult {
        self.controls.update(frame.input);

        if self.controls.action(CONTROL_EXIT).pressed() {
            return GameResult::Exit;
        }
        if self.world.game_over {
            if self.controls.action(CONTROL_HARD_DROP).pressed() {
                self.world.restart();
                self.accumulator = Duration::ZERO;
            }
            return GameResult::Continue;
        }
        if self.controls.action(CONTROL_LEFT).pressed() {
            self.world.translate(-1, 0);
        }
        if self.controls.action(CONTROL_RIGHT).pressed() {
            self.world.translate(1, 0);
        }
        if self.controls.action(CONTROL_ROTATE).pressed() {
            self.world.rotate();
        }
        if self.controls.action(CONTROL_HARD_DROP).pressed() {
            self.world.hard_drop();
            self.accumulator = Duration::ZERO;
        }
        GameResult::Continue
    }

    fn render(&self, frame: &mut Frame<'_>) {
        let fb = &mut frame.framebuffer;
        fb.clear(BG);
        fb.draw_rect(
            BOARD_X - 1,
            BOARD_Y - 1,
            (BOARD_WIDTH * CELL_SIZE + 2) as u32,
            (BOARD_HEIGHT * CELL_SIZE + 2) as u32,
            BORDER,
        );
        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                let px = BOARD_X + x * CELL_SIZE;
                let py = BOARD_Y + y * CELL_SIZE;
                fb.draw_rect(px, py, CELL_SIZE as u32, CELL_SIZE as u32, GRID);
                if let Some(kind) = self.world.board[y as usize][x as usize] {
                    draw_block(fb, x, y, kind);
                }
            }
        }
        if !self.world.game_over {
            for (x, y) in self.world.active.cells() {
                if y >= 0 {
                    draw_block(fb, x, y, self.world.active.kind);
                }
            }
        }
        fb.draw_text(8, 4, "TETRIS", TEXT);
        fb.draw_text(120, 24, "SCORE", TEXT);
        fb.draw_text(120, 34, &self.world.score.to_string(), TEXT);
        fb.draw_text(120, 54, "LINES", TEXT);
        fb.draw_text(120, 64, &self.world.lines.to_string(), TEXT);
        fb.draw_text(120, 86, "NEXT", TEXT);
        for (x, y) in base_cells(self.world.next) {
            draw_preview_block(fb, 126 + x * 8, 100 + y * 8, self.world.next);
        }
        fb.draw_text(120, 145, "ARROWS/PAD", TEXT);
        fb.draw_text(120, 155, "MOVE", TEXT);
        fb.draw_text(120, 170, "UP/EAST ROTATE", TEXT);
        fb.draw_text(120, 185, "SPACE/SOUTH DROP", TEXT);
        if self.world.game_over {
            fb.fill_rect(18, 92, 80, 42, BG);
            fb.draw_rect(18, 92, 80, 42, GAME_OVER);
            fb.draw_text(25, 101, "GAME OVER", GAME_OVER);
            fb.draw_text(24, 117, "DROP TO REPLAY", TEXT);
        }
    }
}

impl Game for TetrisGame {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let result = self.input(frame);
        if result == GameResult::Exit {
            return result;
        }
        if !self.world.game_over {
            self.accumulator = self.accumulator.saturating_add(frame.delta_time);
            let period = if self.controls.action(CONTROL_SOFT_DROP).held() {
                SOFT_DROP
            } else {
                GRAVITY
            };
            while self.accumulator >= period && !self.world.game_over {
                self.accumulator -= period;
                self.world.gravity_step();
            }
        }
        self.render(frame);
        GameResult::Continue
    }
}

fn default_controls() -> ControlMap {
    let mut controls = ControlMap::new();
    controls
        .bind_key(CONTROL_LEFT, Key::Left)
        .bind_key(CONTROL_LEFT, Key::A)
        .bind_gamepad(CONTROL_LEFT, GamepadButton::DPadLeft)
        .bind_gamepad(CONTROL_LEFT, GamepadButton::LeftStickLeft)
        .bind_key(CONTROL_RIGHT, Key::Right)
        .bind_key(CONTROL_RIGHT, Key::D)
        .bind_gamepad(CONTROL_RIGHT, GamepadButton::DPadRight)
        .bind_gamepad(CONTROL_RIGHT, GamepadButton::LeftStickRight)
        .bind_key(CONTROL_ROTATE, Key::Up)
        .bind_key(CONTROL_ROTATE, Key::W)
        .bind_gamepad(CONTROL_ROTATE, GamepadButton::DPadUp)
        .bind_gamepad(CONTROL_ROTATE, GamepadButton::East)
        .bind_key(CONTROL_SOFT_DROP, Key::Down)
        .bind_key(CONTROL_SOFT_DROP, Key::S)
        .bind_gamepad(CONTROL_SOFT_DROP, GamepadButton::DPadDown)
        .bind_gamepad(CONTROL_SOFT_DROP, GamepadButton::LeftStickDown)
        .bind_key(CONTROL_HARD_DROP, Key::Space)
        .bind_gamepad(CONTROL_HARD_DROP, GamepadButton::South)
        .bind_key(CONTROL_EXIT, Key::Escape);
    controls
}

fn spawn(kind: Kind) -> Piece {
    Piece {
        kind,
        rotation: 0,
        x: 3,
        y: -1,
    }
}
fn base_cells(kind: Kind) -> [(i32, i32); 4] {
    match kind {
        Kind::I => [(0, 1), (1, 1), (2, 1), (3, 1)],
        Kind::J => [(0, 0), (0, 1), (1, 1), (2, 1)],
        Kind::L => [(2, 0), (0, 1), (1, 1), (2, 1)],
        Kind::O => [(1, 0), (2, 0), (1, 1), (2, 1)],
        Kind::S => [(1, 0), (2, 0), (0, 1), (1, 1)],
        Kind::T => [(1, 0), (0, 1), (1, 1), (2, 1)],
        Kind::Z => [(0, 0), (1, 0), (1, 1), (2, 1)],
    }
}
fn draw_block(fb: &mut gotoo_pixel_engine::Framebuffer, x: i32, y: i32, kind: Kind) {
    let px = BOARD_X + x * CELL_SIZE;
    let py = BOARD_Y + y * CELL_SIZE;
    fb.fill_rect(
        px + 1,
        py + 1,
        (CELL_SIZE - 1) as u32,
        (CELL_SIZE - 1) as u32,
        COLORS[kind.index()],
    );
}
fn draw_preview_block(fb: &mut gotoo_pixel_engine::Framebuffer, x: i32, y: i32, kind: Kind) {
    fb.fill_rect(x, y, 7, 7, COLORS[kind.index()]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bag_contains_every_piece_once() {
        let mut bag = Bag::new(42);
        let mut seen = [false; 7];
        for _ in 0..7 {
            seen[bag.next().index()] = true;
        }
        assert!(seen.into_iter().all(|value| value));
    }

    #[test]
    fn piece_cannot_leave_board() {
        let mut world = TetrisWorld::new();
        while world.translate(-1, 0) {}
        assert!(world.active.cells().iter().all(|&(x, _)| x >= 0));
    }

    #[test]
    fn hard_drop_locks_piece() {
        let mut world = TetrisWorld::new();
        let before = world.active.kind;
        world.hard_drop();
        assert!(
            world
                .board
                .iter()
                .flatten()
                .any(|&cell| cell == Some(before))
        );
    }

    #[test]
    fn full_line_is_cleared() {
        let mut world = TetrisWorld::new();
        world.board[(BOARD_HEIGHT - 1) as usize] = [Some(Kind::I); BOARD_WIDTH as usize];
        assert_eq!(world.clear_lines(), 1);
        assert!(
            world.board[(BOARD_HEIGHT - 1) as usize]
                .iter()
                .all(Option::is_none)
        );
    }

    #[test]
    fn restart_resets_game() {
        let mut world = TetrisWorld::new();
        world.score = 999;
        world.game_over = true;
        world.restart();
        assert_eq!(world.score, 0);
        assert!(!world.game_over);
    }
}
