use gotoo_pixel_engine::{
    ActionId, ControlMap, EngineConfig, EngineError, Frame, Framebuffer, Game, GameResult,
    GamepadButton, Key, Pixel, Size, SoundBank, SoundId, pcm16_mono_wav, run,
    ui::{PauseConfig, PauseGame},
};

const FRAMEBUFFER_WIDTH: u32 = 180;
const FRAMEBUFFER_HEIGHT: u32 = 320;

const MOVE_LEFT: ActionId = ActionId::new("void_canticle.left");
const MOVE_RIGHT: ActionId = ActionId::new("void_canticle.right");
const MOVE_UP: ActionId = ActionId::new("void_canticle.up");
const MOVE_DOWN: ActionId = ActionId::new("void_canticle.down");
const FIRE: ActionId = ActionId::new("void_canticle.fire");

const FIRE_SOUND: SoundId = SoundId::new("void_canticle.fire");
const ENEMY_HIT_SOUND: SoundId = SoundId::new("void_canticle.enemy_hit");
const ENEMY_FIRE_SOUND: SoundId = SoundId::new("void_canticle.enemy_fire");
const CINDER_SOUND: SoundId = SoundId::new("void_canticle.cinder");
const PLAYER_HIT_SOUND: SoundId = SoundId::new("void_canticle.player_hit");
const AUDIO_SAMPLE_RATE: u32 = 44_100;

const BG: Pixel = Pixel::rgb(5, 6, 14);
const STAR_DIM: Pixel = Pixel::rgb(72, 78, 104);
const STAR_BRIGHT: Pixel = Pixel::rgb(180, 190, 220);
const PILGRIM: Pixel = Pixel::rgb(225, 222, 205);
const PILGRIM_CORE: Pixel = Pixel::rgb(245, 170, 70);
const SHOT: Pixel = Pixel::rgb(245, 235, 180);
const ENEMY: Pixel = Pixel::rgb(150, 78, 95);
const ENEMY_EYE: Pixel = Pixel::rgb(255, 110, 70);
const ENEMY_SHOT: Pixel = Pixel::rgb(232, 80, 110);
const CINDER: Pixel = Pixel::rgb(255, 195, 80);
const TEXT: Pixel = Pixel::rgb(205, 210, 225);
const ACCENT: Pixel = Pixel::rgb(195, 132, 74);
const DANGER: Pixel = Pixel::rgb(255, 75, 70);
const CORE_BG: Pixel = Pixel::rgb(42, 35, 42);

const PLAYER_SPEED: f32 = 118.0;
const PLAYER_SHOT_SPEED: f32 = 250.0;
const ENEMY_SHOT_SPEED: f32 = 72.0;
const FIRE_PERIOD: f32 = 0.09;
const PLAYER_INVULNERABILITY: f32 = 1.05;
const CINDER_CHARGE: u32 = 12;
const WAVE_CYCLE: f32 = 10.5;

#[derive(Debug, Clone, Copy)]
struct Bullet {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    alive: bool,
}

#[derive(Debug, Clone, Copy)]
struct CarrionDrone {
    base_x: f32,
    x: f32,
    y: f32,
    age: f32,
    phase: f32,
    curve_amplitude: f32,
    fire_timer: f32,
    alive: bool,
}

impl CarrionDrone {
    fn new(spec: SpawnSpec) -> Self {
        Self {
            base_x: spec.base_x,
            x: spec.base_x,
            y: -10.0,
            age: 0.0,
            phase: spec.phase,
            curve_amplitude: spec.curve_amplitude,
            fire_timer: spec.first_shot,
            alive: true,
        }
    }

    fn update_position(&mut self, dt: f32) {
        self.age += dt;
        self.x = curved_x(self.base_x, self.age, self.phase, self.curve_amplitude);
        self.y = -10.0 + self.age * 43.0;
    }
}

#[derive(Debug, Clone, Copy)]
struct CinderDrop {
    x: f32,
    y: f32,
    age: f32,
    alive: bool,
}

#[derive(Debug, Clone, Copy)]
struct Burst {
    x: f32,
    y: f32,
    remaining: f32,
}

