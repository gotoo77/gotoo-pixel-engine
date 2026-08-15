use gotoo_pixel_engine::{
    ActionId, ControlMap, Frame, Framebuffer, Game, GameResult, GamepadButton, GamepadId, Input, Key,
    Pixel, Rect, SoundBank, SoundId, pcm16_mono_wav,
    ui::{MenuState, VirtualButton, VirtualPad, draw_menu_item, draw_panel, draw_text_centered,
        menu_confirm_pressed, menu_down_pressed, menu_up_pressed},
};

pub const FRAMEBUFFER_WIDTH: u32 = 320;
pub const FRAMEBUFFER_HEIGHT: u32 = 180;
pub const TOUCH_FRAMEBUFFER_HEIGHT: u32 = 260;

const P1_UP: ActionId = ActionId::new("pong.p1.up");
const P1_DOWN: ActionId = ActionId::new("pong.p1.down");
const P2_UP: ActionId = ActionId::new("pong.p2.up");
const P2_DOWN: ActionId = ActionId::new("pong.p2.down");
const TOUCH_ACTION: ActionId = ActionId::new("pong.touch.action");

const SERVE_SOUND: SoundId = SoundId::new("pong.serve");
const PADDLE_HIT_SOUND: SoundId = SoundId::new("pong.paddle_hit");
const POINT_SOUND: SoundId = SoundId::new("pong.point");
const AUDIO_SAMPLE_RATE: u32 = 44_100;

const BG: Pixel = Pixel::rgb(6, 10, 14);
const FG: Pixel = Pixel::rgb(230, 238, 230);
const ACCENT: Pixel = Pixel::rgb(110, 235, 180);
const BORDER: Pixel = Pixel::rgb(80, 150, 220);
const TOUCH_ACCENT: Pixel = Pixel::rgb(245, 190, 90);
const TOUCH_P1_UP: Rect = Rect {
    x: 8,
    y: 188,
    width: 56,
    height: 30,
};
const TOUCH_P1_DOWN: Rect = Rect {
    x: 8,
    y: 224,
    width: 56,
    height: 30,
};
const TOUCH_P2_UP: Rect = Rect {
    x: 256,
    y: 188,
    width: 56,
    height: 30,
};
const TOUCH_P2_DOWN: Rect = Rect {
    x: 256,
    y: 224,
    width: 56,
    height: 30,
};
const TOUCH_ACTION_RECT: Rect = Rect {
    x: 120,
    y: 206,
    width: 80,
    height: 32,
};

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
enum MatchState {
    WaitingServe,
    Playing,
    MatchOver,
}

pub struct PongGame {
    end_menu: MenuState,
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
    virtual_pad: Option<VirtualPad>,
    touch_controls: ControlMap,
    sounds: SoundBank,
}

impl Default for PongGame {
    fn default() -> Self {
        Self::new()
    }
}

impl PongGame {
    pub fn new() -> Self {
        Self::new_with_touch(false)
    }

    pub fn new_touch() -> Self {
        Self::new_with_touch(true)
    }

    fn new_with_touch(touch: bool) -> Self {
        let mut sounds = SoundBank::new();
        sounds
            .insert_wav(SERVE_SOUND, synthesize_chirp(620.0, 880.0, 0.055, 0.34))
            .expect("Pong serve sound id should be unique");
        sounds
            .insert_wav(
                PADDLE_HIT_SOUND,
                synthesize_chirp(430.0, 350.0, 0.040, 0.30),
            )
            .expect("Pong paddle hit sound id should be unique");
        sounds
            .insert_wav(POINT_SOUND, synthesize_chirp(230.0, 105.0, 0.180, 0.38))
            .expect("Pong point sound id should be unique");

        let mut game = Self {
            end_menu: MenuState::new(2),
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
            virtual_pad: touch.then(|| {
                VirtualPad::new([
                    VirtualButton::new(P1_UP, TOUCH_P1_UP),
                    VirtualButton::new(P1_DOWN, TOUCH_P1_DOWN),
                    VirtualButton::new(P2_UP, TOUCH_P2_UP),
                    VirtualButton::new(P2_DOWN, TOUCH_P2_DOWN),
                    VirtualButton::new(TOUCH_ACTION, TOUCH_ACTION_RECT),
                ])
            }),
            touch_controls: ControlMap::new(),
            sounds,
        };
        game.rebuild_controls();
        game
    }

    pub(super) fn sync_gamepads(&mut self, input: &Input) {
        self.update_assignments(input);
    }

    pub(super) fn gamepad_connected(&self, player: usize) -> bool {
        self.assigned_gamepads.get(player).copied().flatten().is_some()
    }

