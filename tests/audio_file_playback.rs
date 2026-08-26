use std::path::Path;

use gotoo_pixel_engine::{Audio, AudioBus, NoopAudio, PlaybackState};

#[test]
fn noop_file_playback_exposes_non_fatal_blocked_state() {
    let mut audio = NoopAudio::default();
    let playback = audio
        .start_file(Path::new("missing.mp3"), AudioBus::Music, 0.75)
        .expect("noop backend should keep file playback non-fatal");

    assert_eq!(audio.playback_state(playback), Some(PlaybackState::Blocked));
    assert!(audio.pause_playback(playback).is_ok());
    assert!(audio.resume_playback(playback).is_ok());
    assert!(audio.take_finished_playbacks().is_empty());
    assert!(audio.stop_playback(playback).is_ok());
    assert_eq!(audio.playback_state(playback), None);
}

#[test]
fn file_playback_gain_uses_same_unit_interval_contract() {
    let mut audio = NoopAudio::default();

    assert!(
        audio
            .start_file(Path::new("ignored.mp3"), AudioBus::Music, -0.01)
            .is_err()
    );
    assert!(
        audio
            .start_file(Path::new("ignored.mp3"), AudioBus::Music, 1.01)
            .is_err()
    );
}
