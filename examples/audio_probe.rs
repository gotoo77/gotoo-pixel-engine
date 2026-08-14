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

const BG: Pixel = Pixel::rgb(8, 12, 16);
const TEXT: Pixel = Pixel::rgb(220, 235, 220);
const ACCENT: Pixel = Pixel::rgb(90, 220, 180);

struct AudioProbe {
    sounds: SoundBank,
    long_tone_started: bool,
    flash_remaining: Duration,
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

        Self {
            sounds,
            long_tone_started: false,
            flash_remaining: Duration::ZERO,
        }
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
            self.flash_remaining = Duration::from_millis(150);
            eprintln!("audio probe: SPACE -> flash + blip");
            let _ = self.sounds.play(frame.audio, BLIP);
        } else {
            self.flash_remaining = self.flash_remaining.saturating_sub(frame.delta_time);
        }

        frame.framebuffer.clear(BG);
        frame.framebuffer.draw_text(16, 18, "GPE AUDIO PROBE", ACCENT);
        frame
            .framebuffer
            .draw_text(16, 46, "1  LISTEN TO 5 SECOND TONE", TEXT);
        frame
            .framebuffer
            .draw_text(16, 62, "2  WAIT UNTIL IT ENDS", TEXT);
        frame
            .framebuffer
            .draw_text(16, 78, "3  SPACE  FLASH PLUS BLIP", TEXT);
        frame.framebuffer.draw_text(16, 148, "ESC  QUIT", TEXT);

        if !self.flash_remaining.is_zero() {
            frame.framebuffer.fill_rect(16, 104, 288, 28, ACCENT);
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
