use gotoo_pixel_engine::{
    ActionId, ControlMap, EngineConfig, EngineError, Frame, Framebuffer, Game, GameResult,
    GamepadButton, Key, Pixel, Rect, run,
    ui::{
        MenuState, draw_menu_item, draw_panel, draw_text_centered, menu_confirm_pressed,
        menu_down_pressed, menu_up_pressed,
    },
};

const FRAMEBUFFER_WIDTH: u32 = 320;
const FRAMEBUFFER_HEIGHT: u32 = 180;
const PLAY_TOP: f32 = 24.0;

const MOVE_LEFT: ActionId = ActionId::new("breakout.left");
const MOVE_RIGHT: ActionId = ActionId::new("breakout.right");
const RESTART: ActionId = ActionId::new("breakout.restart");

const BG: Pixel = Pixel::rgb(7, 10, 14);
const FG: Pixel = Pixel::rgb(230, 238, 228);
const ACCENT: Pixel = Pixel::rgb(120, 235, 180);
const BORDER: Pixel = Pixel::rgb(80, 150, 220);
const BRICK_COLORS: [Pixel; 5] = [
    Pixel::rgb(236, 92, 92),
    Pixel::rgb(236, 144, 84),
    Pixel::rgb(228, 204, 92),
    Pixel::rgb(116, 210, 126),
    Pixel::rgb(92, 162, 232),
];

const PADDLE_WIDTH: u32 = 48;
const PADDLE_HEIGHT: u32 = 6;
const PADDLE_Y: f32 = 164.0;
const PADDLE_SPEED: f32 = 190.0;

const BALL_SIZE: u32 = 6;
const BALL_SPEED_X: f32 = 112.0;
const BALL_SPEED_Y: f32 = -118.0;
const BALL_SPEED_MAX_X: f32 = 190.0;

const BRICK_COLUMNS: usize = 10;
const BRICK_ROWS: usize = 5;
const BRICK_WIDTH: u32 = 26;
const BRICK_HEIGHT: u32 = 10;
const BRICK_GAP_X: i32 = 2;
const BRICK_GAP_Y: i32 = 3;
const BRICK_START_X: i32 = 21;
const BRICK_START_Y: i32 = 38;

const INITIAL_LIVES: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Main,
    Controls,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RoundState {
    Playing,
    Won,
    Lost,
}

#[derive(Debug, Clone, Copy)]
struct Brick {
    rect: Rect,
    active: bool,
    color: Pixel,
}

struct BreakoutApp {
    page: Page,
    menu: MenuState,
    playing: bool,
    round_state: RoundState,
    paddle_x: f32,
    ball_x: f32,
    ball_y: f32,
    ball_vx: f32,
    ball_vy: f32,
    score: u32,
    lives: u32,
    bricks: Vec<Brick>,
    controls: ControlMap,
}

impl BreakoutApp {
    fn new() -> Self {
        let mut app = Self {
            page: Page::Main,
            menu: MenuState::new(3),
            playing: false,
            round_state: RoundState::Playing,
            paddle_x: centered_paddle_x(),
            ball_x: centered_ball_x(),
            ball_y: PADDLE_Y - BALL_SIZE as f32 - 8.0,
            ball_vx: BALL_SPEED_X,
            ball_vy: BALL_SPEED_Y,
            score: 0,
            lives: INITIAL_LIVES,
            bricks: make_bricks(),
            controls: breakout_controls(),
        };
        app.reset_ball();
        app
    }

    fn update_menu(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if frame.input.key(Key::Escape).pressed() {
            return GameResult::Exit;
        }

        match self.page {
            Page::Main => {
                if menu_up_pressed(frame.input) {
                    self.menu.select_previous();
                }
                if menu_down_pressed(frame.input) {
                    self.menu.select_next();
                }
                if menu_confirm_pressed(frame.input) {
                    match self.menu.selected() {
                        Some(0) => self.playing = true,
                        Some(1) => self.page = Page::Controls,
                        Some(2) => return GameResult::Exit,
                        _ => {}
                    }
                }
                self.render_main_menu(frame.framebuffer);
            }
            Page::Controls => {
                if frame.input.key(Key::Escape).pressed()
                    || frame.input.gamepad_button_any(GamepadButton::East).pressed()
                    || menu_confirm_pressed(frame.input)
                {
                    self.page = Page::Main;
                }
                self.render_controls(frame.framebuffer);
            }
        }

        GameResult::Continue
    }

