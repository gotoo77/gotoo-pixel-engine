#![cfg(not(target_arch = "wasm32"))]

use std::fs::File;
use std::path::{Path, PathBuf};

use rodio::{Decoder, Source};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("audio")
        .join(name)
}

fn assert_mp3_decodes(name: &str, expected_sample_rate: u32) {
    let file = File::open(fixture(name)).expect("MP3 fixture should open");
    let mut decoder = Decoder::try_from(file).expect("MP3 fixture should decode through rodio");

    assert_eq!(decoder.sample_rate().get(), expected_sample_rate);
    assert_eq!(decoder.channels().get(), 1);
    assert!(decoder.by_ref().take(64).count() > 0);
}

#[test]
fn decodes_cbr_44100_mp3_without_audio_device() {
    assert_mp3_decodes("cbr_44100.mp3", 44_100);
}

#[test]
fn decodes_vbr_48000_mp3_without_audio_device() {
    assert_mp3_decodes("vbr_48000.mp3", 48_000);
}
