use std::f32::consts::PI;
use std::time::Duration;

use gotoo_pixel_engine::{
    EngineConfig, EngineError, Frame, Game, GameResult, Key, Pixel, SoundBank, SoundId,
    pcm16_mono_wav, run,
};

const FRAMEBUFFER_WIDTH: u32 = 320;
const FRAMEBUFFER_HEIGHT: u32 = 180;
const AUDIO_SAMPLE_RATE: u32 = 44_100;

const LONG_TONE: SoundId = SoundId::new("audio_probe.long_tone");
const BLIP: SoundId = SoundId::new("audio_probe.blip");
const ALIEN: SoundId = SoundId::new("audio_probe.space_invaders.alien");
const BUNKER: SoundId = SoundId::new("audio_probe.space_invaders.bunker");
const PLAYER: SoundId = SoundId::new("audio_probe.space_invaders.player");

const BG: Pixel = Pixel::rgb(8, 12, 16);
const TEXT: Pixel = Pixel::rgb(220, 235, 220);
const ACCENT: Pixel = Pixel::rgb(90, 220, 180);
const DANGER: Pixel = Pixel::rgb(255, 95, 80);

#[derive(Debug, Clone, Copy)]
enum SpaceInvadersSound {
    Alien,
    Bunker,
    Player,
}

struct AudioProbe {
    sounds: SoundBank,
    long_tone_started: bool,
    flash_remaining: Duration,
    flash_color: Pixel,
}

impl AudioProbe {
    fn new() -> Self {
        let mut sounds = SoundBank::new();
        sounds
            .insert_wav(LONG_TONE, sine_wav(440.0, Duration::from_secs(5), 0.22))
            .expect("audio probe long tone id should be unique");
        sounds
            .insert_wav(BLIP, sine_wav(880.0, Duration::from_millis(120), 0.30))
            .expect("audio probe blip id should be unique");
        sounds
            .insert_wav(ALIEN, space_invaders_wav(SpaceInvadersSound::Alien))
            .expect("audio probe alien sound id should be unique");
        sounds
            .insert_wav(BUNKER, space_invaders_wav(SpaceInvadersSound::Bunker))
            .expect("audio probe bunker sound id should be unique");
        sounds
            .insert_wav(PLAYER, space_invaders_wav(SpaceInvadersSound::Player))
            .expect("audio probe player sound id should be unique");

        Self {
            sounds,
            long_tone_started: false,
            flash_remaining: Duration::ZERO,
            flash_color: ACCENT,
        }
    }

    fn trigger(&mut self, frame: &mut Frame<'_>, id: SoundId, label: &str, color: Pixel) {
        self.flash_remaining = Duration::from_millis(150);
        self.flash_color = color;
        eprintln!("audio probe: {label}");
        let _ = self.sounds.play(frame.audio, id);
    }
}

impl Game for AudioProbe {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if frame.input.key(Key::Escape).pressed() {
            return GameResult::Exit;
        }

        if !self.long_tone_started {
            self.long_tone_started = true;
            eprintln!("audio probe: starting 5 second continuous tone");
            let _ = self.sounds.play(frame.audio, LONG_TONE);
        }

        if frame.input.key(Key::Space).pressed() {
            self.trigger(frame, BLIP, "SPACE -> generic blip", ACCENT);
        } else if frame.input.key(Key::A).pressed() {
            self.trigger(frame, ALIEN, "A -> Space Invaders alien", ACCENT);
        } else if frame.input.key(Key::S).pressed() {
            self.trigger(frame, BUNKER, "S -> Space Invaders bunker", Pixel::rgb(255, 225, 120));
        } else if frame.input.key(Key::D).pressed() {
            self.trigger(frame, PLAYER, "D -> Space Invaders player", DANGER);
        } else {
            self.flash_remaining = self.flash_remaining.saturating_sub(frame.delta_time);
        }

        frame.framebuffer.clear(BG);
        frame.framebuffer.draw_text(16, 14, "GPE AUDIO PROBE", ACCENT);
        frame
            .framebuffer
            .draw_text(16, 38, "START  5 SECOND TONE", TEXT);
        frame
            .framebuffer
            .draw_text(16, 58, "SPACE  GENERIC BLIP", TEXT);
        frame
            .framebuffer
            .draw_text(16, 74, "A  ALIEN EXPLOSION", TEXT);
        frame
            .framebuffer
            .draw_text(16, 90, "S  BUNKER IMPACT", TEXT);
        frame
            .framebuffer
            .draw_text(16, 106, "D  PLAYER EXPLOSION", TEXT);
        frame.framebuffer.draw_text(16, 150, "ESC  QUIT", TEXT);

        if !self.flash_remaining.is_zero() {
            frame
                .framebuffer
                .fill_rect(16, 126, 288, 14, self.flash_color);
        }

        GameResult::Continue
    }
}

fn sine_wav(frequency: f32, duration: Duration, amplitude: f32) -> Vec<u8> {
    let sample_count = (duration.as_secs_f32() * AUDIO_SAMPLE_RATE as f32) as usize;
    let samples = (0..sample_count)
        .map(|index| {
            let t = index as f32 / AUDIO_SAMPLE_RATE as f32;
            let sample = (2.0 * PI * frequency * t).sin() * amplitude;
            (sample * i16::MAX as f32) as i16
        })
        .collect::<Vec<_>>();

    pcm16_mono_wav(AUDIO_SAMPLE_RATE, &samples)
        .expect("audio probe should synthesize a supported PCM WAV")
}

fn space_invaders_wav(kind: SpaceInvadersSound) -> Vec<u8> {
    let (duration, mut seed): (f32, u32) = match kind {
        SpaceInvadersSound::Alien => (0.16, 0xA11E_0001),
        SpaceInvadersSound::Player => (0.38, 0xC0DE_0002),
        SpaceInvadersSound::Bunker => (0.075, 0xB00B_0003),
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
            SpaceInvadersSound::Alien => {
                let frequency = 520.0 - 330.0 * progress;
                phase += frequency / AUDIO_SAMPLE_RATE as f32;
                let square = if phase.fract() < 0.5 { 1.0 } else { -1.0 };
                (0.62 * square + 0.38 * noise) * envelope * envelope * 0.55
            }
            SpaceInvadersSound::Player => {
                let frequency = 170.0 - 100.0 * progress;
                phase += frequency / AUDIO_SAMPLE_RATE as f32;
                let square = if phase.fract() < 0.5 { 1.0 } else { -1.0 };
                (0.34 * square + 0.66 * noise) * envelope * 0.58
            }
            SpaceInvadersSound::Bunker => {
                phase += 180.0 / AUDIO_SAMPLE_RATE as f32;
                let square = if phase.fract() < 0.5 { 1.0 } else { -1.0 };
                (0.72 * square + 0.28 * noise) * envelope * envelope * envelope * 0.45
            }
        };
        samples.push((sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16);
    }

    pcm16_mono_wav(AUDIO_SAMPLE_RATE, &samples)
        .expect("Space Invaders probe audio should use a supported PCM format")
}

fn main() -> Result<(), EngineError> {
    run(
        EngineConfig {
            title: "GPE Audio Probe".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width: 960,
            window_height: 540,
        },
        AudioProbe::new(),
    )
}
