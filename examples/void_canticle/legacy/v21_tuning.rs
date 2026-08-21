const VC21_TUNING_FALLBACK: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/void_canticle/tuning.json"
));

#[derive(Debug, Clone, Copy)]
struct Vc21Tuning {
    player_hull: f32,
    player_shield: f32,
    impact_damage: f32,
    post_hit_invulnerability: f32,
    shield_regen: bool,
}

impl Default for Vc21Tuning {
    fn default() -> Self {
        Self {
            player_hull: 60.0,
            player_shield: 25.0,
            impact_damage: 35.0,
            post_hit_invulnerability: 0.0,
            shield_regen: false,
        }
    }
}

impl Vc21Tuning {
    fn load() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let source = std::fs::read_to_string("assets/void_canticle/tuning.json")
            .unwrap_or_else(|_| VC21_TUNING_FALLBACK.to_string());

        #[cfg(target_arch = "wasm32")]
        let source = VC21_TUNING_FALLBACK.to_string();

        Self::parse(&source).unwrap_or_default()
    }

    fn parse(source: &str) -> Option<Self> {
        let value: serde_json::Value = serde_json::from_str(source).ok()?;
        let number = |key: &str, fallback: f32| {
            value
                .get(key)
                .and_then(serde_json::Value::as_f64)
                .map(|value| value as f32)
                .filter(|value| value.is_finite())
                .unwrap_or(fallback)
        };
        let boolean = |key: &str, fallback: bool| {
            value
                .get(key)
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(fallback)
        };

        let defaults = Self::default();
        Some(Self {
            player_hull: number("player_hull", defaults.player_hull).clamp(1.0, 999.0),
            player_shield: number("player_shield", defaults.player_shield).clamp(0.0, 999.0),
            impact_damage: number("impact_damage", defaults.impact_damage).clamp(1.0, 999.0),
            post_hit_invulnerability: number(
                "post_hit_invulnerability",
                defaults.post_hit_invulnerability,
            )
            .clamp(0.0, 5.0),
            shield_regen: boolean("shield_regen", defaults.shield_regen),
        })
    }
}

#[cfg(test)]
mod v21_tuning_tests {
    use super::*;

    #[test]
    fn checked_in_tuning_is_valid() {
        let tuning = Vc21Tuning::parse(VC21_TUNING_FALLBACK).expect("tuning should parse");
        assert!(tuning.player_hull > 0.0);
        assert!(tuning.impact_damage > 0.0);
    }

    #[test]
    fn current_default_has_no_post_hit_grace() {
        let tuning = Vc21Tuning::parse(VC21_TUNING_FALLBACK).expect("tuning should parse");
        assert_eq!(tuning.post_hit_invulnerability, 0.0);
    }
}