#[derive(Debug, Clone, Copy)]
struct SpawnSpec {
    at: f32,
    base_x: f32,
    phase: f32,
    curve_amplitude: f32,
    first_shot: f32,
}

const WAVE: &[SpawnSpec] = &[
    SpawnSpec { at: 0.45, base_x: 38.0, phase: 0.0, curve_amplitude: 22.0, first_shot: 2.10 },
    SpawnSpec { at: 0.78, base_x: 38.0, phase: 0.5, curve_amplitude: 22.0, first_shot: 2.20 },
    SpawnSpec { at: 1.11, base_x: 38.0, phase: 1.0, curve_amplitude: 22.0, first_shot: 2.30 },
    SpawnSpec { at: 2.50, base_x: 142.0, phase: 3.2, curve_amplitude: -24.0, first_shot: 1.70 },
    SpawnSpec { at: 2.83, base_x: 142.0, phase: 3.7, curve_amplitude: -24.0, first_shot: 1.85 },
    SpawnSpec { at: 3.16, base_x: 142.0, phase: 4.2, curve_amplitude: -24.0, first_shot: 2.00 },
    SpawnSpec { at: 4.75, base_x: 90.0, phase: 0.0, curve_amplitude: 50.0, first_shot: 1.35 },
    SpawnSpec { at: 5.12, base_x: 90.0, phase: 1.6, curve_amplitude: 50.0, first_shot: 1.55 },
    SpawnSpec { at: 5.49, base_x: 90.0, phase: 3.2, curve_amplitude: 50.0, first_shot: 1.75 },
    SpawnSpec { at: 7.20, base_x: 54.0, phase: 0.7, curve_amplitude: 30.0, first_shot: 1.20 },
    SpawnSpec { at: 7.55, base_x: 126.0, phase: 3.8, curve_amplitude: -30.0, first_shot: 1.20 },
];

struct VoidCanticleGame {
    controls: ControlMap,
    sounds: SoundBank,
    player_x: f32,
    player_y: f32,
    player_bullets: Vec<Bullet>,
    enemy_bullets: Vec<Bullet>,
    enemies: Vec<CarrionDrone>,
    cinders: Vec<CinderDrop>,
    bursts: Vec<Burst>,
    fire_cooldown: f32,
    stage_time: f32,
    next_spawn: usize,
    scroll: f32,
    score: u32,
    lives: u32,
    core_charge: u32,
    invulnerability: f32,
    game_over: bool,
}

impl VoidCanticleGame {
    fn new() -> Self {
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
            .bind_key(MOVE_UP, Key::Up)
            .bind_key(MOVE_UP, Key::W)
            .bind_gamepad(MOVE_UP, GamepadButton::DPadUp)
            .bind_gamepad(MOVE_UP, GamepadButton::LeftStickUp)
            .bind_key(MOVE_DOWN, Key::Down)
            .bind_key(MOVE_DOWN, Key::S)
            .bind_gamepad(MOVE_DOWN, GamepadButton::DPadDown)
            .bind_gamepad(MOVE_DOWN, GamepadButton::LeftStickDown)
            .bind_key(FIRE, Key::Space)
            .bind_gamepad(FIRE, GamepadButton::South);

        let mut sounds = SoundBank::new();
        for (id, wav) in [
            (FIRE_SOUND, synthesize_chirp(920.0, 520.0, 0.050, 0.14)),
            (ENEMY_HIT_SOUND, synthesize_noise_burst(0.105, 0.34, 0xC411_10A1)),
            (ENEMY_FIRE_SOUND, synthesize_chirp(260.0, 410.0, 0.070, 0.13)),
            (CINDER_SOUND, synthesize_chirp(730.0, 1120.0, 0.095, 0.24)),
            (PLAYER_HIT_SOUND, synthesize_noise_burst(0.240, 0.48, 0x5157_0001)),
        ] {
            sounds
                .insert_wav(id, wav)
                .expect("Void Canticle sound ids should be unique");
        }

        let mut game = Self {
            controls,
            sounds,
            player_x: FRAMEBUFFER_WIDTH as f32 / 2.0,
            player_y: FRAMEBUFFER_HEIGHT as f32 - 42.0,
            player_bullets: Vec::new(),
            enemy_bullets: Vec::new(),
            enemies: Vec::new(),
            cinders: Vec::new(),
            bursts: Vec::new(),
            fire_cooldown: 0.0,
            stage_time: 0.0,
            next_spawn: 0,
            scroll: 0.0,
            score: 0,
            lives: 3,
            core_charge: 0,
            invulnerability: 0.0,
            game_over: false,
        };
        game.reset_run();
        game
    }

