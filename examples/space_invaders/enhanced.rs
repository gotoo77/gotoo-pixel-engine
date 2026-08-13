include!("game.rs");

const ALIEN_DESTROYED_SOUND: gotoo_pixel_engine::SoundId =
    gotoo_pixel_engine::SoundId::new("space_invaders.alien_destroyed");
const PLAYER_DESTROYED_SOUND: gotoo_pixel_engine::SoundId =
    gotoo_pixel_engine::SoundId::new("space_invaders.player_destroyed");
const BUNKER_HIT_SOUND: gotoo_pixel_engine::SoundId =
    gotoo_pixel_engine::SoundId::new("space_invaders.bunker_hit");

const ALIEN_EXPLOSION_DURATION: Duration = Duration::from_millis(220);
const PLAYER_EXPLOSION_DURATION: Duration = Duration::from_millis(420);
const BUNKER_IMPACT_DURATION: Duration = Duration::from_millis(140);
const IMPACT_COLOR: Pixel = Pixel::rgb(255, 225, 120);
const AUDIO_SAMPLE_RATE: u32 = 44_100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FeedbackKind {
    Alien { row: usize },
    Player,
    Bunker,
}

#[derive(Debug, Clone, Copy)]
struct FeedbackEffect {
    kind: FeedbackKind,
    x: i32,
    y: i32,
    remaining: Duration,
}

impl FeedbackEffect {
    fn new(kind: FeedbackKind, x: i32, y: i32) -> Self {
        let remaining = match kind {
            FeedbackKind::Alien { .. } => ALIEN_EXPLOSION_DURATION,
            FeedbackKind::Player => PLAYER_EXPLOSION_DURATION,
            FeedbackKind::Bunker => BUNKER_IMPACT_DURATION,
        };
        Self {
            kind,
            x,
            y,
            remaining,
        }
    }
}

struct FeedbackSnapshot {
    state: RoundState,
    aliens_alive: Vec<bool>,
    player_x: f32,
    lives: u32,
    bunkers: Vec<[[bool; BUNKER_COLS]; BUNKER_ROWS]>,
}

impl FeedbackSnapshot {
    fn capture(world: &SpaceInvadersWorld) -> Self {
        Self {
            state: world.state,
            aliens_alive: world.aliens.iter().map(|alien| alien.alive).collect(),
            player_x: world.player_x,
            lives: world.lives,
            bunkers: world.bunkers.iter().map(|bunker| bunker.cells).collect(),
        }
    }
}

#[derive(Debug)]
pub struct EnhancedSpaceInvadersGame {
    core: SpaceInvadersGame,
    effects: Vec<FeedbackEffect>,
    audio_initialized: bool,
}

impl EnhancedSpaceInvadersGame {
    pub fn new() -> Self {
        Self {
            core: SpaceInvadersGame::new(),
            effects: Vec::new(),
            audio_initialized: false,
        }
    }

    fn ensure_audio(&mut self, frame: &mut Frame<'_>) {
        if self.audio_initialized {
            return;
        }

        for (id, sound) in [
            (ALIEN_DESTROYED_SOUND, FeedbackKind::Alien { row: 0 }),
            (PLAYER_DESTROYED_SOUND, FeedbackKind::Player),
            (BUNKER_HIT_SOUND, FeedbackKind::Bunker),
        ] {
            let wav: &'static [u8] = Box::leak(synthesize_sound(sound).into_boxed_slice());
            let _ = frame.audio.register_wav(id, wav);
        }
        self.audio_initialized = true;
    }

    fn update_effects(&mut self, dt: Duration) {
        for effect in &mut self.effects {
            effect.remaining = effect.remaining.saturating_sub(dt);
        }
        self.effects.retain(|effect| !effect.remaining.is_zero());
    }

