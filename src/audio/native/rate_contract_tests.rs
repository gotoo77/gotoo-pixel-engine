use super::*;
use crate::audio::loop_rate_tests::WAV;

#[test]
fn no_device_native_keeps_logical_rate_and_cleans_up() {
    let mut audio = NativeAudio::default();
    audio.register_wav(SoundId::new("rate"), WAV).unwrap();
    let id = audio.start_loop(SoundId::new("rate")).unwrap();
    assert_eq!(audio.active_loops[&id], 1.0);
    for rate in [1.5, 0.75, 1.0, 0.5, 2.0] {
        audio.set_loop_playback_rate(id, rate).unwrap();
        assert_eq!(audio.active_loops[&id], rate);
    }
    let count = audio.next_playback_id;
    assert!(audio.set_loop_playback_rate(id, f32::NAN).is_err());
    assert_eq!(audio.next_playback_id, count);
    audio.stop_loop(id).unwrap();
    assert!(audio.active_loops.is_empty());
    assert!(audio.loop_players.is_empty());
    assert!(audio.set_loop_playback_rate(id, 1.0).is_err());
}

#[test]
fn native_targets_the_same_player_and_rejects_without_mutation() {
    let mut audio = NativeAudio::default();
    audio.register_wav(SoundId::new("rate"), WAV).unwrap();
    let a = audio.start_loop(SoundId::new("rate")).unwrap();
    let b = audio.start_loop(SoundId::new("rate")).unwrap();
    let (pa, _oa) = Player::new();
    let (pb, _ob) = Player::new();
    audio.loop_players.insert(a, pa);
    audio.loop_players.insert(b, pb);
    let pointer = std::ptr::from_ref(&audio.loop_players[&a]);
    let buses = audio.loop_buses.clone();
    let controls = audio.controls.clone();
    let next_id = audio.next_playback_id;
    for rate in [1.5, 0.75, 1.0] {
        audio.set_loop_playback_rate(a, rate).unwrap();
        assert_eq!(audio.loop_players[&a].speed(), rate);
        assert_eq!(audio.loop_players[&b].speed(), 1.0);
        assert_eq!(std::ptr::from_ref(&audio.loop_players[&a]), pointer);
        assert_eq!(audio.loop_buses, buses);
        assert_eq!(audio.controls, controls);
        assert_eq!(audio.next_playback_id, next_id);
    }
    for rate in [
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        -1.0,
        0.0,
        0.49,
        2.01,
    ] {
        assert!(audio.set_loop_playback_rate(a, rate).is_err());
        assert_eq!(audio.loop_players[&a].speed(), 1.0);
    }
    let file = audio
        .start_file(Path::new("not-opened-without-device"), AudioBus::Music, 1.0)
        .unwrap();
    for id in [file, PlaybackId::new(0), PlaybackId::new(u64::MAX)] {
        assert!(audio.set_loop_playback_rate(id, 1.5).is_err());
    }
    audio.stop_loop(a).unwrap();
    audio.stop_loop(b).unwrap();
    assert!(audio.loop_players.is_empty());
}
