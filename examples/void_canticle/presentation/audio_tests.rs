mod authored_choice_audio_tests {
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
                "rapid_fire_hover",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/void_canticle/ui/choice/rapid_fire_hover.wav"
                )),
            ),
            (
                "rapid_fire_confirm",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/void_canticle/ui/choice/rapid_fire_confirm.wav"
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
            (
                "nanite_repair_hover",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/void_canticle/ui/choice/nanite_repair_hover.wav"
                )),
            ),
            (
                "nanite_repair_confirm",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/void_canticle/ui/choice/nanite_repair_confirm.wav"
                )),
            ),
        ] {
            assert!(
                vc27_choice_wav_supported(bytes),
                "{name} authored SFX should use supported GPE WAV encoding"
            );
        }
    }

    #[test]
    fn authored_audio_covers_each_choice_family() {
        let entries = vc27_choice_manifest_entries(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/void_canticle/ui/choice/manifest.json"
        )))
        .expect("choice manifest should parse");

        for slug in ["bulwark", "rapid_fire", "death_nova", "nanite_repair"] {
            let entry = entries
                .get(slug)
                .unwrap_or_else(|| panic!("missing {slug}"));
            assert!(entry.hover_sfx.is_some(), "{slug} hover override");
            assert!(entry.confirm_sfx.is_some(), "{slug} confirm override");
        }
    }
}