    fn collect_feedback(
        &mut self,
        before: FeedbackSnapshot,
        frame: &mut Frame<'_>,
    ) {
        let world = &self.core.world;

        if before.state != RoundState::Playing && world.state == RoundState::Playing {
            self.effects.clear();
            return;
        }

        let mut new_effects = Vec::new();
        let mut sounds = Vec::new();

        for (index, was_alive) in before.aliens_alive.into_iter().enumerate() {
            let alien = world.aliens[index];
            if was_alive && !alien.alive {
                let (x, y) = world.alien_position(alien);
                new_effects.push(FeedbackEffect::new(
                    FeedbackKind::Alien { row: alien.row },
                    x + ALIEN_W / 2,
                    y + ALIEN_H / 2,
                ));
                sounds.push(ALIEN_DESTROYED_SOUND);
            }
        }

        if world.lives < before.lives {
            new_effects.push(FeedbackEffect::new(
                FeedbackKind::Player,
                before.player_x.round() as i32 + PLAYER_W / 2,
                PLAYER_Y + 3,
            ));
            sounds.push(PLAYER_DESTROYED_SOUND);
        }

        for (bunker_index, old_cells) in before.bunkers.into_iter().enumerate() {
            let bunker = &world.bunkers[bunker_index];
            let mut sum_x = 0_i32;
            let mut sum_y = 0_i32;
            let mut changed = 0_i32;

            for y in 0..BUNKER_ROWS {
                for x in 0..BUNKER_COLS {
                    if old_cells[y][x] && !bunker.cells[y][x] {
                        sum_x += bunker.x + x as i32 * BUNKER_CELL + BUNKER_CELL / 2;
                        sum_y += BUNKER_Y + y as i32 * BUNKER_CELL + BUNKER_CELL / 2;
                        changed += 1;
                    }
                }
            }

            if changed > 0 {
                new_effects.push(FeedbackEffect::new(
                    FeedbackKind::Bunker,
                    sum_x / changed,
                    sum_y / changed,
                ));
                sounds.push(BUNKER_HIT_SOUND);
            }
        }

        self.effects.extend(new_effects);
        for sound in sounds {
            let _ = frame.audio.play(sound);
        }
    }

    fn render_effects(&self, fb: &mut Framebuffer) {
        for effect in &self.effects {
            match effect.kind {
                FeedbackKind::Alien { row } => {
                    let progress = 1.0
                        - effect.remaining.as_secs_f32()
                            / ALIEN_EXPLOSION_DURATION.as_secs_f32();
                    let radius = if progress < 0.5 { 2 } else { 5 };
                    draw_burst(fb, effect.x, effect.y, radius, alien_feedback_color(row));
                }
                FeedbackKind::Player => {
                    let progress = 1.0
                        - effect.remaining.as_secs_f32()
                            / PLAYER_EXPLOSION_DURATION.as_secs_f32();
                    let radius = if progress < 0.34 {
                        3
                    } else if progress < 0.68 {
                        6
                    } else {
                        9
                    };
                    draw_burst(fb, effect.x, effect.y, radius, DANGER);
                    if radius >= 6 {
                        draw_burst(fb, effect.x, effect.y, radius - 3, SHOT);
                    }
                }
                FeedbackKind::Bunker => {
                    let progress = 1.0
                        - effect.remaining.as_secs_f32()
                            / BUNKER_IMPACT_DURATION.as_secs_f32();
                    draw_burst(
                        fb,
                        effect.x,
                        effect.y,
                        if progress < 0.5 { 1 } else { 3 },
                        IMPACT_COLOR,
                    );
                }
            }
        }
    }
}

impl Game for EnhancedSpaceInvadersGame {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.ensure_audio(frame);
        self.update_effects(frame.delta_time);
        let before = FeedbackSnapshot::capture(&self.core.world);

        let result = self.core.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        self.collect_feedback(before, frame);
        self.render_effects(frame.framebuffer);
        GameResult::Continue
    }
}

