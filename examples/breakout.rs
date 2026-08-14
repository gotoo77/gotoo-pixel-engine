use gotoo_pixel_engine::{
    ActionId, ControlMap, EngineConfig, EngineError, Frame, Framebuffer, Game, GameResult,
    GamepadButton, Key, Pixel, Rect, SoundBank, SoundId, pcm16_mono_wav, run,
    ui::{
        MenuState, VirtualButton, VirtualPad, draw_menu_item, draw_panel, draw_text_centered,
        menu_confirm_pressed, menu_down_pressed, menu_up_pressed,
    },
};

pub const FRAMEBUFFER_WIDTH: u32 = 320;
pub const TOUCH_FRAMEBUFFER_WIDTH: u32 = 480;
pub const FRAMEBUFFER_HEIGHT: u32 = 180;
const PLAY_TOP: f32 = 24.0;

const MOVE_LEFT: ActionId = ActionId::new("breakout.left");
const MOVE_RIGHT: ActionId = ActionId::new("breakout.right");
const ACTION: ActionId = ActionId::new("breakout.action");

const LAUNCH_SOUND: SoundId = SoundId::new("breakout.launch");
const PADDLE_HIT_SOUND: SoundId = SoundId::new("breakout.paddle_hit");
const BRICK_HIT_SOUND: SoundId = SoundId::new("breakout.brick_hit");
const LIFE_LOST_SOUND: SoundId = SoundId::new("breakout.life_lost");
const LEVEL_CLEAR_SOUND: SoundId = SoundId::new("breakout.level_clear");
const GAME_OVER_SOUND: SoundId = SoundId::new("breakout.game_over");
const AUDIO_SAMPLE_RATE: u32 = 44_100;

const BG: Pixel = Pixel::rgb(7, 10, 14);
const FG: Pixel = Pixel::rgb(230, 238, 228);
const ACCENT: Pixel = Pixel::rgb(120, 235, 180);
const BORDER: Pixel = Pixel::rgb(80, 150, 220);
const TOUCH_FILL: Pixel = Pixel::rgb(18, 28, 34);
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
const BALL_SPEED_Y: f32 = 118.0;
const BALL_SPEED_MAX_X: f32 = 190.0;
const BALL_SUBSTEP: f32 = 3.0;
const LEVEL_SPEED_STEP: f32 = 0.08;
const LEVEL_SPEED_SCALE_MAX: f32 = 1.5;

const BRICK_COLUMNS: usize = 10;
const BRICK_ROWS: usize = 5;
const BRICK_WIDTH: u32 = 26;
const BRICK_HEIGHT: u32 = 10;
const BRICK_GAP_X: i32 = 2;
const BRICK_GAP_Y: i32 = 3;
const BRICK_START_X: i32 = 21;
const BRICK_START_Y: i32 = 38;

