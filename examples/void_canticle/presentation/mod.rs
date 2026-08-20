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
    game: VoidCanticleV23Sustain,
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
