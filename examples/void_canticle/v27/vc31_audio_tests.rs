mod vc31_authored_audio_tests {
    use super::*;

    #[test]
    fn checked_in_authored_choice_audio_uses_supported_wav_encoding() {
        for (name, bytes) in [
            (
                "bulwark_hover",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/void_canticle/ui/choice/bulwark_hover.wav"
                )) as &[u8],
            ),
            (
                "bulwark_confirm",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/void_canticle/ui/choice/bulwark_confirm.wav"
                )),
            ),
            (
                "death_nova_hover",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/void_canticle/ui/choice/death_nova_hover.wav"
                )),
            ),
            (
                "death_nova_confirm",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/void_canticle/ui/choice/death_nova_confirm.wav"
                )),
            ),
        ] {
            assert!(
                vc27_choice_wav_supported(bytes),
                "{name} authored SFX should use supported GPE WAV encoding"
            );
        }
    }
}
