use gotoo_pixel_engine::{
    ActionId, ControlMap, EngineConfig, EngineError, Frame, Framebuffer, Game, GameResult,
    GamepadButton, Key, MouseButton, Pixel, Size, SoundBank, SoundId, pcm16_mono_wav, run,
    ui::{PauseConfig, PauseGame},
};

const FRAMEBUFFER_WIDTH: u32 = 180;
const FRAMEBUFFER_HEIGHT: u32 = 320;

const MOVE_LEFT: ActionId = ActionId::new("void_canticle.left");
const MOVE_RIGHT: ActionId = ActionId::new("void_canticle.right");
const MOVE_UP: ActionId = ActionId::new("void_canticle.up");
const MOVE_DOWN: ActionId = ActionId::new("void_canticle.down");
const FIRE: ActionId = ActionId::new("void_canticle.fire");
const FOCUS: ActionId = ActionId::new("void_canticle.focus");
const CANTICLE: ActionId = ActionId::new("void_canticle.canticle");

const FIRE_SOUND: SoundId = SoundId::new("void_canticle.fire");
const ENEMY_HIT_SOUND: SoundId = SoundId::new("void_canticle.enemy_hit");
const ENEMY_FIRE_SOUND: SoundId = SoundId::new("void_canticle.enemy_fire");
const CINDER_SOUND: SoundId = SoundId::new("void_canticle.cinder");
const PLAYER_HIT_SOUND: SoundId = SoundId::new("void_canticle.player_hit");
const CANTICLE_SOUND: SoundId = SoundId::new("void_canticle.canticle");
const BELL_SOUND: SoundId = SoundId::new("void_canticle.bellkeeper_bell");
const AUDIO_SAMPLE_RATE: u32 = 44_100;

const BG: Pixel = Pixel::rgb(5, 6, 14);
const BG_CANTICLE: Pixel = Pixel::rgb(20, 13, 28);
const NEBULA: Pixel = Pixel::rgb(12, 14, 28);
const STAR_DIM: Pixel = Pixel::rgb(72, 78, 104);
const STAR_BRIGHT: Pixel = Pixel::rgb(180, 190, 220);
const WRECK_FAR: Pixel = Pixel::rgb(22, 26, 39);
const WRECK_MID: Pixel = Pixel::rgb(31, 34, 48);
const WRECK_NEAR: Pixel = Pixel::rgb(43, 43, 55);
const WRECK_LIGHT: Pixel = Pixel::rgb(76, 70, 72);
const PILGRIM: Pixel = Pixel::rgb(225, 222, 205);
const PILGRIM_CORE: Pixel = Pixel::rgb(245, 170, 70);
const FOCUS_COLOR: Pixel = Pixel::rgb(255, 230, 155);
const SHOT: Pixel = Pixel::rgb(245, 235, 180);
const ENEMY: Pixel = Pixel::rgb(150, 78, 95);
const ENEMY_EYE: Pixel = Pixel::rgb(255, 110, 70);
const ENEMY_SHOT: Pixel = Pixel::rgb(232, 80, 110);
const ENEMY_SHOT_ALT: Pixel = Pixel::rgb(180, 105, 225);
const CINDER: Pixel = Pixel::rgb(255, 195, 80);
const TEXT: Pixel = Pixel::rgb(205, 210, 225);
const ACCENT: Pixel = Pixel::rgb(195, 132, 74);
const DANGER: Pixel = Pixel::rgb(255, 75, 70);
const CORE_BG: Pixel = Pixel::rgb(42, 35, 42);
const CANTICLE_COLOR: Pixel = Pixel::rgb(255, 220, 150);
const BELL_METAL: Pixel = Pixel::rgb(126, 111, 92);
const BELL_LIGHT: Pixel = Pixel::rgb(222, 190, 126);
const BELL_DARK: Pixel = Pixel::rgb(70, 61, 58);

const PLAYER_SPEED: f32 = 118.0;
const FOCUS_SPEED: f32 = 54.0;
const PLAYER_SHOT_SPEED: f32 = 250.0;
const ENEMY_SHOT_SPEED: f32 = 68.0;
const FIRE_PERIOD: f32 = 0.09;
const PLAYER_INVULNERABILITY: f32 = 1.05;
const CINDER_CHARGE: u32 = 20;
const CORE_MAX: u32 = 100;
const CANTICLE_DURATION: f32 = 0.68;
const BOSS_INTRO_DURATION: f32 = 2.85;
const BELLKEEPER_MAX_HP: u32 = 120;
const CANTICLE_BOSS_DAMAGE: u32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShotPattern {
    Aimed,
    Fan3,
    Fan5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EncounterPhase {
    Waves,
    BossIntro,
    BossFight,
    Cleared,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BellPhase {
    Procession,
    Resonance,
    FinalToll,
}

#[derive(Debug, Clone, Copy)]
struct Bullet {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    alive: bool,
    alternate: bool,
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
    pattern: ShotPattern,
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
            pattern: spec.pattern,
            alive: true,
        }
    }

    fn update_position(&mut self, dt: f32) {
        self.age += dt;
        self.x = curved_x(self.base_x, self.age, self.phase, self.curve_amplitude);
        self.y = -10.0 + self.age * 42.0;
    }
}

#[derive(Debug, Clone, Copy)]
struct Bellkeeper {
    x: f32,
    y: f32,
    age: f32,
    shot_timer: f32,
    ring_rotation: f32,
    pattern_step: u32,
    hp: u32,
}

impl Bellkeeper {
    fn new() -> Self {
        Self {
            x: FRAMEBUFFER_WIDTH as f32 / 2.0,
            y: -32.0,
            age: 0.0,
            shot_timer: 0.95,
            ring_rotation: 0.0,
            pattern_step: 0,
            hp: BELLKEEPER_MAX_HP,
        }
    }

    fn phase(self) -> BellPhase {
        bell_phase_for_hp(self.hp)
    }
}

#[derive(Debug, Clone, Copy)]
struct CinderDrop {
    x: f32,
    y: f32,
    age: f32,
    phase: f32,
    alive: bool,
}

#[derive(Debug, Clone, Copy)]
struct Burst {
    x: f32,
    y: f32,
    remaining: f32,
    duration: f32,
    color: Pixel,
}

