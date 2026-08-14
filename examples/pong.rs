use gotoo_pixel_engine::{
    ActionId, ControlMap, EngineConfig, EngineError, Frame, Framebuffer, Game, GameResult,
    GamepadButton, GamepadId, Input, Key, Pixel, Rect, run,
    ui::{
        MenuState, draw_menu_item, draw_panel, draw_text_centered, menu_confirm_pressed,
        menu_down_pressed, menu_up_pressed,
    },
};

const FRAMEBUFFER_WIDTH: u32 = 320;
const FRAMEBUFFER_HEIGHT: u32 = 180;

const P1_UP: ActionId = ActionId::new("pong.p1.up");
const P1_DOWN: ActionId = ActionId::new("pong.p1.down");
const P2_UP: ActionId = ActionId::new("pong.p2.up");
const P2_DOWN: ActionId = ActionId::new("pong.p2.down");

const BG: Pixel = Pixel::rgb(6, 10, 14);
const FG: Pixel = Pixel::rgb(230, 238, 230);
const ACCENT: Pixel = Pixel::rgb(110, 235, 180);
const BORDER: Pixel = Pixel::rgb(80, 150, 220);

const PADDLE_WIDTH: u32 = 6;
const PADDLE_HEIGHT: u32 = 30;
const PADDLE_SPEED: f32 = 150.0;
const BALL_SIZE: u32 = 6;
const BALL_SPEED_X: f32 = 125.0;
const BALL_SPEED_Y: f32 = 72.0;
const BALL_SPEED_MAX: f32 = 220.0;
const BALL_SUBSTEP: f32 = 3.0;
const MAX_BOUNCE_ANGLE: f32 = std::f32::consts::PI / 3.0;
const WIN_SCORE: u32 = 7;
const PLAY_TOP: f32 = 24.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Main,
    Controls,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatchState {
    WaitingServe,
    Playing,
    MatchOver,
}

struct PongApp {
    page: Page,
    menu: MenuState,
    playing: bool,
    match_state: MatchState,
    p1_y: f32,
    p2_y: f32,
    ball_x: f32,
    ball_y: f32,
    ball_vx: f32,
    ball_vy: f32,
    p1_score: u32,
    p2_score: u32,
    serve_direction: f32,
    assigned_gamepads: [Option<GamepadId>; 2],
    p1_controls: ControlMap,
    p2_controls: ControlMap,
}

impl PongApp {
    fn new() -> Self {
        let mut app = Self {
            page: Page::Main,
            menu: MenuState::new(3),
            playing: false,
            match_state: MatchState::WaitingServe,
            p1_y: centered_paddle_y(),
            p2_y: centered_paddle_y(),
            ball_x: centered_ball_x(),
            ball_y: centered_ball_y(),
            ball_vx: 0.0,
            ball_vy: 0.0,
            p1_score: 0,
            p2_score: 0,
            serve_direction: 1.0,
            assigned_gamepads: [None, None],
            p1_controls: ControlMap::new(),
            p2_controls: ControlMap::new(),
        };
        app.rebuild_controls();
        app
    }

    fn update_assignments(&mut self, input: &Input) {
        let mut ids = input.gamepad_ids().collect::<Vec<_>>();
        ids.sort_by_key(|id| id.as_usize());
        let next = [ids.first().copied(), ids.get(1).copied()];
        if next != self.assigned_gamepads {
            self.assigned_gamepads = next;
            self.rebuild_controls();
        }
    }

    fn rebuild_controls(&mut self) {
        self.p1_controls = player_controls(
            P1_UP,
            P1_DOWN,
            Key::W,
            Key::S,
            self.assigned_gamepads[0],
        );
        self.p2_controls = player_controls(
            P2_UP,
            P2_DOWN,
            Key::Up,
            Key::Down,
            self.assigned_gamepads[1],
        );
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
                        Some(0) => {
                            self.reset_match();
                            self.playing = true;
                        }
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

        self.p1_controls.update(frame.input);
        self.p2_controls.update(frame.input);

        let dt = frame.delta_time.as_secs_f32().min(0.05);
        if self.match_state != MatchState::MatchOver {
            self.p1_y = move_paddle(
                self.p1_y,
                self.p1_controls.action(P1_UP).held(),
                self.p1_controls.action(P1_DOWN).held(),
                dt,
            );
            self.p2_y = move_paddle(
                self.p2_y,
                self.p2_controls.action(P2_UP).held(),
                self.p2_controls.action(P2_DOWN).held(),
                dt,
            );
        }

        let confirm = frame.input.key(Key::Space).pressed()
            || frame.input.gamepad_button_any(GamepadButton::South).pressed();

        match self.match_state {
            MatchState::WaitingServe if confirm => self.serve(),
            MatchState::MatchOver if confirm => self.reset_match(),
            _ => {}
        }

        if self.match_state == MatchState::Playing {
            self.update_ball(dt);
        }

        self.render_game(frame.framebuffer);
        GameResult::Continue
    }

