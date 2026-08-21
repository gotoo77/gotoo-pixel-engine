include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/presentation/combat/hit_reactions.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/presentation/combat/projectile_provenance.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/presentation/combat/bestiary.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/presentation/combat/telegraphs.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/presentation/combat/projectiles.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/presentation/combat/fx.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/presentation/combat/orchestrator.rs"
));

#[cfg(test)]
mod presentation_combat_boundary_tests {
    #[test]
    fn combat_and_hud_do_not_traverse_legacy_wrapper_chain() {
        let sources = [
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/examples/void_canticle/presentation/hud.rs"
            )),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/examples/void_canticle/presentation/combat/hit_reactions.rs"
            )),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/examples/void_canticle/presentation/combat/projectile_provenance.rs"
            )),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/examples/void_canticle/presentation/combat/bestiary.rs"
            )),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/examples/void_canticle/presentation/combat/telegraphs.rs"
            )),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/examples/void_canticle/presentation/combat/projectiles.rs"
            )),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/examples/void_canticle/presentation/combat/fx.rs"
            )),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/examples/void_canticle/presentation/combat/orchestrator.rs"
            )),
        ];
        let nested_wrapper_access = [".game", ".game"].concat();

        for source in sources {
            assert!(!source.contains(&nested_wrapper_access));
        }
    }
}