fn alien_feedback_color(row: usize) -> Pixel {
    match row {
        0 => ALIEN_TOP_COLOR,
        1 | 2 => ALIEN_MIDDLE_COLOR,
        _ => ALIEN_BOTTOM_COLOR,
    }
}

fn draw_burst(fb: &mut Framebuffer, x: i32, y: i32, radius: i32, color: Pixel) {
    let points = [
        (-radius, 0),
        (radius, 0),
        (0, -radius),
        (0, radius),
        (-radius, -radius),
        (radius, -radius),
        (-radius, radius),
        (radius, radius),
    ];

    fb.fill_rect(x - 1, y - 1, 3, 3, color);
    for (dx, dy) in points {
        fb.fill_rect(x + dx, y + dy, 1, 1, color);
    }
}

fn synthesize_sound(kind: FeedbackKind) -> Vec<u8> {
    let (duration, mut seed): (f32, u32) = match kind {
        FeedbackKind::Alien { .. } => (0.16, 0xA11E_0001),
        FeedbackKind::Player => (0.38, 0xC0DE_0002),
        FeedbackKind::Bunker => (0.075, 0xB00B_0003),
    };
    let sample_count = (AUDIO_SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(sample_count);
    let mut phase = 0.0_f32;

    for index in 0..sample_count {
        let t = index as f32 / AUDIO_SAMPLE_RATE as f32;
        let progress = t / duration;
        let envelope = (1.0 - progress).max(0.0);
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let noise = ((seed >> 8) as f32 / 16_777_215.0) * 2.0 - 1.0;

        let sample = match kind {
            FeedbackKind::Alien { .. } => {
                let frequency = 520.0 - 330.0 * progress;
                phase += frequency / AUDIO_SAMPLE_RATE as f32;
                let square = if phase.fract() < 0.5 { 1.0 } else { -1.0 };
                (0.62 * square + 0.38 * noise) * envelope * envelope * 0.55
            }
            FeedbackKind::Player => {
                let frequency = 170.0 - 100.0 * progress;
                phase += frequency / AUDIO_SAMPLE_RATE as f32;
                let square = if phase.fract() < 0.5 { 1.0 } else { -1.0 };
                (0.34 * square + 0.66 * noise) * envelope * 0.58
            }
            FeedbackKind::Bunker => {
                phase += 180.0 / AUDIO_SAMPLE_RATE as f32;
                let square = if phase.fract() < 0.5 { 1.0 } else { -1.0 };
                (0.72 * square + 0.28 * noise) * envelope * envelope * envelope * 0.45
            }
        };
        samples.push((sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16);
    }

    pcm16_mono_wav(&samples)
}

fn pcm16_mono_wav(samples: &[i16]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut wav = Vec::with_capacity(44 + data_len as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16_u32.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&AUDIO_SAMPLE_RATE.to_le_bytes());
    wav.extend_from_slice(&(AUDIO_SAMPLE_RATE * 2).to_le_bytes());
    wav.extend_from_slice(&2_u16.to_le_bytes());
    wav.extend_from_slice(&16_u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    for sample in samples {
        wav.extend_from_slice(&sample.to_le_bytes());
    }
    wav
}

#[cfg(test)]
mod feedback_tests {
    use super::*;

    #[test]
    fn synthesized_feedback_is_pcm_wav() {
        for kind in [
            FeedbackKind::Alien { row: 0 },
            FeedbackKind::Player,
            FeedbackKind::Bunker,
        ] {
            let wav = synthesize_sound(kind);
            assert_eq!(&wav[0..4], b"RIFF");
            assert_eq!(&wav[8..12], b"WAVE");
            assert!(wav.len() > 44);
        }
    }

    #[test]
    fn feedback_effects_have_distinct_lifetimes() {
        assert!(PLAYER_EXPLOSION_DURATION > ALIEN_EXPLOSION_DURATION);
        assert!(ALIEN_EXPLOSION_DURATION > BUNKER_IMPACT_DURATION);
    }
}