    pub(super) fn reset_match(&mut self) {
        self.p1_y = centered_paddle_y();
        self.p2_y = centered_paddle_y();
        self.p1_score = 0;
        self.p2_score = 0;
        self.serve_direction = 1.0;
        self.end_menu = MenuState::new(2);
        self.reset_ball();
        self.match_state = MatchState::WaitingServe;
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
        self.p1_controls =
            player_controls(P1_UP, P1_DOWN, Key::W, Key::S, self.assigned_gamepads[0]);
        self.p2_controls = player_controls(
            P2_UP,
            P2_DOWN,
            Key::Up,
            Key::Down,
            self.assigned_gamepads[1],
        );
    }

    fn update_game(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if let Some(virtual_pad) = &mut self.virtual_pad {
            virtual_pad.update(frame.input, &mut self.touch_controls);
        }
        self.touch_controls.update(frame.input);
        let touch_action_pressed = self.touch_controls.action(TOUCH_ACTION).pressed();

        if frame.input.key(Key::Escape).pressed() {
            return GameResult::Exit;
        }

        if self.match_state == MatchState::MatchOver {
            if touch_action_pressed {
                self.reset_match();
                self.render_game(frame.framebuffer);
                return GameResult::Continue;
            }
            if menu_up_pressed(frame.input) {
                self.end_menu.select_previous();
            }
            if menu_down_pressed(frame.input) {
                self.end_menu.select_next();
            }
            if menu_confirm_pressed(frame.input) {
                match self.end_menu.selected() {
                    Some(0) => self.reset_match(),
                    Some(1) => return GameResult::Exit,
                    _ => {}
                }
            }
            self.render_game(frame.framebuffer);
            return GameResult::Continue;
        }

        self.p1_controls.update(frame.input);
        self.p2_controls.update(frame.input);

        let dt = frame.delta_time.as_secs_f32().min(0.05);
        let p1_up =
            self.p1_controls.action(P1_UP).held() || self.touch_controls.action(P1_UP).held();
        let p1_down =
            self.p1_controls.action(P1_DOWN).held() || self.touch_controls.action(P1_DOWN).held();
        let p2_up =
            self.p2_controls.action(P2_UP).held() || self.touch_controls.action(P2_UP).held();
        let p2_down =
            self.p2_controls.action(P2_DOWN).held() || self.touch_controls.action(P2_DOWN).held();
        self.p1_y = move_paddle(self.p1_y, p1_up, p1_down, dt);
        self.p2_y = move_paddle(self.p2_y, p2_up, p2_down, dt);

        if self.match_state == MatchState::WaitingServe
            && (menu_confirm_pressed(frame.input) || touch_action_pressed)
        {
            self.serve();
            let _ = self.sounds.play(frame.audio, SERVE_SOUND);
        }

        if self.match_state == MatchState::Playing {
            let (paddle_hit, point_scored) = self.update_ball(dt);
            if paddle_hit {
                let _ = self.sounds.play(frame.audio, PADDLE_HIT_SOUND);
            }
            if point_scored {
                let _ = self.sounds.play(frame.audio, POINT_SOUND);
            }
        }

        self.render_game(frame.framebuffer);
        GameResult::Continue
    }

    fn serve(&mut self) {
        if self.match_state != MatchState::WaitingServe {
            return;
        }

        self.ball_vx = BALL_SPEED_X * self.serve_direction;
        self.ball_vy = if (self.p1_score + self.p2_score).is_multiple_of(2) {
            BALL_SPEED_Y
        } else {
            -BALL_SPEED_Y
        };
        self.match_state = MatchState::Playing;
    }

    fn update_ball(&mut self, dt: f32) -> (bool, bool) {
        let largest_delta = self.ball_vx.abs().max(self.ball_vy.abs()) * dt;
        let steps = ((largest_delta / BALL_SUBSTEP).ceil() as u32).clamp(1, 32);
        let step_dt = dt / steps as f32;
        let mut paddle_hit = false;
        let mut point_scored = false;

        for _ in 0..steps {
            if self.match_state != MatchState::Playing {
                break;
            }
            let (step_paddle_hit, step_point_scored) = self.step_ball(step_dt);
            paddle_hit |= step_paddle_hit;
            point_scored |= step_point_scored;
        }

        (paddle_hit, point_scored)
    }

