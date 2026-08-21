const CARRION_TELEGRAPH_WINDOW: f32 = 0.24;
const WRAITH_TELEGRAPH_WINDOW: f32 = 0.30;
const BOSS_TELEGRAPH_WINDOW: f32 = 0.34;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EnemyShotStyle {
    Carrion,
    Wraith,
    VoidPulse,
    Void,
    Bellkeeper,
}

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/presentation/hd_bestiary.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/presentation/chassis_showcase.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/presentation/upgrade_showcase.rs"
));

struct VoidCanticlePresentation {
    game: GameplayRuntime,
    legacy_sink: Framebuffer,
    clean_background: Framebuffer,
    presentation_time: f32,
    hit_reactions: HitReactionState,
    projectile_provenance: ProjectileProvenance,
}

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/presentation/runtime.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/presentation/combat.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/presentation/hud.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/presentation/modals.rs"
));

#[cfg(test)]
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/presentation/audio_tests.rs"
));

#[cfg(test)]
mod semantic_naming_tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    fn rust_files(root: &Path, files: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(root).expect("presentation directory should be readable") {
            let path = entry.expect("presentation entry should be readable").path();
            if path.is_dir() {
                rust_files(&path, files);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
    }

    fn has_numbered_prefix(source: &str, prefix: &str, suffix: Option<u8>) -> bool {
        let bytes = source.as_bytes();
        let prefix = prefix.as_bytes();
        bytes.windows(prefix.len() + 2 + usize::from(suffix.is_some())).any(|window| {
            window.starts_with(prefix)
                && window[prefix.len()].is_ascii_digit()
                && window[prefix.len() + 1].is_ascii_digit()
                && suffix.is_none_or(|suffix| window[prefix.len() + 2] == suffix)
        })
    }

    fn versioned_tokens(source: &str) -> bool {
        let historical_type_prefix = ["VoidCanticle", "V"].concat();
        source.contains(&historical_type_prefix)
            || has_numbered_prefix(source, "vc", Some(b'_'))
            || has_numbered_prefix(source, "Vc", None)
            || has_numbered_prefix(source, "VC", Some(b'_'))
            || has_numbered_prefix(source, ".v", None)
    }

    #[test]
    fn active_presentation_contains_no_version_numbered_architecture_names() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/void_canticle/presentation");
        let mut files = Vec::new();
        rust_files(&root, &mut files);

        let offenders = files
            .into_iter()
            .filter(|path| {
                let source = fs::read_to_string(path).expect("presentation source should be UTF-8");
                versioned_tokens(&source)
            })
            .map(|path| {
                path.strip_prefix(env!("CARGO_MANIFEST_DIR"))
                    .unwrap_or(&path)
                    .display()
                    .to_string()
            })
            .collect::<Vec<_>>();

        assert!(
            offenders.is_empty(),
            "active presentation still contains historical version-numbered names: {}",
            offenders.join(", ")
        );
    }
}
