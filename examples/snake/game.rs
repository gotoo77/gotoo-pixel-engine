use std::time::Duration;

use gotoo_pixel_engine::{
    ActionId, Audio, ControlMap, Frame, Framebuffer, Game, GameResult, GamepadButton, Key,
    LocalStorage, MouseButton, Pixel, Rect, Size, SoundBank, SoundId, Touch, TouchPhase,
    pcm16_mono_wav,
    ui::{VirtualButton, VirtualPad, VirtualPadUpdate},
};

#[path = "world.rs"]
mod world;
use world::{Cell, Direction, GRID_HEIGHT, GRID_WIDTH, Phase, SnakeWorld};

pub const KEYBOARD_FRAMEBUFFER_WIDTH: u32 = 320;
pub const TOUCH_FRAMEBUFFER_WIDTH: u32 = 480;
pub const FRAMEBUFFER_HEIGHT: u32 = 204;

const CELL_SIZE: i32 = 10;
const PLAYFIELD_WIDTH: u32 = GRID_WIDTH as u32 * CELL_SIZE as u32;
const PLAYFIELD_HEIGHT: u32 = GRID_HEIGHT as u32 * CELL_SIZE as u32;
const INITIAL_SEED: u32 = 0x5EED_1234;
const MAX_CATCH_UP: usize = 5;
const TICK_PERIOD: Duration = Duration::from_millis(120);
const EXIT_KEY: Key = Key::Escape;
const RESTART_KEY: Key = Key::Space;
const BEST_SCORE_KEY: &str = "gotoo-pixel-engine.snake.best_score.v1";

const CONTROL_UP: ActionId = ActionId::new("snake.up");
const CONTROL_RIGHT: ActionId = ActionId::new("snake.right");
const CONTROL_DOWN: ActionId = ActionId::new("snake.down");
const CONTROL_LEFT: ActionId = ActionId::new("snake.left");
const CONTROL_RESTART: ActionId = ActionId::new("snake.restart");
const CONTROL_EXIT: ActionId = ActionId::new("snake.exit");

