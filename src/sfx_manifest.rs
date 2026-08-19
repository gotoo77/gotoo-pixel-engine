use std::collections::BTreeMap;

use crate::AudioError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfxManifest {
    entries: BTreeMap<String, Vec<String>>,
}

impl SfxManifest {
    pub fn parse(json: &str) -> Result<Self, AudioError> {
        let value: serde_json::Value = serde_json::from_str(json)
            .map_err(|error| AudioError::new(format!("invalid SFX manifest JSON: {error}")))?;
        let object = value
            .as_object()
            .ok_or_else(|| AudioError::new("invalid SFX manifest JSON: root must be an object"))?;

        let mut entries = BTreeMap::new();
        for (key, value) in object {
            if key.trim().is_empty() {
                return Err(AudioError::new("SFX manifest keys must not be empty"));
            }

            let paths = match value {
                serde_json::Value::String(path) => vec![validate_path(key, path)?.to_string()],
                serde_json::Value::Array(values) => {
                    if values.is_empty() {
                        return Err(AudioError::new(format!(
                            "SFX manifest entry '{key}' must contain at least one WAV path"
                        )));
                    }
                    values
                        .iter()
                        .map(|value| {
                            let path = value.as_str().ok_or_else(|| {
                                AudioError::new(format!(
                                    "SFX manifest entry '{key}' must contain only file path strings"
                                ))
                            })?;
                            validate_path(key, path).map(str::to_string)
                        })
                        .collect::<Result<Vec<_>, _>>()?
                }
                _ => {
                    return Err(AudioError::new(format!(
                        "SFX manifest entry '{key}' must be a WAV path string or an array of WAV paths"
                    )));
                }
            };

            entries.insert(key.clone(), paths);
        }

        Ok(Self { entries })
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn paths(&self, key: &str) -> Option<&[String]> {
        self.entries.get(key).map(Vec::as_slice)
    }

    pub fn path(&self, key: &str) -> Result<&str, AudioError> {
        let paths = self.entries.get(key).ok_or_else(|| {
            AudioError::new(format!("missing required SFX manifest entry '{key}'"))
        })?;
        if paths.len() != 1 {
            return Err(AudioError::new(format!(
                "SFX manifest entry '{key}' must contain exactly one WAV path"
            )));
        }
        Ok(paths[0].as_str())
    }

    pub fn require_keys(&self, keys: &[&str]) -> Result<(), AudioError> {
        for key in keys {
            if !self.entries.contains_key(*key) {
                return Err(AudioError::new(format!(
                    "missing required SFX manifest entry '{key}'"
                )));
            }
        }
        Ok(())
    }
}

fn validate_path<'a>(key: &str, path: &'a str) -> Result<&'a str, AudioError> {
    if path.trim().is_empty() {
        return Err(AudioError::new(format!(
            "SFX manifest entry '{key}' contains an empty path"
        )));
    }
    if !path.to_ascii_lowercase().ends_with(".wav") {
        return Err(AudioError::new(format!(
            "unsupported audio format for SFX manifest entry '{key}': '{path}'"
        )));
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_and_variant_wav_paths() {
        let manifest = SfxManifest::parse(
            r#"{
                "hit": "assets/hit.wav",
                "death": ["assets/death1.wav", "assets/death2.wav"]
            }"#,
        )
        .expect("valid manifest should parse");

        assert_eq!(manifest.path("hit").unwrap(), "assets/hit.wav");
        assert_eq!(
            manifest.paths("death").unwrap(),
            &["assets/death1.wav".to_string(), "assets/death2.wav".to_string()]
        );
    }

    #[test]
    fn reports_missing_required_key() {
        let manifest = SfxManifest::parse(r#"{"hit":"assets/hit.wav"}"#).unwrap();
        let error = manifest
            .require_keys(&["hit", "death"])
            .expect_err("missing key should fail");
        assert!(error.to_string().contains("death"));
    }

    #[test]
    fn rejects_non_wav_paths() {
        let error = SfxManifest::parse(r#"{"hit":"assets/hit.mp3"}"#)
            .expect_err("unsupported format should fail");
        assert!(error.to_string().contains("unsupported audio format"));
    }

    #[test]
    fn rejects_empty_variant_lists() {
        let error = SfxManifest::parse(r#"{"death":[]}"#)
            .expect_err("empty variant list should fail");
        assert!(error.to_string().contains("at least one WAV path"));
    }
}