    fn update_game(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if frame.input.key(Key::Escape).pressed() {
            return GameResult::Exit;
        }

        self.controls.update(frame.input);
        if self.round_state != RoundState::Playing {
            if self.controls.action(RESTART).pressed() {
                self.restart_round();
            }
            self.render_game(frame.framebuffer);
            return GameResult::Continue;
        }

        let dt = frame.delta_time.as_secs_f32().min(0.05);
        self.update_paddle(dt);
        self.update_ball(dt);
        self.render_game(frame.framebuffer);
        GameResult::Continue
    }

    fn update_paddle(&mut self, dt: f32) {
        let left = self.controls.action(MOVE_LEFT).held();
        let right = self.controls.action(MOVE_RIGHT).held();
        let direction = match (left, right) {
            (true, false) => -1.0,
            (false, true) => 1.0,
            _ => 0.0,
        };
        self.paddle_x = move_paddle(self.paddle_x, direction, dt);
    }

    fn update_ball(&mut self, dt: f32) {
        self.ball_x += self.ball_vx * dt;
        self.ball_y += self.ball_vy * dt;

        if self.ball_x <= 0.0 {
            self.ball_x = 0.0;
            self.ball_vx = self.ball_vx.abs();
        }
        let right = FRAMEBUFFER_WIDTH as f32 - BALL_SIZE as f32;
        if self.ball_x >= right {
            self.ball_x = right;
            self.ball_vx = -self.ball_vx.abs();
        }
        if self.ball_y <= PLAY_TOP {
            self.ball_y = PLAY_TOP;
            self.ball_vy = self.ball_vy.abs();
        }

        let paddle = Rect {
            x: self.paddle_x.round() as i32,
            y: PADDLE_Y as i32,
            width: PADDLE_WIDTH,
            height: PADDLE_HEIGHT,
        };
        let ball = ball_rect(self.ball_x, self.ball_y);
        if self.ball_vy > 0.0 && overlaps(ball, paddle) {
            self.ball_y = PADDLE_Y - BALL_SIZE as f32;
            self.ball_vy = -self.ball_vy.abs();

            let paddle_center = self.paddle_x + PADDLE_WIDTH as f32 / 2.0;
            let ball_center = self.ball_x + BALL_SIZE as f32 / 2.0;
            let offset = ((ball_center - paddle_center) / (PADDLE_WIDTH as f32 / 2.0))
                .clamp(-1.0, 1.0);
            self.ball_vx = (self.ball_vx + offset * 45.0)
                .clamp(-BALL_SPEED_MAX_X, BALL_SPEED_MAX_X);
        }

        if let Some(index) = self
            .bricks
            .iter()
            .position(|brick| {
                brick.active && overlaps(ball_rect(self.ball_x, self.ball_y), brick.rect)
            })
        {
            let brick = self.bricks[index];
            self.bricks[index].active = false;
            self.score = self.score.saturating_add(100);
            bounce_from_brick(
                brick.rect,
                self.ball_x,
                self.ball_y,
                &mut self.ball_vx,
                &mut self.ball_vy,
            );

            if self.bricks.iter().all(|brick| !brick.active) {
                self.round_state = RoundState::Won;
            }
        }

        if self.ball_y > FRAMEBUFFER_HEIGHT as f32 {
            self.lives = self.lives.saturating_sub(1);
            if self.lives == 0 {
                self.round_state = RoundState::Lost;
            } else {
                self.reset_ball();
            }
        }
    }

    fn reset_ball(&mut self) {
        self.paddle_x = centered_paddle_x();
        self.ball_x = centered_ball_x();
        self.ball_y = PADDLE_Y - BALL_SIZE as f32 - 8.0;
        self.ball_vx = if self.lives % 2 == 0 {
            -BALL_SPEED_X
        } else {
            BALL_SPEED_X
        };
        self.ball_vy = BALL_SPEED_Y;
    }

    fn restart_round(&mut self) {
        self.round_state = RoundState::Playing;
        self.score = 0;
        self.lives = INITIAL_LIVES;
        self.bricks = make_bricks();
        self.reset_ball();
    }