    fn step_ball(&mut self, dt: f32) -> (bool, bool) {
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
        let mut paddle_hit = false;

        if self.ball_vx < 0.0 && ball.intersects(p1) {
            self.ball_x = (p1.x + p1.width as i32) as f32;
            bounce_from_paddle(
                &mut self.ball_vx,
                &mut self.ball_vy,
                self.ball_y,
                self.p1_y,
                1.0,
            );
            paddle_hit = true;
        } else if self.ball_vx > 0.0 && ball.intersects(p2) {
            self.ball_x = (p2.x - BALL_SIZE as i32) as f32;
            bounce_from_paddle(
                &mut self.ball_vx,
                &mut self.ball_vy,
                self.ball_y,
                self.p2_y,
                -1.0,
            );
            paddle_hit = true;
        }

        let point_scored = if self.ball_x + (BALL_SIZE as f32) < 0.0 {
            self.p2_score = self.p2_score.saturating_add(1);
            self.after_point(-1.0);
            true
        } else if self.ball_x > FRAMEBUFFER_WIDTH as f32 {
            self.p1_score = self.p1_score.saturating_add(1);
            self.after_point(1.0);
            true
        } else {
            false
        };

        (paddle_hit, point_scored)
    }

    fn after_point(&mut self, direction_toward_loser: f32) {
        self.serve_direction = direction_toward_loser;
        self.reset_ball();
        if self.p1_score >= WIN_SCORE || self.p2_score >= WIN_SCORE {
            self.match_state = MatchState::MatchOver;
            self.end_menu = MenuState::new(2);
        } else {
            self.match_state = MatchState::WaitingServe;
        }
    }

    fn reset_ball(&mut self) {
        self.ball_x = centered_ball_x();
        self.ball_y = centered_ball_y();
        self.ball_vx = 0.0;
        self.ball_vy = 0.0;
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

    fn render_game(&self, framebuffer: &mut Framebuffer) {
        framebuffer.clear(BG);

        for y in (28..FRAMEBUFFER_HEIGHT as i32).step_by(12) {
            framebuffer.fill_rect(FRAMEBUFFER_WIDTH as i32 / 2 - 1, y, 2, 6, BORDER);
        }

        framebuffer.fill_rect(
            12,
            self.p1_y.round() as i32,
            PADDLE_WIDTH,
            PADDLE_HEIGHT,
            FG,
        );
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
                if self.virtual_pad.is_none() {
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
            }
            MatchState::Playing => {}
            MatchState::MatchOver => self.render_end_menu(framebuffer),
        }

        if self.virtual_pad.is_some() {
            draw_touch_controls(framebuffer, self.match_state);
        }
    }

