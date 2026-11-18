use super::*;

pub(super) const WAV: &[u8] = &[
    82, 73, 70, 70, 40, 0, 0, 0, 87, 65, 86, 69, 102, 109, 116, 32, 16, 0, 0, 0, 1, 0, 1, 0, 68,
    172, 0, 0, 136, 88, 1, 0, 2, 0, 16, 0, 100, 97, 116, 97, 4, 0, 0, 0, 0, 0, 255, 127,
];

#[test]
fn noop_rate_is_loop_local_and_preserves_identity_and_mix() {
    let mut audio = NoopAudio::default();
    audio.register_wav(SoundId::new("rate.a"), WAV).unwrap();
    audio.set_master_volume(0.8).unwrap();
    audio.set_bus_volume(AudioBus::Sfx, 0.4).unwrap();
    let a = audio.start_loop(SoundId::new("rate.a")).unwrap();
    let b = audio
        .start_loop_on_bus(SoundId::new("rate.a"), AudioBus::Music)
        .unwrap();
    let original = audio.clone();
    audio.set_loop_playback_rate(a, 1.0).unwrap();
    assert_eq!(audio, original, "initial rate must be 1");
    for rate in [1.5, 0.75] {
        audio.set_loop_playback_rate(a, rate).unwrap();
        let mut other = original.clone();
        other.set_loop_playback_rate(b, rate).unwrap();
        assert_ne!(audio, other, "only the targeted ID changes");
        assert_eq!(audio.loop_playbacks, original.loop_playbacks);
        assert_eq!(audio.loop_buses, original.loop_buses);
        assert_eq!(audio.controls, original.controls);
        assert_eq!(audio.next_playback_id, original.next_playback_id);
    }
    audio.set_loop_playback_rate(a, 1.0).unwrap();
    assert_eq!(audio, original);
    audio.set_bus_muted(AudioBus::Sfx, true).unwrap();
    audio.set_loop_playback_rate(a, 2.0).unwrap();
    audio.set_bus_muted(AudioBus::Sfx, false).unwrap();
    audio.set_loop_playback_rate(a, 1.0).unwrap();
    assert_eq!(audio, original);
    audio.stop_loop(a).unwrap();
    audio.stop_loop(b).unwrap();
    assert!(audio.loop_rates.is_empty());
}

#[test]
fn noop_rejects_invalid_rates_and_non_loop_ids_without_mutation() {
    let mut audio = NoopAudio::default();
    audio.register_wav(SoundId::new("rate"), WAV).unwrap();
    let id = audio.start_loop(SoundId::new("rate")).unwrap();
    for rate in [
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        -1.0,
        0.0,
        0.49,
        2.01,
    ] {
        let before = audio.clone();
        assert!(audio.set_loop_playback_rate(id, rate).is_err());
        assert_eq!(audio, before);
    }
    for rate in [0.5, 2.0] {
        audio.set_loop_playback_rate(id, rate).unwrap();
    }
    let file = audio
        .start_file(Path::new("not-opened-by-noop.wav"), AudioBus::Music, 1.0)
        .unwrap();
    audio.stop_loop(id).unwrap();
    for bad in [id, file, PlaybackId::new(0), PlaybackId::new(u64::MAX)] {
        let before = audio.clone();
        assert!(audio.set_loop_playback_rate(bad, 1.5).is_err());
        assert_eq!(audio, before);
    }
    let fresh = audio.start_loop(SoundId::new("rate")).unwrap();
    assert_ne!(fresh, id);
    assert_eq!(audio.loop_rates[&fresh], 1.0);
}

#[test]
fn custom_audio_can_keep_default_unsupported_implementation() {
    struct Legacy;
    impl Audio for Legacy {
        fn register_wav(&mut self, _: SoundId, _: &[u8]) -> Result<(), AudioError> {
            Ok(())
        }
        fn play(&mut self, _: SoundId) -> Result<(), AudioError> {
            Ok(())
        }
    }
    assert!(
        Legacy
            .set_loop_playback_rate(PlaybackId::new(1), 1.5)
            .is_err()
    );
}
