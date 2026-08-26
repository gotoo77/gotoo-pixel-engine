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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioBus {
    Music,
    Sfx,
    Ui,
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

    fn play_on_bus(&mut self, id: SoundId, _bus: AudioBus) -> Result<(), AudioError> {
        self.play(id)
    }

    fn start_loop(&mut self, id: SoundId) -> Result<PlaybackId, AudioError> {
        Err(AudioError::new(format!(
            "looped playback is not supported for sound '{}'",
            id.as_str()
        )))
    }

    fn start_loop_on_bus(&mut self, id: SoundId, _bus: AudioBus) -> Result<PlaybackId, AudioError> {
        self.start_loop(id)
    }

    fn stop_loop(&mut self, playback: PlaybackId) -> Result<(), AudioError> {
        Err(AudioError::new(format!(
            "looped playback '{}' is not active",
            playback.0
        )))
    }

    fn set_master_volume(&mut self, _volume: f32) -> Result<(), AudioError> {
        Err(AudioError::new("master volume control is not supported"))
    }

    fn master_volume(&self) -> f32 {
        1.0
    }

    fn set_master_muted(&mut self, _muted: bool) -> Result<(), AudioError> {
        Err(AudioError::new("master mute control is not supported"))
    }

    fn master_muted(&self) -> bool {
        false
    }

    fn set_bus_volume(&mut self, _bus: AudioBus, _volume: f32) -> Result<(), AudioError> {
        Err(AudioError::new("audio bus volume control is not supported"))
    }

    fn bus_volume(&self, _bus: AudioBus) -> f32 {
        1.0
    }

    fn set_bus_muted(&mut self, _bus: AudioBus, _muted: bool) -> Result<(), AudioError> {
        Err(AudioError::new("audio bus mute control is not supported"))
    }

    fn bus_muted(&self, _bus: AudioBus) -> bool {
        false
    }

    fn effective_gain(&self, bus: AudioBus) -> f32 {
        let master = if self.master_muted() {
            0.0
        } else {
            self.master_volume()
        };
        let bus_gain = if self.bus_muted(bus) {
            0.0
        } else {
            self.bus_volume(bus)
        };
        master * bus_gain
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct GainControl {
    volume: f32,
    muted: bool,
}

impl Default for GainControl {
    fn default() -> Self {
        Self {
            volume: 1.0,
            muted: false,
        }
    }
}

impl GainControl {
    fn gain(self) -> f32 {
        if self.muted { 0.0 } else { self.volume }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
struct AudioControlState {
    master: GainControl,
    music: GainControl,
    sfx: GainControl,
    ui: GainControl,
}

impl AudioControlState {
    fn bus(&self, bus: AudioBus) -> GainControl {
        match bus {
            AudioBus::Music => self.music,
            AudioBus::Sfx => self.sfx,
            AudioBus::Ui => self.ui,
        }
    }

    fn bus_mut(&mut self, bus: AudioBus) -> &mut GainControl {
        match bus {
            AudioBus::Music => &mut self.music,
            AudioBus::Sfx => &mut self.sfx,
            AudioBus::Ui => &mut self.ui,
        }
    }

    fn set_master_volume(&mut self, volume: f32) -> Result<(), AudioError> {
        self.master.volume = validate_volume(volume)?;
        Ok(())
    }

    fn set_bus_volume(&mut self, bus: AudioBus, volume: f32) -> Result<(), AudioError> {
        self.bus_mut(bus).volume = validate_volume(volume)?;
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn effective_gain(&self, bus: AudioBus) -> f32 {
        self.master.gain() * self.bus(bus).gain()
    }
}

fn validate_volume(volume: f32) -> Result<f32, AudioError> {
    if volume.is_finite() && (0.0..=1.0).contains(&volume) {
        Ok(volume)
    } else {
        Err(AudioError::new(
            "audio volume must be finite and within 0.0..=1.0",
        ))
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

    pub fn play_on_bus(
        &mut self,
        audio: &mut dyn Audio,
        id: SoundId,
        bus: AudioBus,
    ) -> Result<(), AudioError> {
        self.ensure_registered(audio, id)?;
        audio.play_on_bus(id, bus)
    }

    pub fn start_loop(
        &mut self,
        audio: &mut dyn Audio,
        id: SoundId,
    ) -> Result<PlaybackId, AudioError> {
        self.ensure_registered(audio, id)?;
        audio.start_loop(id)
    }

    pub fn start_loop_on_bus(
        &mut self,
        audio: &mut dyn Audio,
        id: SoundId,
        bus: AudioBus,
    ) -> Result<PlaybackId, AudioError> {
        self.ensure_registered(audio, id)?;
        audio.start_loop_on_bus(id, bus)
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
    loop_buses: HashMap<PlaybackId, AudioBus>,
    controls: AudioControlState,
    next_playback_id: u64,
}

impl Audio for NoopAudio {
    fn register_wav(&mut self, id: SoundId, bytes: &[u8]) -> Result<(), AudioError> {
        register_decoded_wav(&mut self.sounds, id, bytes)
    }

    fn play(&mut self, id: SoundId) -> Result<(), AudioError> {
        self.play_on_bus(id, AudioBus::Sfx)
    }

    fn play_on_bus(&mut self, id: SoundId, _bus: AudioBus) -> Result<(), AudioError> {
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
        self.start_loop_on_bus(id, AudioBus::Sfx)
    }

    fn start_loop_on_bus(&mut self, id: SoundId, bus: AudioBus) -> Result<PlaybackId, AudioError> {
        if !self.sounds.contains_key(&id) {
            return Err(AudioError::new(format!(
                "sound '{}' is not registered",
                id.as_str()
            )));
        }
        self.next_playback_id += 1;
        let playback = PlaybackId::new(self.next_playback_id);
        self.loop_playbacks.insert(playback, id);
        self.loop_buses.insert(playback, bus);
        Ok(playback)
    }

    fn stop_loop(&mut self, playback: PlaybackId) -> Result<(), AudioError> {
        if self.loop_playbacks.remove(&playback).is_some() {
            self.loop_buses.remove(&playback);
            Ok(())
        } else {
            Err(AudioError::new(format!(
                "looped playback '{}' is not active",
                playback.0
            )))
        }
    }

    fn set_master_volume(&mut self, volume: f32) -> Result<(), AudioError> {
        self.controls.set_master_volume(volume)
    }

    fn master_volume(&self) -> f32 {
        self.controls.master.volume
    }

    fn set_master_muted(&mut self, muted: bool) -> Result<(), AudioError> {
        self.controls.master.muted = muted;
        Ok(())
    }

    fn master_muted(&self) -> bool {
        self.controls.master.muted
    }

    fn set_bus_volume(&mut self, bus: AudioBus, volume: f32) -> Result<(), AudioError> {
        self.controls.set_bus_volume(bus, volume)
    }

    fn bus_volume(&self, bus: AudioBus) -> f32 {
        self.controls.bus(bus).volume
    }

    fn set_bus_muted(&mut self, bus: AudioBus, muted: bool) -> Result<(), AudioError> {
        self.controls.bus_mut(bus).muted = muted;
        Ok(())
    }

    fn bus_muted(&self, bus: AudioBus) -> bool {
        self.controls.bus(bus).muted
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

    use super::{
        Audio, AudioBus, AudioControlState, AudioError, DecodedSound, PlatformAudio, PlaybackId,
        SoundId, register_decoded_wav,
    };

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
        one_shot_players: Vec<(AudioBus, Player)>,
        active_loops: HashSet<PlaybackId>,
        loop_players: HashMap<PlaybackId, Player>,
        loop_buses: HashMap<PlaybackId, AudioBus>,
        controls: AudioControlState,
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
                ..Self::default()
            })
        }

        fn prune_one_shots(&mut self) {
            self.one_shot_players.retain(|(_, player)| !player.empty());
        }

        fn refresh_active_gains(&mut self) {
            self.prune_one_shots();

            for (bus, player) in &self.one_shot_players {
                player.set_volume(self.controls.effective_gain(*bus));
            }

            for (playback, player) in &self.loop_players {
                if let Some(bus) = self.loop_buses.get(playback) {
                    player.set_volume(self.controls.effective_gain(*bus));
                }
            }
        }
    }

    impl Audio for NativeAudio {
        fn register_wav(&mut self, id: SoundId, bytes: &[u8]) -> Result<(), AudioError> {
            register_decoded_wav(&mut self.sounds, id, bytes)
        }

        fn play(&mut self, id: SoundId) -> Result<(), AudioError> {
            self.play_on_bus(id, AudioBus::Sfx)
        }

        fn play_on_bus(&mut self, id: SoundId, bus: AudioBus) -> Result<(), AudioError> {
            self.prune_one_shots();

            let Some(sound) = self.sounds.get(&id) else {
                return Err(AudioError::new(format!(
                    "sound '{}' is not registered",
                    id.as_str()
                )));
            };
            let Some(sink) = self.sink.as_ref() else {
                return Ok(());
            };

            let player = Player::connect_new(sink.mixer());
            player.set_volume(self.controls.effective_gain(bus));
            player.append(SamplesBuffer::new(
                sound.channels,
                sound.sample_rate,
                sound.samples.to_vec(),
            ));
            self.one_shot_players.push((bus, player));
            Ok(())
        }

        fn start_loop(&mut self, id: SoundId) -> Result<PlaybackId, AudioError> {
            self.start_loop_on_bus(id, AudioBus::Sfx)
        }

        fn start_loop_on_bus(
            &mut self,
            id: SoundId,
            bus: AudioBus,
        ) -> Result<PlaybackId, AudioError> {
            let Some(sound) = self.sounds.get(&id) else {
                return Err(AudioError::new(format!(
                    "sound '{}' is not registered",
                    id.as_str()
                )));
            };

            self.next_playback_id += 1;
            let playback = PlaybackId::new(self.next_playback_id);
            self.active_loops.insert(playback);
            self.loop_buses.insert(playback, bus);

            if let Some(sink) = self.sink.as_ref() {
                let player = Player::connect_new(sink.mixer());
                player.set_volume(self.controls.effective_gain(bus));
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
            self.loop_buses.remove(&playback);
            if let Some(player) = self.loop_players.remove(&playback) {
                player.stop();
            }
            Ok(())
        }

        fn set_master_volume(&mut self, volume: f32) -> Result<(), AudioError> {
            self.controls.set_master_volume(volume)?;
            self.refresh_active_gains();
            Ok(())
        }

        fn master_volume(&self) -> f32 {
            self.controls.master.volume
        }

        fn set_master_muted(&mut self, muted: bool) -> Result<(), AudioError> {
            self.controls.master.muted = muted;
            self.refresh_active_gains();
            Ok(())
        }

        fn master_muted(&self) -> bool {
            self.controls.master.muted
        }

        fn set_bus_volume(&mut self, bus: AudioBus, volume: f32) -> Result<(), AudioError> {
            self.controls.set_bus_volume(bus, volume)?;
            self.refresh_active_gains();
            Ok(())
        }

        fn bus_volume(&self, bus: AudioBus) -> f32 {
            self.controls.bus(bus).volume
        }

        fn set_bus_muted(&mut self, bus: AudioBus, muted: bool) -> Result<(), AudioError> {
            self.controls.bus_mut(bus).muted = muted;
            self.refresh_active_gains();
            Ok(())
        }

        fn bus_muted(&self, bus: AudioBus) -> bool {
            self.controls.bus(bus).muted
        }
    }

    impl PlatformAudio for NativeAudio {}
}

#[cfg(target_arch = "wasm32")]
mod web {
    use std::collections::{HashMap, HashSet};

    use web_sys::{AudioBuffer, AudioBufferSourceNode, AudioContext, AudioContextState, GainNode};

    use super::{
        Audio, AudioBus, AudioControlState, AudioError, DecodedSound, PlatformAudio, PlaybackId,
        SoundId, decode_wav, ensure_same_sound,
    };

    #[derive(Clone)]
    struct WebMix {
        master: GainNode,
        music: GainNode,
        sfx: GainNode,
        ui: GainNode,
    }

    impl WebMix {
        fn bus_node(&self, bus: AudioBus) -> &GainNode {
            match bus {
                AudioBus::Music => &self.music,
                AudioBus::Sfx => &self.sfx,
                AudioBus::Ui => &self.ui,
            }
        }
    }

    #[derive(Default)]
    pub(crate) struct WebAudio {
        context: Option<AudioContext>,
        unavailable: bool,
        sounds: HashMap<SoundId, AudioBuffer>,
        decoded_sounds: HashMap<SoundId, DecodedSound>,
        active_loops: HashSet<PlaybackId>,
        loop_sources: HashMap<PlaybackId, AudioBufferSourceNode>,
        controls: AudioControlState,
        mix: Option<WebMix>,
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

        fn ensure_mix(&mut self) -> Result<WebMix, AudioError> {
            if let Some(mix) = self.mix.as_ref() {
                return Ok(mix.clone());
            }

            let context = self.context()?;
            let master = context.create_gain().map_err(|err| {
                AudioError::new(js_error_message("failed to create master GainNode", err))
            })?;
            let music = context.create_gain().map_err(|err| {
                AudioError::new(js_error_message("failed to create music GainNode", err))
            })?;
            let sfx = context.create_gain().map_err(|err| {
                AudioError::new(js_error_message("failed to create SFX GainNode", err))
            })?;
            let ui = context.create_gain().map_err(|err| {
                AudioError::new(js_error_message("failed to create UI GainNode", err))
            })?;

            master
                .connect_with_audio_node(&context.destination())
                .map_err(|err| {
                    AudioError::new(js_error_message("failed to connect master GainNode", err))
                })?;
            for node in [&music, &sfx, &ui] {
                node.connect_with_audio_node(&master).map_err(|err| {
                    AudioError::new(js_error_message("failed to connect bus GainNode", err))
                })?;
            }

            let mix = WebMix {
                master,
                music,
                sfx,
                ui,
            };
            Self::apply_mix_state(&mix, &self.controls);
            self.mix = Some(mix.clone());
            Ok(mix)
        }

        fn apply_mix_state(mix: &WebMix, controls: &AudioControlState) {
            mix.master.gain().set_value(controls.master.gain());
            mix.music.gain().set_value(controls.music.gain());
            mix.sfx.gain().set_value(controls.sfx.gain());
            mix.ui.gain().set_value(controls.ui.gain());
        }

        fn refresh_mix_gains(&self) {
            if let Some(mix) = self.mix.as_ref() {
                Self::apply_mix_state(mix, &self.controls);
            }
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
            self.play_on_bus(id, AudioBus::Sfx)
        }

        fn play_on_bus(&mut self, id: SoundId, bus: AudioBus) -> Result<(), AudioError> {
            let Some(buffer) = self.sounds.get(&id).cloned() else {
                return Err(AudioError::new(format!(
                    "sound '{}' is not registered",
                    id.as_str()
                )));
            };
            let mix = self.ensure_mix()?;
            let context = self.context()?;
            if context.state() != AudioContextState::Running {
                return Ok(());
            }

            let source = context.create_buffer_source().map_err(|err| {
                AudioError::new(js_error_message(
                    "failed to create AudioBufferSourceNode",
                    err,
                ))
            })?;
            source.set_buffer(Some(&buffer));
            source
                .connect_with_audio_node(mix.bus_node(bus))
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
            self.start_loop_on_bus(id, AudioBus::Sfx)
        }

        fn start_loop_on_bus(
            &mut self,
            id: SoundId,
            bus: AudioBus,
        ) -> Result<PlaybackId, AudioError> {
            let Some(buffer) = self.sounds.get(&id).cloned() else {
                return Err(AudioError::new(format!(
                    "sound '{}' is not registered",
                    id.as_str()
                )));
            };
            let mix = self.ensure_mix()?;
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
                .connect_with_audio_node(mix.bus_node(bus))
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

        fn set_master_volume(&mut self, volume: f32) -> Result<(), AudioError> {
            self.controls.set_master_volume(volume)?;
            self.refresh_mix_gains();
            Ok(())
        }

        fn master_volume(&self) -> f32 {
            self.controls.master.volume
        }

        fn set_master_muted(&mut self, muted: bool) -> Result<(), AudioError> {
            self.controls.master.muted = muted;
            self.refresh_mix_gains();
            Ok(())
        }

        fn master_muted(&self) -> bool {
            self.controls.master.muted
        }

        fn set_bus_volume(&mut self, bus: AudioBus, volume: f32) -> Result<(), AudioError> {
            self.controls.set_bus_volume(bus, volume)?;
            self.refresh_mix_gains();
            Ok(())
        }

        fn bus_volume(&self, bus: AudioBus) -> f32 {
            self.controls.bus(bus).volume
        }

        fn set_bus_muted(&mut self, bus: AudioBus, muted: bool) -> Result<(), AudioError> {
            self.controls.bus_mut(bus).muted = muted;
            self.refresh_mix_gains();
            Ok(())
        }

        fn bus_muted(&self, bus: AudioBus) -> bool {
            self.controls.bus(bus).muted
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
    use super::{Audio, AudioBus, NoopAudio, SoundBank, SoundId, decode_wav};

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
        assert_eq!(audio.loop_buses.get(&playback), Some(&AudioBus::Sfx));
        assert!(audio.stop_loop(playback).is_ok());
        assert!(!audio.loop_playbacks.contains_key(&playback));
        assert!(!audio.loop_buses.contains_key(&playback));
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
    fn sound_bank_routes_clips_to_explicit_bus() {
        let mut bank = SoundBank::new();
        bank.insert_wav(TEST_SOUND, VALID_WAV)
            .expect("bank insert should succeed");
        let mut audio = NoopAudio::default();

        let playback = bank
            .start_loop_on_bus(&mut audio, TEST_SOUND, AudioBus::Ui)
            .expect("bank should route loop to UI bus");

        assert_eq!(audio.loop_buses.get(&playback), Some(&AudioBus::Ui));
        assert!(
            bank.play_on_bus(&mut audio, TEST_SOUND, AudioBus::Ui)
                .is_ok()
        );
    }

    #[test]
    fn control_plane_defaults_to_full_unmuted_gain() {
        let audio = NoopAudio::default();

        assert_eq!(audio.master_volume(), 1.0);
        assert!(!audio.master_muted());
        assert_eq!(audio.bus_volume(AudioBus::Music), 1.0);
        assert!(!audio.bus_muted(AudioBus::Music));
        assert_eq!(audio.effective_gain(AudioBus::Music), 1.0);
    }

    #[test]
    fn control_plane_composes_master_and_bus_gain() {
        let mut audio = NoopAudio::default();

        audio
            .set_master_volume(0.8)
            .expect("master volume should be valid");
        audio
            .set_bus_volume(AudioBus::Music, 0.7)
            .expect("music volume should be valid");

        assert!((audio.effective_gain(AudioBus::Music) - 0.56).abs() < f32::EPSILON);
    }

    #[test]
    fn mute_does_not_destroy_remembered_volume() {
        let mut audio = NoopAudio::default();

        audio
            .set_bus_volume(AudioBus::Music, 0.65)
            .expect("music volume should be valid");
        audio
            .set_bus_muted(AudioBus::Music, true)
            .expect("music mute should be supported");

        assert_eq!(audio.bus_volume(AudioBus::Music), 0.65);
        assert_eq!(audio.effective_gain(AudioBus::Music), 0.0);

        audio
            .set_bus_muted(AudioBus::Music, false)
            .expect("music unmute should be supported");

        assert_eq!(audio.bus_volume(AudioBus::Music), 0.65);
        assert_eq!(audio.effective_gain(AudioBus::Music), 0.65);
    }

    #[test]
    fn control_plane_rejects_invalid_volume() {
        let mut audio = NoopAudio::default();

        assert!(audio.set_master_volume(-0.1).is_err());
        assert!(audio.set_master_volume(1.1).is_err());
        assert!(audio.set_bus_volume(AudioBus::Sfx, f32::NAN).is_err());
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
