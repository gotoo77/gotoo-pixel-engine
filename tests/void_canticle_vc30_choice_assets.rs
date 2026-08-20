use std::io::{BufReader, Cursor};

const ICONS: [(&str, &[u8]); 15] = [
    (
        "bulwark",
        include_bytes!("../assets/void_canticle/ui/choice/bulwark.png"),
    ),
    (
        "pilgrim",
        include_bytes!("../assets/void_canticle/ui/choice/pilgrim.png"),
    ),
    (
        "wraith",
        include_bytes!("../assets/void_canticle/ui/choice/wraith.png"),
    ),
    (
        "rapid_fire",
        include_bytes!("../assets/void_canticle/ui/choice/rapid_fire.png"),
    ),
    (
        "magnet_field",
        include_bytes!("../assets/void_canticle/ui/choice/magnet_field.png"),
    ),
    (
        "stellar_power",
        include_bytes!("../assets/void_canticle/ui/choice/stellar_power.png"),
    ),
    (
        "xp_hunger",
        include_bytes!("../assets/void_canticle/ui/choice/xp_hunger.png"),
    ),
    (
        "vital_spark",
        include_bytes!("../assets/void_canticle/ui/choice/vital_spark.png"),
    ),
    (
        "core_surge",
        include_bytes!("../assets/void_canticle/ui/choice/core_surge.png"),
    ),
    (
        "piercing_lance",
        include_bytes!("../assets/void_canticle/ui/choice/piercing_lance.png"),
    ),
    (
        "split_volley",
        include_bytes!("../assets/void_canticle/ui/choice/split_volley.png"),
    ),
    (
        "death_nova",
        include_bytes!("../assets/void_canticle/ui/choice/death_nova.png"),
    ),
    (
        "orbitals",
        include_bytes!("../assets/void_canticle/ui/choice/orbitals.png"),
    ),
    (
        "nanite_repair",
        include_bytes!("../assets/void_canticle/ui/choice/nanite_repair.png"),
    ),
    (
        "shield_capacitor",
        include_bytes!("../assets/void_canticle/ui/choice/shield_capacitor.png"),
    ),
];

#[test]
fn vc30_authored_choice_icon_set_is_complete_and_decodable() {
    let manifest = serde_json::from_str::<serde_json::Value>(include_str!(
        "../assets/void_canticle/ui/choice/manifest.json"
    ))
    .expect("VC3.0 choice manifest should parse");
    let entries = manifest
        .as_object()
        .expect("VC3.0 choice manifest should be an object");

    assert_eq!(ICONS.len(), 15);
    assert_eq!(entries.len(), ICONS.len());

    for (slug, bytes) in ICONS {
        let expected_filename = format!("{slug}.png");
        let descriptor = entries
            .get(slug)
            .unwrap_or_else(|| panic!("missing VC3.0 manifest entry for {slug}"));
        assert_eq!(
            descriptor.get("icon").and_then(serde_json::Value::as_str),
            Some(expected_filename.as_str()),
            "{slug} should point at its authored icon"
        );

        let cursor = Cursor::new(bytes);
        let mut decoder = png::Decoder::new(BufReader::new(cursor));
        decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
        let mut reader = decoder
            .read_info()
            .unwrap_or_else(|error| panic!("{slug} authored icon should parse: {error}"));
        let buffer_size = reader
            .output_buffer_size()
            .unwrap_or_else(|| panic!("{slug} authored icon output should fit memory"));
        let mut buffer = vec![0; buffer_size];
        let info = reader
            .next_frame(&mut buffer)
            .unwrap_or_else(|error| panic!("{slug} authored icon should decode: {error}"));

        assert_eq!((info.width, info.height), (72, 72), "{slug}");
        assert!(info.buffer_size() > 0, "{slug}");
    }
}
