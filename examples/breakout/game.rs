use gotoo_pixel_engine::{Frame, Framebuffer, Game, GameResult, Key, Pixel};

pub const FRAMEBUFFER_WIDTH: u32 = 320;
pub const FRAMEBUFFER_HEIGHT: u32 = 180;

const WORLD_W: f32 = FRAMEBUFFER_WIDTH as f32;
const WORLD_H: f32 = FRAMEBUFFER_HEIGHT as f32;

const PADDLE_W: f32 = 42.0;
const PADDLE_H: f32 = 5.0;
const PADDLE_Y: f32 = 164.0;
const PADDLE_SPEED: f32 = 190.0;

const BALL_SIZE: f32 = 4.0;
const BASE_BALL_SPEED: f32 = 112.0;
const MAX_BALL_SPEED: f32 = 190.0;

const BRICK_COLS: usize = 10;
const BRICK_ROWS: usize = 6;
const BRICK_W: f32 = 28.0;
const BRICK_H: f32 = 8.0;
const BRICK_GAP: f32 = 2.0;
const BRICK_START_X: f32 = 11.0;
const BRICK_START_Y: f32 = 28.0;

const BG: Pixel = Pixel::rgb(8, 11, 16);
const TEXT: Pixel = Pixel::rgb(226, 234, 218);
const BALL: Pixel = Pixel::rgb(244, 244, 232);
const PADDLE: Pixel = Pixel::rgb(110, 214, 255);
const BORDER: Pixel = Pixel::rgb(48, 60, 72);
const GAME_OVER: Pixel = Pixel::rgb(245, 76, 76);

const BRICK_COLORS: [Pixel; BRICK_ROWS] = [
    Pixel::rgb(240, 82, 82),
    Pixel::rgb(244, 135, 72),
    Pixel::rgb(239, 203, 82),
    Pixel::rgb(100, 209, 117),
    Pixel::rgb(84, 170, 237),
    Pixel::rgb(167, 108, 230),
];

#[derive(Debug, Clone)]
struct Brick {
    x: f32,
    y: f32,
    active: bool,
    row: usize,
}

#[derive(Debug, Clone, Copy)]
struct BallState {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    stuck: bool,
}

#[derive(Debug, Clone)]
struct BreakoutWorld {
    paddle_x: f32,
    ball: BallState,
    bricks: Vec<Brick>,
    score: u32,
    lives: u32,
    level: u32,
    game_over: bool,
}

impl BreakoutWorld {
    fn new() -> Self {
        let mut world = Self {
            paddle_x: (WORLD_W - PADDLE_W) * 0.5,
            ball: BallState {
                x: 0.0,
                y: 0.0,
                vx: 0.0,
                vy: 0.0,
                stuck: true,
            },
            bricks: make_bricks(),
            score: 0,
            lives: 3,
            level: 1,
            game_over: false,
        };
        world.reset_ball();
        world
    }

    fn restart(&mut self) {
        *self = Self::new();
    }

    fn level_speed(&self) -> f32 {
        (BASE_BALL_SPEED + (self.level.saturating_sub(1) as f32) * 12.0).min(MAX_BALL_SPEED)
    }

    fn reset_ball(&mut self) {
        let speed = self.level_speed();
        self.ball = BallState {
            x: self.paddle_x + (PADDLE_W - BALL_SIZE) * 0.5,
            y: PADDLE_Y - BALL_SIZE - 2.0,
            vx: speed * 0.58,
            vy: -speed * 0.82,
            stuck: true,
        };
    }

    fn launch(&mut self) {
        if !self.game_over {
            self.ball.stuck = false;
        }
    }

    fn move_paddle(&mut self, direction: f32, dt: f32) {
        self.paddle_x = (self.paddle_x + direction * PADDLE_SPEED * dt)
            .clamp(0.0, WORLD_W - PADDLE_W);

        if self.ball.stuck {
            self.ball.x = self.paddle_x + (PADDLE_W - BALL_SIZE) * 0.5;
            self.ball.y = PADDLE_Y - BALL_SIZE - 2.0;
        }
    }

    fn update_ball(&mut self, dt: f32) {
        if self.game_over || self.ball.stuck {
            return;
        }

        let dt = dt.min(0.05);
        let travel = self.ball.vx.abs().max(self.ball.vy.abs()) * dt;
        let steps = ((travel / 1.5).ceil() as u32).max(1);
        let step_dt = dt / steps as f32;

        for _ in 0..steps {
            if self.ball.stuck || self.game_over {
                break;
            }

            self.step_ball(step_dt);
        }
    }