    fn reset_run(&mut self) {
        self.player_x = FRAMEBUFFER_WIDTH as f32 / 2.0;
        self.player_y = FRAMEBUFFER_HEIGHT as f32 - 42.0;
        self.player_bullets.clear();
        self.enemy_bullets.clear();
        self.enemies.clear();
        self.cinders.clear();
        self.bursts.clear();
        self.fire_cooldown = 0.0;
        self.stage_time = 0.0;
        self.next_spawn = 0;
        self.score = 0;
        self.lives = 3;
        self.core_charge = 0;
        self.invulnerability = 0.0;
        self.game_over = false;
    }

    fn update_player(&mut self, dt: f32) {
        let left = self.controls.action(MOVE_LEFT).held();
        let right = self.controls.action(MOVE_RIGHT).held();
        let up = self.controls.action(MOVE_UP).held();
        let down = self.controls.action(MOVE_DOWN).held();

        let mut dx = (right as i8 - left as i8) as f32;
        let mut dy = (down as i8 - up as i8) as f32;
        if dx != 0.0 && dy != 0.0 {
            const INV_SQRT_2: f32 = 0.707_106_77;
            dx *= INV_SQRT_2;
            dy *= INV_SQRT_2;
        }

        self.player_x = (self.player_x + dx * PLAYER_SPEED * dt)
            .clamp(8.0, FRAMEBUFFER_WIDTH as f32 - 8.0);
        self.player_y = (self.player_y + dy * PLAYER_SPEED * dt)
            .clamp(30.0, FRAMEBUFFER_HEIGHT as f32 - 10.0);
    }

    fn update_player_fire(&mut self, dt: f32, frame: &mut Frame<'_>) {
        self.fire_cooldown = (self.fire_cooldown - dt).max(0.0);
        if self.controls.action(FIRE).held() && self.fire_cooldown <= 0.0 {
            self.player_bullets.push(Bullet {
                x: self.player_x,
                y: self.player_y - 9.0,
                vx: 0.0,
                vy: -PLAYER_SHOT_SPEED,
                alive: true,
            });
            self.fire_cooldown = FIRE_PERIOD;
            let _ = self.sounds.play(frame.audio, FIRE_SOUND);
        }
    }

    fn update_wave_timeline(&mut self, dt: f32) {
        self.stage_time += dt;

        while self.next_spawn < WAVE.len() && WAVE[self.next_spawn].at <= self.stage_time {
            self.enemies.push(CarrionDrone::new(WAVE[self.next_spawn]));
            self.next_spawn += 1;
        }

        if self.stage_time >= WAVE_CYCLE {
            self.stage_time -= WAVE_CYCLE;
            self.next_spawn = 0;
        }
    }

    fn update_enemies(&mut self, dt: f32, frame: &mut Frame<'_>) {
        let mut spawned_shots = Vec::new();

        for enemy in &mut self.enemies {
            enemy.update_position(dt);
            enemy.fire_timer -= dt;

            if enemy.fire_timer <= 0.0 && (42.0..218.0).contains(&enemy.y) {
                let (vx, vy) = aimed_velocity(
                    enemy.x,
                    enemy.y,
                    self.player_x,
                    self.player_y,
                    ENEMY_SHOT_SPEED,
                );
                spawned_shots.push(Bullet {
                    x: enemy.x,
                    y: enemy.y + 6.0,
                    vx,
                    vy,
                    alive: true,
                });
                enemy.fire_timer += 2.15;
            }

            if enemy.y > FRAMEBUFFER_HEIGHT as f32 + 18.0 {
                enemy.alive = false;
            }
        }

        if !spawned_shots.is_empty() {
            let _ = self.sounds.play(frame.audio, ENEMY_FIRE_SOUND);
            self.enemy_bullets.extend(spawned_shots);
        }
        self.enemies.retain(|enemy| enemy.alive);
    }