impl Burst {
    fn new(x: f32, y: f32, duration: f32, color: Pixel) -> Self {
        Self {
            x,
            y,
            remaining: duration,
            duration,
            color,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SpawnSpec {
    at: f32,
    base_x: f32,
    phase: f32,
    curve_amplitude: f32,
    first_shot: f32,
    pattern: ShotPattern,
}

const WAVE: &[SpawnSpec] = &[
    SpawnSpec { at: 0.45, base_x: 38.0, phase: 0.0, curve_amplitude: 22.0, first_shot: 2.05, pattern: ShotPattern::Aimed },
    SpawnSpec { at: 0.78, base_x: 38.0, phase: 0.5, curve_amplitude: 22.0, first_shot: 2.20, pattern: ShotPattern::Aimed },
    SpawnSpec { at: 1.11, base_x: 38.0, phase: 1.0, curve_amplitude: 22.0, first_shot: 2.35, pattern: ShotPattern::Aimed },
    SpawnSpec { at: 2.50, base_x: 142.0, phase: 3.2, curve_amplitude: -24.0, first_shot: 1.75, pattern: ShotPattern::Fan3 },
    SpawnSpec { at: 2.88, base_x: 142.0, phase: 3.7, curve_amplitude: -24.0, first_shot: 1.95, pattern: ShotPattern::Fan3 },
    SpawnSpec { at: 4.55, base_x: 90.0, phase: 0.0, curve_amplitude: 50.0, first_shot: 1.35, pattern: ShotPattern::Fan3 },
    SpawnSpec { at: 4.92, base_x: 90.0, phase: 1.6, curve_amplitude: 50.0, first_shot: 1.55, pattern: ShotPattern::Aimed },
    SpawnSpec { at: 5.29, base_x: 90.0, phase: 3.2, curve_amplitude: 50.0, first_shot: 1.75, pattern: ShotPattern::Fan3 },
    SpawnSpec { at: 7.25, base_x: 50.0, phase: 0.7, curve_amplitude: 31.0, first_shot: 1.15, pattern: ShotPattern::Fan5 },
    SpawnSpec { at: 7.58, base_x: 130.0, phase: 3.8, curve_amplitude: -31.0, first_shot: 1.15, pattern: ShotPattern::Fan5 },
];

struct VoidCanticleGame {
    controls: ControlMap,
    sounds: SoundBank,
    player_x: f32,
    player_y: f32,
    player_bullets: Vec<Bullet>,
    enemy_bullets: Vec<Bullet>,
    enemies: Vec<CarrionDrone>,
    boss: Option<Bellkeeper>,
    cinders: Vec<CinderDrop>,
    bursts: Vec<Burst>,
    fire_cooldown: f32,
    stage_time: f32,
    next_spawn: usize,
    encounter_phase: EncounterPhase,
    boss_intro_timer: f32,
    scroll: f32,
    score: u32,
    lives: u32,
    core_charge: u32,
    invulnerability: f32,
    canticle_timer: f32,
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
            .bind_gamepad(FIRE, GamepadButton::South)
            .bind_key(FOCUS, Key::LeftShift)
            .bind_gamepad(FOCUS, GamepadButton::LeftShoulder)
            .bind_key(CANTICLE, Key::X)
            .bind_gamepad(CANTICLE, GamepadButton::East);

        let mut sounds = SoundBank::new();
        for (id, wav) in [
            (FIRE_SOUND, synthesize_chirp(920.0, 520.0, 0.050, 0.12)),
            (ENEMY_HIT_SOUND, synthesize_noise_burst(0.105, 0.34, 0xC411_10A1)),
            (ENEMY_FIRE_SOUND, synthesize_chirp(260.0, 410.0, 0.070, 0.10)),
            (CINDER_SOUND, synthesize_chirp(730.0, 1120.0, 0.095, 0.22)),
            (PLAYER_HIT_SOUND, synthesize_noise_burst(0.240, 0.48, 0x5157_0001)),
            (CANTICLE_SOUND, synthesize_canticle_sound()),
            (BELL_SOUND, synthesize_bell_sound()),
        ] {
            sounds.insert_wav(id, wav).expect("Void Canticle sound ids should be unique");
        }

        let mut game = Self {
            controls,
            sounds,
            player_x: 0.0,
            player_y: 0.0,
            player_bullets: Vec::new(),
            enemy_bullets: Vec::new(),
            enemies: Vec::new(),
            boss: None,
            cinders: Vec::new(),
            bursts: Vec::new(),
            fire_cooldown: 0.0,
            stage_time: 0.0,
            next_spawn: 0,
            encounter_phase: EncounterPhase::Waves,
            boss_intro_timer: 0.0,
            scroll: 0.0,
            score: 0,
            lives: 3,
            core_charge: 0,
            invulnerability: 0.0,
            canticle_timer: 0.0,
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
        self.boss = None;
        self.cinders.clear();
        self.bursts.clear();
        self.fire_cooldown = 0.0;
        self.stage_time = 0.0;
        self.next_spawn = 0;
        self.encounter_phase = EncounterPhase::Waves;
        self.boss_intro_timer = 0.0;
        self.scroll = 0.0;
        self.score = 0;
        self.lives = 3;
        self.core_charge = 0;
        self.invulnerability = 0.0;
        self.canticle_timer = 0.0;
        self.game_over = false;
    }

    fn focus_held(&self, frame: &Frame<'_>) -> bool {
        self.controls.action(FOCUS).held() || frame.input.mouse_button(MouseButton::Right).held()
    }

    fn canticle_pressed(&self, frame: &Frame<'_>) -> bool {
        self.controls.action(CANTICLE).pressed() || frame.input.mouse_button(MouseButton::Left).pressed()
    }

    fn update_player(&mut self, dt: f32, focused: bool) {
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

        let speed = if focused { FOCUS_SPEED } else { PLAYER_SPEED };
        self.player_x = (self.player_x + dx * speed * dt).clamp(8.0, FRAMEBUFFER_WIDTH as f32 - 8.0);
        self.player_y = (self.player_y + dy * speed * dt).clamp(30.0, FRAMEBUFFER_HEIGHT as f32 - 16.0);
    }

    fn update_player_fire(&mut self, dt: f32, frame: &mut Frame<'_>) {
        self.fire_cooldown = (self.fire_cooldown - dt).max(0.0);
        if self.controls.action(FIRE).held() && self.fire_cooldown <= 0.0 {
            for offset in [-2.0_f32, 2.0] {
                self.player_bullets.push(Bullet {
                    x: self.player_x + offset,
                    y: self.player_y - 9.0,
                    vx: 0.0,
                    vy: -PLAYER_SHOT_SPEED,
                    alive: true,
                    alternate: false,
                });
            }
            self.fire_cooldown = FIRE_PERIOD;
            let _ = self.sounds.play(frame.audio, FIRE_SOUND);
        }
    }

    fn update_encounter(&mut self, dt: f32, frame: &mut Frame<'_>) {
        match self.encounter_phase {
            EncounterPhase::Waves => {
                self.stage_time += dt;
                while self.next_spawn < WAVE.len() && WAVE[self.next_spawn].at <= self.stage_time {
                    self.enemies.push(CarrionDrone::new(WAVE[self.next_spawn]));
                    self.next_spawn += 1;
                }
                if self.next_spawn == WAVE.len() && self.stage_time >= 10.4 && self.enemies.is_empty() {
                    self.begin_boss_intro(frame);
                }
            }
            EncounterPhase::BossIntro => {
                self.boss_intro_timer = (self.boss_intro_timer - dt).max(0.0);
                if let Some(boss) = self.boss.as_mut() {
                    boss.y = (boss.y + 34.0 * dt).min(64.0);
                }
                if self.boss_intro_timer <= 0.0 {
                    self.encounter_phase = EncounterPhase::BossFight;
                }
            }
            EncounterPhase::BossFight => self.update_boss(dt, frame),
            EncounterPhase::Cleared => {}
        }
    }

    fn begin_boss_intro(&mut self, frame: &mut Frame<'_>) {
        self.encounter_phase = EncounterPhase::BossIntro;
        self.boss_intro_timer = BOSS_INTRO_DURATION;
        self.enemy_bullets.clear();
        self.boss = Some(Bellkeeper::new());
        let _ = self.sounds.play(frame.audio, BELL_SOUND);
    }

    fn update_boss(&mut self, dt: f32, frame: &mut Frame<'_>) {
        let mut spawned = Vec::new();
        let mut tolled = false;
        if let Some(boss) = self.boss.as_mut() {
            boss.age += dt;
            boss.x = FRAMEBUFFER_WIDTH as f32 / 2.0 + (boss.age * 0.72).sin() * 42.0;
            boss.y = 64.0 + (boss.age * 1.25).sin() * 5.0;
            boss.shot_timer -= dt;
            if boss.shot_timer <= 0.0 {
                match boss.phase() {
                    BellPhase::Procession => {
                        if boss.pattern_step % 2 == 0 {
                            spawn_ring(&mut spawned, boss.x, boss.y + 12.0, 12, boss.ring_rotation, 54.0);
                        } else {
                            spawn_fan(&mut spawned, boss.x, boss.y + 12.0, self.player_x, self.player_y, 7, 0.86, 62.0);
                        }
                        boss.ring_rotation += 0.24;
                        boss.shot_timer += 1.08;
                    }
                    BellPhase::Resonance => {
                        spawn_ring(&mut spawned, boss.x, boss.y + 12.0, 14, boss.ring_rotation, 59.0);
                        spawn_ring(&mut spawned, boss.x, boss.y + 12.0, 14, boss.ring_rotation + 0.19, 42.0);
                        if boss.pattern_step % 3 == 2 {
                            spawn_fan(&mut spawned, boss.x, boss.y + 12.0, self.player_x, self.player_y, 5, 0.56, 72.0);
                        }
                        boss.ring_rotation += 0.31;
                        boss.shot_timer += 0.92;
                    }
                    BellPhase::FinalToll => {
                        spawn_ring(&mut spawned, boss.x, boss.y + 12.0, 16, boss.ring_rotation, 67.0);
                        spawn_ring(&mut spawned, boss.x, boss.y + 12.0, 16, boss.ring_rotation + 0.28, 49.0);
                        spawn_fan(&mut spawned, boss.x, boss.y + 12.0, self.player_x, self.player_y, 7, 0.78, 76.0);
                        boss.ring_rotation += 0.39;
                        boss.shot_timer += 0.78;
                    }
                }
                boss.pattern_step = boss.pattern_step.wrapping_add(1);
                tolled = true;
            }
        }
        if !spawned.is_empty() { self.enemy_bullets.extend(spawned); }
        if tolled { let _ = self.sounds.play(frame.audio, BELL_SOUND); }
    }

    fn update_enemies(&mut self, dt: f32, frame: &mut Frame<'_>) {
        let mut spawned_shots = Vec::new();
        for enemy in &mut self.enemies {
            enemy.update_position(dt);
            enemy.fire_timer -= dt;
            if enemy.fire_timer <= 0.0 && (42.0..225.0).contains(&enemy.y) {
                spawn_pattern(&mut spawned_shots, enemy.pattern, enemy.x, enemy.y + 6.0, self.player_x, self.player_y);
                enemy.fire_timer += match enemy.pattern { ShotPattern::Aimed => 2.15, ShotPattern::Fan3 => 2.55, ShotPattern::Fan5 => 3.20 };
            }
            if enemy.y > FRAMEBUFFER_HEIGHT as f32 + 18.0 { enemy.alive = false; }
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
            bullet.alive = bullet.x > -8.0 && bullet.x < FRAMEBUFFER_WIDTH as f32 + 8.0 && bullet.y > 18.0 && bullet.y < FRAMEBUFFER_HEIGHT as f32 + 8.0;
        }
        self.enemy_bullets.retain(|bullet| bullet.alive);
    }

    fn resolve_player_shots(&mut self, frame: &mut Frame<'_>) {
        let mut destroyed = Vec::new();
        let mut boss_hits = 0_u32;
        for bullet in &mut self.player_bullets {
            if !bullet.alive { continue; }
            for enemy in &mut self.enemies {
                if enemy.alive && point_near(bullet.x, bullet.y, enemy.x, enemy.y, 9.0) {
                    bullet.alive = false;
                    enemy.alive = false;
                    destroyed.push((enemy.x, enemy.y));
                    break;
                }
            }
            if bullet.alive {
                if let Some(boss) = self.boss.as_mut() {
                    if self.encounter_phase == EncounterPhase::BossFight && point_near(bullet.x, bullet.y, boss.x, boss.y, 20.0) {
                        bullet.alive = false;
                        boss.hp = boss.hp.saturating_sub(1);
                        boss_hits += 1;
                    }
                }
            }
        }
        self.player_bullets.retain(|bullet| bullet.alive);
        self.enemies.retain(|enemy| enemy.alive);
        for (x, y) in destroyed {
            self.score = self.score.saturating_add(100);
            self.cinders.push(CinderDrop { x, y, age: 0.0, phase: x * 0.11 + y * 0.07, alive: true });
            self.bursts.push(Burst::new(x, y, 0.22, ENEMY_EYE));
            let _ = self.sounds.play(frame.audio, ENEMY_HIT_SOUND);
        }
        if boss_hits > 0 {
            self.score = self.score.saturating_add(boss_hits * 5);
            if let Some(boss) = self.boss { self.bursts.push(Burst::new(boss.x, boss.y + 4.0, 0.10, BELL_LIGHT)); }
        }
        self.finish_boss_if_destroyed(frame);
    }

    fn update_cinders(&mut self, dt: f32, frame: &mut Frame<'_>) {
        let player_x = self.player_x;
        let player_y = self.player_y;
        let mut collected = 0_u32;
        for cinder in &mut self.cinders {
            cinder.age += dt;
            cinder.y += 24.0 * dt;
            cinder.x += (cinder.age * 4.2 + cinder.phase).sin() * 10.0 * dt;
            if point_near(cinder.x, cinder.y, player_x, player_y, 10.0) {
                cinder.alive = false;
                collected += 1;
            } else if cinder.y > FRAMEBUFFER_HEIGHT as f32 + 8.0 { cinder.alive = false; }
        }
        self.cinders.retain(|cinder| cinder.alive);
        if collected > 0 {
            self.core_charge = (self.core_charge + collected * CINDER_CHARGE).min(CORE_MAX);
            self.score = self.score.saturating_add(collected * 25);
            let _ = self.sounds.play(frame.audio, CINDER_SOUND);
        }
    }

    fn trigger_canticle(&mut self, frame: &mut Frame<'_>) {
        if self.core_charge < CORE_MAX || self.canticle_timer > 0.0 { return; }
        self.core_charge = 0;
        self.canticle_timer = CANTICLE_DURATION;
        self.invulnerability = self.invulnerability.max(0.72);
        let erased_bullets = self.enemy_bullets.len() as u32;
        self.enemy_bullets.clear();
        self.score = self.score.saturating_add(erased_bullets * 2);
        let mut destroyed = Vec::new();
        for enemy in &mut self.enemies {
            if enemy.alive && enemy.y >= 18.0 && enemy.y < FRAMEBUFFER_HEIGHT as f32 {
                enemy.alive = false;
                destroyed.push((enemy.x, enemy.y));
            }
        }
        self.enemies.retain(|enemy| enemy.alive);
        self.score = self.score.saturating_add((destroyed.len() as u32).saturating_mul(50));
        for (x, y) in destroyed { self.bursts.push(Burst::new(x, y, 0.38, CANTICLE_COLOR)); }
        if self.encounter_phase == EncounterPhase::BossFight {
            if let Some(boss) = self.boss.as_mut() {
                boss.hp = boss.hp.saturating_sub(CANTICLE_BOSS_DAMAGE);
                self.bursts.push(Burst::new(boss.x, boss.y, 0.52, CANTICLE_COLOR));
            }
        }
        self.bursts.push(Burst::new(self.player_x, self.player_y, CANTICLE_DURATION, CANTICLE_COLOR));
        let _ = self.sounds.play(frame.audio, CANTICLE_SOUND);
        self.finish_boss_if_destroyed(frame);
    }

    fn finish_boss_if_destroyed(&mut self, frame: &mut Frame<'_>) {
        let destroyed = self.boss.is_some_and(|boss| boss.hp == 0);
        if !destroyed || self.encounter_phase == EncounterPhase::Cleared { return; }
        if let Some(boss) = self.boss {
            for (dx, dy) in [(-22.0, -10.0), (22.0, -10.0), (-15.0, 12.0), (15.0, 12.0), (0.0, -4.0), (0.0, 14.0)] {
                self.bursts.push(Burst::new(boss.x + dx, boss.y + dy, 0.66, CANTICLE_COLOR));
            }
        }
        self.enemy_bullets.clear();
        self.score = self.score.saturating_add(5_000);
        self.encounter_phase = EncounterPhase::Cleared;
        let _ = self.sounds.play(frame.audio, CANTICLE_SOUND);
    }

    fn resolve_player_hits(&mut self, frame: &mut Frame<'_>) {
        if self.invulnerability > 0.0 || self.game_over { return; }
        let bullet_hit = self.enemy_bullets.iter().any(|bullet| point_near(bullet.x, bullet.y, self.player_x, self.player_y, 4.0));
        let body_hit = self.enemies.iter().any(|enemy| point_near(enemy.x, enemy.y, self.player_x, self.player_y, 8.0));
        if !bullet_hit && !body_hit { return; }
        self.lives = self.lives.saturating_sub(1);
        self.invulnerability = PLAYER_INVULNERABILITY;
        self.enemy_bullets.clear();
        self.bursts.push(Burst::new(self.player_x, self.player_y, 0.42, DANGER));
        let _ = self.sounds.play(frame.audio, PLAYER_HIT_SOUND);
        if self.lives == 0 { self.game_over = true; }
    }

    fn update_feedback(&mut self, dt: f32) {
        self.invulnerability = (self.invulnerability - dt).max(0.0);
        self.canticle_timer = (self.canticle_timer - dt).max(0.0);
        for burst in &mut self.bursts { burst.remaining = (burst.remaining - dt).max(0.0); }
        self.bursts.retain(|burst| burst.remaining > 0.0);
    }

    fn render(&self, framebuffer: &mut Framebuffer, focused: bool) {
        framebuffer.clear(if self.canticle_timer > 0.46 { BG_CANTICLE } else { BG });
        render_grave_orbit_background(framebuffer, self.scroll);
        for cinder in &self.cinders {
            let x = cinder.x.round() as i32;
            let y = cinder.y.round() as i32;
            framebuffer.fill_circle(x, y, 2, CINDER);
            framebuffer.draw(x, y - 4, CANTICLE_COLOR);
        }
        for enemy in &self.enemies { render_carrion_drone(framebuffer, *enemy); }
        if let Some(boss) = self.boss {
            if self.encounter_phase != EncounterPhase::Cleared { render_bellkeeper(framebuffer, boss); }
        }
        for bullet in &self.player_bullets {
            framebuffer.fill_rect(bullet.x.round() as i32 - 1, bullet.y.round() as i32 - 4, 2, 8, SHOT);
        }
        for bullet in &self.enemy_bullets {
            let color = if bullet.alternate { ENEMY_SHOT_ALT } else { ENEMY_SHOT };
            framebuffer.fill_circle(bullet.x.round() as i32, bullet.y.round() as i32, 2, color);
        }
        for burst in &self.bursts { render_burst(framebuffer, *burst); }
        render_pilgrim(framebuffer, self.player_x.round() as i32, self.player_y.round() as i32, focused, self.invulnerability);
        if self.canticle_timer > 0.0 {
            render_canticle(framebuffer, self.player_x.round() as i32, self.player_y.round() as i32, self.canticle_timer);
        }
        self.render_hud(framebuffer);
        self.render_encounter_overlay(framebuffer);
        if self.game_over {
            framebuffer.fill_rect(20, 132, 140, 48, Pixel::rgb(12, 10, 18));
            framebuffer.draw_rect(20, 132, 140, 48, DANGER);
            framebuffer.draw_text(41, 143, "PILGRIM FALLEN", DANGER);
            framebuffer.draw_text(37, 160, "SPACE TO RETURN", TEXT);
        }
    }

    fn render_hud(&self, framebuffer: &mut Framebuffer) {
        framebuffer.draw_text(4, 4, "VOID CANTICLE", ACCENT);
        framebuffer.draw_text(4, 15, "GRAVE ORBIT / VC0.5", TEXT);
        framebuffer.draw_text(4, 27, &format!("LIVES {}", self.lives), TEXT);
        framebuffer.draw_text(126, 4, &format!("{}", self.score), PILGRIM_CORE);
        if let Some(boss) = self.boss {
            if self.encounter_phase == EncounterPhase::BossFight {
                framebuffer.draw_text(4, 39, "BELLKEEPER", BELL_LIGHT);
                framebuffer.fill_rect(66, 40, 106, 5, CORE_BG);
                let width = 106_u32.saturating_mul(boss.hp) / BELLKEEPER_MAX_HP;
                framebuffer.fill_rect(66, 40, width, 5, DANGER);
                let phase = match boss.phase() { BellPhase::Procession => "TOLL I", BellPhase::Resonance => "TOLL II", BellPhase::FinalToll => "FINAL TOLL" };
                framebuffer.draw_text(4, 49, phase, WRECK_LIGHT);
            }
        }
        framebuffer.draw_text(4, 279, "CORE", TEXT);
        framebuffer.fill_rect(34, 280, 138, 6, CORE_BG);
        let charge_width = 138_u32.saturating_mul(self.core_charge) / CORE_MAX;
        framebuffer.fill_rect(34, 280, charge_width, 6, CINDER);
        if self.core_charge >= CORE_MAX { framebuffer.draw_text(112, 269, "READY", CANTICLE_COLOR); }
        framebuffer.draw_text(4, 294, "SPACE FIRE", TEXT);
        framebuffer.draw_text(76, 294, "SHIFT FOCUS", TEXT);
        framebuffer.draw_text(4, 305, "X CANTICLE", TEXT);
    }

    fn render_encounter_overlay(&self, framebuffer: &mut Framebuffer) {
        match self.encounter_phase {
            EncounterPhase::BossIntro => {
                let pulse = ((self.boss_intro_timer * 7.0) as i32 & 1) == 0;
                let border = if pulse { BELL_LIGHT } else { BELL_METAL };
                framebuffer.draw_line(0, 118, FRAMEBUFFER_WIDTH as i32 - 1, 118, border);
                framebuffer.draw_line(0, 194, FRAMEBUFFER_WIDTH as i32 - 1, 194, border);
                framebuffer.fill_rect(13, 132, 154, 52, Pixel::rgb(10, 8, 14));
                framebuffer.draw_rect(13, 132, 154, 52, border);
                framebuffer.draw_text(55, 142, "WARNING", DANGER);
                framebuffer.draw_text(31, 159, "THE BELLKEEPER", BELL_LIGHT);
                framebuffer.draw_text(45, 171, "TOLLS FOR YOU", TEXT);
            }
            EncounterPhase::Cleared => {
                framebuffer.fill_rect(12, 136, 156, 44, Pixel::rgb(10, 8, 14));
                framebuffer.draw_rect(12, 136, 156, 44, CANTICLE_COLOR);
                framebuffer.draw_text(28, 147, "GRAVE ORBIT CLEARED", CANTICLE_COLOR);
                framebuffer.draw_text(48, 164, "THE PATH OPENS", TEXT);
            }
            EncounterPhase::Waves | EncounterPhase::BossFight => {}
        }
    }
}

impl Game for VoidCanticleGame {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.controls.update(frame.input);
        if self.game_over {
            if self.controls.action(FIRE).pressed() { self.reset_run(); }
            self.render(frame.framebuffer, false);
            return GameResult::Continue;
        }
        let dt = frame.delta_time.as_secs_f32().min(0.05);
        let focused = self.focus_held(frame);
        let canticle_pressed = self.canticle_pressed(frame);
        self.scroll = (self.scroll + 34.0 * dt) % FRAMEBUFFER_HEIGHT as f32;
        self.update_feedback(dt);
        self.update_player(dt, focused);
        self.update_player_fire(dt, frame);
        self.update_encounter(dt, frame);
        self.update_enemies(dt, frame);
        self.update_projectiles(dt);
        self.resolve_player_shots(frame);
        self.update_cinders(dt, frame);
        if canticle_pressed { self.trigger_canticle(frame); }
        self.resolve_player_hits(frame);
        self.render(frame.framebuffer, focused);
        GameResult::Continue
    }
}

fn bell_phase_for_hp(hp: u32) -> BellPhase {
    if hp > BELLKEEPER_MAX_HP * 2 / 3 { BellPhase::Procession }
    else if hp > BELLKEEPER_MAX_HP / 3 { BellPhase::Resonance }
    else { BellPhase::FinalToll }
}

fn spawn_pattern(output: &mut Vec<Bullet>, pattern: ShotPattern, x: f32, y: f32, target_x: f32, target_y: f32) {
    let (count, spread) = match pattern { ShotPattern::Aimed => (1, 0.0), ShotPattern::Fan3 => (3, 0.44), ShotPattern::Fan5 => (5, 0.76) };
    spawn_fan(output, x, y, target_x, target_y, count, spread, ENEMY_SHOT_SPEED);
}

fn spawn_fan(output: &mut Vec<Bullet>, x: f32, y: f32, target_x: f32, target_y: f32, count: usize, spread: f32, speed: f32) {
    let base_angle = (target_y - y).atan2(target_x - x);
    let center = (count.saturating_sub(1)) as f32 / 2.0;
    let step = if count > 1 { spread / (count - 1) as f32 } else { 0.0 };
    for index in 0..count {
        let offset = (index as f32 - center) * step;
        let angle = base_angle + offset;
        output.push(Bullet { x, y, vx: angle.cos() * speed, vy: angle.sin() * speed, alive: true, alternate: index % 2 == 1 });
    }
}

fn spawn_ring(output: &mut Vec<Bullet>, x: f32, y: f32, count: usize, rotation: f32, speed: f32) {
    for index in 0..count {
        let angle = rotation + index as f32 * std::f32::consts::TAU / count as f32;
        output.push(Bullet { x, y, vx: angle.cos() * speed, vy: angle.sin() * speed, alive: true, alternate: index % 2 == 1 });
    }
}

fn curved_x(base_x: f32, age: f32, phase: f32, amplitude: f32) -> f32 {
    (base_x + (age * 2.05 + phase).sin() * amplitude).clamp(10.0, FRAMEBUFFER_WIDTH as f32 - 10.0)
}

fn point_near(ax: f32, ay: f32, bx: f32, by: f32, radius: f32) -> bool {
    let dx = ax - bx;
    let dy = ay - by;
    dx * dx + dy * dy <= radius * radius
}

fn wrapped_y(base: f32, scroll: f32, speed: f32, period: f32) -> i32 {
    ((base + scroll * speed).rem_euclid(period) - 48.0).round() as i32
}

fn render_grave_orbit_background(framebuffer: &mut Framebuffer, scroll: f32) {
    render_nebula(framebuffer, scroll);
    render_far_wrecks(framebuffer, scroll);
    render_starfield(framebuffer, scroll);
    render_mid_wrecks(framebuffer, scroll);
    render_near_wrecks(framebuffer, scroll);
}

fn render_nebula(framebuffer: &mut Framebuffer, scroll: f32) {
    let y0 = wrapped_y(35.0, scroll, 0.16, 416.0);
    let y1 = wrapped_y(230.0, scroll, 0.13, 416.0);
    framebuffer.fill_circle(-18, y0, 74, NEBULA);
    framebuffer.fill_circle(194, y1, 82, NEBULA);
    framebuffer.draw_circle(85, wrapped_y(145.0, scroll, 0.09, 416.0), 58, WRECK_FAR);
}

fn render_far_wrecks(framebuffer: &mut Framebuffer, scroll: f32) {
    render_cross_satellite(framebuffer, 28, wrapped_y(82.0, scroll, 0.24, 408.0), WRECK_FAR);
    let ring_y = wrapped_y(274.0, scroll, 0.20, 408.0);
    framebuffer.draw_circle(146, ring_y, 35, WRECK_FAR);
    framebuffer.draw_circle(146, ring_y, 29, WRECK_FAR);
    framebuffer.draw_line(111, ring_y, 128, ring_y - 8, WRECK_FAR);
}

fn render_mid_wrecks(framebuffer: &mut Framebuffer, scroll: f32) {
    render_broken_hull(framebuffer, 138, wrapped_y(168.0, scroll, 0.46, 430.0), WRECK_MID);
    render_reliquary_wreck(framebuffer, 49, wrapped_y(360.0, scroll, 0.39, 430.0), WRECK_MID);
}

fn render_near_wrecks(framebuffer: &mut Framebuffer, scroll: f32) {
    render_broken_hull(framebuffer, 12, wrapped_y(42.0, scroll, 0.78, 470.0), WRECK_NEAR);
    let mast_y = wrapped_y(275.0, scroll, 0.70, 470.0);
    framebuffer.fill_rect(160, mast_y - 32, 5, 65, WRECK_NEAR);
    framebuffer.fill_rect(147, mast_y - 3, 30, 5, WRECK_NEAR);
    framebuffer.draw_line(162, mast_y - 32, 171, mast_y - 43, WRECK_LIGHT);
}

fn render_cross_satellite(framebuffer: &mut Framebuffer, x: i32, y: i32, color: Pixel) {
    framebuffer.fill_rect(x - 2, y - 17, 5, 35, color);
    framebuffer.fill_rect(x - 17, y - 2, 35, 5, color);
    framebuffer.draw_rect(x - 6, y - 6, 13, 13, color);
    framebuffer.draw_line(x - 17, y, x - 25, y + 7, color);
    framebuffer.draw_line(x + 17, y, x + 24, y - 8, color);
}

fn render_broken_hull(framebuffer: &mut Framebuffer, x: i32, y: i32, color: Pixel) {
    framebuffer.fill_rect(x - 24, y - 7, 43, 15, color);
    framebuffer.fill_rect(x - 13, y - 13, 25, 7, color);
    framebuffer.draw_line(x + 18, y - 7, x + 31, y - 15, color);
    framebuffer.draw_line(x + 18, y + 7, x + 29, y + 16, color);
    framebuffer.draw_line(x - 24, y - 7, x - 32, y - 14, color);
    framebuffer.fill_rect(x - 7, y - 3, 4, 4, BG);
    framebuffer.fill_rect(x + 5, y + 2, 6, 3, BG);
}

fn render_reliquary_wreck(framebuffer: &mut Framebuffer, x: i32, y: i32, color: Pixel) {
    framebuffer.draw_rect(x - 12, y - 20, 25, 41, color);
    framebuffer.draw_line(x - 12, y - 20, x, y - 32, color);
    framebuffer.draw_line(x + 12, y - 20, x, y - 32, color);
    framebuffer.fill_rect(x - 2, y - 28, 5, 17, color);
    framebuffer.draw_circle(x, y, 7, color);
    framebuffer.draw_line(x, y + 20, x, y + 32, color);
}

fn render_starfield(framebuffer: &mut Framebuffer, scroll: f32) {
    for index in 0..46_i32 {
        let x = (index * 47 + 19) % FRAMEBUFFER_WIDTH as i32;
        let base_y = (index * 83 + 11) % FRAMEBUFFER_HEIGHT as i32;
        let speed = if index % 3 == 0 { 0.52 } else { 1.0 };
        let y = (base_y + (scroll * speed).round() as i32).rem_euclid(FRAMEBUFFER_HEIGHT as i32);
        let bright = index % 7 == 0;
        framebuffer.fill_rect(x, y, if bright { 2 } else { 1 }, if bright { 2 } else { 1 }, if bright { STAR_BRIGHT } else { STAR_DIM });
    }
}

fn render_pilgrim(framebuffer: &mut Framebuffer, x: i32, y: i32, focused: bool, invulnerability: f32) {
    let visible = invulnerability <= 0.0 || ((invulnerability * 16.0) as i32 % 2 == 0);
    if !visible { return; }
    framebuffer.fill_rect(x - 4, y - 10, 9, 1, PILGRIM);
    framebuffer.fill_rect(x - 2, y - 8, 5, 3, PILGRIM);
    framebuffer.fill_rect(x - 3, y - 5, 7, 8, PILGRIM);
    framebuffer.fill_rect(x - 6, y - 3, 3, 5, PILGRIM);
    framebuffer.fill_rect(x + 4, y - 3, 3, 5, PILGRIM);
    framebuffer.fill_rect(x - 1, y - 3, 3, 4, PILGRIM_CORE);
    framebuffer.fill_rect(x - 3, y + 3, 2, 4, PILGRIM);
    framebuffer.fill_rect(x + 2, y + 3, 2, 4, PILGRIM);
    if focused {
        framebuffer.draw_circle(x, y - 1, 4, FOCUS_COLOR);
        framebuffer.fill_circle(x, y - 1, 1, FOCUS_COLOR);
    }
}

fn render_carrion_drone(framebuffer: &mut Framebuffer, enemy: CarrionDrone) {
    let x = enemy.x.round() as i32;
    let y = enemy.y.round() as i32;
    framebuffer.fill_rect(x - 6, y - 3, 13, 6, ENEMY);
    framebuffer.draw_line(x - 6, y - 2, x - 12, y - 7, ENEMY);
    framebuffer.draw_line(x + 6, y - 2, x + 12, y - 7, ENEMY);
    framebuffer.draw_line(x - 6, y + 2, x - 11, y + 6, ENEMY);
    framebuffer.draw_line(x + 6, y + 2, x + 11, y + 6, ENEMY);
    framebuffer.fill_rect(x - 1, y - 1, 3, 3, ENEMY_EYE);
}

fn render_bellkeeper(framebuffer: &mut Framebuffer, boss: Bellkeeper) {
    let x = boss.x.round() as i32;
    let y = boss.y.round() as i32;
    let pulse = ((boss.age * 5.0) as i32 & 1) == 0;
    let glow = if pulse { BELL_LIGHT } else { BELL_METAL };
    let clapper = y + 14 + (boss.age * 5.0).sin().round() as i32 * 2;
    framebuffer.draw_circle(x, y - 2, 28, BELL_DARK);
    framebuffer.draw_line(x - 22, y - 8, x - 34, y - 17, BELL_METAL);
    framebuffer.draw_line(x + 22, y - 8, x + 34, y - 17, BELL_METAL);
    framebuffer.draw_line(x - 34, y - 17, x - 38, y + 3, BELL_METAL);
    framebuffer.draw_line(x + 34, y - 17, x + 38, y + 3, BELL_METAL);
    framebuffer.draw_circle(x - 38, y + 5, 4, glow);
    framebuffer.draw_circle(x + 38, y + 5, 4, glow);
    framebuffer.fill_rect(x - 12, y - 17, 25, 5, BELL_METAL);
    framebuffer.fill_rect(x - 16, y - 12, 33, 18, BELL_METAL);
    framebuffer.fill_rect(x - 20, y + 6, 41, 6, glow);
    framebuffer.draw_rect(x - 16, y - 12, 33, 18, BELL_LIGHT);
    framebuffer.fill_rect(x - 2, clapper, 5, 8, DANGER);
    framebuffer.fill_circle(x, y - 4, 3, ENEMY_EYE);
    match boss.phase() {
        BellPhase::Procession => {}
        BellPhase::Resonance => framebuffer.draw_circle(x, y - 3, 23, BELL_LIGHT),
        BellPhase::FinalToll => {
            framebuffer.draw_circle(x, y - 3, 23, DANGER);
            framebuffer.draw_circle(x, y - 3, 33, BELL_LIGHT);
        }
    }
}

fn render_burst(framebuffer: &mut Framebuffer, burst: Burst) {
    let progress = 1.0 - burst.remaining / burst.duration;
    let radius = (2.0 + progress * 11.0).round() as i32;
    let x = burst.x.round() as i32;
    let y = burst.y.round() as i32;
    let points = [(-radius, 0), (radius, 0), (0, -radius), (0, radius), (-radius, -radius), (radius, -radius), (-radius, radius), (radius, radius)];
    framebuffer.fill_rect(x - 1, y - 1, 3, 3, burst.color);
    for (dx, dy) in points { framebuffer.fill_rect(x + dx, y + dy, 2, 2, burst.color); }
}

fn render_canticle(framebuffer: &mut Framebuffer, x: i32, y: i32, remaining: f32) {
    let progress = 1.0 - remaining / CANTICLE_DURATION;
    let radius = (8.0 + progress * 190.0).round() as u32;
    framebuffer.draw_circle(x, y, radius, CANTICLE_COLOR);
    framebuffer.draw_circle(x, y, radius.saturating_add(5), PILGRIM_CORE);
    if progress < 0.30 { framebuffer.fill_circle(x, y, 7, CANTICLE_COLOR); }
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
    pcm16_mono_wav(AUDIO_SAMPLE_RATE, &samples).expect("Void Canticle procedural audio should use supported PCM")
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
    pcm16_mono_wav(AUDIO_SAMPLE_RATE, &samples).expect("Void Canticle noise should use supported PCM")
}

fn synthesize_canticle_sound() -> Vec<u8> {
    let duration = 0.62_f32;
    let sample_count = (AUDIO_SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(sample_count);
    let mut low_phase = 0.0_f32;
    let mut high_phase = 0.0_f32;
    for index in 0..sample_count {
        let progress = index as f32 / sample_count.max(1) as f32;
        low_phase += (72.0 + progress * 54.0) / AUDIO_SAMPLE_RATE as f32;
        high_phase += (420.0 + progress * 740.0) / AUDIO_SAMPLE_RATE as f32;
        let low = (low_phase * std::f32::consts::TAU).sin();
        let high = if high_phase.fract() < 0.5 { 1.0 } else { -1.0 };
        let attack = (progress / 0.08).min(1.0);
        let release = (1.0 - progress).powf(0.55);
        let sample = (low * 0.70 + high * 0.30) * attack * release * 0.52;
        samples.push((sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16);
    }
    pcm16_mono_wav(AUDIO_SAMPLE_RATE, &samples).expect("Void Canticle Canticle audio should use supported PCM")
}

fn synthesize_bell_sound() -> Vec<u8> {
    let duration = 0.78_f32;
    let sample_count = (AUDIO_SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(sample_count);
    for index in 0..sample_count {
        let t = index as f32 / AUDIO_SAMPLE_RATE as f32;
        let progress = (t / duration).clamp(0.0, 1.0);
        let fundamental = (std::f32::consts::TAU * 108.0 * t).sin();
        let partial = (std::f32::consts::TAU * 271.0 * t).sin() * 0.46;
        let high = (std::f32::consts::TAU * 431.0 * t).sin() * 0.20;
        let strike = (1.0 - progress).powf(2.4);
        let tail = (-4.2 * progress).exp();
        let sample = (fundamental + partial + high) * strike.max(tail * 0.55) * 0.40;
        samples.push((sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16);
    }
    pcm16_mono_wav(AUDIO_SAMPLE_RATE, &samples).expect("Void Canticle bell audio should use supported PCM")
}

fn window_size_for(is_wsl: bool) -> (u32, u32) {
    if is_wsl { (960, 612) } else { (FRAMEBUFFER_WIDTH * 3, FRAMEBUFFER_HEIGHT * 3) }
}

fn window_size() -> (u32, u32) {
    window_size_for(std::env::var_os("WSL_DISTRO_NAME").is_some())
}

fn main() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig { title: "Void Canticle - Gotoo Pixel Engine".into(), framebuffer_width: FRAMEBUFFER_WIDTH, framebuffer_height: FRAMEBUFFER_HEIGHT, window_width, window_height },
        PauseGame::new(VoidCanticleGame::new(), PauseConfig::new(Size { width: FRAMEBUFFER_WIDTH, height: FRAMEBUFFER_HEIGHT })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aimed_pattern_points_toward_target() {
        let mut bullets = Vec::new();
        spawn_pattern(&mut bullets, ShotPattern::Aimed, 50.0, 50.0, 50.0, 150.0);
        assert_eq!(bullets.len(), 1);
        assert!(bullets[0].vy > 0.0);
        assert!(bullets[0].vx.abs() < 0.001);
    }

    #[test]
    fn fan_patterns_have_expected_density() {
        let mut three = Vec::new();
        let mut five = Vec::new();
        spawn_pattern(&mut three, ShotPattern::Fan3, 90.0, 60.0, 90.0, 200.0);
        spawn_pattern(&mut five, ShotPattern::Fan5, 90.0, 60.0, 90.0, 200.0);
        assert_eq!(three.len(), 3);
        assert_eq!(five.len(), 5);
    }

    #[test]
    fn generic_fan_has_requested_density() {
        let mut bullets = Vec::new();
        spawn_fan(&mut bullets, 90.0, 64.0, 90.0, 240.0, 7, 0.8, 70.0);
        assert_eq!(bullets.len(), 7);
    }

    #[test]
    fn bellkeeper_ring_has_requested_density() {
        let mut bullets = Vec::new();
        spawn_ring(&mut bullets, 90.0, 64.0, 16, 0.2, 60.0);
        assert_eq!(bullets.len(), 16);
    }

    #[test]
    fn bellkeeper_phases_follow_health_thirds() {
        assert_eq!(bell_phase_for_hp(BELLKEEPER_MAX_HP), BellPhase::Procession);
        assert_eq!(bell_phase_for_hp(BELLKEEPER_MAX_HP * 2 / 3), BellPhase::Resonance);
        assert_eq!(bell_phase_for_hp(BELLKEEPER_MAX_HP / 3), BellPhase::FinalToll);
    }

    #[test]
    fn background_wrap_keeps_values_in_expected_band() {
        let y = wrapped_y(200.0, 1000.0, 0.5, 430.0);
        assert!((-48..382).contains(&y));
    }

    #[test]
    fn focus_speed_is_slower_than_normal_speed() { assert!(FOCUS_SPEED < PLAYER_SPEED); }

    #[test]
    fn five_cinders_fill_the_core() { assert_eq!(CINDER_CHARGE * 5, CORE_MAX); }

    #[test]
    fn canticle_does_not_one_shot_full_health_boss() { assert!(CANTICLE_BOSS_DAMAGE < BELLKEEPER_MAX_HP); }

    #[test]
    fn procedural_sounds_are_pcm_wav() {
        for wav in [synthesize_chirp(900.0, 500.0, 0.05, 0.1), synthesize_noise_burst(0.1, 0.2, 42), synthesize_canticle_sound(), synthesize_bell_sound()] {
            assert_eq!(&wav[0..4], b"RIFF");
            assert_eq!(&wav[8..12], b"WAVE");
        }
    }

    #[test]
    fn wsl_uses_known_stable_surface_size() {
        assert_eq!(window_size_for(true), (960, 612));
        assert_eq!(window_size_for(false), (FRAMEBUFFER_WIDTH * 3, FRAMEBUFFER_HEIGHT * 3));
    }
}
