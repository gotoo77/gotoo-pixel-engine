use std::collections::HashMap;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioError {
    message: String,
}

impl AudioError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.message) }
}
impl std::error::Error for AudioError {}

pub trait Audio {
    fn register_wav(&mut self, id: SoundId, bytes: &[u8]) -> Result<(), AudioError>;
    fn play(&mut self, id: SoundId) -> Result<(), AudioError>;
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct NoopAudio { sounds: HashMap<SoundId, DecodedSound> }

impl Audio for NoopAudio {
    fn register_wav(&mut self, id: SoundId, bytes: &[u8]) -> Result<(), AudioError> {
        register_decoded_wav(&mut self.sounds, id, bytes)
    }
    fn play(&mut self, id: SoundId) -> Result<(), AudioError> {
        if self.sounds.contains_key(&id) { Ok(()) } else { Err(AudioError::new(format!("sound '{}' is not registered", id.as_str()))) }
    }
}

pub(crate) trait PlatformAudio: Audio { fn activate(&mut self) {} }
impl PlatformAudio for NoopAudio {}

pub(crate) fn platform_audio() -> Box<dyn PlatformAudio> {
    #[cfg(target_arch = "wasm32")]
    { Box::new(web::WebAudio::default()) }
    #[cfg(not(target_arch = "wasm32"))]
    { Box::new(native::NativeAudio::new().unwrap_or_default()) }
}

#[derive(Debug, Clone, PartialEq)]
struct DecodedSound { channels: NonZero<u16>, sample_rate: NonZero<u32>, samples: Arc<[f32]> }

fn register_decoded_wav(sounds: &mut HashMap<SoundId, DecodedSound>, id: SoundId, bytes: &[u8]) -> Result<(), AudioError> {
    if sounds.contains_key(&id) { return Ok(()); }
    let sound = decode_wav(bytes)?;
    sounds.insert(id, sound);
    Ok(())
}

fn decode_wav(bytes: &[u8]) -> Result<DecodedSound, AudioError> {
    let mut reader = hound::WavReader::new(Cursor::new(bytes)).map_err(|err| AudioError::new(format!("failed to read WAV: {err}")))?;
    let spec = reader.spec();
    if spec.sample_format != hound::SampleFormat::Int || spec.bits_per_sample != 16 { return Err(AudioError::new("only 16-bit PCM WAV sounds are supported")); }
    if spec.channels != 1 && spec.channels != 2 { return Err(AudioError::new("only mono and stereo WAV sounds are supported")); }
    if spec.sample_rate != 44_100 && spec.sample_rate != 48_000 { return Err(AudioError::new("only 44100 Hz and 48000 Hz WAV sounds are supported")); }
    let samples = reader.samples::<i16>().map(|sample| sample.map(|sample| f32::from(sample) / 32768.0).map_err(|err| AudioError::new(format!("failed to decode WAV sample: {err}")))).collect::<Result<Vec<_>, _>>()?;
    if samples.is_empty() { return Err(AudioError::new("WAV contains no samples")); }
    if samples.len() % usize::from(spec.channels) != 0 { return Err(AudioError::new("WAV sample count is not aligned to its channel count")); }
    Ok(DecodedSound {
        channels: NonZero::new(spec.channels).expect("validated channel count should be non-zero"),
        sample_rate: NonZero::new(spec.sample_rate).expect("validated sample rate should be non-zero"),
        samples: Arc::from(samples),
    })
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::collections::HashMap;
    use rodio::buffer::SamplesBuffer;
    use rodio::{DeviceSinkBuilder, MixerDeviceSink};
    use super::{Audio, AudioError, DecodedSound, PlatformAudio, SoundId, register_decoded_wav};

    #[derive(Default)]
    pub(crate) struct NativeAudio { sink: Option<MixerDeviceSink>, sounds: HashMap<SoundId, DecodedSound> }
    impl NativeAudio {
        pub(crate) fn new() -> Result<Self, AudioError> {
            let mut sink = DeviceSinkBuilder::open_default_sink().map_err(|err| AudioError::new(format!("audio device unavailable: {err}")))?;
            sink.log_on_drop(false);
            Ok(Self { sink: Some(sink), sounds: HashMap::new() })
        }
    }
    impl Audio for NativeAudio {
        fn register_wav(&mut self, id: SoundId, bytes: &[u8]) -> Result<(), AudioError> { register_decoded_wav(&mut self.sounds, id, bytes) }
        fn play(&mut self, id: SoundId) -> Result<(), AudioError> {
            let Some(sound) = self.sounds.get(&id) else { return Err(AudioError::new(format!("sound '{}' is not registered", id.as_str()))); };
            let Some(sink) = self.sink.as_ref() else { return Ok(()); };
            sink.mixer().add(SamplesBuffer::new(sound.channels, sound.sample_rate, sound.samples.to_vec()));
            Ok(())
        }
    }
    impl PlatformAudio for NativeAudio {}
}

#[cfg(target_arch = "wasm32")]
mod web {
    use std::collections::HashMap;
    use web_sys::{AudioBuffer, AudioContext, AudioContextState};
    use super::{Audio, AudioError, DecodedSound, PlatformAudio, SoundId, decode_wav};

