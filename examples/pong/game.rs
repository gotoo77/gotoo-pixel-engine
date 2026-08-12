use gotoo_pixel_engine::{Frame, Game, GameResult, Key, Pixel};

pub const FRAMEBUFFER_WIDTH: u32 = 320;
pub const FRAMEBUFFER_HEIGHT: u32 = 180;

const PADDLE_WIDTH: f32 = 5.0;
const PADDLE_HEIGHT: f32 = 30.0;
const PADDLE_MARGIN: f32 = 12.0;
const PADDLE_SPEED: f32 = 125.0;
const BALL_RADIUS: f32 = 3.0;
const INITIAL_BALL_SPEED: f32 = 105.0;
const MAX_BALL_SPEED: f32 = 190.0;
const SPEEDUP_PER_HIT: f32 = 1.035;
const MAX_BOUNCE_ANGLE: f32 = std::f32::consts::PI / 3.0;
const WIN_SCORE: u8 = 7;

const BG: Pixel = Pixel::rgb(8, 11, 15);
const FG: Pixel = Pixel::rgb(226, 234, 218);
const DIM: Pixel = Pixel::rgb(70, 82, 92);
const ACCENT: Pixel = Pixel::rgb(96, 220, 180);
const WIN: Pixel = Pixel::rgb(245, 207, 82);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RoundState {
    WaitingServe,
    Playing,
    MatchOver,
}

#[derive(Debug, Clone)]
struct PongWorld {
    left_y: f32,
    right_y: f32,
    ball_x: f32,
    ball_y: f32,
    ball_vx: f32,
    ball_vy: f32,
    left_score: u8,
    right_score: u8,
    serve_direction: f32,
    state: RoundState,
}

impl PongWorld {
    fn new() -> Self {
        let mut world = Self {
            left_y: 0.0,
            right_y: 0.0,
            ball_x: 0.0,
            ball_y: 0.0,
            ball_vx: 0.0,
            ball_vy: 0.0,
            left_score: 0,
            right_score: 0,
            serve_direction: 1.0,
            state: RoundState::WaitingServe,
        };
        world.reset_match();
        world
    }

    fn reset_match(&mut self) {
        self.left_score = 0;
        self.right_score = 0;
        self.left_y = centered_paddle_y();
        self.right_y = centered_paddle_y();
        self.serve_direction = 1.0;
        self.reset_ball();
        self.state = RoundState::WaitingServe;
    }

    fn reset_ball(&mut self) {
        self.ball_x = FRAMEBUFFER_WIDTH as f32 * 0.5;
        self.ball_y = FRAMEBUFFER_HEIGHT as f32 * 0.5;
        self.ball_vx = 0.0;
        self.ball_vy = 0.0;
    }

    fn serve(&mut self) {
        if self.state != RoundState::WaitingServe {
            return;
        }
        let vertical = if (self.left_score + self.right_score) % 2 == 0 {
            0.32
        } else {
            -0.32
        };
        let horizontal = (1.0_f32 - vertical * vertical).sqrt();
        self.ball_vx = self.serve_direction * INITIAL_BALL_SPEED * horizontal;
        self.ball_vy = INITIAL_BALL_SPEED * vertical;
        self.state = RoundState::Playing;
    }

    fn move_left(&mut self, dy: f32) {
        self.left_y = clamp_paddle(self.left_y + dy);
    }

    fn move_right(&mut self, dy: f32) {
        self.right_y = clamp_paddle(self.right_y + dy);
    }

    fn update_ball(&mut self, dt: f32) {
        if self.state != RoundState::Playing {
            return;
        }

        let dt = dt.min(0.05);
        let total_dx = self.ball_vx * dt;
        let total_dy = self.ball_vy * dt;
        let largest_delta = total_dx.abs().max(total_dy.abs());
        let steps = ((largest_delta / BALL_RADIUS.max(1.0)).ceil() as u32).clamp(1, 32);
        let step_dt = dt / steps as f32;

        for _ in 0..steps {
            if self.state != RoundState::Playing {
                break;
            }
            self.ball_x += self.ball_vx * step_dt;
            self.ball_y += self.ball_vy * step_dt;
            self.resolve_walls();
            self.resolve_paddles();
            self.resolve_goal();
        }
    }

    fn resolve_walls(&mut self) {
        if self.ball_y - BALL_RADIUS <= 0.0 && self.ball_vy < 0.0 {
            self.ball_y = BALL_RADIUS;
            self.ball_vy = self.ball_vy.abs();
        }
        let bottom = FRAMEBUFFER_HEIGHT as f32 - BALL_RADIUS;
        if self.ball_y + BALL_RADIUS >= FRAMEBUFFER_HEIGHT as f32 && self.ball_vy > 0.0 {
            self.ball_y = bottom;
            self.ball_vy = -self.ball_vy.abs();
        }
    }