    fn update_projectiles(&mut self, dt: f32) {
        for bullet in &mut self.player_bullets {
            bullet.x += bullet.vx * dt;
            bullet.y += bullet.vy * dt;
            bullet.alive = bullet.y > 18.0;
        }
        self.player_bullets.retain(|bullet| bullet.alive);

        for bullet in &mut self.enemy_bullets {
            bullet.x += bullet.vx * dt;
            bullet.y += bullet.vy * dt;
            bullet.alive = bullet.x > -8.0
                && bullet.x < FRAMEBUFFER_WIDTH as f32 + 8.0
                && bullet.y > 18.0
                && bullet.y < FRAMEBUFFER_HEIGHT as f32 + 8.0;
        }
        self.enemy_bullets.retain(|bullet| bullet.alive);
    }

    fn resolve_player_shots(&mut self, frame: &mut Frame<'_>) {
        let mut kills = Vec::new();

        for bullet in &mut self.player_bullets {
            if !bullet.alive {
                continue;
            }
            for enemy in &mut self.enemies {
                if enemy.alive && bullet_hits_drone(*bullet, *enemy) {
                    bullet.alive = false;
                    enemy.alive = false;
                    kills.push((enemy.x, enemy.y));
                    break;
                }
            }
        }

        self.player_bullets.retain(|bullet| bullet.alive);
        self.enemies.retain(|enemy| enemy.alive);

        for (x, y) in kills {
            self.score = self.score.saturating_add(100);
            self.cinders.push(CinderDrop {
                x,
                y,
                age: 0.0,
                alive: true,
            });
            self.bursts.push(Burst {
                x,
                y,
                remaining: 0.18,
            });
            let _ = self.sounds.play(frame.audio, ENEMY_HIT_SOUND);
        }
    }

    fn update_cinders(&mut self, dt: f32, frame: &mut Frame<'_>) {
        let mut collected = 0_u32;

        for cinder in &mut self.cinders {
            cinder.age += dt;
            cinder.y += 34.0 * dt;
            cinder.x += (cinder.age * 4.0).sin() * 7.0 * dt;

            if distance_squared(cinder.x, cinder.y, self.player_x, self.player_y) <= 9.0 * 9.0 {
                cinder.alive = false;
                collected += 1;
            } else if cinder.y > FRAMEBUFFER_HEIGHT as f32 + 8.0 {
                cinder.alive = false;
            }
        }

        self.cinders.retain(|cinder| cinder.alive);
        if collected > 0 {
            self.core_charge = (self.core_charge + collected * CINDER_CHARGE).min(100);
            let _ = self.sounds.play(frame.audio, CINDER_SOUND);
        }
    }

    fn resolve_player_hits(&mut self, frame: &mut Frame<'_>) {
        if self.invulnerability > 0.0 || self.game_over {
            return;
        }

        let mut hit = false;
        for bullet in &mut self.enemy_bullets {
            if bullet.alive
                && distance_squared(bullet.x, bullet.y, self.player_x, self.player_y) <= 4.0 * 4.0
            {
                bullet.alive = false;
                hit = true;
                break;
            }
        }
        self.enemy_bullets.retain(|bullet| bullet.alive);

        if hit {
            self.lives = self.lives.saturating_sub(1);
            self.invulnerability = PLAYER_INVULNERABILITY;
            self.bursts.push(Burst {
                x: self.player_x,
                y: self.player_y,
                remaining: 0.34,
            });
            let _ = self.sounds.play(frame.audio, PLAYER_HIT_SOUND);

            if self.lives == 0 {
                self.game_over = true;
                self.enemy_bullets.clear();
            }
        }
    }

    fn update_bursts(&mut self, dt: f32) {
        for burst in &mut self.bursts {
            burst.remaining = (burst.remaining - dt).max(0.0);
        }
        self.bursts.retain(|burst| burst.remaining > 0.0);
    }