    fn step_ball(&mut self, dt: f32) {
        self.ball.x += self.ball.vx * dt;

        if self.ball.x <= 0.0 {
            self.ball.x = 0.0;
            self.ball.vx = self.ball.vx.abs();
        } else if self.ball.x + BALL_SIZE >= WORLD_W {
            self.ball.x = WORLD_W - BALL_SIZE;
            self.ball.vx = -self.ball.vx.abs();
        }

        if self.hit_brick_at(self.ball.x, self.ball.y) {
            self.ball.x -= self.ball.vx * dt;
            self.ball.vx = -self.ball.vx;
            self.advance_level_if_cleared();
            if self.ball.stuck {
                return;
            }
        }

        self.ball.y += self.ball.vy * dt;

        if self.ball.y <= 18.0 {
            self.ball.y = 18.0;
            self.ball.vy = self.ball.vy.abs();
        }

        if self.ball.vy > 0.0 && overlaps(
            self.ball.x,
            self.ball.y,
            BALL_SIZE,
            BALL_SIZE,
            self.paddle_x,
            PADDLE_Y,
            PADDLE_W,
            PADDLE_H,
        ) {
            self.ball.y = PADDLE_Y - BALL_SIZE;
            self.bounce_from_paddle();
        } else if self.hit_brick_at(self.ball.x, self.ball.y) {
            self.ball.y -= self.ball.vy * dt;
            self.ball.vy = -self.ball.vy;
            self.advance_level_if_cleared();
            if self.ball.stuck {
                return;
            }
        }

        if self.ball.y > WORLD_H {
            self.lose_life();
        }
    }

    fn bounce_from_paddle(&mut self) {
        let paddle_center = self.paddle_x + PADDLE_W * 0.5;
        let ball_center = self.ball.x + BALL_SIZE * 0.5;
        let offset = ((ball_center - paddle_center) / (PADDLE_W * 0.5)).clamp(-1.0, 1.0);
        let speed = self.ball.vx.hypot(self.ball.vy).max(self.level_speed());

        self.ball.vx = speed * 0.82 * offset;
        let vertical = (speed * speed - self.ball.vx * self.ball.vx).max(0.0).sqrt();
        self.ball.vy = -vertical.max(speed * 0.45);
    }

    fn hit_brick_at(&mut self, x: f32, y: f32) -> bool {
        for brick in &mut self.bricks {
            if !brick.active {
                continue;
            }

            if overlaps(x, y, BALL_SIZE, BALL_SIZE, brick.x, brick.y, BRICK_W, BRICK_H) {
                brick.active = false;
                self.score = self.score.saturating_add((BRICK_ROWS - brick.row) as u32 * 10);
                return true;
            }
        }

        false
    }

    fn advance_level_if_cleared(&mut self) {
        if self.bricks.iter().any(|brick| brick.active) {
            return;
        }

        self.level = self.level.saturating_add(1);
        self.bricks = make_bricks();
        self.paddle_x = (WORLD_W - PADDLE_W) * 0.5;
        self.reset_ball();
    }

    fn lose_life(&mut self) {
        self.lives = self.lives.saturating_sub(1);
        if self.lives == 0 {
            self.game_over = true;
            self.ball.stuck = true;
        } else {
            self.paddle_x = (WORLD_W - PADDLE_W) * 0.5;
            self.reset_ball();
        }
    }
}

#[derive(Debug)]
pub struct BreakoutGame {
    world: BreakoutWorld,
}

impl BreakoutGame {
    pub fn new() -> Self {
        Self {
            world: BreakoutWorld::new(),
        }
    }

    fn input(&mut self, frame: &Frame<'_>) -> GameResult {
        if frame.input.key(Key::Escape).pressed() {
            return GameResult::Exit;
        }

        if self.world.game_over {
            if frame.input.key(Key::Space).pressed() {
                self.world.restart();
            }
            return GameResult::Continue;
        }

        let mut direction = 0.0;
        if frame.input.key(Key::Left).held() || frame.input.key(Key::A).held() {
            direction -= 1.0;
        }
        if frame.input.key(Key::Right).held() || frame.input.key(Key::D).held() {
            direction += 1.0;
        }

        self.world
            .move_paddle(direction, frame.delta_time.as_secs_f32().min(0.05));

        if frame.input.key(Key::Space).pressed() {
            self.world.launch();
        }

        GameResult::Continue
    }