    fn render_end_menu(&self, framebuffer: &mut Framebuffer) {
        draw_panel(
            framebuffer,
            Rect {
                x: 82,
                y: 46,
                width: 156,
                height: 98,
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
                y: 60,
                width: 132,
                height: 16,
            },
            winner,
            1,
            ACCENT,
        );

        for (index, (label, y)) in [("REPLAY", 91), ("QUIT", 115)].into_iter().enumerate() {
            draw_menu_item(
                framebuffer,
                Rect {
                    x: 100,
                    y,
                    width: 120,
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

fn draw_touch_controls(framebuffer: &mut Framebuffer, state: MatchState) {
    framebuffer.draw_line(
        0,
        FRAMEBUFFER_HEIGHT as i32,
        FRAMEBUFFER_WIDTH as i32 - 1,
        FRAMEBUFFER_HEIGHT as i32,
        BORDER,
    );

    for (rect, label) in [
        (TOUCH_P1_UP, "P1 UP"),
        (TOUCH_P1_DOWN, "P1 DOWN"),
        (TOUCH_P2_UP, "P2 UP"),
        (TOUCH_P2_DOWN, "P2 DOWN"),
    ] {
        framebuffer.draw_rect(rect.x, rect.y, rect.width, rect.height, BORDER);
        draw_text_centered(framebuffer, rect, label, 1, TOUCH_ACCENT);
    }

    let action_label = match state {
        MatchState::WaitingServe => Some("SERVE"),
        MatchState::MatchOver => Some("REPLAY"),
        MatchState::Playing => None,
    };
    if let Some(label) = action_label {
        framebuffer.draw_rect(
            TOUCH_ACTION_RECT.x,
            TOUCH_ACTION_RECT.y,
            TOUCH_ACTION_RECT.width,
            TOUCH_ACTION_RECT.height,
            TOUCH_ACCENT,
        );
        draw_text_centered(framebuffer, TOUCH_ACTION_RECT, label, 1, TOUCH_ACCENT);
    }
}

impl Game for PongGame {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.update_assignments(frame.input);
        self.update_game(frame)
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

fn bounce_from_paddle(vx: &mut f32, vy: &mut f32, ball_y: f32, paddle_y: f32, direction: f32) {
    let paddle_center = paddle_y + PADDLE_HEIGHT as f32 / 2.0;
    let ball_center = ball_y + BALL_SIZE as f32 / 2.0;
    let offset =
        ((ball_center - paddle_center) / (PADDLE_HEIGHT as f32 / 2.0)).clamp(-1.0, 1.0);
    let angle = offset * MAX_BOUNCE_ANGLE;
    let speed = (vx.hypot(*vy) * 1.04).clamp(BALL_SPEED_X.hypot(BALL_SPEED_Y), BALL_SPEED_MAX);

    *vx = direction * speed * angle.cos();
    *vy = speed * angle.sin();
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
        .expect("Pong procedural audio should use a supported PCM format")
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

fn centered_paddle_y() -> f32 {
    (FRAMEBUFFER_HEIGHT - PADDLE_HEIGHT) as f32 / 2.0
}

fn centered_ball_x() -> f32 {
    (FRAMEBUFFER_WIDTH - BALL_SIZE) as f32 / 2.0
}

fn centered_ball_y() -> f32 {
    (FRAMEBUFFER_HEIGHT - BALL_SIZE) as f32 / 2.0
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
        assert!(ball_rect(15.0, 70.0).intersects(paddle));
        assert!(!ball_rect(30.0, 70.0).intersects(paddle));
    }

    #[test]
    fn new_match_waits_for_serve() {
        let game = PongGame::new();
        assert_eq!(game.match_state, MatchState::WaitingServe);
        assert_eq!(game.ball_vx, 0.0);
        assert_eq!(game.ball_vy, 0.0);
    }

    #[test]
    fn serve_starts_rally() {
        let mut game = PongGame::new();
        game.serve();
        assert_eq!(game.match_state, MatchState::Playing);
        assert!(game.ball_vx > 0.0);
        assert_ne!(game.ball_vy, 0.0);
    }

    #[test]
    fn point_waits_for_next_serve() {
        let mut game = PongGame::new();
        game.match_state = MatchState::Playing;
        game.p1_score = 1;
        game.after_point(1.0);
        assert_eq!(game.match_state, MatchState::WaitingServe);
        assert_eq!(game.ball_vx, 0.0);
        assert_eq!(game.ball_vy, 0.0);
        assert_eq!(game.serve_direction, 1.0);
    }

    #[test]
    fn first_player_to_seven_wins_match() {
        let mut game = PongGame::new();
        game.p1_score = WIN_SCORE;
        game.after_point(1.0);
        assert_eq!(game.match_state, MatchState::MatchOver);
        assert_eq!(game.winner(), Some(1));
        assert_eq!(game.end_menu.selected(), Some(0));
    }

    #[test]
    fn end_menu_has_replay_and_quit_choices() {
        let mut game = PongGame::new();
        assert_eq!(game.end_menu.selected(), Some(0));
        game.end_menu.select_next();
        assert_eq!(game.end_menu.selected(), Some(1));
    }

    #[test]
    fn substeps_catch_fast_paddle_collision() {
        let mut game = PongGame::new();
        game.match_state = MatchState::Playing;
        game.p1_y = 70.0;
        game.ball_x = 30.0;
        game.ball_y = 80.0;
        game.ball_vx = -900.0;
        game.ball_vy = 0.0;

        let (paddle_hit, point_scored) = game.update_ball(0.03);

        assert!(paddle_hit);
        assert!(!point_scored);
        assert!(game.ball_vx > 0.0);
        assert_eq!(game.match_state, MatchState::Playing);
    }

    #[test]
    fn touch_mode_has_two_player_virtual_controls() {
        let game = PongGame::new_touch();
        let pad = game
            .virtual_pad
            .as_ref()
            .expect("touch mode owns a virtual pad");
        let actions = pad
            .buttons()
            .iter()
            .map(|button| button.action)
            .collect::<Vec<_>>();
        assert_eq!(actions.len(), 5);
        for action in [P1_UP, P1_DOWN, P2_UP, P2_DOWN, TOUCH_ACTION] {
            assert!(actions.contains(&action));
        }
        assert!(TOUCH_FRAMEBUFFER_HEIGHT > FRAMEBUFFER_HEIGHT);
        assert!(
            pad.buttons()
                .iter()
                .all(|button| button.rect.y >= FRAMEBUFFER_HEIGHT as i32)
        );
    }

    #[test]
    fn sound_bank_owns_pong_feedback_assets() {
        let game = PongGame::new();
        assert!(game.sounds.contains(SERVE_SOUND));
        assert!(game.sounds.contains(PADDLE_HIT_SOUND));
        assert!(game.sounds.contains(POINT_SOUND));
    }
}