    #[derive(Default)]
    pub(crate) struct WebAudio { context: Option<AudioContext>, unavailable: bool, sounds: HashMap<SoundId, AudioBuffer> }
    impl WebAudio {
        fn context(&mut self) -> Result<AudioContext, AudioError> {
            if self.unavailable { return Err(AudioError::new("WebAudio is unavailable")); }
            if let Some(context) = self.context.as_ref() { return Ok(context.clone()); }
            match AudioContext::new() {
                Ok(context) => { self.context = Some(context.clone()); Ok(context) }
                Err(err) => { self.unavailable = true; Err(AudioError::new(js_error_message("failed to create AudioContext", err))) }
            }
        }
        fn create_buffer(context: &AudioContext, sound: &DecodedSound) -> Result<AudioBuffer, AudioError> {
            let channels = u32::from(sound.channels.get());
            let frames = sound.samples.len() / channels as usize;
            let buffer = context.create_buffer(channels, frames as u32, sound.sample_rate.get() as f32).map_err(|err| AudioError::new(js_error_message("failed to create AudioBuffer", err)))?;
            for channel in 0..channels {
                let channel_samples = sound.samples.iter().skip(channel as usize).step_by(channels as usize).copied().collect::<Vec<_>>();
                buffer.copy_to_channel(&channel_samples, channel as i32).map_err(|err| AudioError::new(js_error_message("failed to fill AudioBuffer", err)))?;
            }
            Ok(buffer)
        }
    }
    impl Audio for WebAudio {
        fn register_wav(&mut self, id: SoundId, bytes: &[u8]) -> Result<(), AudioError> {
            if self.sounds.contains_key(&id) { return Ok(()); }
            let sound = decode_wav(bytes)?;
            let context = self.context()?;
            let buffer = Self::create_buffer(&context, &sound)?;
            self.sounds.insert(id, buffer);
            Ok(())
        }
        fn play(&mut self, id: SoundId) -> Result<(), AudioError> {
            let Some(buffer) = self.sounds.get(&id) else { return Err(AudioError::new(format!("sound '{}' is not registered", id.as_str()))); };
            let Some(context) = self.context.as_ref() else { return Ok(()); };
            if context.state() != AudioContextState::Running { return Ok(()); }
            let source = context.create_buffer_source().map_err(|err| AudioError::new(js_error_message("failed to create AudioBufferSourceNode", err)))?;
            source.set_buffer(Some(buffer));
            source.connect_with_audio_node(&context.destination()).map_err(|err| AudioError::new(js_error_message("failed to connect AudioBufferSourceNode", err)))?;
            source.start().map_err(|err| AudioError::new(js_error_message("failed to start sound", err)))
        }
    }
    impl PlatformAudio for WebAudio {
        fn activate(&mut self) {
            if let Ok(context) = self.context() { if context.state() == AudioContextState::Suspended { let _ = context.resume(); } }
        }
    }
    fn js_error_message(context: &str, error: wasm_bindgen::JsValue) -> String {
        match error.as_string() { Some(error) => format!("{context}: {error}"), None => context.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::{Audio, NoopAudio, SoundId, decode_wav};
    const TEST_SOUND: SoundId = SoundId::new("test.sound");
    const OTHER_SOUND: SoundId = SoundId::new("test.other");
    const VALID_WAV: &[u8] = &[82,73,70,70,40,0,0,0,87,65,86,69,102,109,116,32,16,0,0,0,1,0,1,0,68,172,0,0,136,88,1,0,2,0,16,0,100,97,116,97,4,0,0,0,0,0,255,127];
    const INVALID_WAV: &[u8] = b"not a wav";

    #[test] fn sound_id_keeps_stable_namespace() { assert_eq!(SoundId::new("snake.eat").as_str(), "snake.eat"); }
    #[test] fn decodes_valid_pcm_16_wav() { let sound = decode_wav(VALID_WAV).expect("valid wav should decode"); assert_eq!(sound.channels.get(), 1); assert_eq!(sound.sample_rate.get(), 44_100); assert_eq!(sound.samples.len(), 2); }
    #[test] fn decodes_runtime_owned_wav_buffer() { let bytes = VALID_WAV.to_vec(); let sound = decode_wav(&bytes).expect("owned runtime buffer should decode"); assert_eq!(sound.samples.len(), 2); }
    #[test] fn rejects_invalid_wav() { assert!(decode_wav(INVALID_WAV).is_err()); }
    #[test] fn noop_audio_registers_valid_wav() { let mut audio = NoopAudio::default(); assert!(audio.register_wav(TEST_SOUND, VALID_WAV).is_ok()); }
    #[test] fn noop_audio_registers_runtime_owned_wav() { let mut audio = NoopAudio::default(); let bytes = VALID_WAV.to_vec(); audio.register_wav(TEST_SOUND, &bytes).expect("runtime buffer should register"); drop(bytes); assert!(audio.play(TEST_SOUND).is_ok()); }
    #[test] fn noop_audio_rejects_invalid_wav() { let mut audio = NoopAudio::default(); assert!(audio.register_wav(TEST_SOUND, INVALID_WAV).is_err()); }
    #[test] fn noop_audio_registration_is_idempotent() { let mut audio = NoopAudio::default(); audio.register_wav(TEST_SOUND, VALID_WAV).expect("first register should succeed"); audio.register_wav(TEST_SOUND, INVALID_WAV).expect("same id should already be cached"); }
    #[test] fn noop_audio_reports_unknown_sound() { let mut audio = NoopAudio::default(); assert!(audio.play(TEST_SOUND).is_err()); }
    #[test] fn noop_audio_allows_multiple_plays_of_registered_sound() { let mut audio = NoopAudio::default(); audio.register_wav(TEST_SOUND, VALID_WAV).expect("register should succeed"); assert!(audio.play(TEST_SOUND).is_ok()); assert!(audio.play(TEST_SOUND).is_ok()); }
    #[test] fn different_sound_ids_are_distinct() { let mut audio = NoopAudio::default(); audio.register_wav(TEST_SOUND, VALID_WAV).expect("register should succeed"); assert!(audio.play(OTHER_SOUND).is_err()); }
}