    fn render(&self, framebuffer: &mut Framebuffer) {
        framebuffer.clear(BG);
        render_starfield(framebuffer, self.scroll);

        for cinder in &self.cinders {
            render_cinder(framebuffer, cinder.x.round() as i32, cinder.y.round() as i32);
        }
        for enemy in &self.enemies {
            render_carrion_drone(framebuffer, enemy.x.round() as i32, enemy.y.round() as i32);
        }
        for bullet in &self.enemy_bullets {
            framebuffer.fill_rect(
                bullet.x.round() as i32 - 1,
                bullet.y.round() as i32 - 1,
                3,
                3,
                ENEMY_SHOT,
            );
        }
        for bullet in &self.player_bullets {
            framebuffer.fill_rect(
                bullet.x.round() as i32 - 1,
                bullet.y.round() as i32 - 3,
                2,
                6,
                SHOT,
            );
        }
        for burst in &self.bursts {
            render_burst(framebuffer, *burst);
        }

        let blink_off = self.invulnerability > 0.0 && (self.invulnerability * 18.0) as i32 % 2 == 0;
        if !blink_off {
            render_pilgrim(
                framebuffer,
                self.player_x.round() as i32,
                self.player_y.round() as i32,
            );
        }

        render_hud(framebuffer, self.score, self.lives, self.core_charge);

        if self.game_over {
            framebuffer.fill_rect(18, 132, 144, 54, Pixel::rgb(14, 10, 18));
            framebuffer.draw_rect(18, 132, 144, 54, DANGER);
            framebuffer.draw_text(55, 145, "PILGRIM FALLEN", DANGER);
            framebuffer.draw_text(43, 165, "SPACE / SOUTH RETRY", TEXT);
        }
    }
}

impl Game for VoidCanticleGame {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.controls.update(frame.input);
        let dt = frame.delta_time.as_secs_f32().min(0.05);
        self.scroll = (self.scroll + 36.0 * dt) % FRAMEBUFFER_HEIGHT as f32;
        self.invulnerability = (self.invulnerability - dt).max(0.0);
        self.update_bursts(dt);

        if self.game_over {
            if self.controls.action(FIRE).pressed() {
                self.reset_run();
            }
            self.render(frame.framebuffer);
            return GameResult::Continue;
        }

        self.update_player(dt);
        self.update_player_fire(dt, frame);
        self.update_wave_timeline(dt);
        self.update_enemies(dt, frame);
        self.update_projectiles(dt);
        self.resolve_player_shots(frame);
        self.update_cinders(dt, frame);
        self.resolve_player_hits(frame);
        self.render(frame.framebuffer);
        GameResult::Continue
    }
}

fn curved_x(base_x: f32, age: f32, phase: f32, amplitude: f32) -> f32 {
    base_x + (age * 2.15 + phase).sin() * amplitude
}

fn aimed_velocity(from_x: f32, from_y: f32, to_x: f32, to_y: f32, speed: f32) -> (f32, f32) {
    let dx = to_x - from_x;
    let dy = to_y - from_y;
    let length = (dx * dx + dy * dy).sqrt().max(0.001);
    (dx / length * speed, dy / length * speed)
}

fn bullet_hits_drone(bullet: Bullet, enemy: CarrionDrone) -> bool {
    bullet.x >= enemy.x - 9.0
        && bullet.x <= enemy.x + 9.0
        && bullet.y >= enemy.y - 6.0
        && bullet.y <= enemy.y + 6.0
}

fn distance_squared(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let dx = ax - bx;
    let dy = ay - by;
    dx * dx + dy * dy
}

fn render_starfield(framebuffer: &mut Framebuffer, scroll: f32) {
    for index in 0..48_i32 {
        let x = (index * 47 + 19) % FRAMEBUFFER_WIDTH as i32;
        let base_y = (index * 83 + 11) % FRAMEBUFFER_HEIGHT as i32;
        let y = (base_y + scroll.round() as i32).rem_euclid(FRAMEBUFFER_HEIGHT as i32);
        let bright = index % 7 == 0;
        framebuffer.fill_rect(
            x,
            y,
            if bright { 2 } else { 1 },
            if bright { 2 } else { 1 },
            if bright { STAR_BRIGHT } else { STAR_DIM },
        );
    }

    for index in 0..5_i32 {
        let y = ((index * 79 + 31) as f32 + scroll * 0.38)
            .round() as i32
            % FRAMEBUFFER_HEIGHT as i32;
        let x = 22 + index * 34;
        framebuffer.fill_rect(x, y, 16, 1, Pixel::rgb(28, 30, 44));
        framebuffer.fill_rect(x + 4, y - 2, 8, 1, Pixel::rgb(38, 40, 54));
    }
}