    fn serve(&mut self) {
        if self.match_state != MatchState::WaitingServe {
            return;
        }

        self.ball_vx = BALL_SPEED_X * self.serve_direction;
        self.ball_vy = if (self.p1_score + self.p2_score) % 2 == 0 {
            BALL_SPEED_Y
        } else {
            -BALL_SPEED_Y
        };
        self.match_state = MatchState::Playing;
    }

    fn update_ball(&mut self, dt: f32) {
        let largest_delta = self.ball_vx.abs().max(self.ball_vy.abs()) * dt;
        let steps = ((largest_delta / BALL_SUBSTEP).ceil() as u32).clamp(1, 32);
        let step_dt = dt / steps as f32;

        for _ in 0..steps {
            if self.match_state != MatchState::Playing {
                break;
            }
            self.step_ball(step_dt);
        }
    }

    fn step_ball(&mut self, dt: f32) {
        self.ball_x += self.ball_vx * dt;
        self.ball_y += self.ball_vy * dt;

        if self.ball_y <= PLAY_TOP && self.ball_vy < 0.0 {
            self.ball_y = PLAY_TOP;
            self.ball_vy = self.ball_vy.abs();
        }
        let bottom = FRAMEBUFFER_HEIGHT as f32 - BALL_SIZE as f32;
        if self.ball_y >= bottom && self.ball_vy > 0.0 {
            self.ball_y = bottom;
            self.ball_vy = -self.ball_vy.abs();
        }

        let p1 = paddle_rect(12, self.p1_y);
        let p2 = paddle_rect(FRAMEBUFFER_WIDTH as i32 - 18, self.p2_y);
        let ball = ball_rect(self.ball_x, self.ball_y);

        if self.ball_vx < 0.0 && overlaps(ball, p1) {
            self.ball_x = (p1.x + p1.width as i32) as f32;
            bounce_from_paddle(
                &mut self.ball_vx,
                &mut self.ball_vy,
                self.ball_y,
                self.p1_y,
                1.0,
            );
        } else if self.ball_vx > 0.0 && overlaps(ball, p2) {
            self.ball_x = (p2.x - BALL_SIZE as i32) as f32;
            bounce_from_paddle(
                &mut self.ball_vx,
                &mut self.ball_vy,
                self.ball_y,
                self.p2_y,
                -1.0,
            );
        }

        if self.ball_x + BALL_SIZE as f32 < 0.0 {
            self.p2_score = self.p2_score.saturating_add(1);
            self.after_point(-1.0);
        } else if self.ball_x > FRAMEBUFFER_WIDTH as f32 {
            self.p1_score = self.p1_score.saturating_add(1);
            self.after_point(1.0);
        }
    }

    fn after_point(&mut self, direction_toward_loser: f32) {
        self.serve_direction = direction_toward_loser;
        self.reset_ball();
        self.match_state = if self.p1_score >= WIN_SCORE || self.p2_score >= WIN_SCORE {
            MatchState::MatchOver
        } else {
            MatchState::WaitingServe
        };
    }

    fn reset_ball(&mut self) {
        self.ball_x = centered_ball_x();
        self.ball_y = centered_ball_y();
        self.ball_vx = 0.0;
        self.ball_vy = 0.0;
    }

    fn reset_match(&mut self) {
        self.p1_y = centered_paddle_y();
        self.p2_y = centered_paddle_y();
        self.p1_score = 0;
        self.p2_score = 0;
        self.serve_direction = 1.0;
        self.reset_ball();
        self.match_state = MatchState::WaitingServe;
    }

    fn winner(&self) -> Option<u8> {
        if self.p1_score >= WIN_SCORE {
            Some(1)
        } else if self.p2_score >= WIN_SCORE {
            Some(2)
        } else {
            None
        }
    }

