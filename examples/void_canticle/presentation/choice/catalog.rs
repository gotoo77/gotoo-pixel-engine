#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Vc27ChoiceArtId {
    Bulwark,
    Pilgrim,
    Wraith,
    RapidFire,
    MagnetField,
    StellarPower,
    XpHunger,
    VitalSpark,
    CoreSurge,
    PiercingLance,
    SplitVolley,
    DeathNova,
    Orbitals,
    NaniteRepair,
    ShieldCapacitor,
}

impl Vc27ChoiceArtId {
    const ALL: [Self; 15] = [
        Self::Bulwark,
        Self::Pilgrim,
        Self::Wraith,
        Self::RapidFire,
        Self::MagnetField,
        Self::StellarPower,
        Self::XpHunger,
        Self::VitalSpark,
        Self::CoreSurge,
        Self::PiercingLance,
        Self::SplitVolley,
        Self::DeathNova,
        Self::Orbitals,
        Self::NaniteRepair,
        Self::ShieldCapacitor,
    ];

    fn from_label(label: &str) -> Option<Self> {
        match label {
            "BULWARK" => Some(Self::Bulwark),
            "PILGRIM" => Some(Self::Pilgrim),
            "WRAITH" => Some(Self::Wraith),
            "RAPID FIRE" => Some(Self::RapidFire),
            "MAGNET FIELD" => Some(Self::MagnetField),
            "STELLAR POWER" => Some(Self::StellarPower),
            "XP HUNGER" => Some(Self::XpHunger),
            "VITAL SPARK" => Some(Self::VitalSpark),
            "CORE SURGE" => Some(Self::CoreSurge),
            "PIERCING LANCE" => Some(Self::PiercingLance),
            "SPLIT VOLLEY" => Some(Self::SplitVolley),
            "DEATH NOVA" => Some(Self::DeathNova),
            "ORBITALS" => Some(Self::Orbitals),
            "NANITE REPAIR" => Some(Self::NaniteRepair),
            "SHIELD CAPACITOR" => Some(Self::ShieldCapacitor),
            _ => None,
        }
    }

