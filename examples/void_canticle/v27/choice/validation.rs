const VC29_CHOICE_ASSET_FIELDS: [&str; 3] = ["icon", "hover_sfx", "confirm_sfx"];

fn vc29_validate_choice_manifest(manifest_text: &str) -> Result<(), String> {
    let manifest = serde_json::from_str::<serde_json::Value>(manifest_text)
        .map_err(|error| format!("invalid JSON: {error}"))?;
    let entries = manifest
        .as_object()
        .ok_or_else(|| "choice asset manifest must be a JSON object".to_owned())?;

    let known_ids = Vc27ChoiceArtId::ALL
        .into_iter()
        .map(Vc27ChoiceArtId::slug)
        .collect::<std::collections::BTreeSet<_>>();
    let mut errors = Vec::new();

    for (choice_id, value) in entries {
        if !known_ids.contains(choice_id.as_str()) {
            errors.push(format!("unknown choice id `{choice_id}`"));
            continue;
        }

        if let Some(icon) = value.as_str() {
            vc29_validate_choice_asset_path(choice_id, "icon", icon, ".png", &mut errors);
            continue;
        }

        let Some(descriptor) = value.as_object() else {
            errors.push(format!(
                "choice `{choice_id}` must be a legacy PNG string or an asset descriptor object"
            ));
            continue;
        };

        if descriptor.is_empty() {
            errors.push(format!(
                "choice `{choice_id}` descriptor must contain at least one asset field"
            ));
        }

        for field in descriptor.keys() {
            if !VC29_CHOICE_ASSET_FIELDS.contains(&field.as_str()) {
                errors.push(format!(
                    "choice `{choice_id}` contains unknown asset field `{field}`"
                ));
            }
        }

        for (field, extension) in [
            ("icon", ".png"),
            ("hover_sfx", ".wav"),
            ("confirm_sfx", ".wav"),
        ] {
            let Some(value) = descriptor.get(field) else {
                continue;
            };
            let Some(path) = value.as_str() else {
                errors.push(format!(
                    "choice `{choice_id}` field `{field}` must be a string"
                ));
                continue;
            };
            vc29_validate_choice_asset_path(choice_id, field, path, extension, &mut errors);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

fn vc29_validate_choice_asset_path(
    choice_id: &str,
    field: &str,
    path: &str,
    expected_extension: &str,
    errors: &mut Vec<String>,
) {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        errors.push(format!(
            "choice `{choice_id}` field `{field}` must not be empty"
        ));
        return;
    }
    if trimmed != path {
        errors.push(format!(
            "choice `{choice_id}` field `{field}` must not contain surrounding whitespace"
        ));
    }

    if path.starts_with('/')
        || path.starts_with('\\')
        || path.contains('\\')
        || path.contains(':')
        || path.contains('?')
        || path.contains('#')
    {
        errors.push(format!(
            "choice `{choice_id}` field `{field}` must be a relative asset path using `/` separators"
        ));
    }

    if path
        .split('/')
        .any(|component| component.is_empty() || component == "." || component == "..")
    {
        errors.push(format!(
            "choice `{choice_id}` field `{field}` contains an unsafe path component"
        ));
    }

    if !path.ends_with(expected_extension) {
        errors.push(format!(
            "choice `{choice_id}` field `{field}` must use the `{expected_extension}` extension"
        ));
    }
}

#[cfg(test)]
mod choice_manifest_validation_tests {
    use super::*;

    #[test]
    fn checked_in_vc29_choice_manifest_is_strictly_valid() {
        vc29_validate_choice_manifest(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/void_canticle/ui/choice/manifest.json"
        )))
        .expect("checked-in VC2.9 choice asset manifest should be valid");
    }

    #[test]
    fn validator_keeps_legacy_png_shorthand() {
        vc29_validate_choice_manifest(r#"{"death_nova":"death_nova.png"}"#)
            .expect("legacy PNG shorthand remains supported");
    }

    #[test]
    fn validator_accepts_partial_descriptors_without_requiring_asset_files() {
        vc29_validate_choice_manifest(
            r#"{
                "death_nova": {"icon":"mutations/death_nova.png"},
                "vital_spark": {"hover_sfx":"audio/vital_spark_hover.wav"},
                "orbitals": {"confirm_sfx":"audio/orbitals_confirm.wav"}
            }"#,
        )
        .expect("each descriptor field is independently optional");
    }

    #[test]
    fn validator_rejects_unknown_choice_ids_and_fields() {
        let error = vc29_validate_choice_manifest(
            r#"{
                "clint_auris": {"icon":"clint.png"},
                "death_nova": {"portrait":"death_nova.png"}
            }"#,
        )
        .expect_err("unknown ids and fields must fail validation");

        assert!(error.contains("unknown choice id `clint_auris`"));
        assert!(error.contains("unknown asset field `portrait`"));
    }

    #[test]
    fn validator_rejects_wrong_json_types_and_empty_descriptors() {
        let error = vc29_validate_choice_manifest(
            r#"{
                "death_nova": 12,
                "orbitals": {},
                "vital_spark": {"confirm_sfx":42}
            }"#,
        )
        .expect_err("wrong descriptor types must fail validation");

        assert!(error.contains("legacy PNG string or an asset descriptor object"));
        assert!(error.contains("descriptor must contain at least one asset field"));
        assert!(error.contains("field `confirm_sfx` must be a string"));
    }

    #[test]
    fn validator_rejects_path_traversal_absolute_paths_and_backslashes() {
        let error = vc29_validate_choice_manifest(
            r#"{
                "death_nova": {"icon":"../death_nova.png"},
                "orbitals": {"hover_sfx":"/tmp/orbitals.wav"},
                "vital_spark": {"confirm_sfx":"audio\\vital.wav"}
            }"#,
        )
        .expect_err("unsafe paths must fail validation");

        assert!(error.contains("unsafe path component"));
        assert!(error.contains("relative asset path using `/` separators"));
    }

    #[test]
    fn validator_rejects_incoherent_extensions_and_whitespace() {
        let error = vc29_validate_choice_manifest(
            r#"{
                "death_nova": {"icon":"death_nova.wav"},
                "orbitals": {"hover_sfx":"orbitals.png"},
                "vital_spark": {"confirm_sfx":" vital_spark.wav "}
            }"#,
        )
        .expect_err("field extensions are part of the asset contract");

        assert!(error.contains("field `icon` must use the `.png` extension"));
        assert!(error.contains("field `hover_sfx` must use the `.wav` extension"));
        assert!(error.contains("must not contain surrounding whitespace"));
    }
}