    fn render_main_menu(&self, framebuffer: &mut Framebuffer) {
        framebuffer.clear(BG);
        draw_panel(
            framebuffer,
            Rect {
                x: 66,
                y: 24,
                width: 188,
                height: 132,
            },
            BG,
            BORDER,
        );
        draw_text_centered(
            framebuffer,
            Rect {
                x: 80,
                y: 38,
                width: 160,
                height: 24,
            },
            "PONG",
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
                x: 36,
                y: 22,
                width: 248,
                height: 136,
            },
            BG,
            BORDER,
        );
        draw_text_centered(
            framebuffer,
            Rect {
                x: 48,
                y: 34,
                width: 224,
                height: 16,
            },
            "CONTROLS",
            1,
            ACCENT,
        );
        framebuffer.draw_text(58, 64, "P1  W/S", FG);
        framebuffer.draw_text(58, 78, "P2  UP/DOWN", FG);
        framebuffer.draw_text(
            58,
            100,
            if self.assigned_gamepads[0].is_some() {
                "P1 PAD CONNECTED"
            } else {
                "P1 PAD NONE"
            },
            FG,
        );
        framebuffer.draw_text(
            58,
            114,
            if self.assigned_gamepads[1].is_some() {
                "P2 PAD CONNECTED"
            } else {
                "P2 PAD NONE"
            },
            FG,
        );
        draw_text_centered(
            framebuffer,
            Rect {
                x: 48,
                y: 136,
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

        for y in (28..FRAMEBUFFER_HEIGHT as i32).step_by(12) {
            framebuffer.fill_rect(FRAMEBUFFER_WIDTH as i32 / 2 - 1, y, 2, 6, BORDER);
        }

        framebuffer.fill_rect(12, self.p1_y.round() as i32, PADDLE_WIDTH, PADDLE_HEIGHT, FG);
        framebuffer.fill_rect(
            FRAMEBUFFER_WIDTH as i32 - 18,
            self.p2_y.round() as i32,
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

        draw_text_centered(
            framebuffer,
            Rect {
                x: 112,
                y: 4,
                width: 96,
                height: 16,
            },
            &format!("{}   {}", self.p1_score, self.p2_score),
            1,
            FG,
        );

        match self.match_state {
            MatchState::WaitingServe => {
                draw_text_centered(
                    framebuffer,
                    Rect {
                        x: 72,
                        y: 158,
                        width: 176,
                        height: 12,
                    },
                    "SPACE/SOUTH SERVE",
                    1,
                    FG,
                );
            }
            MatchState::Playing => {}
            MatchState::MatchOver => {
                draw_panel(
                    framebuffer,
                    Rect {
                        x: 82,
                        y: 62,
                        width: 156,
                        height: 56,
                    },
                    BG,
                    BORDER,
                );
                let winner = match self.winner() {
                    Some(1) => "P1 WINS",
                    Some(2) => "P2 WINS",
                    _ => "MATCH OVER",
                };
                draw_text_centered(
                    framebuffer,
                    Rect {
                        x: 94,
                        y: 76,
                        width: 132,
                        height: 12,
                    },
                    winner,
                    1,
                    ACCENT,
                );
                draw_text_centered(
                    framebuffer,
                    Rect {
                        x: 94,
                        y: 96,
                        width: 132,
                        height: 12,
                    },
                    "SPACE/SOUTH REPLAY",
                    1,
                    FG,
                );
            }
        }
    }
}

impl Game for PongApp {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.update_assignments(frame.input);
        if self.playing {
            self.update_game(frame)
        } else {
            self.update_menu(frame)
        }
    }
}

fn player_controls(
    up: ActionId,
    down: ActionId,
    up_key: Key,
    down_key: Key,
    gamepad_id: Option<GamepadId>,
) -> ControlMap {
    let mut controls = ControlMap::new();
    controls.bind_key(up, up_key).bind_key(down, down_key);

    if let Some(id) = gamepad_id {
        controls
            .bind_gamepad_device(up, id, GamepadButton::DPadUp)
            .bind_gamepad_device(up, id, GamepadButton::LeftStickUp)
            .bind_gamepad_device(down, id, GamepadButton::DPadDown)
            .bind_gamepad_device(down, id, GamepadButton::LeftStickDown);
    }

    controls
}

fn move_paddle(y: f32, up: bool, down: bool, dt: f32) -> f32 {
    let direction = match (up, down) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    };
    let max_y = FRAMEBUFFER_HEIGHT as f32 - PADDLE_HEIGHT as f32;
    (y + direction * PADDLE_SPEED * dt).clamp(PLAY_TOP, max_y)
}