const EAT_SOUND: SoundId = SoundId::new("snake.eat");
const DEATH_SOUND: SoundId = SoundId::new("snake.death");
const TURN_SOUND: SoundId = SoundId::new("snake.turn");
const SNAKE_AUDIO_SAMPLE_RATE: u32 = 44_100;
const SNAKE_SOUNDS: [(SoundId, &[u8]); 2] = [
    (EAT_SOUND, include_bytes!("assets/eat.wav")),
    (DEATH_SOUND, include_bytes!("assets/death.wav")),
];
const HUD_TEXT_SCALE: u32 = 1;
const GAME_OVER_TEXT_SCALE: u32 = 2;
const HUD_HEIGHT: u32 = 24;
const CONTROLS_WIDTH: u32 = 160;
const D_PAD_BUTTON_SIZE: u32 = 52;
const GAME_OVER_PANEL_SIZE: Size = Size {
    width: 192,
    height: 108,
};
const REPLAY_BUTTON_SIZE: Size = Size {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SnakeInteractionMode {
    Keyboard,
    Touch,
}

impl SnakeInteractionMode {
    pub fn framebuffer_size(self) -> Size {
        SnakeLayout::for_mode(self).framebuffer_size
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SnakeLayout {
    framebuffer_size: Size,
    hud: Rect,
    playfield: Rect,
    controls: Option<Rect>,
    d_pad: Option<DPadLayout>,
    replay: Rect,
    game_over_panel: Rect,
}

impl SnakeLayout {
    fn for_mode(mode: SnakeInteractionMode) -> Self {
        match mode {
            SnakeInteractionMode::Keyboard => Self::keyboard(),
            SnakeInteractionMode::Touch => Self::touch(),
        }
    }

    fn keyboard() -> Self {
        Self::new(KEYBOARD_FRAMEBUFFER_WIDTH, None)
    }

    fn touch() -> Self {
        Self::new(TOUCH_FRAMEBUFFER_WIDTH, Some(CONTROLS_WIDTH))
    }

    fn new(framebuffer_width: u32, controls_width: Option<u32>) -> Self {
        let framebuffer_size = Size {
            width: framebuffer_width,
            height: FRAMEBUFFER_HEIGHT,
        };
        let hud = Rect {
            x: 0,
            y: 0,
            width: framebuffer_width,
            height: HUD_HEIGHT,
        };
        let playfield = Rect {
            x: 0,
            y: hud.height as i32,
            width: PLAYFIELD_WIDTH,
            height: PLAYFIELD_HEIGHT,
        };
        let controls = controls_width.map(|width| Rect {
            x: playfield.width as i32,
            y: playfield.y,
            width,
            height: playfield.height,
        });
        let d_pad = controls.map(DPadLayout::new);
        let game_over_panel = Rect {
            x: playfield.x + centered_offset(playfield.width, GAME_OVER_PANEL_SIZE.width),
            y: playfield.y + centered_offset(playfield.height, GAME_OVER_PANEL_SIZE.height),
            width: GAME_OVER_PANEL_SIZE.width,
            height: GAME_OVER_PANEL_SIZE.height,
        };
        let replay = Rect {
            x: playfield.x + centered_offset(playfield.width, REPLAY_BUTTON_SIZE.width),
            y: game_over_panel.y + 72,
            width: REPLAY_BUTTON_SIZE.width,
            height: REPLAY_BUTTON_SIZE.height,
        };

        Self {
            framebuffer_size,
            hud,
            playfield,
            controls,
            d_pad,
            replay,
            game_over_panel,
        }
    }

    fn cell_origin(self, cell: Cell) -> (i32, i32) {
        (
            self.playfield.x + cell.x * CELL_SIZE,
            self.playfield.y + cell.y * CELL_SIZE,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DPadLayout {
    up: Rect,
    left: Rect,
    center: Rect,
    right: Rect,
    down: Rect,
}

impl DPadLayout {
    fn new(controls: Rect) -> Self {
        let left_x = controls.x + 2;
        let center_x = left_x + D_PAD_BUTTON_SIZE as i32;
        let right_x = center_x + D_PAD_BUTTON_SIZE as i32;
        let center_y = controls.y + 64;
        let up_y = center_y - D_PAD_BUTTON_SIZE as i32;
        let down_y = center_y + D_PAD_BUTTON_SIZE as i32;

        Self {
            up: Rect {
                x: center_x,
                y: up_y,
                width: D_PAD_BUTTON_SIZE,
                height: D_PAD_BUTTON_SIZE,
            },
            left: Rect {
                x: left_x,
                y: center_y,
                width: D_PAD_BUTTON_SIZE,
                height: D_PAD_BUTTON_SIZE,
            },
            center: Rect {
                x: center_x,
                y: center_y,
                width: D_PAD_BUTTON_SIZE,
                height: D_PAD_BUTTON_SIZE,
            },
            right: Rect {
                x: right_x,
                y: center_y,
                width: D_PAD_BUTTON_SIZE,
                height: D_PAD_BUTTON_SIZE,
            },
            down: Rect {
                x: center_x,
                y: down_y,
                width: D_PAD_BUTTON_SIZE,
                height: D_PAD_BUTTON_SIZE,
            },
        }
    }

    fn direction_zones(self) -> [DirectionZone; 4] {
        [
            DirectionZone {
                rect: self.up,
                direction: Direction::Up,
            },
            DirectionZone {
                rect: self.left,
                direction: Direction::Left,
            },
            DirectionZone {
                rect: self.right,
                direction: Direction::Right,
            },
            DirectionZone {
                rect: self.down,
                direction: Direction::Down,
            },
        ]
    }
}

fn virtual_pad_for_mode(mode: SnakeInteractionMode) -> Option<VirtualPad> {
    let d_pad = SnakeLayout::for_mode(mode).d_pad?;
    Some(VirtualPad::new([
        VirtualButton::new(CONTROL_UP, d_pad.up),
        VirtualButton::new(CONTROL_RIGHT, d_pad.right),
        VirtualButton::new(CONTROL_DOWN, d_pad.down),
        VirtualButton::new(CONTROL_LEFT, d_pad.left),
    ]))
}

#[derive(Debug)]
pub struct SnakeGame {
    world: SnakeWorld,
    accumulator: Duration,
    interaction_mode: SnakeInteractionMode,
    virtual_pad: Option<VirtualPad>,
    controls: ControlMap,
    sounds: SoundBank,
    best_score: u32,
    best_score_loaded: bool,
}

impl SnakeGame {
    pub fn new(interaction_mode: SnakeInteractionMode) -> Self {
        let virtual_pad = virtual_pad_for_mode(interaction_mode);
        Self {
            world: SnakeWorld::new(INITIAL_SEED),
            accumulator: Duration::ZERO,
            interaction_mode,
            virtual_pad,
            controls: default_controls(),
            sounds: snake_sound_bank(),
            best_score: 0,
            best_score_loaded: false,
        }
    }

    fn layout(&self) -> SnakeLayout {
        SnakeLayout::for_mode(self.interaction_mode)
    }

    fn update_logic(&mut self, delta_time: Duration, controls: SnakeControls) -> SnakeEvents {
        let mut events = SnakeEvents::default();

        if controls.restart && self.world.phase() == Phase::GameOver {
            self.restart();
        }

        for direction in controls.directions {
            self.world.queue_direction(direction);
        }

        if self.world.phase() != Phase::Running {
            return events;
        }

        self.accumulator += delta_time;

        let mut ticks = 0;
        while self.accumulator >= TICK_PERIOD && ticks < MAX_CATCH_UP {
            let result = self.world.tick();
            if result.turned {
                events.turns += 1;
            }
            if result.ate_food {
                events.food_eaten += 1;
            }
            if result.game_over {
                events.game_over = true;
            }
            self.accumulator -= TICK_PERIOD;
            ticks += 1;

            if self.world.phase() != Phase::Running {
                self.reset_virtual_pad();
                break;
            }
        }

        events
    }

    fn restart(&mut self) {
        self.world.restart();
        self.accumulator = Duration::ZERO;
        self.reset_virtual_pad();
    }

    fn reset_virtual_pad(&mut self) {
        if let Some(virtual_pad) = &mut self.virtual_pad {
            virtual_pad.reset(&mut self.controls);
        }
    }

    fn load_best_score_once(&mut self, storage: &mut dyn LocalStorage) {
        if self.best_score_loaded {
            return;
        }

        self.best_score = read_best_score(storage);
        self.best_score_loaded = true;
    }

    fn persist_best_score_if_needed(&mut self, storage: &mut dyn LocalStorage) {
        let score = self.world.score();
        if score <= self.best_score {
            return;
        }

        self.best_score = score;
        let _ = storage.set(BEST_SCORE_KEY, &score.to_string());
    }

    fn play_sounds(&mut self, audio: &mut dyn Audio, events: SnakeEvents) {
        for _ in 0..events.turns {
            let _ = self.sounds.play(audio, TURN_SOUND);
        }
        for _ in 0..events.food_eaten {
            let _ = self.sounds.play(audio, EAT_SOUND);
        }
        if events.game_over {
            let _ = self.sounds.play(audio, DEATH_SOUND);
        }
    }

    fn draw(&self, frame: &mut Frame<'_>) {
        let framebuffer = &mut frame.framebuffer;
        let layout = self.layout();

        framebuffer.clear(BACKGROUND);
        draw_grid(framebuffer, layout);

        if let Some(food) = self.world.food() {
            let (cell_x, cell_y) = layout.cell_origin(food);
            let center_x = cell_x + CELL_SIZE / 2;
            let center_y = cell_y + CELL_SIZE / 2;
            framebuffer.fill_circle(center_x, center_y, 4, FOOD);
        }

        for (index, cell) in self.world.snake().iter().enumerate().rev() {
            let color = if index == 0 { SNAKE_HEAD } else { SNAKE_BODY };
            let (cell_x, cell_y) = layout.cell_origin(*cell);
            framebuffer.fill_rect(
                cell_x + 1,
                cell_y + 1,
                (CELL_SIZE - 2) as u32,
                (CELL_SIZE - 2) as u32,
                color,
            );
        }

        framebuffer.draw_rect(
            layout.playfield.x,
            layout.playfield.y,
            layout.playfield.width,
            layout.playfield.height,
            BORDER,
        );
        draw_score_hud(framebuffer, layout, self.world.score(), self.best_score);

        if self.world.phase() == Phase::GameOver {
            draw_game_over(framebuffer, layout, self.world.score());
        } else if self.virtual_pad.as_ref().is_some_and(VirtualPad::visible)
            && layout.d_pad.is_some()
        {
            draw_d_pad(framebuffer, layout);
        }
    }
}

impl Game for SnakeGame {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.load_best_score_once(frame.storage);

        let phase = self.world.phase();
        let virtual_update = match phase {
            Phase::Running => self
                .virtual_pad
                .as_mut()
                .map(|virtual_pad| virtual_pad.update(frame.input, &mut self.controls)),
            Phase::GameOver => {
                self.reset_virtual_pad();
                None
            }
        };
        self.controls.update(frame.input);

        if self.controls.action(CONTROL_EXIT).pressed() {
            return GameResult::Exit;
        }

        let layout = self.layout();
        let controls = SnakeControls::from_frame(
            frame,
            phase,
            virtual_update.as_ref(),
            layout,
            &self.controls,
        );
        let events = self.update_logic(frame.delta_time, controls);
        self.play_sounds(frame.audio, events);
        self.persist_best_score_if_needed(frame.storage);
        self.draw(frame);

        GameResult::Continue
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct SnakeEvents {
    turns: u32,
    food_eaten: u32,
    game_over: bool,
}

fn default_controls() -> ControlMap {
    let mut controls = ControlMap::new();
    controls
        .bind_key(CONTROL_UP, Key::Up)
        .bind_key(CONTROL_UP, Key::W)
        .bind_gamepad(CONTROL_UP, GamepadButton::DPadUp)
        .bind_gamepad(CONTROL_UP, GamepadButton::LeftStickUp)
        .bind_key(CONTROL_RIGHT, Key::Right)
        .bind_key(CONTROL_RIGHT, Key::D)
        .bind_gamepad(CONTROL_RIGHT, GamepadButton::DPadRight)
        .bind_gamepad(CONTROL_RIGHT, GamepadButton::LeftStickRight)
        .bind_key(CONTROL_DOWN, Key::Down)
        .bind_key(CONTROL_DOWN, Key::S)
        .bind_gamepad(CONTROL_DOWN, GamepadButton::DPadDown)
        .bind_gamepad(CONTROL_DOWN, GamepadButton::LeftStickDown)
        .bind_key(CONTROL_LEFT, Key::Left)
        .bind_key(CONTROL_LEFT, Key::A)
        .bind_gamepad(CONTROL_LEFT, GamepadButton::DPadLeft)
        .bind_gamepad(CONTROL_LEFT, GamepadButton::LeftStickLeft)
        .bind_key(CONTROL_RESTART, RESTART_KEY)
        .bind_gamepad(CONTROL_RESTART, GamepadButton::South)
        .bind_gamepad(CONTROL_RESTART, GamepadButton::Start)
        .bind_key(CONTROL_EXIT, EXIT_KEY);
    controls
}

fn snake_sound_bank() -> SoundBank {
    let mut sounds = SoundBank::new();
    for (id, bytes) in SNAKE_SOUNDS {
        sounds
            .insert_wav(id, bytes.to_vec())
            .expect("Snake sound ids should be unique");
    }
    sounds
        .insert_wav(TURN_SOUND, synthesize_turn_sound())
        .expect("Snake turn sound id should be unique");
    sounds
}

fn synthesize_turn_sound() -> Vec<u8> {
    let duration = 0.035_f32;
    let sample_count = (SNAKE_AUDIO_SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(sample_count);
    let mut phase = 0.0_f32;

    for index in 0..sample_count {
        let progress = index as f32 / sample_count as f32;
        let envelope = (1.0 - progress).powi(2);
        let frequency = 175.0 + 45.0 * progress;
        phase += frequency / SNAKE_AUDIO_SAMPLE_RATE as f32;
        let wave = (phase * std::f32::consts::TAU).sin();
        let sample = wave * envelope * 0.11;
        samples.push((sample * i16::MAX as f32) as i16);
    }

    pcm16_mono_wav(SNAKE_AUDIO_SAMPLE_RATE, &samples)
        .expect("Snake procedural turn sound should use a supported PCM format")
}

fn draw_grid(framebuffer: &mut Framebuffer, layout: SnakeLayout) {
    for x in 1..GRID_WIDTH {
        let pixel_x = layout.playfield.x + x * CELL_SIZE;
        framebuffer.draw_line(
            pixel_x,
            layout.playfield.y,
            pixel_x,
            layout.playfield.y + layout.playfield.height as i32 - 1,
            GRID_LINE,
        );
    }
    for y in 1..GRID_HEIGHT {
        let pixel_y = layout.playfield.y + y * CELL_SIZE;
        framebuffer.draw_line(
            layout.playfield.x,
            pixel_y,
            layout.playfield.x + layout.playfield.width as i32 - 1,
            pixel_y,
            GRID_LINE,
        );
    }
}

fn read_best_score(storage: &mut dyn LocalStorage) -> u32 {
    storage
        .get(BEST_SCORE_KEY)
        .ok()
        .flatten()
        .and_then(|value| value.trim().parse::<u32>().ok())
        .unwrap_or(0)
}

fn draw_score_hud(framebuffer: &mut Framebuffer, layout: SnakeLayout, score: u32, best_score: u32) {
    let text = format!("SCORE {score}    BEST {best_score}");
    let (_, height) = Framebuffer::text_size(&text, HUD_TEXT_SCALE);
    let text_y = layout.hud.y + centered_offset(layout.hud.height, height);

    framebuffer.fill_rect(
        layout.hud.x,
        layout.hud.y,
        layout.hud.width,
        layout.hud.height,
        HUD_BACKDROP,
    );
    framebuffer.draw_rect(
        layout.hud.x,
        layout.hud.y,
        layout.hud.width,
        layout.hud.height,
        BORDER,
    );
    framebuffer.draw_text_scaled(4, text_y, &text, HUD_TEXT_SCALE, HUD_TEXT);
}

fn draw_game_over(framebuffer: &mut Framebuffer, layout: SnakeLayout, score: u32) {
    framebuffer.fill_rect(
        layout.game_over_panel.x,
        layout.game_over_panel.y,
        layout.game_over_panel.width,
        layout.game_over_panel.height,
        PANEL_FILL,
    );
    framebuffer.draw_rect(
        layout.game_over_panel.x,
        layout.game_over_panel.y,
        layout.game_over_panel.width,
        layout.game_over_panel.height,
        GAME_OVER,
    );

    draw_centered_text_in_rect(
        framebuffer,
        layout.playfield,
        layout.game_over_panel.y + 14,
        "GAME OVER",
        GAME_OVER_TEXT_SCALE,
        GAME_OVER,
    );
    draw_centered_text_in_rect(
        framebuffer,
        layout.playfield,
        layout.game_over_panel.y + 40,
        &format!("SCORE {score}"),
        GAME_OVER_TEXT_SCALE,
        HUD_TEXT,
    );
    draw_replay_button(framebuffer, layout);
}

fn draw_replay_button(framebuffer: &mut Framebuffer, layout: SnakeLayout) {
    framebuffer.fill_rect(
        layout.replay.x,
        layout.replay.y,
        layout.replay.width,
        layout.replay.height,
        BUTTON_FILL,
    );
    framebuffer.draw_rect(
        layout.replay.x,
        layout.replay.y,
        layout.replay.width,
        layout.replay.height,
        BUTTON_BORDER,
    );

    let text = "REJOUER";
    let (text_width, text_height) = Framebuffer::text_size(text, GAME_OVER_TEXT_SCALE);
    let text_x = layout.replay.x + centered_offset(layout.replay.width, text_width);
    let text_y = layout.replay.y + centered_offset(layout.replay.height, text_height);
    framebuffer.draw_text_scaled(text_x, text_y, text, GAME_OVER_TEXT_SCALE, BUTTON_TEXT);
}

fn draw_d_pad(framebuffer: &mut Framebuffer, layout: SnakeLayout) {
    let Some(controls) = layout.controls else {
        return;
    };
    let Some(d_pad) = layout.d_pad else {
        return;
    };

    framebuffer.draw_rect(
        controls.x,
        controls.y,
        controls.width,
        controls.height,
        BORDER,
    );

    for zone in d_pad.direction_zones() {
        draw_d_pad_button(framebuffer, zone.rect, zone.direction);
    }

    framebuffer.fill_rect(
        d_pad.center.x,
        d_pad.center.y,
        d_pad.center.width,
        d_pad.center.height,
        D_PAD_CENTER_FILL,
    );
    framebuffer.draw_rect(
        d_pad.center.x,
        d_pad.center.y,
        d_pad.center.width,
        d_pad.center.height,
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

fn draw_centered_text_in_rect(
    framebuffer: &mut Framebuffer,
    rect: Rect,
    y: i32,
    text: &str,
    scale: u32,
    pixel: Pixel,
) {
    let (width, _) = Framebuffer::text_size(text, scale);
    let x = rect.x + centered_offset(rect.width, width);
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

    fn from_frame(
        frame: &Frame<'_>,
        phase: Phase,
        virtual_update: Option<&VirtualPadUpdate>,
        layout: SnakeLayout,
        controls: &ControlMap,
    ) -> Self {
        match phase {
            Phase::Running => Self::from_running_inputs(controls, virtual_update),
            Phase::GameOver => Self {
                directions: Vec::new(),
                restart: replay_requested(
                    controls.action(CONTROL_RESTART).pressed(),
                    frame.input.mouse_button(MouseButton::Left).pressed(),
                    frame.input.mouse_position(),
                    frame.input.touches(),
                    layout,
                ),
            },
        }
    }

    fn from_running_inputs(
        controls: &ControlMap,
        virtual_update: Option<&VirtualPadUpdate>,
    ) -> Self {
        let mut result = Self::none();
        let virtual_actions = virtual_update
            .map(VirtualPadUpdate::pressed_actions)
            .unwrap_or(&[]);

        for action in virtual_actions.iter().copied() {
            if let Some(direction) = direction_for_action(action) {
                result.directions.push(direction);
            }
        }

        for (action, direction) in [
            (CONTROL_UP, Direction::Up),
            (CONTROL_RIGHT, Direction::Right),
            (CONTROL_DOWN, Direction::Down),
            (CONTROL_LEFT, Direction::Left),
        ] {
            if controls.action(action).pressed() && !virtual_actions.contains(&action) {
                result.directions.push(direction);
            }
        }

        result.restart = controls.action(CONTROL_RESTART).pressed();
        result
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

fn direction_for_action(action: ActionId) -> Option<Direction> {
    if action == CONTROL_UP {
        Some(Direction::Up)
    } else if action == CONTROL_RIGHT {
        Some(Direction::Right)
    } else if action == CONTROL_DOWN {
        Some(Direction::Down)
    } else if action == CONTROL_LEFT {
        Some(Direction::Left)
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DirectionZone {
    rect: Rect,
    direction: Direction,
}

fn replay_requested(
    keyboard_restart_pressed: bool,
    mouse_left_pressed: bool,
    mouse_position: Option<(i32, i32)>,
    touches: &[Touch],
    layout: SnakeLayout,
) -> bool {
    keyboard_restart_pressed
        || (mouse_left_pressed
            && mouse_position.is_some_and(|position| layout.replay.contains(position)))
        || touches.iter().any(|touch| {
            touch.phase == TouchPhase::Started
                && touch
                    .position
                    .is_some_and(|position| layout.replay.contains(position))
        })
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, VecDeque};
    use std::time::Duration;

    use super::{
        BEST_SCORE_KEY, BUTTON_BORDER, BUTTON_TEXT, CONTROL_DOWN, CONTROL_LEFT, CONTROL_RIGHT,
        CONTROL_UP, Cell, D_PAD_ARROW, D_PAD_CENTER_FILL, D_PAD_FILL, DEATH_SOUND, Direction,
        EAT_SOUND, FOOD, FRAMEBUFFER_HEIGHT, GAME_OVER, GRID_LINE, HUD_BACKDROP, HUD_TEXT,
        KEYBOARD_FRAMEBUFFER_WIDTH, PANEL_FILL, Phase, SNAKE_BODY, SNAKE_HEAD, SNAKE_SOUNDS,
        SnakeControls, SnakeGame, SnakeInteractionMode, SnakeLayout, TICK_PERIOD,
        TOUCH_FRAMEBUFFER_WIDTH, TURN_SOUND, default_controls, direction_for_action, draw_d_pad,
        draw_game_over, draw_grid, draw_score_hud, replay_requested, virtual_pad_for_mode,
    };
    use gotoo_pixel_engine::{
        Audio, AudioError, Frame, Framebuffer, Game, GameResult, Input, LocalStorage, NoopAudio,
        Pixel, Rect, Size, SoundId, StorageError, Touch, TouchPhase, Viewport, ui::VirtualButton,
    };

    #[derive(Debug, Default, Clone, PartialEq, Eq)]
    struct TestStorage {
        entries: HashMap<String, String>,
        fail_reads: bool,
        fail_writes: bool,
    }

    #[derive(Default)]
    struct TestAudio {
        plays: Vec<SoundId>,
        fail_play: bool,
    }

    impl TestStorage {
        fn with_entry(key: &str, value: &str) -> Self {
            let mut storage = Self::default();
            storage.entries.insert(key.into(), value.into());
            storage
        }

        fn failing_reads() -> Self {
            Self {
                fail_reads: true,
                ..Self::default()
            }
        }

        fn failing_writes() -> Self {
            Self {
                fail_writes: true,
                ..Self::default()
            }
        }
    }

    impl LocalStorage for TestStorage {
        fn get(&mut self, key: &str) -> Result<Option<String>, StorageError> {
            if self.fail_reads {
                return Err(StorageError::new("read failed"));
            }
            Ok(self.entries.get(key).cloned())
        }

        fn set(&mut self, key: &str, value: &str) -> Result<(), StorageError> {
            if self.fail_writes {
                return Err(StorageError::new("write failed"));
            }
            self.entries.insert(key.into(), value.into());
            Ok(())
        }
    }

    impl TestAudio {
        fn failing_play() -> Self {
            Self {
                fail_play: true,
                ..Self::default()
            }
        }

        fn count(&self, id: SoundId) -> usize {
            self.plays.iter().filter(|sound| **sound == id).count()
        }
    }

    impl Audio for TestAudio {
        fn register_wav(&mut self, _id: SoundId, _bytes: &[u8]) -> Result<(), AudioError> {
            Ok(())
        }

        fn play(&mut self, id: SoundId) -> Result<(), AudioError> {
            if self.fail_play {
                return Err(AudioError::new("play failed"));
            }
            self.plays.push(id);
            Ok(())
        }
    }

    fn keyboard_game() -> SnakeGame {
        SnakeGame::new(SnakeInteractionMode::Keyboard)
    }

    fn touch_game() -> SnakeGame {
        SnakeGame::new(SnakeInteractionMode::Touch)
    }

    fn head(game: &SnakeGame) -> Cell {
        game.world.snake[0]
    }

    fn tick(game: &mut SnakeGame) {
        game.update_logic(TICK_PERIOD, SnakeControls::none());
    }

    fn touch_layout() -> SnakeLayout {
        SnakeLayout::touch()
    }

    fn keyboard_layout() -> SnakeLayout {
        SnakeLayout::keyboard()
    }

    fn rect_center(rect: Rect) -> (i32, i32) {
        (
            rect.x + rect.width as i32 / 2,
            rect.y + rect.height as i32 / 2,
        )
    }

    fn touch(id: u64, phase: TouchPhase, position: Option<(i32, i32)>) -> Touch {
        Touch {
            id,
            phase,
            position,
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

    fn count_pixels_in_rect(framebuffer: &Framebuffer, pixel: Pixel, rect: Rect) -> usize {
        let mut count = 0;
        for y in rect.y..rect.y + rect.height as i32 {
            for x in rect.x..rect.x + rect.width as i32 {
                if framebuffer.pixel(x, y) == Some(pixel) {
                    count += 1;
                }
            }
        }
        count
    }

    fn count_pixels_outside_rect(framebuffer: &Framebuffer, pixel: Pixel, rect: Rect) -> usize {
        let mut count = 0;
        for y in 0..framebuffer.height() as i32 {
            for x in 0..framebuffer.width() as i32 {
                if !rect.contains((x, y)) && framebuffer.pixel(x, y) == Some(pixel) {
                    count += 1;
                }
            }
        }
        count
    }

    fn assert_color_only_inside(framebuffer: &Framebuffer, pixel: Pixel, rect: Rect) {
        assert!(count_pixels_in_rect(framebuffer, pixel, rect) > 0);
        assert_eq!(count_pixels_outside_rect(framebuffer, pixel, rect), 0);
    }

    fn update_with_services(
        game: &mut SnakeGame,
        storage: &mut dyn LocalStorage,
        audio: &mut dyn Audio,
        delta_time: Duration,
        surface_size: Size,
    ) -> GameResult {
        let framebuffer_size = game.layout().framebuffer_size;
        let mut framebuffer = Framebuffer::new(framebuffer_size.width, framebuffer_size.height);
        let input = Input::default();
        let mut frame = Frame {
            framebuffer: &mut framebuffer,
            input: &input,
            delta_time,
            surface_size,
            viewport: Viewport::new(surface_size, framebuffer_size),
            storage,
            audio,
        };
        game.update(&mut frame)
    }

    fn update_with_storage(
        game: &mut SnakeGame,
        storage: &mut dyn LocalStorage,
        delta_time: Duration,
    ) -> GameResult {
        let mut audio = TestAudio::default();
        let size = game.layout().framebuffer_size;
        update_with_services(game, storage, &mut audio, delta_time, size)
    }

    #[test]
    fn interaction_modes_expose_expected_layouts() {
        let keyboard = keyboard_layout();
        let touch = touch_layout();

        assert_eq!(
            keyboard.framebuffer_size,
            Size {
                width: KEYBOARD_FRAMEBUFFER_WIDTH,
                height: FRAMEBUFFER_HEIGHT,
            }
        );
        assert_eq!(keyboard.controls, None);
        assert_eq!(keyboard.d_pad, None);

        assert_eq!(
            touch.framebuffer_size,
            Size {
                width: TOUCH_FRAMEBUFFER_WIDTH,
                height: FRAMEBUFFER_HEIGHT,
            }
        );
        assert_eq!(
            touch.playfield,
            Rect {
                x: 0,
                y: 24,
                width: 320,
                height: 180,
            }
        );
        assert!(touch.controls.is_some());
        assert!(touch.d_pad.is_some());
    }

    #[test]
    fn touch_virtual_pad_is_wired_to_snake_actions_and_layout() {
        let layout = touch_layout();
        let d_pad = layout.d_pad.expect("touch layout should have a D-pad");
        let pad = virtual_pad_for_mode(SnakeInteractionMode::Touch)
            .expect("touch mode should create a virtual pad");

        assert_eq!(
            pad.buttons(),
            &[
                VirtualButton::new(CONTROL_UP, d_pad.up),
                VirtualButton::new(CONTROL_RIGHT, d_pad.right),
                VirtualButton::new(CONTROL_DOWN, d_pad.down),
                VirtualButton::new(CONTROL_LEFT, d_pad.left),
            ]
        );
        assert!(!pad.visible());
    }

    #[test]
    fn keyboard_mode_has_no_virtual_pad() {
        assert!(virtual_pad_for_mode(SnakeInteractionMode::Keyboard).is_none());
        assert!(keyboard_game().virtual_pad.is_none());
    }

    #[test]
    fn snake_action_mapping_is_explicit() {
        assert_eq!(direction_for_action(CONTROL_UP), Some(Direction::Up));
        assert_eq!(direction_for_action(CONTROL_RIGHT), Some(Direction::Right));
        assert_eq!(direction_for_action(CONTROL_DOWN), Some(Direction::Down));
        assert_eq!(direction_for_action(CONTROL_LEFT), Some(Direction::Left));
    }

    #[test]
    fn virtual_control_state_feeds_running_snake_controls() {
        let mut controls = default_controls();
        controls.set_virtual(CONTROL_UP, true);
        controls.update(&Input::default());

        let snake_controls = SnakeControls::from_running_inputs(&controls, None);
        assert_eq!(snake_controls.directions, [Direction::Up]);
    }

    #[test]
    fn layout_zones_do_not_overlap_playfield() {
        let layout = touch_layout();
        let controls = layout.controls.expect("touch layout should have controls");
        let d_pad = layout.d_pad.expect("touch layout should have a D-pad");

        assert!(!layout.hud.intersects(layout.playfield));
        assert!(!controls.intersects(layout.playfield));
        for rect in [d_pad.up, d_pad.left, d_pad.center, d_pad.right, d_pad.down] {
            assert!(controls.contains((rect.x, rect.y)));
            assert!(controls.contains((
                rect.x + rect.width as i32 - 1,
                rect.y + rect.height as i32 - 1,
            )));
            assert!(!rect.intersects(layout.playfield));
        }
    }

    #[test]
    fn grid_is_drawn_only_inside_playfield() {
        let mut framebuffer = Framebuffer::new(TOUCH_FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT);
        let layout = touch_layout();
        draw_grid(&mut framebuffer, layout);

        assert_color_only_inside(&framebuffer, GRID_LINE, layout.playfield);
    }

    #[test]
    fn d_pad_is_drawn_only_inside_controls_rect() {
        let mut framebuffer = Framebuffer::new(TOUCH_FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT);
        let layout = touch_layout();
        let controls = layout.controls.expect("touch layout should have controls");
        draw_d_pad(&mut framebuffer, layout);

        assert_color_only_inside(&framebuffer, D_PAD_FILL, controls);
        assert_color_only_inside(&framebuffer, D_PAD_CENTER_FILL, controls);
        assert_color_only_inside(&framebuffer, D_PAD_ARROW, controls);
    }

    #[test]
    fn keyboard_game_draws_no_d_pad() {
        let game = keyboard_game();
        let mut framebuffer = Framebuffer::new(KEYBOARD_FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT);
        let mut storage = TestStorage::default();
        let mut audio = TestAudio::default();
        let surface_size = SnakeInteractionMode::Keyboard.framebuffer_size();
        let input = Input::default();
        let mut frame = Frame {
            framebuffer: &mut framebuffer,
            input: &input,
            delta_time: Duration::ZERO,
            storage: &mut storage,
            audio: &mut audio,
            surface_size,
            viewport: Viewport::new(surface_size, surface_size),
        };

        game.draw(&mut frame);

        assert_eq!(count_pixels(&framebuffer, D_PAD_FILL), 0);
        assert_eq!(count_pixels(&framebuffer, D_PAD_CENTER_FILL), 0);
        assert_eq!(count_pixels(&framebuffer, D_PAD_ARROW), 0);
    }

    #[test]
    fn game_over_panel_and_replay_are_centered_in_playfield() {
        for layout in [keyboard_layout(), touch_layout()] {
            assert_eq!(
                layout.game_over_panel.x,
                layout.playfield.x
                    + (layout.playfield.width - layout.game_over_panel.width) as i32 / 2
            );
            assert!(layout.playfield.contains(rect_center(layout.replay)));
        }
    }

    #[test]
    fn replay_request_accepts_keyboard_mouse_and_touch() {
        let layout = touch_layout();
        let replay_center = rect_center(layout.replay);

        assert!(replay_requested(true, false, None, &[], layout));
        assert!(replay_requested(
            false,
            true,
            Some(replay_center),
            &[],
            layout
        ));
        assert!(replay_requested(
            false,
            false,
            None,
            &[touch(1, TouchPhase::Started, Some(replay_center))],
            layout
        ));
    }

    #[test]
    fn replay_request_ignores_outside_and_non_started_touch() {
        let layout = touch_layout();
        let replay_center = rect_center(layout.replay);

        assert!(!replay_requested(false, true, Some((0, 0)), &[], layout));
        assert!(!replay_requested(
            false,
            false,
            None,
            &[touch(1, TouchPhase::Moved, Some(replay_center))],
            layout
        ));
        assert!(!replay_requested(
            false,
            false,
            None,
            &[touch(1, TouchPhase::Started, Some((0, 0)))],
            layout
        ));
    }

    #[test]
    fn replay_hitbox_works_in_keyboard_layout_too() {
        let layout = keyboard_layout();
        let center = rect_center(layout.replay);
        assert!(replay_requested(false, true, Some(center), &[], layout));
        assert!(replay_requested(
            false,
            false,
            None,
            &[touch(1, TouchPhase::Started, Some(center))],
            layout
        ));
    }

    #[test]
    fn game_over_render_contains_panel_score_and_button() {
        let mut framebuffer = Framebuffer::new(TOUCH_FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT);
        let layout = touch_layout();
        draw_game_over(&mut framebuffer, layout, 7);

        assert!(count_pixels(&framebuffer, GAME_OVER) > 0);
        assert!(count_pixels(&framebuffer, HUD_TEXT) > 0);
        assert!(count_pixels(&framebuffer, BUTTON_TEXT) > 0);
        assert_eq!(
            framebuffer.pixel(layout.replay.x, layout.replay.y),
            Some(BUTTON_BORDER)
        );
        assert_color_only_inside(&framebuffer, PANEL_FILL, layout.playfield);
    }

    #[test]
    fn snake_and_food_are_rendered_only_inside_playfield() {
        let mut game = touch_game();
        game.world.food = Some(Cell { x: 4, y: 5 });
        let mut framebuffer = Framebuffer::new(TOUCH_FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT);
        let mut storage = TestStorage::default();
        let mut audio = TestAudio::default();
        let surface_size = SnakeInteractionMode::Touch.framebuffer_size();
        let input = Input::default();
        let mut frame = Frame {
            framebuffer: &mut framebuffer,
            input: &input,
            delta_time: Duration::ZERO,
            storage: &mut storage,
            audio: &mut audio,
            surface_size,
            viewport: Viewport::new(surface_size, surface_size),
        };

        game.draw(&mut frame);

        assert_color_only_inside(&framebuffer, FOOD, touch_layout().playfield);
        assert_color_only_inside(&framebuffer, SNAKE_HEAD, touch_layout().playfield);
        assert_color_only_inside(&framebuffer, SNAKE_BODY, touch_layout().playfield);
    }

    #[test]
    fn hud_draws_score_and_best_inside_hud() {
        let mut framebuffer = Framebuffer::new(TOUCH_FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT);
        let layout = touch_layout();
        draw_score_hud(&mut framebuffer, layout, 12, 37);

        assert_color_only_inside(&framebuffer, HUD_BACKDROP, layout.hud);
        assert_color_only_inside(&framebuffer, HUD_TEXT, layout.hud);
    }

    #[test]
    fn snake_does_not_move_before_complete_tick() {
        let mut game = touch_game();
        let initial_head = head(&game);
        game.update_logic(
            TICK_PERIOD - Duration::from_millis(1),
            SnakeControls::none(),
        );
        assert_eq!(head(&game), initial_head);
    }

    #[test]
    fn small_delta_times_accumulate_into_tick() {
        let mut game = touch_game();
        for _ in 0..3 {
            game.update_logic(Duration::from_millis(40), SnakeControls::none());
        }
        assert_eq!(head(&game), Cell { x: 17, y: 9 });
    }

    #[test]
    fn eating_food_grows_and_scores() {
        let mut game = touch_game();
        game.world.food = Some(Cell { x: 17, y: 9 });
        let initial_len = game.world.snake.len();
        tick(&mut game);

        assert_eq!(game.world.snake.len(), initial_len + 1);
        assert_eq!(game.world.score(), 1);
    }

    #[test]
    fn invalid_reverse_is_rejected() {
        let mut game = touch_game();
        game.update_logic(
            TICK_PERIOD,
            SnakeControls::with_directions([Direction::Left]),
        );

        assert_eq!(head(&game), Cell { x: 17, y: 9 });
        assert_eq!(game.world.direction, Direction::Right);
    }

    #[test]
    fn two_valid_turns_are_applied_on_successive_ticks() {
        let mut game = touch_game();
        game.update_logic(
            TICK_PERIOD,
            SnakeControls::with_directions([Direction::Up, Direction::Left]),
        );
        assert_eq!(game.world.direction, Direction::Up);
        tick(&mut game);
        assert_eq!(game.world.direction, Direction::Left);
    }

    #[test]
    fn restart_restores_world_and_accumulator() {
        let mut game = touch_game();
        game.world.phase = Phase::GameOver;
        game.world.snake = VecDeque::from([Cell { x: 1, y: 1 }]);
        game.world.direction = Direction::Down;
        game.world.turn_queue = VecDeque::from([Direction::Left]);
        game.world.score = 4;
        game.accumulator = TICK_PERIOD;

        game.update_logic(Duration::ZERO, SnakeControls::restart());

        assert_eq!(game.world.phase, Phase::Running);
        assert_eq!(game.world.snake, super::world::initial_snake());
        assert_eq!(game.world.direction, Direction::Right);
        assert!(game.world.turn_queue.is_empty());
        assert_eq!(game.world.score(), 0);
        assert_eq!(game.accumulator, Duration::ZERO);
        assert!(game.world.food.is_some());
    }

    #[test]
    fn best_score_defaults_to_zero_when_storage_absent() {
        let mut game = touch_game();
        let mut storage = TestStorage::default();
        update_with_storage(&mut game, &mut storage, Duration::ZERO);

        assert_eq!(game.best_score, 0);
        assert!(game.best_score_loaded);
    }

    #[test]
    fn best_score_loads_once() {
        let mut game = touch_game();
        let mut storage = TestStorage::with_entry(BEST_SCORE_KEY, "37");
        update_with_storage(&mut game, &mut storage, Duration::ZERO);
        storage.entries.insert(BEST_SCORE_KEY.into(), "99".into());
        update_with_storage(&mut game, &mut storage, Duration::ZERO);

        assert_eq!(game.best_score, 37);
    }

    #[test]
    fn corrupt_or_failed_best_score_read_falls_back_to_zero() {
        let mut corrupt = touch_game();
        let mut corrupt_storage = TestStorage::with_entry(BEST_SCORE_KEY, "not a number");
        update_with_storage(&mut corrupt, &mut corrupt_storage, Duration::ZERO);
        assert_eq!(corrupt.best_score, 0);

        let mut failed = touch_game();
        let mut failed_storage = TestStorage::failing_reads();
        update_with_storage(&mut failed, &mut failed_storage, Duration::ZERO);
        assert_eq!(failed.best_score, 0);
    }

    #[test]
    fn new_record_persists_immediately() {
        let mut game = touch_game();
        let mut storage = TestStorage::default();
        game.world.food = Some(Cell { x: 17, y: 9 });
        update_with_storage(&mut game, &mut storage, TICK_PERIOD);

        assert_eq!(game.best_score, 1);
        assert_eq!(
            storage.entries.get(BEST_SCORE_KEY).map(String::as_str),
            Some("1")
        );
    }

    #[test]
    fn best_score_write_failure_keeps_record_in_memory() {
        let mut game = touch_game();
        let mut storage = TestStorage::failing_writes();
        game.world.score = 5;
        update_with_storage(&mut game, &mut storage, Duration::ZERO);

        assert_eq!(game.best_score, 5);
        assert!(!storage.entries.contains_key(BEST_SCORE_KEY));
    }

    #[test]
    fn surface_resize_does_not_restart_world() {
        let mut game = touch_game();
        let mut storage = TestStorage::default();
        let mut audio = TestAudio::default();
        game.world.food = Some(Cell { x: 17, y: 9 });
        let normal_size = game.layout().framebuffer_size;
        update_with_services(
            &mut game,
            &mut storage,
            &mut audio,
            TICK_PERIOD,
            normal_size,
        );
        let world_before = game.world.clone();

        update_with_services(
            &mut game,
            &mut storage,
            &mut audio,
            Duration::ZERO,
            Size {
                width: 412,
                height: 915,
            },
        );

        assert_eq!(game.world, world_before);
    }

    #[test]
    fn food_and_death_request_expected_audio_once() {
        let mut eat_game = touch_game();
        let mut eat_storage = TestStorage::default();
        let mut eat_audio = TestAudio::default();
        eat_game.world.food = Some(Cell { x: 17, y: 9 });
        let eat_size = eat_game.layout().framebuffer_size;
        update_with_services(
            &mut eat_game,
            &mut eat_storage,
            &mut eat_audio,
            TICK_PERIOD,
            eat_size,
        );
        assert_eq!(eat_audio.count(EAT_SOUND), 1);
        assert_eq!(eat_audio.count(DEATH_SOUND), 0);

        let mut death_game = touch_game();
        let mut death_storage = TestStorage::default();
        let mut death_audio = TestAudio::default();
        death_game.world.snake = VecDeque::from([
            Cell { x: 31, y: 9 },
            Cell { x: 30, y: 9 },
            Cell { x: 29, y: 9 },
        ]);
        death_game.world.direction = Direction::Right;
        let death_size = death_game.layout().framebuffer_size;
        update_with_services(
            &mut death_game,
            &mut death_storage,
            &mut death_audio,
            TICK_PERIOD,
            death_size,
        );
        update_with_services(
            &mut death_game,
            &mut death_storage,
            &mut death_audio,
            TICK_PERIOD,
            death_size,
        );
        assert_eq!(death_audio.count(DEATH_SOUND), 1);
    }

    #[test]
    fn audio_play_error_does_not_stop_gameplay() {
        let mut game = touch_game();
        let mut storage = TestStorage::default();
        let mut audio = TestAudio::failing_play();
        game.world.food = Some(Cell { x: 17, y: 9 });
        let size = game.layout().framebuffer_size;

        assert_eq!(
            update_with_services(&mut game, &mut storage, &mut audio, TICK_PERIOD, size),
            GameResult::Continue
        );
        assert_eq!(game.world.score(), 1);
    }

    #[test]
    fn snake_audio_assets_and_turn_sound_are_playable() {
        let mut audio = NoopAudio::default();
        for (id, bytes) in SNAKE_SOUNDS {
            audio
                .register_wav(id, bytes)
                .expect("snake audio asset should be valid");
            audio.play(id).expect("registered sound should play");
        }

        let mut bank = super::snake_sound_bank();
        bank.play(&mut audio, TURN_SOUND)
            .expect("generated turn sound should play");
    }
}