const INITIAL_LIVES: u32 = 3;
const TOUCH_PANEL: Rect = Rect {
    x: FRAMEBUFFER_WIDTH as i32,
    y: 0,
    width: TOUCH_FRAMEBUFFER_WIDTH - FRAMEBUFFER_WIDTH,
    height: FRAMEBUFFER_HEIGHT,
};
const TOUCH_ACTION: Rect = Rect {
    x: 354,
    y: 44,
    width: 92,
    height: 48,
};
const TOUCH_LEFT: Rect = Rect {
    x: 326,
    y: 112,
    width: 60,
    height: 48,
};
const TOUCH_RIGHT: Rect = Rect {
    x: 414,
    y: 112,
    width: 60,
    height: 48,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Main,
    Controls,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RoundState {
    Playing,
    Lost,
}

#[derive(Debug, Clone, Copy)]
struct Brick {
    rect: Rect,
    active: bool,
    color: Pixel,
    row: usize,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct BallFeedback {
    paddle_hit: bool,
    brick_hits: u8,
    life_lost: bool,
    level_cleared: bool,
    game_over: bool,
}

pub struct BreakoutApp {
    page: Page,
    menu: MenuState,
    end_menu: MenuState,
    playing: bool,
    round_state: RoundState,
    paddle_x: f32,
    ball_x: f32,
    ball_y: f32,
    ball_vx: f32,
    ball_vy: f32,
    ball_stuck: bool,
    score: u32,
    lives: u32,
    level: u32,
    bricks: Vec<Brick>,
    controls: ControlMap,
    virtual_pad: Option<VirtualPad>,
    sounds: SoundBank,
}

impl BreakoutApp {
    pub fn new() -> Self {
        Self::new_with_touch(false)
    }

    pub fn new_touch() -> Self {
        let mut app = Self::new_with_touch(true);
        app.restart_round();
        app.playing = true;
        app
    }

    fn new_with_touch(touch: bool) -> Self {
        let mut sounds = SoundBank::new();
        sounds
            .insert_wav(LAUNCH_SOUND, synthesize_chirp(520.0, 760.0, 0.055, 0.32))
            .expect("Breakout launch sound id should be unique");
        sounds
            .insert_wav(
                PADDLE_HIT_SOUND,
                synthesize_chirp(330.0, 255.0, 0.040, 0.30),
            )
            .expect("Breakout paddle hit sound id should be unique");
        sounds
            .insert_wav(BRICK_HIT_SOUND, synthesize_chirp(920.0, 670.0, 0.035, 0.26))
            .expect("Breakout brick hit sound id should be unique");
        sounds
            .insert_wav(LIFE_LOST_SOUND, synthesize_chirp(210.0, 100.0, 0.160, 0.34))
            .expect("Breakout life lost sound id should be unique");
        sounds
            .insert_wav(
                LEVEL_CLEAR_SOUND,
                synthesize_chirp(430.0, 980.0, 0.220, 0.34),
            )
            .expect("Breakout level clear sound id should be unique");
        sounds
            .insert_wav(GAME_OVER_SOUND, synthesize_chirp(190.0, 62.0, 0.360, 0.38))
            .expect("Breakout game over sound id should be unique");

        let virtual_pad = touch.then(|| {
            VirtualPad::new([
                VirtualButton::new(MOVE_LEFT, TOUCH_LEFT),
                VirtualButton::new(MOVE_RIGHT, TOUCH_RIGHT),
                VirtualButton::new(ACTION, TOUCH_ACTION),
            ])
        });

        let mut app = Self {
            page: Page::Main,
            menu: MenuState::new(3),
            end_menu: MenuState::new(2),
            playing: false,
            round_state: RoundState::Playing,
            paddle_x: centered_paddle_x(),
            ball_x: centered_ball_x(),
            ball_y: PADDLE_Y - BALL_SIZE as f32 - 2.0,
            ball_vx: 0.0,
            ball_vy: 0.0,
            ball_stuck: true,
            score: 0,
            lives: INITIAL_LIVES,
            level: 1,
            bricks: make_bricks(),
            controls: breakout_controls(),
            virtual_pad,
            sounds,
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
                        Some(0) => {
                            self.restart_round();
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
                    || frame
                        .input
                        .gamepad_button_any(GamepadButton::East)
                        .pressed()
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

        if let Some(virtual_pad) = &mut self.virtual_pad {
            virtual_pad.update(frame.input, &mut self.controls);
        }
        self.controls.update(frame.input);

        if self.round_state == RoundState::Lost {
            if menu_up_pressed(frame.input) || self.controls.action(MOVE_LEFT).pressed() {
                self.end_menu.select_previous();
            }
            if menu_down_pressed(frame.input) || self.controls.action(MOVE_RIGHT).pressed() {
                self.end_menu.select_next();
            }
            if menu_confirm_pressed(frame.input) || self.controls.action(ACTION).pressed() {
                match self.end_menu.selected() {
                    Some(0) => self.restart_round(),
                    Some(1) => return GameResult::Exit,
                    _ => {}
                }
            }
            self.render_game(frame.framebuffer);
            return GameResult::Continue;
        }

        let dt = frame.delta_time.as_secs_f32().min(0.05);
        self.update_paddle(dt);

        if self.controls.action(ACTION).pressed() && self.launch_ball() {
            let _ = self.sounds.play(frame.audio, LAUNCH_SOUND);
        }
        if !self.ball_stuck {
            let feedback = self.update_ball(dt);
            if feedback.paddle_hit {
                let _ = self.sounds.play(frame.audio, PADDLE_HIT_SOUND);
            }
            for _ in 0..feedback.brick_hits {
                let _ = self.sounds.play(frame.audio, BRICK_HIT_SOUND);
            }
            if feedback.level_cleared {
                let _ = self.sounds.play(frame.audio, LEVEL_CLEAR_SOUND);
            }
            if feedback.game_over {
                let _ = self.sounds.play(frame.audio, GAME_OVER_SOUND);
            } else if feedback.life_lost {
                let _ = self.sounds.play(frame.audio, LIFE_LOST_SOUND);
            }
        }

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

        if self.ball_stuck {
            self.attach_ball_to_paddle();
        }
    }

    fn launch_ball(&mut self) -> bool {
        if self.round_state == RoundState::Playing && self.ball_stuck {
            self.ball_stuck = false;
            true
        } else {
            false
        }
    }

    fn update_ball(&mut self, dt: f32) -> BallFeedback {
        let largest_delta = self.ball_vx.abs().max(self.ball_vy.abs()) * dt;
        let steps = ((largest_delta / BALL_SUBSTEP).ceil() as u32).clamp(1, 32);
        let step_dt = dt / steps as f32;
        let mut feedback = BallFeedback::default();

        for _ in 0..steps {
            if self.ball_stuck || self.round_state != RoundState::Playing {
                break;
            }
            self.step_ball(step_dt, &mut feedback);
        }

        feedback
    }

    fn step_ball(&mut self, dt: f32, feedback: &mut BallFeedback) {
        self.ball_x += self.ball_vx * dt;
        self.ball_y += self.ball_vy * dt;

        if self.ball_x <= 0.0 && self.ball_vx < 0.0 {
            self.ball_x = 0.0;
            self.ball_vx = self.ball_vx.abs();
        }
        let right = FRAMEBUFFER_WIDTH as f32 - BALL_SIZE as f32;
        if self.ball_x >= right && self.ball_vx > 0.0 {
            self.ball_x = right;
            self.ball_vx = -self.ball_vx.abs();
        }
        if self.ball_y <= PLAY_TOP && self.ball_vy < 0.0 {
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
            let offset =
                ((ball_center - paddle_center) / (PADDLE_WIDTH as f32 / 2.0)).clamp(-1.0, 1.0);
            self.ball_vx =
                (self.ball_vx + offset * 45.0).clamp(-BALL_SPEED_MAX_X, BALL_SPEED_MAX_X);
            feedback.paddle_hit = true;
        }

        if let Some(index) = self.bricks.iter().position(|brick| {
            brick.active && overlaps(ball_rect(self.ball_x, self.ball_y), brick.rect)
        }) {
            let brick = self.bricks[index];
            self.bricks[index].active = false;
            self.score = self.score.saturating_add(brick_score(brick.row));
            feedback.brick_hits = feedback.brick_hits.saturating_add(1);
            bounce_from_brick(
                brick.rect,
                self.ball_x,
                self.ball_y,
                &mut self.ball_vx,
                &mut self.ball_vy,
            );

            if self.bricks.iter().all(|brick| !brick.active) {
                self.advance_level();
                feedback.level_cleared = true;
                return;
            }
        }

        if self.ball_y > FRAMEBUFFER_HEIGHT as f32 {
            self.lose_life();
            if self.round_state == RoundState::Lost {
                feedback.game_over = true;
            } else {
                feedback.life_lost = true;
            }
        }
    }

    fn lose_life(&mut self) {
        self.lives = self.lives.saturating_sub(1);
        if self.lives == 0 {
            self.round_state = RoundState::Lost;
            self.ball_stuck = true;
            self.end_menu = MenuState::new(2);
        } else {
            self.paddle_x = centered_paddle_x();
            self.reset_ball();
        }
    }

    fn advance_level(&mut self) {
        self.level = self.level.saturating_add(1);
        self.bricks = make_bricks();
        self.paddle_x = centered_paddle_x();
        self.reset_ball();
    }

    fn reset_ball(&mut self) {
        let speed_scale = level_speed_scale(self.level);
        self.ball_vx = if self.lives.is_multiple_of(2) {
            -BALL_SPEED_X * speed_scale
        } else {
            BALL_SPEED_X * speed_scale
        };
        self.ball_vy = -BALL_SPEED_Y * speed_scale;
        self.ball_stuck = true;
        self.attach_ball_to_paddle();
    }

    fn attach_ball_to_paddle(&mut self) {
        self.ball_x = self.paddle_x + (PADDLE_WIDTH - BALL_SIZE) as f32 / 2.0;
        self.ball_y = PADDLE_Y - BALL_SIZE as f32 - 2.0;
    }

    fn restart_round(&mut self) {
        self.round_state = RoundState::Playing;
        self.end_menu = MenuState::new(2);
        self.score = 0;
        self.lives = INITIAL_LIVES;
        self.level = 1;
        self.bricks = make_bricks();
        self.paddle_x = centered_paddle_x();
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
        framebuffer.draw_text(58, 104, "ACTION    SPACE / SOUTH", FG);
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
        framebuffer.draw_text(140, 8, &format!("LV {}", self.level), FG);
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

        if self.round_state == RoundState::Lost {
            self.render_game_over(framebuffer);
        } else if self.ball_stuck {
            draw_text_centered(
                framebuffer,
                Rect {
                    x: 72,
                    y: 146,
                    width: 176,
                    height: 12,
                },
                if self.virtual_pad.is_some() {
                    "ACTION TO LAUNCH"
                } else {
                    "SPACE/SOUTH LAUNCH"
                },
                1,
                FG,
            );
        }

        if self.virtual_pad.is_some() {
            draw_touch_controls(framebuffer);
        }
    }

    fn render_game_over(&self, framebuffer: &mut Framebuffer) {
        draw_panel(
            framebuffer,
            Rect {
                x: 70,
                y: 64,
                width: 180,
                height: 96,
            },
            BG,
            BORDER,
        );
        draw_text_centered(
            framebuffer,
            Rect {
                x: 84,
                y: 76,
                width: 152,
                height: 14,
            },
            "GAME OVER",
            1,
            ACCENT,
        );
        draw_text_centered(
            framebuffer,
            Rect {
                x: 84,
                y: 94,
                width: 152,
                height: 12,
            },
            &format!("SCORE {}  LV {}", self.score, self.level),
            1,
            FG,
        );

        for (index, (label, y)) in [("REPLAY", 116), ("QUIT", 138)].into_iter().enumerate() {
            draw_menu_item(
                framebuffer,
                Rect {
                    x: 96,
                    y,
                    width: 128,
                    height: 16,
                },
                label,
                self.end_menu.selected() == Some(index),
                1,
                FG,
                ACCENT,
            );
        }
    }
}

impl Default for BreakoutApp {
    fn default() -> Self {
        Self::new()
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
        .bind_key(ACTION, Key::Space)
        .bind_gamepad(ACTION, GamepadButton::South);
    controls
}

fn draw_touch_controls(framebuffer: &mut Framebuffer) {
    draw_panel(framebuffer, TOUCH_PANEL, BG, BORDER);
    draw_text_centered(
        framebuffer,
        Rect {
            x: TOUCH_PANEL.x,
            y: 12,
            width: TOUCH_PANEL.width,
            height: 18,
        },
        "TOUCH",
        1,
        FG,
    );

    for (rect, label) in [
        (TOUCH_ACTION, "ACTION"),
        (TOUCH_LEFT, "LEFT"),
        (TOUCH_RIGHT, "RIGHT"),
    ] {
        framebuffer.fill_rect(rect.x, rect.y, rect.width, rect.height, TOUCH_FILL);
        framebuffer.draw_rect(rect.x, rect.y, rect.width, rect.height, BORDER);
        draw_text_centered(framebuffer, rect, label, 1, ACCENT);
    }
}

fn make_bricks() -> Vec<Brick> {
    let mut bricks = Vec::with_capacity(BRICK_COLUMNS * BRICK_ROWS);
    for (row, color) in BRICK_COLORS.into_iter().enumerate() {
        for column in 0..BRICK_COLUMNS {
            bricks.push(Brick {
                rect: Rect {
                    x: BRICK_START_X + column as i32 * (BRICK_WIDTH as i32 + BRICK_GAP_X),
                    y: BRICK_START_Y + row as i32 * (BRICK_HEIGHT as i32 + BRICK_GAP_Y),
                    width: BRICK_WIDTH,
                    height: BRICK_HEIGHT,
                },
                active: true,
                color,
                row,
            });
        }
    }
    bricks
}

fn brick_score(row: usize) -> u32 {
    BRICK_ROWS.saturating_sub(row) as u32 * 10
}

fn level_speed_scale(level: u32) -> f32 {
    (1.0 + level.saturating_sub(1) as f32 * LEVEL_SPEED_STEP).min(LEVEL_SPEED_SCALE_MAX)
}

fn move_paddle(x: f32, direction: f32, dt: f32) -> f32 {
    let max_x = FRAMEBUFFER_WIDTH as f32 - PADDLE_WIDTH as f32;
    (x + direction * PADDLE_SPEED * dt).clamp(0.0, max_x)
}

fn bounce_from_brick(brick: Rect, ball_x: f32, ball_y: f32, ball_vx: &mut f32, ball_vy: &mut f32) {
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

fn synthesize_chirp(start_hz: f32, end_hz: f32, duration: f32, gain: f32) -> Vec<u8> {
    let sample_count = (AUDIO_SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(sample_count);
    let mut phase = 0.0_f32;

    for index in 0..sample_count {
        let progress = index as f32 / sample_count.max(1) as f32;
        let frequency = start_hz + (end_hz - start_hz) * progress;
        phase += frequency / AUDIO_SAMPLE_RATE as f32;
        let square = if phase.fract() < 0.5 { 1.0 } else { -1.0 };
        let envelope = (1.0 - progress).max(0.0);
        let sample = square * envelope * envelope * gain;
        samples.push((sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16);
    }

    pcm16_mono_wav(AUDIO_SAMPLE_RATE, &samples)
        .expect("Breakout procedural audio should use a supported PCM format")
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

    #[test]
    fn ball_starts_attached_and_launch_releases_it() {
        let mut app = BreakoutApp::new();
        assert!(app.ball_stuck);
        let initial_x = app.ball_x;

        app.paddle_x += 20.0;
        app.attach_ball_to_paddle();
        assert!(app.ball_x > initial_x);

        assert!(app.launch_ball());
        assert!(!app.ball_stuck);
        assert!(!app.launch_ball());
    }

    #[test]
    fn touch_mode_has_three_virtual_actions_and_starts_in_game() {
        let app = BreakoutApp::new_touch();
        assert!(app.playing);
        assert_eq!(
            app.virtual_pad.as_ref().map(|pad| pad.buttons().len()),
            Some(3)
        );
    }

    #[test]
    fn upper_rows_score_more_points() {
        assert!(brick_score(0) > brick_score(BRICK_ROWS - 1));
        assert_eq!(brick_score(0), BRICK_ROWS as u32 * 10);
    }

    #[test]
    fn level_progression_increases_speed_and_relaunches() {
        let mut app = BreakoutApp::new();
        let first_speed = app.ball_vx.abs();
        app.advance_level();
        assert_eq!(app.level, 2);
        assert!(app.ball_vx.abs() > first_speed);
        assert!(app.ball_stuck);
        assert!(app.bricks.iter().all(|brick| brick.active));
    }

    #[test]
    fn substeps_catch_fast_brick_collision() {
        let mut app = BreakoutApp::new();
        // Approach the bottom brick row from below so no other active brick can
        // legitimately intercept the ball before the brick under test.
        let brick_index = (BRICK_ROWS - 1) * BRICK_COLUMNS;
        let brick = app.bricks[brick_index];
        app.ball_stuck = false;
        app.ball_x = brick.rect.x as f32 + 4.0;
        app.ball_y = brick.rect.y as f32 + brick.rect.height as f32 + 8.0;
        app.ball_vx = 0.0;
        app.ball_vy = -900.0;

        let feedback = app.update_ball(0.02);

        assert!(!app.bricks[brick_index].active);
        assert!(app.score > 0);
        assert!(feedback.brick_hits > 0);
    }

    #[test]
    fn game_over_menu_defaults_to_replay() {
        let mut app = BreakoutApp::new();
        app.lives = 1;
        app.lose_life();

        assert_eq!(app.round_state, RoundState::Lost);
        assert_eq!(app.end_menu.selected(), Some(0));
    }

    #[test]
    fn sound_bank_owns_breakout_feedback_assets() {
        let app = BreakoutApp::new();
        assert!(app.sounds.contains(LAUNCH_SOUND));
        assert!(app.sounds.contains(PADDLE_HIT_SOUND));
        assert!(app.sounds.contains(BRICK_HIT_SOUND));
        assert!(app.sounds.contains(LIFE_LOST_SOUND));
        assert!(app.sounds.contains(LEVEL_CLEAR_SOUND));
        assert!(app.sounds.contains(GAME_OVER_SOUND));
    }
}