    fn resolve_paddles(&mut self) {
        let left_x = PADDLE_MARGIN;
        let right_x = FRAMEBUFFER_WIDTH as f32 - PADDLE_MARGIN - PADDLE_WIDTH;

        if self.ball_vx < 0.0
            && ball_intersects_paddle(self.ball_x, self.ball_y, left_x, self.left_y)
        {
            self.ball_x = left_x + PADDLE_WIDTH + BALL_RADIUS;
            self.bounce_from_paddle(self.left_y, 1.0);
        } else if self.ball_vx > 0.0
            && ball_intersects_paddle(self.ball_x, self.ball_y, right_x, self.right_y)
        {
            self.ball_x = right_x - BALL_RADIUS;
            self.bounce_from_paddle(self.right_y, -1.0);
        }
    }

    fn bounce_from_paddle(&mut self, paddle_y: f32, direction: f32) {
        let paddle_center = paddle_y + PADDLE_HEIGHT * 0.5;
        let impact = ((self.ball_y - paddle_center) / (PADDLE_HEIGHT * 0.5)).clamp(-1.0, 1.0);
        let angle = impact * MAX_BOUNCE_ANGLE;
        let speed = (self.ball_vx.hypot(self.ball_vy) * SPEEDUP_PER_HIT)
            .clamp(INITIAL_BALL_SPEED, MAX_BALL_SPEED);
        self.ball_vx = direction * speed * angle.cos();
        self.ball_vy = speed * angle.sin();
    }

    fn resolve_goal(&mut self) {
        if self.ball_x + BALL_RADIUS < 0.0 {
            self.right_score = self.right_score.saturating_add(1);
            self.after_point(-1.0);
        } else if self.ball_x - BALL_RADIUS > FRAMEBUFFER_WIDTH as f32 {
            self.left_score = self.left_score.saturating_add(1);
            self.after_point(1.0);
        }
    }

    fn after_point(&mut self, direction_toward_loser: f32) {
        self.serve_direction = direction_toward_loser;
        self.reset_ball();
        if self.left_score >= WIN_SCORE || self.right_score >= WIN_SCORE {
            self.state = RoundState::MatchOver;
        } else {
            self.state = RoundState::WaitingServe;
        }
    }

