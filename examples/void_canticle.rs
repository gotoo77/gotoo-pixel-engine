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
const HIT_SOUND: SoundId = SoundId::new("void_canticle.hit");
const AUDIO_SAMPLE_RATE: u32 = 44_100;

const BG: Pixel = Pixel::rgb(5, 6, 14);
const STAR_DIM: Pixel = Pixel::rgb(72, 78, 104);
const STAR_BRIGHT: Pixel = Pixel::rgb(180, 190, 220);
const PILGRIM: Pixel = Pixel::rgb(225, 222, 205);
const PILGRIM_CORE: Pixel = Pixel::rgb(245, 170, 70);
const SHOT: Pixel = Pixel::rgb(245, 235, 180);
const ENEMY: Pixel = Pixel::rgb(150, 78, 95);
const ENEMY_EYE: Pixel = Pixel::rgb(255, 110, 70);
const TEXT: Pixel = Pixel::rgb(205, 210, 225);
const ACCENT: Pixel = Pixel::rgb(195, 132, 74);

const PLAYER_SPEED: f32 = 118.0;
const SHOT_SPEED: f32 = 250.0;
const FIRE_PERIOD: f32 = 0.09;

#[derive(Debug, Clone, Copy)]
struct Bullet {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy)]
struct Target {
    x: f32,
    y: f32,
    vx: f32,
}

struct VoidCanticleGame {
    controls: ControlMap,
    sounds: SoundBank,
    player_x: f32,
    player_y: f32,
    bullets: Vec<Bullet>,
    target: Target,
    fire_cooldown: f32,
    scroll: f32,
    score: u32,
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
        sounds
            .insert_wav(FIRE_SOUND, synthesize_fire_sound())
            .expect("Void Canticle fire sound id should be unique");
        sounds
            .insert_wav(HIT_SOUND, synthesize_hit_sound())
            .expect("Void Canticle hit sound id should be unique");

        Self {
            controls,
            sounds,
            player_x: FRAMEBUFFER_WIDTH as f32 / 2.0,
            player_y: FRAMEBUFFER_HEIGHT as f32 - 42.0,
            bullets: Vec::new(),
            target: Target {
                x: FRAMEBUFFER_WIDTH as f32 / 2.0,
                y: 58.0,
                vx: 34.0,
            },
            fire_cooldown: 0.0,
            scroll: 0.0,
            score: 0,
        }
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

        self.player_x = (self.player_x + dx * PLAYER_SPEED * dt).clamp(8.0, FRAMEBUFFER_WIDTH as f32 - 8.0);
        self.player_y = (self.player_y + dy * PLAYER_SPEED * dt).clamp(28.0, FRAMEBUFFER_HEIGHT as f32 - 10.0);
    }

    fn update_fire(&mut self, dt: f32, frame: &mut Frame<'_>) {
        self.fire_cooldown = (self.fire_cooldown - dt).max(0.0);
        if self.controls.action(FIRE).held() && self.fire_cooldown <= 0.0 {
            self.bullets.push(Bullet {
                x: self.player_x,
                y: self.player_y - 9.0,
            });
            self.fire_cooldown = FIRE_PERIOD;
            let _ = self.sounds.play(frame.audio, FIRE_SOUND);
        }
    }

    fn update_target(&mut self, dt: f32, frame: &mut Frame<'_>) {
        self.target.x += self.target.vx * dt;
        if self.target.x <= 18.0 {
            self.target.x = 18.0;
            self.target.vx = self.target.vx.abs();
        } else if self.target.x >= FRAMEBUFFER_WIDTH as f32 - 18.0 {
            self.target.x = FRAMEBUFFER_WIDTH as f32 - 18.0;
            self.target.vx = -self.target.vx.abs();
        }

        for bullet in &mut self.bullets {
            bullet.y -= SHOT_SPEED * dt;
        }

        let mut hit = false;
        self.bullets.retain(|bullet| {
            if bullet.y < 18.0 {
                return false;
            }
            if bullet_hits_target(*bullet, self.target) {
                hit = true;
                return false;
            }
            true
        });

        if hit {
            self.score = self.score.saturating_add(1);
            let phase = self.score as f32 * 0.91;
            self.target.x = 24.0 + phase.sin().abs() * (FRAMEBUFFER_WIDTH as f32 - 48.0);
            self.target.vx = if self.score.is_multiple_of(2) { 42.0 } else { -42.0 };
            let _ = self.sounds.play(frame.audio, HIT_SOUND);
        }
    }

    fn render(&self, framebuffer: &mut Framebuffer) {
        framebuffer.clear(BG);
        render_starfield(framebuffer, self.scroll);
        render_target(framebuffer, self.target);

        for bullet in &self.bullets {
            framebuffer.fill_rect(bullet.x.round() as i32 - 1, bullet.y.round() as i32 - 3, 2, 6, SHOT);
        }

        render_pilgrim(framebuffer, self.player_x.round() as i32, self.player_y.round() as i32);

        framebuffer.draw_text(5, 5, "VOID CANTICLE", ACCENT);
        framebuffer.draw_text(5, 16, "VC0 / GRAVE ORBIT", TEXT);
        framebuffer.draw_text(5, 292, "MOVE ARROWS/WASD", TEXT);
        framebuffer.draw_text(5, 303, "FIRE SPACE / SOUTH", TEXT);
        framebuffer.draw_text(130, 5, &format!("{}", self.score), PILGRIM_CORE);
    }
}