    const fn slug(self) -> &'static str {
        match self {
            Self::Bulwark => "bulwark",
            Self::Pilgrim => "pilgrim",
            Self::Wraith => "wraith",
            Self::RapidFire => "rapid_fire",
            Self::MagnetField => "magnet_field",
            Self::StellarPower => "stellar_power",
            Self::XpHunger => "xp_hunger",
            Self::VitalSpark => "vital_spark",
            Self::CoreSurge => "core_surge",
            Self::PiercingLance => "piercing_lance",
            Self::SplitVolley => "split_volley",
            Self::DeathNova => "death_nova",
            Self::Orbitals => "orbitals",
            Self::NaniteRepair => "nanite_repair",
            Self::ShieldCapacitor => "shield_capacitor",
        }
    }

    const fn hover_override_sound(self) -> SoundId {
        match self {
            Self::Bulwark => SoundId::new("void_canticle.ui.choice.bulwark.hover"),
            Self::Pilgrim => SoundId::new("void_canticle.ui.choice.pilgrim.hover"),
            Self::Wraith => SoundId::new("void_canticle.ui.choice.wraith.hover"),
            Self::RapidFire => SoundId::new("void_canticle.ui.choice.rapid_fire.hover"),
            Self::MagnetField => SoundId::new("void_canticle.ui.choice.magnet_field.hover"),
            Self::StellarPower => SoundId::new("void_canticle.ui.choice.stellar_power.hover"),
            Self::XpHunger => SoundId::new("void_canticle.ui.choice.xp_hunger.hover"),
            Self::VitalSpark => SoundId::new("void_canticle.ui.choice.vital_spark.hover"),
            Self::CoreSurge => SoundId::new("void_canticle.ui.choice.core_surge.hover"),
            Self::PiercingLance => SoundId::new("void_canticle.ui.choice.piercing_lance.hover"),
            Self::SplitVolley => SoundId::new("void_canticle.ui.choice.split_volley.hover"),
            Self::DeathNova => SoundId::new("void_canticle.ui.choice.death_nova.hover"),
            Self::Orbitals => SoundId::new("void_canticle.ui.choice.orbitals.hover"),
            Self::NaniteRepair => SoundId::new("void_canticle.ui.choice.nanite_repair.hover"),
            Self::ShieldCapacitor => SoundId::new("void_canticle.ui.choice.shield_capacitor.hover"),
        }
    }

    const fn confirm_override_sound(self) -> SoundId {
        match self {
            Self::Bulwark => SoundId::new("void_canticle.ui.choice.bulwark.confirm"),
            Self::Pilgrim => SoundId::new("void_canticle.ui.choice.pilgrim.confirm"),
            Self::Wraith => SoundId::new("void_canticle.ui.choice.wraith.confirm"),
            Self::RapidFire => SoundId::new("void_canticle.ui.choice.rapid_fire.confirm"),
            Self::MagnetField => SoundId::new("void_canticle.ui.choice.magnet_field.confirm"),
            Self::StellarPower => SoundId::new("void_canticle.ui.choice.stellar_power.confirm"),
            Self::XpHunger => SoundId::new("void_canticle.ui.choice.xp_hunger.confirm"),
            Self::VitalSpark => SoundId::new("void_canticle.ui.choice.vital_spark.confirm"),
            Self::CoreSurge => SoundId::new("void_canticle.ui.choice.core_surge.confirm"),
            Self::PiercingLance => SoundId::new("void_canticle.ui.choice.piercing_lance.confirm"),
            Self::SplitVolley => SoundId::new("void_canticle.ui.choice.split_volley.confirm"),
            Self::DeathNova => SoundId::new("void_canticle.ui.choice.death_nova.confirm"),
            Self::Orbitals => SoundId::new("void_canticle.ui.choice.orbitals.confirm"),
            Self::NaniteRepair => SoundId::new("void_canticle.ui.choice.nanite_repair.confirm"),
            Self::ShieldCapacitor => SoundId::new("void_canticle.ui.choice.shield_capacitor.confirm"),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Vc27ChoiceAssetDescriptor {
    icon: Option<String>,
    hover_sfx: Option<String>,
    confirm_sfx: Option<String>,
}

#[derive(Debug, Default)]
struct Vc27ChoiceCatalog {
    sprites: std::collections::BTreeMap<Vc27ChoiceArtId, Sprite>,
    hover_sfx: std::collections::BTreeMap<Vc27ChoiceArtId, Vec<u8>>,
    confirm_sfx: std::collections::BTreeMap<Vc27ChoiceArtId, Vec<u8>>,
}

impl Vc27ChoiceCatalog {
    fn sprite(&self, id: Vc27ChoiceArtId) -> Option<&Sprite> {
        self.sprites.get(&id)
    }

    fn hover_sound(&self, id: Vc27ChoiceArtId) -> Option<SoundId> {
        self.hover_sfx
            .contains_key(&id)
            .then(|| id.hover_override_sound())
    }

    fn confirm_sound(&self, id: Vc27ChoiceArtId) -> Option<SoundId> {
        self.confirm_sfx
            .contains_key(&id)
            .then(|| id.confirm_override_sound())
    }

    fn insert_sprite(&mut self, id: Vc27ChoiceArtId, sprite: Sprite) {
        self.sprites.insert(id, sprite);
    }

    fn insert_hover_sfx(&mut self, id: Vc27ChoiceArtId, bytes: Vec<u8>) {
        self.hover_sfx.insert(id, bytes);
    }

    fn insert_confirm_sfx(&mut self, id: Vc27ChoiceArtId, bytes: Vec<u8>) {
        self.confirm_sfx.insert(id, bytes);
    }

    fn register_sounds(
        &self,
        sounds: &mut SoundBank,
    ) -> Result<(), gotoo_pixel_engine::AudioError> {
        for id in Vc27ChoiceArtId::ALL {
            if let Some(bytes) = self.hover_sfx.get(&id) {
                sounds.insert_wav(id.hover_override_sound(), bytes.clone())?;
            }
            if let Some(bytes) = self.confirm_sfx.get(&id) {
                sounds.insert_wav(id.confirm_override_sound(), bytes.clone())?;
            }
        }
        Ok(())
    }

    fn len(&self) -> usize {
        self.sprites.len()
    }
}

static VC27_CHOICE_CATALOG: std::sync::OnceLock<Vc27ChoiceCatalog> = std::sync::OnceLock::new();

fn vc27_choice_catalog() -> &'static Vc27ChoiceCatalog {
    VC27_CHOICE_CATALOG.get_or_init(vc27_load_choice_catalog)
}

#[cfg(target_arch = "wasm32")]
fn vc27_load_choice_catalog() -> Vc27ChoiceCatalog {
    Vc27ChoiceCatalog::default()
}

#[cfg(not(target_arch = "wasm32"))]
fn vc27_load_choice_catalog() -> Vc27ChoiceCatalog {
    let root = std::env::var_os("GPE_VC_CHOICE_ASSET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("assets/void_canticle/ui/choice"));
    let Ok(manifest_text) = std::fs::read_to_string(root.join("manifest.json")) else {
        return Vc27ChoiceCatalog::default();
    };
    let Ok(entries) = vc27_choice_manifest_entries(&manifest_text) else {
        return Vc27ChoiceCatalog::default();
    };

    let mut catalog = Vc27ChoiceCatalog::default();
    for id in Vc27ChoiceArtId::ALL {
        let Some(entry) = entries.get(id.slug()) else {
            continue;
        };

        if let Some(filename) = entry.icon.as_deref()
            && let Ok(bytes) = std::fs::read(root.join(filename))
            && let Ok(sprite) = vc27_decode_png_sprite(&bytes)
        {
            catalog.insert_sprite(id, sprite);
        }

        if let Some(filename) = entry.hover_sfx.as_deref()
            && let Ok(bytes) = std::fs::read(root.join(filename))
            && vc27_choice_wav_supported(&bytes)
        {
            catalog.insert_hover_sfx(id, bytes);
        }

        if let Some(filename) = entry.confirm_sfx.as_deref()
            && let Ok(bytes) = std::fs::read(root.join(filename))
            && vc27_choice_wav_supported(&bytes)
        {
            catalog.insert_confirm_sfx(id, bytes);
        }
    }
    catalog
}

fn vc27_choice_manifest_entries(
    manifest_text: &str,
) -> Result<std::collections::BTreeMap<String, Vc27ChoiceAssetDescriptor>, String> {
    let manifest = serde_json::from_str::<serde_json::Value>(manifest_text)
        .map_err(|error| error.to_string())?;
    let entries = manifest
        .as_object()
        .ok_or_else(|| "choice asset manifest must be a JSON object".to_owned())?;

    let mut parsed = std::collections::BTreeMap::new();
    for (key, value) in entries {
        let descriptor = if let Some(icon) = value.as_str() {
            Vc27ChoiceAssetDescriptor {
                icon: Some(icon.to_owned()),
                ..Vc27ChoiceAssetDescriptor::default()
            }
        } else if let Some(object) = value.as_object() {
            Vc27ChoiceAssetDescriptor {
                icon: object
                    .get("icon")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned),
                hover_sfx: object
                    .get("hover_sfx")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned),
                confirm_sfx: object
                    .get("confirm_sfx")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned),
            }
        } else {
            continue;
        };
        parsed.insert(key.clone(), descriptor);
    }

    Ok(parsed)
}

