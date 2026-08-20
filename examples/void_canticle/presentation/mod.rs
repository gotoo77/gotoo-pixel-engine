const VC27_PRESENTATION_VERSION: &str = "VC3.2";
const VC27_CARRION_TELEGRAPH_WINDOW: f32 = 0.24;
const VC27_WRAITH_TELEGRAPH_WINDOW: f32 = 0.30;
const VC27_BOSS_TELEGRAPH_WINDOW: f32 = 0.34;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Vc27EnemyShotStyle {
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

struct VoidCanticleV27DirectPresentation {
    game: VoidCanticleV23Sustain,
    legacy_sink: Framebuffer,
    clean_background: Framebuffer,
    presentation_time: f32,
    front_state: Vc27FrontState,
    hit_reactions: Vc27HitReactionState,
    projectile_provenance: Vc27ProjectileProvenance,
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
