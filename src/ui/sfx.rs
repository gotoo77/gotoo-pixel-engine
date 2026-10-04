use crate::{Audio, AudioBus, AudioError, SoundBank, SoundId};

const OPEN: SoundId = SoundId::new("gpe.ui.open");
const CONFIRM: SoundId = SoundId::new("gpe.ui.confirm");
const ADJUST: SoundId = SoundId::new("gpe.ui.adjust");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiSoundCue {
    Open,
    Confirm,
    Adjust,
}

#[derive(Debug, Clone)]
pub struct UiSfx {
    bank: SoundBank,
}

impl Default for UiSfx {
    fn default() -> Self {
        Self::new()
    }
}

impl UiSfx {
    pub fn new() -> Self {
        let mut bank = SoundBank::new();
        bank.insert_wav(OPEN, tone_sequence(&[(420.0, 28), (560.0, 42)], 0.055))
            .expect("generated UI open sound id must be unique");
        bank.insert_wav(CONFIRM, tone_sequence(&[(690.0, 44)], 0.05))
            .expect("generated UI confirm sound id must be unique");
        bank.insert_wav(ADJUST, tone_sequence(&[(880.0, 24)], 0.035))
            .expect("generated UI adjust sound id must be unique");
        Self { bank }
    }

    pub fn play(
        &mut self,
        audio: &mut dyn Audio,
        cue: UiSoundCue,
    ) -> Result<(), AudioError> {
        let id = match cue {
            UiSoundCue::Open => OPEN,
            UiSoundCue::Confirm => CONFIRM,
            UiSoundCue::Adjust => ADJUST,
        };
        self.bank.play_on_bus(audio, id, AudioBus::Ui)
    }
}

fn tone_sequence(parts: &[(f32, u32)], amplitude: f32) -> Vec<u8> {
    const SAMPLE_RATE: u32 = 48_000;
    const CHANNELS: u16 = 1;
    const BITS_PER_SAMPLE: u16 = 16;
    const BYTES_PER_SAMPLE: u16 = BITS_PER_SAMPLE / 8;

    let total_samples = parts
        .iter()
        .map(|(_, ms)| (u64::from(SAMPLE_RATE) * u64::from(*ms) / 1000) as usize)
        .sum::<usize>();
    let data_len = total_samples * usize::from(BYTES_PER_SAMPLE);
    let mut out = Vec::with_capacity(44 + data_len);

    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36_u32 + data_len as u32).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16_u32.to_le_bytes());
    out.extend_from_slice(&1_u16.to_le_bytes());
    out.extend_from_slice(&CHANNELS.to_le_bytes());
    out.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    let byte_rate = SAMPLE_RATE * u32::from(CHANNELS) * u32::from(BYTES_PER_SAMPLE);
    out.extend_from_slice(&byte_rate.to_le_bytes());
    let block_align = CHANNELS * BYTES_PER_SAMPLE;
    out.extend_from_slice(&block_align.to_le_bytes());
    out.extend_from_slice(&BITS_PER_SAMPLE.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&(data_len as u32).to_le_bytes());

    for &(frequency, ms) in parts {
        let samples = (u64::from(SAMPLE_RATE) * u64::from(ms) / 1000) as usize;
        let attack = (samples / 8).max(1);
        let release = (samples / 3).max(1);
        for index in 0..samples {
            let t = index as f32 / SAMPLE_RATE as f32;
            let attack_gain = (index as f32 / attack as f32).clamp(0.0, 1.0);
            let release_start = samples.saturating_sub(release);
            let release_gain = if index < release_start {
                1.0
            } else {
                ((samples.saturating_sub(index)) as f32 / release as f32).clamp(0.0, 1.0)
            };
            let envelope = attack_gain * release_gain;
            let sample = (core::f32::consts::TAU * frequency * t).sin()
                * amplitude.clamp(0.0, 1.0)
                * envelope;
            let pcm = (sample * i16::MAX as f32).round() as i16;
            out.extend_from_slice(&pcm.to_le_bytes());
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NoopAudio;

    #[test]
    fn generated_ui_cues_are_valid_and_playable() {
        let mut ui = UiSfx::new();
        let mut audio = NoopAudio::default();

        assert!(ui.play(&mut audio, UiSoundCue::Open).is_ok());
        assert!(ui.play(&mut audio, UiSoundCue::Confirm).is_ok());
        assert!(ui.play(&mut audio, UiSoundCue::Adjust).is_ok());
    }
}
