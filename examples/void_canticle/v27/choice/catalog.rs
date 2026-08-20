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
}

#[derive(Debug, Default)]
struct Vc27ChoiceCatalog {
    sprites: std::collections::BTreeMap<Vc27ChoiceArtId, Sprite>,
}

impl Vc27ChoiceCatalog {
    fn sprite(&self, id: Vc27ChoiceArtId) -> Option<&Sprite> {
        self.sprites.get(&id)
    }

    fn insert(&mut self, id: Vc27ChoiceArtId, sprite: Sprite) {
        self.sprites.insert(id, sprite);
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
        let Some(filename) = entries.get(id.slug()) else {
            continue;
        };
        let Ok(bytes) = std::fs::read(root.join(filename)) else {
            continue;
        };
        let Ok(sprite) = vc27_decode_png_sprite(&bytes) else {
            continue;
        };
        catalog.insert(id, sprite);
    }
    catalog
}

fn vc27_choice_manifest_entries(
    manifest_text: &str,
) -> Result<std::collections::BTreeMap<String, String>, String> {
    let manifest = serde_json::from_str::<serde_json::Value>(manifest_text)
        .map_err(|error| error.to_string())?;
    let entries = manifest
        .as_object()
        .ok_or_else(|| "choice asset manifest must be a JSON object".to_owned())?;
    Ok(entries
        .iter()
        .filter_map(|(key, value)| value.as_str().map(|path| (key.clone(), path.to_owned())))
        .collect())
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
pub(crate) async fn vc27_preload_choice_catalog_web(base_url: &str) -> Result<usize, wasm_bindgen::JsValue> {
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
        let Some(filename) = entries.get(id.slug()) else {
            continue;
        };
        let url = format!("{base_url}/{filename}");
        let Ok(bytes) = fetch_bytes(&url).await else {
            continue;
        };
        let Ok(sprite) = vc27_decode_png_sprite(&bytes) else {
            continue;
        };
        catalog.insert(id, sprite);
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
        let manifest = serde_json::from_str::<serde_json::Value>(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/void_canticle/ui/choice/manifest.json"
        )))
        .expect("choice asset manifest should be valid JSON");
        let entries = manifest
            .as_object()
            .expect("choice asset manifest should be a JSON object");
        for id in Vc27ChoiceArtId::ALL {
            assert!(entries.contains_key(id.slug()), "missing {}", id.slug());
        }
    }

    #[test]
    fn manifest_parser_keeps_string_entries_only() {
        let entries = vc27_choice_manifest_entries(
            r#"{"death_nova":"death_nova.png","ignored":12}"#,
        )
        .expect("valid manifest");
        assert_eq!(entries.get("death_nova").map(String::as_str), Some("death_nova.png"));
        assert!(!entries.contains_key("ignored"));
    }

    #[test]
    fn catalog_can_override_one_art_id_without_touching_other_choices() {
        let mut catalog = Vc27ChoiceCatalog::default();
        let sprite = Sprite::new(1, 1, vec![Pixel::WHITE]).expect("valid test sprite");
        catalog.insert(Vc27ChoiceArtId::DeathNova, sprite);
        assert!(catalog.sprite(Vc27ChoiceArtId::DeathNova).is_some());
        assert!(catalog.sprite(Vc27ChoiceArtId::VitalSpark).is_none());
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