fn render_pilgrim(framebuffer: &mut Framebuffer, x: i32, y: i32) {
    framebuffer.fill_rect(x - 5, y - 11, 11, 1, PILGRIM);
    framebuffer.fill_rect(x - 2, y - 9, 5, 3, PILGRIM);
    framebuffer.fill_rect(x - 3, y - 6, 7, 9, PILGRIM);
    framebuffer.fill_rect(x - 7, y - 3, 4, 5, PILGRIM);
    framebuffer.fill_rect(x + 4, y - 3, 4, 5, PILGRIM);
    framebuffer.fill_rect(x - 1, y - 3, 3, 4, PILGRIM_CORE);
    framebuffer.fill_rect(x - 3, y + 3, 2, 4, PILGRIM);
    framebuffer.fill_rect(x + 2, y + 3, 2, 4, PILGRIM);
}

fn render_carrion_drone(framebuffer: &mut Framebuffer, x: i32, y: i32) {
    framebuffer.fill_rect(x - 7, y - 4, 15, 8, ENEMY);
    framebuffer.fill_rect(x - 11, y - 2, 4, 3, ENEMY);
    framebuffer.fill_rect(x + 8, y - 2, 4, 3, ENEMY);
    framebuffer.fill_rect(x - 5, y - 7, 3, 3, ENEMY);
    framebuffer.fill_rect(x + 3, y - 7, 3, 3, ENEMY);
    framebuffer.fill_rect(x - 1, y - 1, 3, 2, ENEMY_EYE);
}

fn render_cinder(framebuffer: &mut Framebuffer, x: i32, y: i32) {
    framebuffer.fill_rect(x, y - 2, 1, 5, CINDER);
    framebuffer.fill_rect(x - 2, y, 5, 1, CINDER);
    framebuffer.fill_rect(x, y, 1, 1, Pixel::rgb(255, 245, 190));
}

fn render_burst(framebuffer: &mut Framebuffer, burst: Burst) {
    let x = burst.x.round() as i32;
    let y = burst.y.round() as i32;
    let radius = if burst.remaining > 0.22 {
        8
    } else if burst.remaining > 0.10 {
        5
    } else {
        3
    };
    let color = if burst.remaining > 0.20 { DANGER } else { CINDER };

    framebuffer.fill_rect(x - 1, y - 1, 3, 3, color);
    for (dx, dy) in [
        (-radius, 0),
        (radius, 0),
        (0, -radius),
        (0, radius),
        (-radius, -radius),
        (radius, -radius),
        (-radius, radius),
        (radius, radius),
    ] {
        framebuffer.fill_rect(x + dx, y + dy, 1, 1, color);
    }
}

fn render_hud(framebuffer: &mut Framebuffer, score: u32, lives: u32, core_charge: u32) {
    framebuffer.fill_rect(0, 0, FRAMEBUFFER_WIDTH, 26, Pixel::rgb(8, 8, 18));
    framebuffer.draw_text(5, 4, "GRAVE ORBIT", ACCENT);
    framebuffer.draw_text(5, 15, &format!("S {:05}", score), TEXT);
    framebuffer.draw_text(69, 15, &format!("L {}", lives), TEXT);
    framebuffer.draw_text(102, 15, "CORE", TEXT);
    framebuffer.fill_rect(134, 16, 40, 5, CORE_BG);
    framebuffer.fill_rect(134, 16, core_charge.min(100) * 40 / 100, 5, CINDER);
}

