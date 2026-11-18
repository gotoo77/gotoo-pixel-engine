//! Opt-in signal capture through the actual NativeAudio implementation.
//! The marker is test instrumentation on the same existing device mixer.
use super::*;
use rodio::{ChannelCount, SampleRate};
use std::{
    io::{self, BufRead, Write},
    sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn unix_ns() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}
struct Clock {
    sequence: AtomicUsize,
    frames: [AtomicU64; 32],
    times: [AtomicU64; 32],
}
struct Marker {
    clock: Arc<Clock>,
    rate: SampleRate,
    frame: u64,
    channel: usize,
    sequence: usize,
    pulse: usize,
    value: f32,
}
impl Iterator for Marker {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.channel == 0 {
            let sequence = self.clock.sequence.load(Ordering::Acquire);
            if sequence != self.sequence {
                self.sequence = sequence;
                self.pulse = 0;
                self.clock.frames[sequence].store(self.frame, Ordering::Release);
                self.clock.times[sequence].store(unix_ns(), Ordering::Release);
            }
            self.value = if self.pulse < 128 {
                0.08 * (std::f32::consts::PI * self.pulse as f32 / 127.0)
                    .sin()
                    .powi(2)
            } else {
                0.0
            };
            self.pulse += 1;
            self.frame += 1;
        }
        let value = if self.channel == 0 {
            self.value
        } else {
            -self.value
        };
        self.channel = (self.channel + 1) % 2;
        Some(value)
    }
}
impl Source for Marker {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> ChannelCount {
        ChannelCount::new(2).unwrap()
    }
    fn sample_rate(&self) -> SampleRate {
        self.rate
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

fn wav(channels: u16, frequency: f32, level: f32, opposed: bool) -> Vec<u8> {
    let mut bytes = io::Cursor::new(Vec::new());
    let spec = hound::WavSpec {
        channels,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    {
        let mut writer = hound::WavWriter::new(&mut bytes, spec).unwrap();
        for frame in 0..44100 {
            let value = level * (std::f32::consts::TAU * frequency * frame as f32 / 44100.0).sin();
            for channel in 0..channels {
                let sign = if opposed && channel == 1 { -1.0 } else { 1.0 };
                writer
                    .write_sample((value * sign * 32767.0).round() as i16)
                    .unwrap();
            }
        }
        writer.finalize().unwrap();
    }
    bytes.into_inner()
}

fn set_rate(audio: &mut NativeAudio, playback: PlaybackId, rate: f32) -> Result<(), AudioError> {
    audio.set_loop_playback_rate(playback, rate)
}

#[test]
#[ignore = "requires a native device and synchronized monitor capture; see loop-rate investigation"]
fn capture() {
    let count: usize = std::env::var("GPE_RATE_PROBE_COUNT")
        .unwrap_or("12".into())
        .parse()
        .unwrap();
    let channels: u16 = std::env::var("GPE_RATE_PROBE_CHANNELS")
        .unwrap_or("1".into())
        .parse()
        .unwrap();
    assert!(count < 30);
    assert!(channels == 1 || channels == 2);
    let mut audio = NativeAudio::new(
        #[cfg(feature = "diagnostics")]
        None,
    )
    .expect("real native device required; no Noop fallback");
    let clock = Arc::new(Clock {
        sequence: AtomicUsize::new(0),
        frames: std::array::from_fn(|_| AtomicU64::new(0)),
        times: std::array::from_fn(|_| AtomicU64::new(0)),
    });
    let sink = audio.sink.as_ref().unwrap();
    assert_eq!(sink.config().channel_count().get(), 2);
    let output_rate = sink.config().sample_rate();
    sink.mixer().add(Marker {
        clock: clock.clone(),
        rate: output_rate,
        frame: 0,
        channel: 0,
        sequence: 0,
        pulse: 128,
        value: 0.0,
    });
    let target_sound = SoundId::new("rate.probe.target");
    let sentinel_sound = SoundId::new("rate.probe.sentinel");
    let motor = std::env::var_os("GPE_RATE_PROBE_WAV");
    let bytes = motor.as_ref().map_or_else(
        || wav(channels, 880.0, 0.05, false),
        |path| std::fs::read(path).unwrap(),
    );
    audio.register_wav(target_sound, &bytes).unwrap();
    assert_eq!(audio.sounds[&target_sound].channels.get(), channels);
    audio
        .register_wav(sentinel_sound, &wav(2, 4000.0, 0.005, true))
        .unwrap();
    let sentinel = audio
        .start_loop_on_bus(sentinel_sound, AudioBus::Music)
        .unwrap();
    println!(
        "{{\"kind\":\"ready\",\"pid\":{},\"output_rate\":{},\"channels\":2,\"source_channels\":{},\"fixture\":\"{}\",\"backend\":\"GPE NativeAudio\"}}",
        std::process::id(),
        output_rate.get(),
        channels,
        if motor.is_some() { "motor" } else { "sine" }
    );
    io::stdout().flush().unwrap();
    let mut command = String::new();
    io::stdin().lock().read_line(&mut command).unwrap();
    assert_eq!(command.trim(), "GO");
    let target = audio.start_loop(target_sound).unwrap();
    let pointer = std::ptr::from_ref(&audio.loop_players[&target]);
    let next_id = audio.next_playback_id;
    let mix = audio.controls.clone();
    let buses = audio.loop_buses.clone();
    let initial = unix_ns();
    clock.sequence.store(1, Ordering::Release);
    println!(
        "{{\"kind\":\"start\",\"sequence\":1,\"command_before_ns\":{},\"command_after_ns\":{},\"rate\":1.0,\"playback_id\":{},\"player_address\":{}}}",
        initial,
        unix_ns(),
        target.0,
        pointer as usize
    );
    io::stdout().flush().unwrap();
    let delays = [
        1650, 1830, 1720, 2020, 1740, 1930, 1690, 1880, 1770, 1990, 1710, 1840,
    ];
    for event in 0..count {
        std::thread::sleep(Duration::from_millis(delays[event % delays.len()]));
        let rate = [1.5, 0.75, 1.0][event % 3];
        let before = unix_ns();
        set_rate(&mut audio, target, rate).unwrap();
        let after = unix_ns();
        clock.sequence.store(event + 2, Ordering::Release);
        assert_eq!(audio.loop_players[&target].speed(), rate);
        assert_eq!(audio.loop_players[&sentinel].speed(), 1.0);
        assert_eq!(std::ptr::from_ref(&audio.loop_players[&target]), pointer);
        assert_eq!(audio.loop_players.len(), 2);
        assert_eq!(audio.next_playback_id, next_id);
        assert_eq!(audio.controls, mix);
        assert_eq!(audio.loop_buses, buses);
        println!(
            "{{\"kind\":\"command\",\"sequence\":{},\"rate\":{},\"command_before_ns\":{},\"command_after_ns\":{},\"playback_id\":{},\"player_address\":{}}}",
            event + 2,
            rate,
            before,
            after,
            target.0,
            pointer as usize
        );
        io::stdout().flush().unwrap();
    }
    std::thread::sleep(Duration::from_millis(2200));
    for sequence in 1..count + 2 {
        println!(
            "{{\"kind\":\"marker\",\"sequence\":{},\"render_frame\":{},\"render_unix_ns\":{}}}",
            sequence,
            clock.frames[sequence].load(Ordering::Acquire),
            clock.times[sequence].load(Ordering::Acquire)
        );
    }
    audio.stop_loop(target).unwrap();
    audio.stop_loop(sentinel).unwrap();
    assert!(set_rate(&mut audio, target, 1.0).is_err());
    assert!(audio.active_loops.is_empty());
    assert!(audio.loop_players.is_empty());
    assert!(audio.loop_buses.is_empty());
    println!(
        "{{\"kind\":\"done\",\"target_players_created\":1,\"target_appends\":1,\"sentinel_preserved\":true,\"cleanup\":true}}"
    );
}