impl Game for VoidCanticleGame {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.controls.update(frame.input);
        let dt = frame.delta_time.as_secs_f32().min(0.05);
        self.scroll = (self.scroll + 36.0 * dt) % FRAMEBUFFER_HEIGHT as f32;

        self.update_player(dt);
        self.update_fire(dt, frame);
        self.update_target(dt, frame);
        self.render(frame.framebuffer);
        GameResult::Continue
    }
}

fn bullet_hits_target(bullet: Bullet, target: Target) -> bool {
    let half_w = 9.0;
    let half_h = 6.0;
    bullet.x >= target.x - half_w
        && bullet.x <= target.x + half_w
        && bullet.y >= target.y - half_h
        && bullet.y <= target.y + half_h
}

fn render_starfield(framebuffer: &mut Framebuffer, scroll: f32) {
    for index in 0..42_i32 {
        let x = (index * 47 + 19) % FRAMEBUFFER_WIDTH as i32;
        let base_y = (index * 83 + 11) % FRAMEBUFFER_HEIGHT as i32;
        let y = (base_y + scroll.round() as i32).rem_euclid(FRAMEBUFFER_HEIGHT as i32);
        let bright = index % 7 == 0;
        framebuffer.fill_rect(x, y, if bright { 2 } else { 1 }, if bright { 2 } else { 1 }, if bright { STAR_BRIGHT } else { STAR_DIM });
    }
}

fn render_pilgrim(framebuffer: &mut Framebuffer, x: i32, y: i32) {
    framebuffer.fill_rect(x - 4, y - 10, 9, 1, PILGRIM);
    framebuffer.fill_rect(x - 2, y - 8, 5, 3, PILGRIM);
    framebuffer.fill_rect(x - 3, y - 5, 7, 8, PILGRIM);
    framebuffer.fill_rect(x - 6, y - 3, 3, 5, PILGRIM);
    framebuffer.fill_rect(x + 4, y - 3, 3, 5, PILGRIM);
    framebuffer.fill_rect(x - 1, y - 3, 3, 4, PILGRIM_CORE);
    framebuffer.fill_rect(x - 3, y + 3, 2, 4, PILGRIM);
    framebuffer.fill_rect(x + 2, y + 3, 2, 4, PILGRIM);
}

fn render_target(framebuffer: &mut Framebuffer, target: Target) {
    let x = target.x.round() as i32;
    let y = target.y.round() as i32;
    framebuffer.fill_rect(x - 8, y - 3, 17, 7, ENEMY);
    framebuffer.fill_rect(x - 12, y - 1, 4, 2, ENEMY);
    framebuffer.fill_rect(x + 9, y - 1, 4, 2, ENEMY);
    framebuffer.fill_rect(x - 4, y - 6, 3, 3, ENEMY);
    framebuffer.fill_rect(x + 2, y - 6, 3, 3, ENEMY);
    framebuffer.fill_rect(x - 1, y, 3, 2, ENEMY_EYE);
}

fn synthesize_fire_sound() -> Vec<u8> {
    synthesize_chirp(920.0, 520.0, 0.055, 0.18)
}

fn synthesize_hit_sound() -> Vec<u8> {
    synthesize_chirp(280.0, 110.0, 0.11, 0.34)
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

fn main() -> Result<(), EngineError> {
    run(
        EngineConfig {
            title: "Void Canticle - Gotoo Pixel Engine".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width: FRAMEBUFFER_WIDTH * 3,
            window_height: FRAMEBUFFER_HEIGHT * 3,
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
    fn projectile_hits_center_of_target() {
        let target = Target {
            x: 90.0,
            y: 60.0,
            vx: 0.0,
        };
        assert!(bullet_hits_target(Bullet { x: 90.0, y: 60.0 }, target));
    }

    #[test]
    fn projectile_outside_target_does_not_hit() {
        let target = Target {
            x: 90.0,
            y: 60.0,
            vx: 0.0,
        };
        assert!(!bullet_hits_target(Bullet { x: 120.0, y: 60.0 }, target));
    }

    #[test]
    fn procedural_sounds_are_pcm_wav() {
        for wav in [synthesize_fire_sound(), synthesize_hit_sound()] {
            assert_eq!(&wav[0..4], b"RIFF");
            assert_eq!(&wav[8..12], b"WAVE");
        }
    }
}
