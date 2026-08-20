use std::collections::{HashMap, HashSet};
use std::fmt;
use std::io::Cursor;
use std::num::NonZero;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundId(&'static str);

impl SoundId {
    pub const fn new(id: &'static str) -> Self {
        Self(id)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlaybackId(u64);

impl PlaybackId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioError {
    message: String,
}

impl AudioError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AudioError {}

pub trait Audio {
    fn register_wav(&mut self, id: SoundId, bytes: &[u8]) -> Result<(), AudioError>;
    fn play(&mut self, id: SoundId) -> Result<(), AudioError>;
    fn start_loop(&mut self, id: SoundId) -> Result<PlaybackId, AudioError> {
        Err(AudioError::new(format!(
            "looped playback is not supported for sound '{}'",
            id.as_str()
        )))
    }
    fn stop_loop(&mut self, playback: PlaybackId) -> Result<(), AudioError> {
        Err(AudioError::new(format!(
            "looped playback '{}' is not active",
            playback.0
        )))
    }
}

#[derive(Debug, Default, Clone)]
pub struct SoundBank {
    assets: HashMap<SoundId, Vec<u8>>,
    registered: HashSet<SoundId>,
}

impl SoundBank {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_wav(&mut self, id: SoundId, bytes: impl Into<Vec<u8>>) -> Result<(), AudioError> {
        if self.assets.contains_key(&id) {
            return Err(AudioError::new(format!(
                "sound '{}' is already present in the bank",
                id.as_str()
            )));
        }
        self.assets.insert(id, bytes.into());
        Ok(())
    }

    pub fn contains(&self, id: SoundId) -> bool {
        self.assets.contains_key(&id)
    }

    pub fn preload(&mut self, audio: &mut dyn Audio) -> Result<(), AudioError> {
        let ids = self.assets.keys().copied().collect::<Vec<_>>();
        for id in ids {
            self.ensure_registered(audio, id)?;
        }
        Ok(())
    }

    pub fn play(&mut self, audio: &mut dyn Audio, id: SoundId) -> Result<(), AudioError> {
        self.ensure_registered(audio, id)?;
        audio.play(id)
    }

    pub fn start_loop(
        &mut self,
        audio: &mut dyn Audio,
        id: SoundId,
    ) -> Result<PlaybackId, AudioError> {
        self.ensure_registered(audio, id)?;
        audio.start_loop(id)
    }

    pub fn stop_loop(
        &mut self,
        audio: &mut dyn Audio,
        playback: PlaybackId,
    ) -> Result<(), AudioError> {
        audio.stop_loop(playback)
    }

    fn ensure_registered(&mut self, audio: &mut dyn Audio, id: SoundId) -> Result<(), AudioError> {
        if self.registered.contains(&id) {
            return Ok(());
        }
        let Some(bytes) = self.assets.get(&id) else {
            return Err(AudioError::new(format!(
                "sound '{}' is not present in the bank",
                id.as_str()
            )));
        };
        audio.register_wav(id, bytes)?;
        self.registered.insert(id);
        Ok(())
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct NoopAudio {
    sounds: HashMap<SoundId, DecodedSound>,
    loop_playbacks: HashMap<PlaybackId, SoundId>,
    next_playback_id: u64,
}

impl Audio for NoopAudio {
    fn register_wav(&mut self, id: SoundId, bytes: &[u8]) -> Result<(), AudioError> {
        register_decoded_wav(&mut self.sounds, id, bytes)
    }

    fn play(&mut self, id: SoundId) -> Result<(), AudioError> {
        if self.sounds.contains_key(&id) {
            Ok(())
        } else {
            Err(AudioError::new(format!(
                "sound '{}' is not registered",
                id.as_str()
            )))
        }
    }

    fn start_loop(&mut self, id: SoundId) -> Result<PlaybackId, AudioError> {
        if !self.sounds.contains_key(&id) {
            return Err(AudioError::new(format!(
                "sound '{}' is not registered",
                id.as_str()
            )));
        }
        self.next_playback_id += 1;
        let playback = PlaybackId::new(self.next_playback_id);
        self.loop_playbacks.insert(playback, id);
        Ok(playback)
    }

    fn stop_loop(&mut self, playback: PlaybackId) -> Result<(), AudioError> {
        if self.loop_playbacks.remove(&playback).is_some() {
            Ok(())
        } else {
            Err(AudioError::new(format!(
                "looped playback '{}' is not active",
                playback.0
            )))
        }
    }
}

pub(crate) trait PlatformAudio: Audio {
    fn activate(&mut self) {}
}

impl PlatformAudio for NoopAudio {}

pub(crate) fn platform_audio() -> Box<dyn PlatformAudio> {
    #[cfg(target_arch = "wasm32")]
    {
        Box::new(web::WebAudio::default())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Box::new(native::NativeAudio::new().unwrap_or_default())
    }
}

#[derive(Debug, Clone, PartialEq)]
struct DecodedSound {
    channels: NonZero<u16>,
    sample_rate: NonZero<u32>,
    samples: Arc<[f32]>,
}

fn register_decoded_wav(
    sounds: &mut HashMap<SoundId, DecodedSound>,
    id: SoundId,
    bytes: &[u8],
) -> Result<(), AudioError> {
    let sound = decode_wav(bytes)?;
    if let Some(existing) = sounds.get(&id) {
        return ensure_same_sound(id, existing, &sound);
    }

    sounds.insert(id, sound);
    Ok(())
}

fn ensure_same_sound(
    id: SoundId,
    existing: &DecodedSound,
    candidate: &DecodedSound,
) -> Result<(), AudioError> {
    if existing == candidate {
        Ok(())
    } else {
        Err(AudioError::new(format!(
            "sound '{}' is already registered with different audio data",
            id.as_str()
        )))
    }
}

fn decode_wav(bytes: &[u8]) -> Result<DecodedSound, AudioError> {
    let mut reader = hound::WavReader::new(Cursor::new(bytes))
        .map_err(|err| AudioError::new(format!("failed to read WAV: {err}")))?;
    let spec = reader.spec();

    if spec.sample_format != hound::SampleFormat::Int || spec.bits_per_sample != 16 {
        return Err(AudioError::new("only 16-bit PCM WAV sounds are supported"));
    }
    if spec.channels != 1 && spec.channels != 2 {
        return Err(AudioError::new(
            "only mono and stereo WAV sounds are supported",
        ));
    }
    if spec.sample_rate != 44_100 && spec.sample_rate != 48_000 {
        return Err(AudioError::new(
            "only 44100 Hz and 48000 Hz WAV sounds are supported",
        ));
    }

    let samples = reader
        .samples::<i16>()
        .map(|sample| {
            sample
                .map(|sample| f32::from(sample) / 32768.0)
                .map_err(|err| AudioError::new(format!("failed to decode WAV sample: {err}")))
        })
        .collect::<Result<Vec<_>, _>>()?;

    if samples.is_empty() {
        return Err(AudioError::new("WAV contains no samples"));
    }
    if samples.len() % usize::from(spec.channels) != 0 {
        return Err(AudioError::new(
            "WAV sample count is not aligned to its channel count",
        ));
    }

    Ok(DecodedSound {
        channels: NonZero::new(spec.channels).expect("validated channel count should be non-zero"),
        sample_rate: NonZero::new(spec.sample_rate)
            .expect("validated sample rate should be non-zero"),
        samples: Arc::from(samples),
    })
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::collections::{HashMap, HashSet};
    use std::sync::atomic::{AtomicBool, Ordering};

    use rodio::buffer::SamplesBuffer;
    use rodio::cpal::StreamError;
    use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player, Source};

    use super::PlaybackId;
    use super::{Audio, AudioError, DecodedSound, PlatformAudio, SoundId, register_decoded_wav};

    static XRUN_REPORTED: AtomicBool = AtomicBool::new(false);

    fn report_stream_error(error: StreamError) {
        if matches!(error, StreamError::BufferUnderrun) {
            if !XRUN_REPORTED.swap(true, Ordering::Relaxed) {
                eprintln!(
                    "audio stream warning: buffer underrun/overrun occurred; suppressing repeats"
                );
            }
            return;
        }

        eprintln!("audio stream error: {error}");
    }

    #[derive(Default)]
    pub(crate) struct NativeAudio {
        sink: Option<MixerDeviceSink>,
        sounds: HashMap<SoundId, DecodedSound>,
        active_loops: HashSet<PlaybackId>,
        loop_players: HashMap<PlaybackId, Player>,
        next_playback_id: u64,
    }

    impl NativeAudio {
        pub(crate) fn new() -> Result<Self, AudioError> {
            let builder = DeviceSinkBuilder::from_default_device()
                .map_err(|err| AudioError::new(format!("audio device unavailable: {err}")))?
                .with_error_callback(report_stream_error as fn(StreamError));
            let mut sink = builder
                .open_sink_or_fallback()
                .map_err(|err| AudioError::new(format!("audio device unavailable: {err}")))?;
            sink.log_on_drop(false);

            Ok(Self {
                sink: Some(sink),
                sounds: HashMap::new(),
                active_loops: HashSet::new(),
                loop_players: HashMap::new(),
                next_playback_id: 0,
            })
        }
    }

    impl Audio for NativeAudio {
        fn register_wav(&mut self, id: SoundId, bytes: &[u8]) -> Result<(), AudioError> {
            register_decoded_wav(&mut self.sounds, id, bytes)
        }

        fn play(&mut self, id: SoundId) -> Result<(), AudioError> {
            let Some(sound) = self.sounds.get(&id) else {
                return Err(AudioError::new(format!(
                    "sound '{}' is not registered",
                    id.as_str()
                )));
            };
            let Some(sink) = self.sink.as_ref() else {
                return Ok(());
            };

            sink.mixer().add(SamplesBuffer::new(
                sound.channels,
                sound.sample_rate,
                sound.samples.to_vec(),
            ));
            Ok(())
        }

        fn start_loop(&mut self, id: SoundId) -> Result<PlaybackId, AudioError> {
            let Some(sound) = self.sounds.get(&id) else {
                return Err(AudioError::new(format!(
                    "sound '{}' is not registered",
                    id.as_str()
                )));
            };

            self.next_playback_id += 1;
            let playback = PlaybackId::new(self.next_playback_id);
            self.active_loops.insert(playback);

            if let Some(sink) = self.sink.as_ref() {
                let player = Player::connect_new(sink.mixer());
                player.append(
                    SamplesBuffer::new(sound.channels, sound.sample_rate, sound.samples.to_vec())
                        .repeat_infinite(),
                );
                self.loop_players.insert(playback, player);
            }

            Ok(playback)
        }

        fn stop_loop(&mut self, playback: PlaybackId) -> Result<(), AudioError> {
            if !self.active_loops.remove(&playback) {
                return Err(AudioError::new(format!(
                    "looped playback '{}' is not active",
                    playback.0
                )));
            }
            if let Some(player) = self.loop_players.remove(&playback) {
                player.stop();
            }
            Ok(())
        }
    }

    impl PlatformAudio for NativeAudio {}
}

#[cfg(target_arch = "wasm32")]
mod web {
    use std::collections::{HashMap, HashSet};

    use web_sys::{AudioBuffer, AudioBufferSourceNode, AudioContext, AudioContextState};

    use super::PlaybackId;
    use super::{
        Audio, AudioError, DecodedSound, PlatformAudio, SoundId, decode_wav, ensure_same_sound,
    };

    #[derive(Default)]
    pub(crate) struct WebAudio {
        context: Option<AudioContext>,
        unavailable: bool,
        sounds: HashMap<SoundId, AudioBuffer>,
        decoded_sounds: HashMap<SoundId, DecodedSound>,
        active_loops: HashSet<PlaybackId>,
        loop_sources: HashMap<PlaybackId, AudioBufferSourceNode>,
        next_playback_id: u64,
    }

    impl WebAudio {
        fn context(&mut self) -> Result<AudioContext, AudioError> {
            if self.unavailable {
                return Err(AudioError::new("WebAudio is unavailable"));
            }
            if let Some(context) = self.context.as_ref() {
                return Ok(context.clone());
            }

            match AudioContext::new() {
                Ok(context) => {
                    self.context = Some(context.clone());
                    Ok(context)
                }
                Err(err) => {
                    self.unavailable = true;
                    Err(AudioError::new(js_error_message(
                        "failed to create AudioContext",
                        err,
                    )))
                }
            }
        }

        fn create_buffer(
            context: &AudioContext,
            sound: &DecodedSound,
        ) -> Result<AudioBuffer, AudioError> {
            let channels = u32::from(sound.channels.get());
            let frames = sound.samples.len() / channels as usize;
            let buffer = context
                .create_buffer(channels, frames as u32, sound.sample_rate.get() as f32)
                .map_err(|err| {
                    AudioError::new(js_error_message("failed to create AudioBuffer", err))
                })?;

            for channel in 0..channels {
                let channel_samples = sound
                    .samples
                    .iter()
                    .skip(channel as usize)
                    .step_by(channels as usize)
                    .copied()
                    .collect::<Vec<_>>();
                buffer
                    .copy_to_channel(&channel_samples, channel as i32)
                    .map_err(|err| {
                        AudioError::new(js_error_message("failed to fill AudioBuffer", err))
                    })?;
            }

            Ok(buffer)
        }
    }

    impl Audio for WebAudio {
        fn register_wav(&mut self, id: SoundId, bytes: &[u8]) -> Result<(), AudioError> {
            let sound = decode_wav(bytes)?;
            if let Some(existing) = self.decoded_sounds.get(&id) {
                return ensure_same_sound(id, existing, &sound);
            }

            let context = self.context()?;
            let buffer = Self::create_buffer(&context, &sound)?;
            self.sounds.insert(id, buffer);
            self.decoded_sounds.insert(id, sound);
            Ok(())
        }

        fn play(&mut self, id: SoundId) -> Result<(), AudioError> {
            let Some(buffer) = self.sounds.get(&id) else {
                return Err(AudioError::new(format!(
                    "sound '{}' is not registered",
                    id.as_str()
                )));
            };
            let Some(context) = self.context.as_ref() else {
                return Ok(());
            };
            if context.state() != AudioContextState::Running {
                return Ok(());
            }

            let source = context.create_buffer_source().map_err(|err| {
                AudioError::new(js_error_message(
                    "failed to create AudioBufferSourceNode",
                    err,
                ))
            })?;
            source.set_buffer(Some(buffer));
            source
                .connect_with_audio_node(&context.destination())
                .map_err(|err| {
                    AudioError::new(js_error_message(
                        "failed to connect AudioBufferSourceNode",
                        err,
                    ))
                })?;
            source
                .start()
                .map_err(|err| AudioError::new(js_error_message("failed to start sound", err)))
        }

        fn start_loop(&mut self, id: SoundId) -> Result<PlaybackId, AudioError> {
            let Some(buffer) = self.sounds.get(&id).cloned() else {
                return Err(AudioError::new(format!(
                    "sound '{}' is not registered",
                    id.as_str()
                )));
            };
            let context = self.context()?;

            self.next_playback_id += 1;
            let playback = PlaybackId::new(self.next_playback_id);
            self.active_loops.insert(playback);

            let source = context.create_buffer_source().map_err(|err| {
                AudioError::new(js_error_message(
                    "failed to create looping AudioBufferSourceNode",
                    err,
                ))
            })?;
            source.set_buffer(Some(&buffer));
            source.set_loop(true);
            source
                .connect_with_audio_node(&context.destination())
                .map_err(|err| {
                    AudioError::new(js_error_message(
                        "failed to connect looping AudioBufferSourceNode",
                        err,
                    ))
                })?;
            source.start().map_err(|err| {
                AudioError::new(js_error_message("failed to start looping sound", err))
            })?;
            self.loop_sources.insert(playback, source);
            Ok(playback)
        }

        fn stop_loop(&mut self, playback: PlaybackId) -> Result<(), AudioError> {
            if !self.active_loops.remove(&playback) {
                return Err(AudioError::new(format!(
                    "looped playback '{}' is not active",
                    playback.0
                )));
            }
            if let Some(source) = self.loop_sources.remove(&playback) {
                stop_source(&source);
            }
            Ok(())
        }
    }

    impl PlatformAudio for WebAudio {
        fn activate(&mut self) {
            if let Ok(context) = self.context() {
                if context.state() == AudioContextState::Suspended {
                    let _ = context.resume();
                }
            }
        }
    }

    fn js_error_message(context: &str, error: wasm_bindgen::JsValue) -> String {
        match error.as_string() {
            Some(error) => format!("{context}: {error}"),
            None => context.into(),
        }
    }

    #[allow(deprecated)]
    fn stop_source(source: &AudioBufferSourceNode) {
        let _ = source.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::{Audio, NoopAudio, SoundBank, SoundId, decode_wav};

    const TEST_SOUND: SoundId = SoundId::new("test.sound");
    const OTHER_SOUND: SoundId = SoundId::new("test.other");
    const VALID_WAV: &[u8] = &[
        82, 73, 70, 70, 40, 0, 0, 0, 87, 65, 86, 69, 102, 109, 116, 32, 16, 0, 0, 0, 1, 0, 1, 0,
        68, 172, 0, 0, 136, 88, 1, 0, 2, 0, 16, 0, 100, 97, 116, 97, 4, 0, 0, 0, 0, 0, 255, 127,
    ];
    const DIFFERENT_VALID_WAV: &[u8] = &[
        82, 73, 70, 70, 40, 0, 0, 0, 87, 65, 86, 69, 102, 109, 116, 32, 16, 0, 0, 0, 1, 0, 1, 0,
        68, 172, 0, 0, 136, 88, 1, 0, 2, 0, 16, 0, 100, 97, 116, 97, 4, 0, 0, 0, 0, 0, 0, 0,
    ];
    const INVALID_WAV: &[u8] = b"not a wav";

    #[test]
    fn sound_id_keeps_stable_namespace() {
        assert_eq!(SoundId::new("snake.eat").as_str(), "snake.eat");
    }

    #[test]
    fn decodes_valid_pcm_16_wav() {
        let sound = decode_wav(VALID_WAV).expect("valid wav should decode");

        assert_eq!(sound.channels.get(), 1);
        assert_eq!(sound.sample_rate.get(), 44_100);
        assert_eq!(sound.samples.len(), 2);
    }

    #[test]
    fn rejects_invalid_wav() {
        assert!(decode_wav(INVALID_WAV).is_err());
    }

    #[test]
    fn noop_audio_registers_valid_wav() {
        let mut audio = NoopAudio::default();

        assert!(audio.register_wav(TEST_SOUND, VALID_WAV).is_ok());
    }

    #[test]
    fn noop_audio_rejects_invalid_wav() {
        let mut audio = NoopAudio::default();

        assert!(audio.register_wav(TEST_SOUND, INVALID_WAV).is_err());
    }

    #[test]
    fn noop_audio_registration_is_idempotent_for_same_sound() {
        let mut audio = NoopAudio::default();

        audio
            .register_wav(TEST_SOUND, VALID_WAV)
            .expect("first register should succeed");
        audio
            .register_wav(TEST_SOUND, VALID_WAV)
            .expect("same id and sound should remain idempotent");
    }

    #[test]
    fn noop_audio_rejects_same_id_with_different_sound() {
        let mut audio = NoopAudio::default();

        audio
            .register_wav(TEST_SOUND, VALID_WAV)
            .expect("first register should succeed");
        let error = audio
            .register_wav(TEST_SOUND, DIFFERENT_VALID_WAV)
            .expect_err("same id with different audio data should be rejected");

        assert!(error.to_string().contains("different audio data"));
    }

    #[test]
    fn noop_audio_reports_unknown_sound() {
        let mut audio = NoopAudio::default();

        assert!(audio.play(TEST_SOUND).is_err());
    }

    #[test]
    fn noop_audio_allows_multiple_plays_of_registered_sound() {
        let mut audio = NoopAudio::default();
        audio
            .register_wav(TEST_SOUND, VALID_WAV)
            .expect("register should succeed");

        assert!(audio.play(TEST_SOUND).is_ok());
        assert!(audio.play(TEST_SOUND).is_ok());
    }

    #[test]
    fn noop_audio_starts_and_stops_identifiable_loop() {
        let mut audio = NoopAudio::default();
        audio
            .register_wav(TEST_SOUND, VALID_WAV)
            .expect("register should succeed");

        let playback = audio
            .start_loop(TEST_SOUND)
            .expect("registered sound should loop");

        assert_eq!(audio.loop_playbacks.get(&playback), Some(&TEST_SOUND));
        assert!(audio.stop_loop(playback).is_ok());
        assert!(!audio.loop_playbacks.contains_key(&playback));
    }

    #[test]
    fn noop_audio_double_stop_reports_inactive_loop() {
        let mut audio = NoopAudio::default();
        audio
            .register_wav(TEST_SOUND, VALID_WAV)
            .expect("register should succeed");

        let playback = audio
            .start_loop(TEST_SOUND)
            .expect("registered sound should loop");

        assert!(audio.stop_loop(playback).is_ok());
        let error = audio
            .stop_loop(playback)
            .expect_err("double stop should be explicit");
        assert!(error.to_string().contains("is not active"));
    }

    #[test]
    fn sound_bank_starts_loop_without_regressing_one_shots() {
        let mut bank = SoundBank::new();
        bank.insert_wav(TEST_SOUND, VALID_WAV)
            .expect("bank insert should succeed");
        let mut audio = NoopAudio::default();

        let playback = bank
            .start_loop(&mut audio, TEST_SOUND)
            .expect("bank should start registered loop");

        assert!(bank.play(&mut audio, TEST_SOUND).is_ok());
        assert!(bank.stop_loop(&mut audio, playback).is_ok());
    }

    #[test]
    fn different_sound_ids_are_distinct() {
        let mut audio = NoopAudio::default();
        audio
            .register_wav(TEST_SOUND, VALID_WAV)
            .expect("register should succeed");

        assert!(audio.play(OTHER_SOUND).is_err());
    }

    #[test]
    fn sound_bank_registers_lazily_and_replays() {
        let mut audio = NoopAudio::default();
        let mut bank = SoundBank::new();
        bank.insert_wav(TEST_SOUND, VALID_WAV.to_vec())
            .expect("asset should be inserted");

        assert!(bank.play(&mut audio, TEST_SOUND).is_ok());
        assert!(bank.play(&mut audio, TEST_SOUND).is_ok());
    }

    #[test]
    fn sound_bank_reports_unknown_asset() {
        let mut audio = NoopAudio::default();
        let mut bank = SoundBank::new();
        assert!(bank.play(&mut audio, TEST_SOUND).is_err());
    }
}