    fn render_main_menu(&self, framebuffer: &mut Framebuffer) {
        framebuffer.clear(BG);
        draw_panel(
            framebuffer,
            Rect {
                x: 62,
                y: 24,
                width: 196,
                height: 132,
            },
            BG,
            BORDER,
        );
        draw_text_centered(
            framebuffer,
            Rect {
                x: 76,
                y: 38,
                width: 168,
                height: 24,
            },
            "BREAKOUT",
            2,
            ACCENT,
        );

        for (index, (label, y)) in [("PLAY", 82), ("CONTROLS", 104), ("QUIT", 126)]
            .into_iter()
            .enumerate()
        {
            draw_menu_item(
                framebuffer,
                Rect {
                    x: 94,
                    y,
                    width: 132,
                    height: 16,
                },
                label,
                self.menu.selected() == Some(index),
                1,
                FG,
                ACCENT,
            );
        }
    }

    fn render_controls(&self, framebuffer: &mut Framebuffer) {
        framebuffer.clear(BG);
        draw_panel(
            framebuffer,
            Rect {
                x: 34,
                y: 24,
                width: 252,
                height: 132,
            },
            BG,
            BORDER,
        );
        draw_text_centered(
            framebuffer,
            Rect {
                x: 48,
                y: 38,
                width: 224,
                height: 16,
            },
            "CONTROLS",
            1,
            ACCENT,
        );
        framebuffer.draw_text(58, 68, "KEYBOARD  LEFT/RIGHT OR A/D", FG);
        framebuffer.draw_text(58, 86, "GAMEPAD   DPAD / LEFT STICK", FG);
        framebuffer.draw_text(58, 104, "RESTART   SPACE / SOUTH", FG);
        draw_text_centered(
            framebuffer,
            Rect {
                x: 48,
                y: 132,
                width: 224,
                height: 12,
            },
            "FIRE OR EAST TO BACK",
            1,
            FG,
        );
    }

    fn render_game(&self, framebuffer: &mut Framebuffer) {
        framebuffer.clear(BG);
        framebuffer.draw_text(8, 8, &format!("SCORE {}", self.score), FG);
        framebuffer.draw_text(242, 8, &format!("LIVES {}", self.lives), FG);
        framebuffer.draw_line(
            0,
            PLAY_TOP as i32 - 1,
            FRAMEBUFFER_WIDTH as i32 - 1,
            PLAY_TOP as i32 - 1,
            BORDER,
        );

        for brick in self.bricks.iter().filter(|brick| brick.active) {
            framebuffer.fill_rect(
                brick.rect.x,
                brick.rect.y,
                brick.rect.width,
                brick.rect.height,
                brick.color,
            );
        }

        framebuffer.fill_rect(
            self.paddle_x.round() as i32,
            PADDLE_Y as i32,
            PADDLE_WIDTH,
            PADDLE_HEIGHT,
            FG,
        );
        framebuffer.fill_rect(
            self.ball_x.round() as i32,
            self.ball_y.round() as i32,
            BALL_SIZE,
            BALL_SIZE,
            ACCENT,
        );

        match self.round_state {
            RoundState::Playing => {}
            RoundState::Won => self.render_round_message(framebuffer, "YOU WIN"),
            RoundState::Lost => self.render_round_message(framebuffer, "GAME OVER"),
        }
    }

    fn render_round_message(&self, framebuffer: &mut Framebuffer, message: &str) {
        draw_panel(
            framebuffer,
            Rect {
                x: 76,
                y: 88,
                width: 168,
                height: 54,
            },
            BG,
            BORDER,
        );
        draw_text_centered(
            framebuffer,
            Rect {
                x: 88,
                y: 98,
                width: 144,
                height: 16,
            },
            message,
            1,
            ACCENT,
        );
        draw_text_centered(
            framebuffer,
            Rect {
                x: 88,
                y: 120,
                width: 144,
                height: 12,
            },
            "SPACE/SOUTH RESTART",
            1,
            FG,
        );
    }
}

impl Game for BreakoutApp {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if self.playing {
            self.update_game(frame)
        } else {
            self.update_menu(frame)
        }
    }
}

fn breakout_controls() -> ControlMap {
    let mut controls = ControlMap::new();
    controls
        .bind_key(MOVE_LEFT, Key::Left)
        .bind_key(MOVE_LEFT, Key::A)
        .bind_gamepad(MOVE_LEFT, GamepadButton::DPadLeft)
        .bind_gamepad(MOVE_LEFT, GamepadButton::LeftStickLeft)
        .bind_key(MOVE_RIGHT, Key::Right)
        .bind_key(MOVE_RIGHT, Key::D)
        .bind_gamepad(MOVE_RIGHT, GamepadButton::DPadRight)
        .bind_gamepad(MOVE_RIGHT, GamepadButton::LeftStickRight)
        .bind_key(RESTART, Key::Space)
        .bind_gamepad(RESTART, GamepadButton::South);
    controls
}