fn vc27_choice_wav_supported(bytes: &[u8]) -> bool {
    const PROBE_SOUND: SoundId = SoundId::new("void_canticle.ui.choice_asset_probe");

    let mut sounds = SoundBank::new();
    if sounds.insert_wav(PROBE_SOUND, bytes.to_vec()).is_err() {
        return false;
    }

    let mut audio = gotoo_pixel_engine::NoopAudio::default();
    sounds.preload(&mut audio).is_ok()
}

fn vc27_decode_png_sprite(bytes: &[u8]) -> Result<Sprite, String> {
    let cursor = std::io::Cursor::new(bytes);
    let mut decoder = png::Decoder::new(std::io::BufReader::new(cursor));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|error| error.to_string())?;
    let buffer_size = reader
        .output_buffer_size()
        .ok_or_else(|| "choice PNG output buffer is too large".to_owned())?;
    let mut buffer = vec![0; buffer_size];
    let info = reader
        .next_frame(&mut buffer)
        .map_err(|error| error.to_string())?;
    let bytes = &buffer[..info.buffer_size()];

    let pixels = match info.color_type {
        png::ColorType::Rgba => bytes
            .chunks_exact(4)
            .map(|rgba| Pixel::rgba(rgba[0], rgba[1], rgba[2], rgba[3]))
            .collect(),
        png::ColorType::Rgb => bytes
            .chunks_exact(3)
            .map(|rgb| Pixel::rgb(rgb[0], rgb[1], rgb[2]))
            .collect(),
        png::ColorType::Grayscale => bytes
            .iter()
            .map(|value| Pixel::rgb(*value, *value, *value))
            .collect(),
        png::ColorType::GrayscaleAlpha => bytes
            .chunks_exact(2)
            .map(|ga| Pixel::rgba(ga[0], ga[0], ga[0], ga[1]))
            .collect(),
        png::ColorType::Indexed => {
            return Err("indexed PNG was not expanded by the decoder".to_owned());
        }
    };

    Sprite::new(info.width, info.height, pixels).map_err(|error| error.to_string())
}