    fn winner(&self) -> Option<u8> {
        if self.left_score >= WIN_SCORE {
            Some(1)
        } else if self.right_score >= WIN_SCORE {
            Some(2)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct PongGame {
    world: PongWorld,
}

impl PongGame {
    pub fn new() -> Self {
        Self {
            world: PongWorld::new(),
        }
    }

    fn handle_input(&mut self, frame: &Frame<'_>) -> GameResult {
        if frame.input.key(Key::Escape).pressed() {
            return GameResult::Exit;
        }

        if self.world.state == RoundState::MatchOver {
            if frame.input.key(Key::Space).pressed() {
                self.world.reset_match();
            }
            return GameResult::Continue;
        }

        let dt = frame.delta_time.as_secs_f32().min(0.05);
        let distance = PADDLE_SPEED * dt;

        if frame.input.key(Key::W).held() {
            self.world.move_left(-distance);
        }
        if frame.input.key(Key::S).held() {
            self.world.move_left(distance);
        }
        if frame.input.key(Key::Up).held() {
            self.world.move_right(-distance);
        }
        if frame.input.key(Key::Down).held() {
            self.world.move_right(distance);
        }

        if frame.input.key(Key::Space).pressed() {
            self.world.serve();
        }

        GameResult::Continue
    }

    fn render(&self, frame: &mut Frame<'_>) {
        let fb = &mut frame.framebuffer;
        fb.clear(BG);

        let center_x = FRAMEBUFFER_WIDTH as i32 / 2;
        for y in (8..FRAMEBUFFER_HEIGHT as i32 - 8).step_by(8) {
            fb.fill_rect(center_x - 1, y, 2, 4, DIM);
        }

        fb.draw_text(8, 6, "P1 W/S", DIM);
        fb.draw_text(255, 6, "P2 UP/DN", DIM);

        let left_score = self.world.left_score.to_string();
        let right_score = self.world.right_score.to_string();
        fb.draw_text_scaled(132, 10, &left_score, 2, FG);
        fb.draw_text_scaled(174, 10, &right_score, 2, FG);

        draw_paddle(fb, PADDLE_MARGIN, self.world.left_y, ACCENT);
        draw_paddle(
            fb,
            FRAMEBUFFER_WIDTH as f32 - PADDLE_MARGIN - PADDLE_WIDTH,
            self.world.right_y,
            ACCENT,
        );

        fb.fill_circle(
            self.world.ball_x.round() as i32,
            self.world.ball_y.round() as i32,
            BALL_RADIUS as u32,
            FG,
        );

        match self.world.state {
            RoundState::WaitingServe => {
                fb.draw_text(116, 158, "SPACE TO SERVE", FG);
            }
            RoundState::Playing => {}
            RoundState::MatchOver => {
                fb.fill_rect(92, 66, 136, 48, BG);
                fb.draw_rect(92, 66, 136, 48, WIN);
                if let Some(player) = self.world.winner() {
                    fb.draw_text(121, 77, &format!("PLAYER {player} WINS"), WIN);
                }
                fb.draw_text(108, 96, "SPACE TO REPLAY", FG);
            }
        }
    }
}

impl Game for PongGame {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let result = self.handle_input(frame);
        if result == GameResult::Exit {
            return result;
        }
        self.world.update_ball(frame.delta_time.as_secs_f32());
        self.render(frame);
        GameResult::Continue
    }
}

fn centered_paddle_y() -> f32 {
    (FRAMEBUFFER_HEIGHT as f32 - PADDLE_HEIGHT) * 0.5
}

fn clamp_paddle(y: f32) -> f32 {
    y.clamp(0.0, FRAMEBUFFER_HEIGHT as f32 - PADDLE_HEIGHT)
}

fn ball_intersects_paddle(ball_x: f32, ball_y: f32, x: f32, y: f32) -> bool {
    let ball_left = ball_x - BALL_RADIUS;
    let ball_right = ball_x + BALL_RADIUS;
    let ball_top = ball_y - BALL_RADIUS;
    let ball_bottom = ball_y + BALL_RADIUS;

    ball_right >= x
        && ball_left <= x + PADDLE_WIDTH
        && ball_bottom >= y
        && ball_top <= y + PADDLE_HEIGHT
}

fn draw_paddle(fb: &mut gotoo_pixel_engine::Framebuffer, x: f32, y: f32, color: Pixel) {
    fb.fill_rect(
        x.round() as i32,
        y.round() as i32,
        PADDLE_WIDTH as u32,
        PADDLE_HEIGHT as u32,
        color,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paddles_are_centered_on_new_match() {
        let world = PongWorld::new();
        assert_eq!(world.left_y, centered_paddle_y());
        assert_eq!(world.right_y, centered_paddle_y());
    }

    #[test]
    fn paddle_movement_is_clamped_to_playfield() {
        let mut world = PongWorld::new();
        world.move_left(-10_000.0);
        world.move_right(10_000.0);
        assert_eq!(world.left_y, 0.0);
        assert_eq!(world.right_y, FRAMEBUFFER_HEIGHT as f32 - PADDLE_HEIGHT);
    }

    #[test]
    fn serve_starts_ball_motion() {
        let mut world = PongWorld::new();
        world.serve();
        assert_eq!(world.state, RoundState::Playing);
        assert!(world.ball_vx > 0.0);
        assert!(world.ball_vy != 0.0);
    }

    #[test]
    fn point_updates_score_and_waits_for_next_serve() {
        let mut world = PongWorld::new();
        world.state = RoundState::Playing;
        world.ball_x = -BALL_RADIUS - 1.0;
        world.resolve_goal();
        assert_eq!(world.right_score, 1);
        assert_eq!(world.state, RoundState::WaitingServe);
        assert_eq!(world.ball_x, FRAMEBUFFER_WIDTH as f32 * 0.5);
    }

    #[test]
    fn first_player_to_seven_wins_match() {
        let mut world = PongWorld::new();
        world.left_score = WIN_SCORE - 1;
        world.state = RoundState::Playing;
        world.ball_x = FRAMEBUFFER_WIDTH as f32 + BALL_RADIUS + 1.0;
        world.resolve_goal();
        assert_eq!(world.left_score, WIN_SCORE);
        assert_eq!(world.state, RoundState::MatchOver);
        assert_eq!(world.winner(), Some(1));
    }

    #[test]
    fn paddle_impact_controls_vertical_direction() {
        let mut world = PongWorld::new();
        world.ball_vx = -INITIAL_BALL_SPEED;
        world.ball_vy = 0.0;
        world.ball_y = world.left_y + 2.0;
        world.bounce_from_paddle(world.left_y, 1.0);
        assert!(world.ball_vx > 0.0);
        assert!(world.ball_vy < 0.0);
    }

    #[test]
    fn reset_match_clears_scores_and_returns_to_serve() {
        let mut world = PongWorld::new();
        world.left_score = 7;
        world.right_score = 5;
        world.state = RoundState::MatchOver;
        world.reset_match();
        assert_eq!(world.left_score, 0);
        assert_eq!(world.right_score, 0);
        assert_eq!(world.state, RoundState::WaitingServe);
    }
}
