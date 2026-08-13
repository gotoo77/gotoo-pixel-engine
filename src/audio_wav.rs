use crate::AudioError;

/// Encodes mono signed 16-bit PCM samples as a WAV asset supported by GPE audio backends.
pub fn pcm16_mono_wav(sample_rate: u32, samples: &[i16]) -> Result<Vec<u8>, AudioError> {
    if sample_rate != 44_100 && sample_rate != 48_000 {
        return Err(AudioError::new(
            "only 44100 Hz and 48000 Hz PCM WAV sounds are supported",
        ));
    }
    if samples.is_empty() {
        return Err(AudioError::new("PCM sound contains no samples"));
    }
    if samples.len() > (u32::MAX as usize / 2) {
        return Err(AudioError::new("PCM sound is too large to encode as WAV"));
    }

    let data_len = (samples.len() * 2) as u32;
    let byte_rate = sample_rate * 2;
    let mut wav = Vec::with_capacity(44 + data_len as usize);

    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16_u32.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&2_u16.to_le_bytes());
    wav.extend_from_slice(&16_u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    for sample in samples {
        wav.extend_from_slice(&sample.to_le_bytes());
    }

    Ok(wav)
}

#[cfg(test)]
mod tests {
    use super::pcm16_mono_wav;
    use crate::{Audio, NoopAudio, SoundId};

    const TEST_SOUND: SoundId = SoundId::new("test.generated");

    #[test]
    fn generated_pcm_wav_is_accepted_by_audio_backend() {
        let wav = pcm16_mono_wav(44_100, &[0, i16::MAX, 0, i16::MIN])
            .expect("supported PCM should encode");
        let mut audio = NoopAudio::default();

        audio
            .register_wav(TEST_SOUND, &wav)
            .expect("generated WAV should be accepted by the audio backend");
        assert!(audio.play(TEST_SOUND).is_ok());
    }

    #[test]
    fn pcm_wav_encoder_rejects_unsupported_sample_rate() {
        assert!(pcm16_mono_wav(22_050, &[0]).is_err());
    }

    #[test]
    fn pcm_wav_encoder_rejects_empty_sound() {
        assert!(pcm16_mono_wav(44_100, &[]).is_err());
    }
}