#[cfg(target_arch = "wasm32")]
pub(crate) async fn vc27_preload_choice_catalog_web(
    base_url: &str,
) -> Result<usize, wasm_bindgen::JsValue> {
    use wasm_bindgen::JsCast as _;

    async fn fetch_response(url: &str) -> Result<web_sys::Response, wasm_bindgen::JsValue> {
        let window = web_sys::window()
            .ok_or_else(|| wasm_bindgen::JsValue::from_str("window is unavailable"))?;
        let value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(url)).await?;
        let response = value.dyn_into::<web_sys::Response>()?;
        if response.ok() {
            Ok(response)
        } else {
            Err(wasm_bindgen::JsValue::from_str(&format!(
                "HTTP {} while loading {url}",
                response.status()
            )))
        }
    }

    async fn fetch_text(url: &str) -> Result<String, wasm_bindgen::JsValue> {
        let response = fetch_response(url).await?;
        let value = wasm_bindgen_futures::JsFuture::from(response.text()?).await?;
        value
            .as_string()
            .ok_or_else(|| wasm_bindgen::JsValue::from_str("response text is not a string"))
    }

    async fn fetch_bytes(url: &str) -> Result<Vec<u8>, wasm_bindgen::JsValue> {
        let response = fetch_response(url).await?;
        let value = wasm_bindgen_futures::JsFuture::from(response.array_buffer()?).await?;
        Ok(js_sys::Uint8Array::new(&value).to_vec())
    }

    let base_url = base_url.trim_end_matches('/');
    let manifest_url = format!("{base_url}/manifest.json");
    let manifest_text = fetch_text(&manifest_url).await?;
    let entries = vc27_choice_manifest_entries(&manifest_text)
        .map_err(|error| wasm_bindgen::JsValue::from_str(&error))?;

    let mut catalog = Vc27ChoiceCatalog::default();
    for id in Vc27ChoiceArtId::ALL {
        let Some(entry) = entries.get(id.slug()) else {
            continue;
        };

        if let Some(filename) = entry.icon.as_deref() {
            let url = format!("{base_url}/{filename}");
            if let Ok(bytes) = fetch_bytes(&url).await
                && let Ok(sprite) = vc27_decode_png_sprite(&bytes)
            {
                catalog.insert_sprite(id, sprite);
            }
        }

        if let Some(filename) = entry.hover_sfx.as_deref() {
            let url = format!("{base_url}/{filename}");
            if let Ok(bytes) = fetch_bytes(&url).await
                && vc27_choice_wav_supported(&bytes)
            {
                catalog.insert_hover_sfx(id, bytes);
            }
        }

        if let Some(filename) = entry.confirm_sfx.as_deref() {
            let url = format!("{base_url}/{filename}");
            if let Ok(bytes) = fetch_bytes(&url).await
                && vc27_choice_wav_supported(&bytes)
            {
                catalog.insert_confirm_sfx(id, bytes);
            }
        }
    }

    let loaded = catalog.len();
    let _ = VC27_CHOICE_CATALOG.set(catalog);
    Ok(loaded)
}

#[cfg(test)]
mod choice_catalog_tests {
    use super::*;