fn synthesize_chirp(start_hz: f32, end_hz: f32, duration: f32, volume: f32) -> Vec<u8> {
    let sample_count = (AUDIO_SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(sample_count);
    let mut phase = 0.0_f32;

    for index in 0..sample_count {
        let t = index as f32 / AUDIO_SAMPLE_RATE as f32;
        let progress = (t / duration).clamp(0.0, 1.0);
        let frequency = start_hz + (end_hz - start_hz) * progress;
        phase += frequency / AUDIO_SAMPLE_RATE as f32;
        let square = if phase.fract() < 0.5 { 1.0 } else { -1.0 };
        let envelope = (1.0 - progress).powi(2);
        samples.push((square * envelope * volume * i16::MAX as f32) as i16);
    }

    pcm16_mono_wav(AUDIO_SAMPLE_RATE, &samples)
        .expect("Void Canticle procedural audio should use supported PCM")
}

fn synthesize_noise_burst(duration: f32, volume: f32, mut seed: u32) -> Vec<u8> {
    let sample_count = (AUDIO_SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(sample_count);

    for index in 0..sample_count {
        let progress = index as f32 / sample_count.max(1) as f32;
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let noise = ((seed >> 8) as f32 / 16_777_215.0) * 2.0 - 1.0;
        let envelope = (1.0 - progress).powi(2);
        samples.push((noise * envelope * volume * i16::MAX as f32) as i16);
    }

    pcm16_mono_wav(AUDIO_SAMPLE_RATE, &samples)
        .expect("Void Canticle procedural audio should use supported PCM")
}

fn window_size_for(is_wsl: bool) -> (u32, u32) {
    if is_wsl {
        // WSLg/Weston is known to crash for some surface sizes. 960x612 is
        // documented as stable in docs/investigations/wslg-surface-present-stall.md.
        (960, 612)
    } else {
        (FRAMEBUFFER_WIDTH * 3, FRAMEBUFFER_HEIGHT * 3)
    }
}

fn window_size() -> (u32, u32) {
    window_size_for(std::env::var_os("WSL_DISTRO_NAME").is_some())
}

fn main() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();

    run(
        EngineConfig {
            title: "Void Canticle - Gotoo Pixel Engine".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width,
            window_height,
        },
        PauseGame::new(
            VoidCanticleGame::new(),
            PauseConfig::new(Size {
                width: FRAMEBUFFER_WIDTH,
                height: FRAMEBUFFER_HEIGHT,
            }),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curved_path_moves_away_from_base_x() {
        let start = curved_x(90.0, 0.0, 0.0, 30.0);
        let later = curved_x(90.0, 0.5, 0.0, 30.0);
        assert_ne!(start, later);
    }

    #[test]
    fn aimed_velocity_has_requested_speed_and_points_downward() {
        let (vx, vy) = aimed_velocity(20.0, 40.0, 90.0, 260.0, ENEMY_SHOT_SPEED);
        let speed = (vx * vx + vy * vy).sqrt();
        assert!((speed - ENEMY_SHOT_SPEED).abs() < 0.001);
        assert!(vy > 0.0);
    }

    #[test]
    fn projectile_hits_center_of_drone() {
        let enemy = CarrionDrone {
            base_x: 90.0,
            x: 90.0,
            y: 60.0,
            age: 0.0,
            phase: 0.0,
            curve_amplitude: 0.0,
            fire_timer: 1.0,
            alive: true,
        };
        let bullet = Bullet {
            x: 90.0,
            y: 60.0,
            vx: 0.0,
            vy: 0.0,
            alive: true,
        };
        assert!(bullet_hits_drone(bullet, enemy));
    }

    #[test]
    fn procedural_sounds_are_pcm_wav() {
        for wav in [
            synthesize_chirp(500.0, 900.0, 0.05, 0.2),
            synthesize_noise_burst(0.08, 0.2, 123),
        ] {
            assert_eq!(&wav[0..4], b"RIFF");
            assert_eq!(&wav[8..12], b"WAVE");
        }
    }

    #[test]
    fn wsl_uses_known_stable_surface_size() {
        assert_eq!(window_size_for(true), (960, 612));
        assert_eq!(
            window_size_for(false),
            (FRAMEBUFFER_WIDTH * 3, FRAMEBUFFER_HEIGHT * 3)
        );
    }
}
