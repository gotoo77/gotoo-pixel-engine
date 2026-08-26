use gotoo_pixel_engine::{Audio, AudioBus, NoopAudio};

#[test]
fn public_audio_control_api_composes_master_and_bus_gain() {
    let mut audio = NoopAudio::default();

    audio
        .set_master_volume(0.8)
        .expect("master volume should be accepted");
    audio
        .set_bus_volume(AudioBus::Music, 0.5)
        .expect("music bus volume should be accepted");

    assert!((audio.effective_gain(AudioBus::Music) - 0.4).abs() < f32::EPSILON);
}

#[test]
fn public_audio_control_api_preserves_volume_across_mute() {
    let mut audio = NoopAudio::default();

    audio
        .set_bus_volume(AudioBus::Ui, 0.7)
        .expect("UI bus volume should be accepted");
    audio
        .set_bus_muted(AudioBus::Ui, true)
        .expect("UI bus mute should be accepted");

    assert_eq!(audio.bus_volume(AudioBus::Ui), 0.7);
    assert_eq!(audio.effective_gain(AudioBus::Ui), 0.0);

    audio
        .set_bus_muted(AudioBus::Ui, false)
        .expect("UI bus unmute should be accepted");

    assert_eq!(audio.bus_volume(AudioBus::Ui), 0.7);
    assert_eq!(audio.effective_gain(AudioBus::Ui), 0.7);
}