    fn render(&self, frame: &mut Frame<'_>) {
        let fb = &mut frame.framebuffer;
        fb.clear(BG);

        fb.draw_text(8, 5, "BREAKOUT", TEXT);
        fb.draw_text(112, 5, "SCORE", TEXT);
        fb.draw_text(151, 5, &self.world.score.to_string(), TEXT);
        fb.draw_text(207, 5, "LIVES", TEXT);
        fb.draw_text(246, 5, &self.world.lives.to_string(), TEXT);
        fb.draw_text(268, 5, "LV", TEXT);
        fb.draw_text(287, 5, &self.world.level.to_string(), TEXT);
        fb.draw_line(0, 17, FRAMEBUFFER_WIDTH as i32 - 1, 17, BORDER);

        for brick in &self.world.bricks {
            if brick.active {
                draw_brick(fb, brick);
            }
        }

        fb.fill_rect(
            self.world.paddle_x.round() as i32,
            PADDLE_Y as i32,
            PADDLE_W as u32,
            PADDLE_H as u32,
            PADDLE,
        );

        fb.fill_rect(
            self.world.ball.x.round() as i32,
            self.world.ball.y.round() as i32,
            BALL_SIZE as u32,
            BALL_SIZE as u32,
            BALL,
        );

        if self.world.game_over {
            fb.fill_rect(91, 109, 138, 38, BG);
            fb.draw_rect(91, 109, 138, 38, GAME_OVER);
            fb.draw_text(123, 117, "GAME OVER", GAME_OVER);
            fb.draw_text(106, 133, "SPACE TO REPLAY", TEXT);
        } else if self.world.ball.stuck {
            fb.draw_text(108, 148, "SPACE TO LAUNCH", TEXT);
        }
    }
}

impl Game for BreakoutGame {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let result = self.input(frame);
        if result == GameResult::Exit {
            return result;
        }

        self.world.update_ball(frame.delta_time.as_secs_f32());
        self.render(frame);
        GameResult::Continue
    }
}

fn make_bricks() -> Vec<Brick> {
    let mut bricks = Vec::with_capacity(BRICK_COLS * BRICK_ROWS);

    for row in 0..BRICK_ROWS {
        for col in 0..BRICK_COLS {
            bricks.push(Brick {
                x: BRICK_START_X + col as f32 * (BRICK_W + BRICK_GAP),
                y: BRICK_START_Y + row as f32 * (BRICK_H + BRICK_GAP),
                active: true,
                row,
            });
        }
    }

    bricks
}

fn draw_brick(fb: &mut Framebuffer, brick: &Brick) {
    let x = brick.x as i32;
    let y = brick.y as i32;
    fb.fill_rect(x, y, BRICK_W as u32, BRICK_H as u32, BRICK_COLORS[brick.row]);
    fb.draw_line(
        x,
        y + BRICK_H as i32 - 1,
        x + BRICK_W as i32 - 1,
        y + BRICK_H as i32 - 1,
        BG,
    );
}

#[allow(clippy::too_many_arguments)]
fn overlaps(
    ax: f32,
    ay: f32,
    aw: f32,
    ah: f32,
    bx: f32,
    by: f32,
    bw: f32,
    bh: f32,
) -> bool {
    ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_starts_with_full_brick_wall() {
        let world = BreakoutWorld::new();
        assert_eq!(world.bricks.len(), BRICK_COLS * BRICK_ROWS);
        assert!(world.bricks.iter().all(|brick| brick.active));
    }

    #[test]
    fn launch_releases_ball() {
        let mut world = BreakoutWorld::new();
        assert!(world.ball.stuck);
        world.launch();
        assert!(!world.ball.stuck);
    }

    #[test]
    fn paddle_is_clamped_to_playfield() {
        let mut world = BreakoutWorld::new();
        world.move_paddle(-1.0, 10.0);
        assert_eq!(world.paddle_x, 0.0);
        world.move_paddle(1.0, 10.0);
        assert_eq!(world.paddle_x, WORLD_W - PADDLE_W);
    }

    #[test]
    fn losing_ball_consumes_a_life() {
        let mut world = BreakoutWorld::new();
        world.ball.stuck = false;
        world.ball.y = WORLD_H + 1.0;
        world.ball.vy = 1.0;
        world.update_ball(0.016);
        assert_eq!(world.lives, 2);
        assert!(world.ball.stuck);
    }

    #[test]
    fn clearing_wall_advances_level_and_rebuilds_bricks() {
        let mut world = BreakoutWorld::new();
        for brick in &mut world.bricks {
            brick.active = false;
        }
        world.advance_level_if_cleared();
        assert_eq!(world.level, 2);
        assert_eq!(world.bricks.len(), BRICK_COLS * BRICK_ROWS);
        assert!(world.bricks.iter().all(|brick| brick.active));
        assert!(world.ball.stuck);
    }

    #[test]
    fn upper_rows_are_worth_more_points() {
        let mut world = BreakoutWorld::new();
        let top = world.bricks[0].clone();
        assert!(world.hit_brick_at(top.x, top.y));
        assert_eq!(world.score, 60);
    }
}