    #[test]
    fn every_known_choice_art_id_has_a_manifest_entry() {
        let entries = vc27_choice_manifest_entries(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/void_canticle/ui/choice/manifest.json"
        )))
        .expect("choice asset manifest should be valid");

        for id in Vc27ChoiceArtId::ALL {
            assert!(entries.contains_key(id.slug()), "missing {}", id.slug());
        }
    }

    #[test]
    fn manifest_parser_preserves_legacy_string_entries() {
        let entries = vc27_choice_manifest_entries(
            r#"{"death_nova":"death_nova.png","ignored":12}"#,
        )
        .expect("valid manifest");
        let death_nova = entries.get("death_nova").expect("legacy entry");

        assert_eq!(death_nova.icon.as_deref(), Some("death_nova.png"));
        assert_eq!(death_nova.hover_sfx, None);
        assert_eq!(death_nova.confirm_sfx, None);
        assert!(!entries.contains_key("ignored"));
    }

    #[test]
    fn manifest_parser_accepts_complete_and_partial_descriptors() {
        let entries = vc27_choice_manifest_entries(
            r#"{
                "death_nova": {
                    "icon": "death_nova.png",
                    "hover_sfx": "death_nova_hover.wav",
                    "confirm_sfx": "death_nova_confirm.wav"
                },
                "vital_spark": {
                    "hover_sfx": "vital_spark_hover.wav",
                    "confirm_sfx": 12
                }
            }"#,
        )
        .expect("valid manifest");

        assert_eq!(
            entries.get("death_nova"),
            Some(&Vc27ChoiceAssetDescriptor {
                icon: Some("death_nova.png".to_owned()),
                hover_sfx: Some("death_nova_hover.wav".to_owned()),
                confirm_sfx: Some("death_nova_confirm.wav".to_owned()),
            })
        );
        assert_eq!(
            entries.get("vital_spark"),
            Some(&Vc27ChoiceAssetDescriptor {
                icon: None,
                hover_sfx: Some("vital_spark_hover.wav".to_owned()),
                confirm_sfx: None,
            })
        );
    }

    #[test]
    fn catalog_can_override_one_art_id_without_touching_other_choices() {
        let mut catalog = Vc27ChoiceCatalog::default();
        let sprite = Sprite::new(1, 1, vec![Pixel::WHITE]).expect("valid test sprite");
        catalog.insert_sprite(Vc27ChoiceArtId::DeathNova, sprite);
        assert!(catalog.sprite(Vc27ChoiceArtId::DeathNova).is_some());
        assert!(catalog.sprite(Vc27ChoiceArtId::VitalSpark).is_none());
    }

    #[test]
    fn catalog_audio_overrides_have_stable_per_choice_sound_ids() {
        let wav = pcm16_mono_wav(44_100, &[0, 1, -1, 0]).expect("valid test WAV");
        let mut catalog = Vc27ChoiceCatalog::default();
        catalog.insert_hover_sfx(Vc27ChoiceArtId::DeathNova, wav.clone());
        catalog.insert_confirm_sfx(Vc27ChoiceArtId::DeathNova, wav);

        assert_eq!(
            catalog.hover_sound(Vc27ChoiceArtId::DeathNova),
            Some(Vc27ChoiceArtId::DeathNova.hover_override_sound())
        );
        assert_eq!(
            catalog.confirm_sound(Vc27ChoiceArtId::DeathNova),
            Some(Vc27ChoiceArtId::DeathNova.confirm_override_sound())
        );
        assert_eq!(catalog.hover_sound(Vc27ChoiceArtId::VitalSpark), None);
        assert_ne!(
            Vc27ChoiceArtId::DeathNova.hover_override_sound(),
            Vc27ChoiceArtId::VitalSpark.hover_override_sound()
        );
    }

    #[test]
    fn choice_wav_validation_rejects_invalid_audio_and_accepts_supported_pcm() {
        assert!(!vc27_choice_wav_supported(b"not a wav"));

        let wav = pcm16_mono_wav(48_000, &[0, 100, -100, 0]).expect("valid test WAV");
        assert!(vc27_choice_wav_supported(&wav));
    }

    #[test]
    fn png_decoder_preserves_transparency() {
        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, 1, 1);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().expect("test PNG header");
            writer
                .write_image_data(&[12, 34, 56, 0])
                .expect("test PNG pixels");
        }
        let sprite = vc27_decode_png_sprite(&bytes).expect("decode test PNG");
        assert_eq!(sprite.pixels(), &[Pixel::rgba(12, 34, 56, 0)]);
    }
}