fn bounce_from_paddle(
    vx: &mut f32,
    vy: &mut f32,
    ball_y: f32,
    paddle_y: f32,
    direction: f32,
) {
    let paddle_center = paddle_y + PADDLE_HEIGHT as f32 / 2.0;
    let ball_center = ball_y + BALL_SIZE as f32 / 2.0;
    let offset = ((ball_center - paddle_center) / (PADDLE_HEIGHT as f32 / 2.0)).clamp(-1.0, 1.0);
    let angle = offset * MAX_BOUNCE_ANGLE;
    let speed = (vx.hypot(*vy) * 1.04).clamp(BALL_SPEED_X.hypot(BALL_SPEED_Y), BALL_SPEED_MAX);

    *vx = direction * speed * angle.cos();
    *vy = speed * angle.sin();
}

fn paddle_rect(x: i32, y: f32) -> Rect {
    Rect {
        x,
        y: y.round() as i32,
        width: PADDLE_WIDTH,
        height: PADDLE_HEIGHT,
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

fn centered_paddle_y() -> f32 {
    (FRAMEBUFFER_HEIGHT - PADDLE_HEIGHT) as f32 / 2.0
}

fn centered_ball_x() -> f32 {
    (FRAMEBUFFER_WIDTH - BALL_SIZE) as f32 / 2.0
}

fn centered_ball_y() -> f32 {
    (FRAMEBUFFER_HEIGHT - BALL_SIZE) as f32 / 2.0
}

fn main() -> Result<(), EngineError> {
    run(
        EngineConfig {
            title: "Pong - Gotoo Pixel Engine".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width: FRAMEBUFFER_WIDTH * 3,
            window_height: FRAMEBUFFER_HEIGHT * 3,
        },
        PongApp::new(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paddle_stays_inside_playfield() {
        assert_eq!(move_paddle(PLAY_TOP, true, false, 1.0), PLAY_TOP);
        assert_eq!(
            move_paddle(FRAMEBUFFER_HEIGHT as f32, false, true, 1.0),
            FRAMEBUFFER_HEIGHT as f32 - PADDLE_HEIGHT as f32
        );
    }

    #[test]
    fn rectangles_overlap_only_when_they_share_area() {
        let paddle = paddle_rect(12, 60.0);
        assert!(overlaps(ball_rect(15.0, 70.0), paddle));
        assert!(!overlaps(ball_rect(30.0, 70.0), paddle));
    }

    #[test]
    fn new_match_waits_for_serve() {
        let app = PongApp::new();
        assert_eq!(app.match_state, MatchState::WaitingServe);
        assert_eq!(app.ball_vx, 0.0);
        assert_eq!(app.ball_vy, 0.0);
    }

    #[test]
    fn serve_starts_rally() {
        let mut app = PongApp::new();
        app.serve();
        assert_eq!(app.match_state, MatchState::Playing);
        assert!(app.ball_vx > 0.0);
        assert_ne!(app.ball_vy, 0.0);
    }

    #[test]
    fn point_waits_for_next_serve() {
        let mut app = PongApp::new();
        app.match_state = MatchState::Playing;
        app.p1_score = 1;
        app.after_point(1.0);
        assert_eq!(app.match_state, MatchState::WaitingServe);
        assert_eq!(app.ball_vx, 0.0);
        assert_eq!(app.ball_vy, 0.0);
        assert_eq!(app.serve_direction, 1.0);
    }

    #[test]
    fn first_player_to_seven_wins_match() {
        let mut app = PongApp::new();
        app.p1_score = WIN_SCORE;
        app.after_point(1.0);
        assert_eq!(app.match_state, MatchState::MatchOver);
        assert_eq!(app.winner(), Some(1));
    }

    #[test]
    fn substeps_catch_fast_paddle_collision() {
        let mut app = PongApp::new();
        app.match_state = MatchState::Playing;
        app.p1_y = 70.0;
        app.ball_x = 30.0;
        app.ball_y = 80.0;
        app.ball_vx = -900.0;
        app.ball_vy = 0.0;

        app.update_ball(0.03);

        assert!(app.ball_vx > 0.0);
        assert_eq!(app.match_state, MatchState::Playing);
    }
}