fn make_bricks() -> Vec<Brick> {
    let mut bricks = Vec::with_capacity(BRICK_COLUMNS * BRICK_ROWS);
    for row in 0..BRICK_ROWS {
        for column in 0..BRICK_COLUMNS {
            bricks.push(Brick {
                rect: Rect {
                    x: BRICK_START_X + column as i32 * (BRICK_WIDTH as i32 + BRICK_GAP_X),
                    y: BRICK_START_Y + row as i32 * (BRICK_HEIGHT as i32 + BRICK_GAP_Y),
                    width: BRICK_WIDTH,
                    height: BRICK_HEIGHT,
                },
                active: true,
                color: BRICK_COLORS[row],
            });
        }
    }
    bricks
}

fn move_paddle(x: f32, direction: f32, dt: f32) -> f32 {
    let max_x = FRAMEBUFFER_WIDTH as f32 - PADDLE_WIDTH as f32;
    (x + direction * PADDLE_SPEED * dt).clamp(0.0, max_x)
}

fn bounce_from_brick(
    brick: Rect,
    ball_x: f32,
    ball_y: f32,
    ball_vx: &mut f32,
    ball_vy: &mut f32,
) {
    let ball_left = ball_x;
    let ball_right = ball_x + BALL_SIZE as f32;
    let ball_top = ball_y;
    let ball_bottom = ball_y + BALL_SIZE as f32;
    let brick_left = brick.x as f32;
    let brick_right = brick_left + brick.width as f32;
    let brick_top = brick.y as f32;
    let brick_bottom = brick_top + brick.height as f32;

    let penetration_left = ball_right - brick_left;
    let penetration_right = brick_right - ball_left;
    let penetration_top = ball_bottom - brick_top;
    let penetration_bottom = brick_bottom - ball_top;
    let horizontal = penetration_left.min(penetration_right);
    let vertical = penetration_top.min(penetration_bottom);

    if horizontal < vertical {
        *ball_vx = -*ball_vx;
    } else {
        *ball_vy = -*ball_vy;
    }
}

fn ball_rect(x: f32, y: f32) -> Rect {
    Rect {
        x: x.round() as i32,
        y: y.round() as i32,
        width: BALL_SIZE,
        height: BALL_SIZE,
    }
}

fn overlaps(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.width as i32
        && a.x + a.width as i32 > b.x
        && a.y < b.y + b.height as i32
        && a.y + a.height as i32 > b.y
}

fn centered_paddle_x() -> f32 {
    (FRAMEBUFFER_WIDTH - PADDLE_WIDTH) as f32 / 2.0
}

fn centered_ball_x() -> f32 {
    (FRAMEBUFFER_WIDTH - BALL_SIZE) as f32 / 2.0
}

fn main() -> Result<(), EngineError> {
    run(
        EngineConfig {
            title: "Breakout - Gotoo Pixel Engine".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width: FRAMEBUFFER_WIDTH * 3,
            window_height: FRAMEBUFFER_HEIGHT * 3,
        },
        BreakoutApp::new(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brick_grid_has_expected_size_and_active_bricks() {
        let bricks = make_bricks();
        assert_eq!(bricks.len(), BRICK_COLUMNS * BRICK_ROWS);
        assert!(bricks.iter().all(|brick| brick.active));
    }

    #[test]
    fn paddle_is_clamped_to_playfield() {
        assert_eq!(move_paddle(0.0, -1.0, 1.0), 0.0);
        assert_eq!(
            move_paddle(FRAMEBUFFER_WIDTH as f32, 1.0, 1.0),
            FRAMEBUFFER_WIDTH as f32 - PADDLE_WIDTH as f32
        );
    }

    #[test]
    fn rectangles_overlap_only_when_they_share_area() {
        let brick = Rect {
            x: 10,
            y: 10,
            width: 20,
            height: 10,
        };
        assert!(overlaps(
            Rect {
                x: 12,
                y: 12,
                width: 4,
                height: 4,
            },
            brick
        ));
        assert!(!overlaps(
            Rect {
                x: 30,
                y: 12,
                width: 4,
                height: 4,
            },
            brick
        ));
    }
}
