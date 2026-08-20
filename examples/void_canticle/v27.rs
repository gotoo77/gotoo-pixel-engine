const VC27_PRESENTATION_VERSION: &str = "VC2.7";
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
    "/examples/void_canticle/v27/hd_bestiary.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/v27/chassis_showcase.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/v27/upgrade_showcase.rs"
));

struct VoidCanticleV27DirectPresentation {
    game: VoidCanticleV23Sustain,
    legacy_sink: Framebuffer,
    clean_background: Framebuffer,
    presentation_time: f32,
    hit_reactions: Vc27HitReactionState,
}

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/v27/runtime.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/v27/combat.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/v27/hud.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/v27/modals.rs"
));
