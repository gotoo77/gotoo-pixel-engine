const CHOICE_CONFIRM_DURATION: f32 = 0.58;
const SYNERGY_REVEAL_DURATION: f32 = 1.05;

fn synergy_after_upgrade(
    build: BuildState,
    mutations: MutationBuild,
    upgrade: UpgradeKind,
) -> Option<&'static str> {
    let before = synergy_mask(build, mutations);
    let mut next = build;
    match upgrade {
        UpgradeKind::RapidFire => next.rapid_fire = next.rapid_fire.saturating_add(1),
        UpgradeKind::MagnetField => next.magnet_field = next.magnet_field.saturating_add(1),
        UpgradeKind::StellarPower => next.stellar_power = next.stellar_power.saturating_add(1),
        UpgradeKind::XpHunger => next.xp_hunger = next.xp_hunger.saturating_add(1),
        UpgradeKind::VitalSpark => next.vital_spark = next.vital_spark.saturating_add(1),
        UpgradeKind::CoreSurge => next.core_surge = next.core_surge.saturating_add(1),
    }
    new_synergy_name(before, synergy_mask(next, mutations))
}

fn synergy_after_mutation(
    build: BuildState,
    mutations: MutationBuild,
    mutation: MutationKind,
) -> Option<&'static str> {
    let before = synergy_mask(build, mutations);
    let mut next = mutations;
    match mutation {
        MutationKind::PiercingLance => {
            next.piercing_lance = next.piercing_lance.saturating_add(1)
        }
        MutationKind::SplitVolley => next.split_volley = next.split_volley.saturating_add(1),
        MutationKind::DeathNova => next.death_nova = next.death_nova.saturating_add(1),
        MutationKind::Orbitals => next.orbitals = next.orbitals.saturating_add(1),
    }
    new_synergy_name(before, synergy_mask(build, next))
}

fn new_synergy_name(before: u8, after: u8) -> Option<&'static str> {
    let discovered = after & !before;
    (discovered != 0).then(|| first_synergy_name(discovered))
}

fn primary_active_synergy(
    build: BuildState,
    mutations: MutationBuild,
) -> Option<&'static str> {
    let active = synergy_mask(build, mutations);
    (active != 0).then(|| first_synergy_name(active))
}

fn render_synergy_hint(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    name: &str,
    selected: bool,
    time: f32,
) {
    let pulse = selected && ((time * 5.0) as i32 & 1) == 0;
    let color = if pulse { SYNERGY_LIGHT } else { SYNERGY_COLOR };
    framebuffer.draw_line(x, y, x + 9, y, SYNERGY_GOLD);
    framebuffer.draw_text(x + 14, y - 3, "SYNERGY >", color);
    framebuffer.draw_text(x + 80, y - 3, name, SYNERGY_GOLD);
}

fn render_active_synergy_strip(
    framebuffer: &mut Framebuffer,
    name: &str,
    time: f32,
) {
    let y = 86_i32;
    let sweep = ((time * 68.0) as i32).rem_euclid(238);
    framebuffer.draw_line(58, y, 302, y, WRECK_MID);
    framebuffer.draw_line(61 + sweep, y, 67 + sweep, y, SYNERGY_COLOR);
    vc_visual_draw_centered_text(
        framebuffer,
        y + 7,
        &format!("ACTIVE SYNERGY / {name}"),
        1,
        SYNERGY_GOLD,
    );
}

fn render_choice_confirmation(
    framebuffer: &mut Framebuffer,
    label: &str,
    accent: Pixel,
    synergy: Option<&str>,
    remaining: f32,
    duration: f32,
) {
    let ratio = if duration > 0.0 {
        (remaining / duration).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let synergy_reveal = synergy.is_some();
    let width = if synergy_reveal { 300_u32 } else { 252_u32 };
    let height = if synergy_reveal { 44_u32 } else { 28_u32 };
    let x = ((VC_VISUAL_PRESENTATION_WIDTH - width) / 2) as i32;
    let y = if synergy_reveal { 84_i32 } else { 92_i32 };
    let edge = if synergy_reveal { SYNERGY_COLOR } else { accent };

    framebuffer.fill_rect(x, y, width, height, Pixel::rgb(6, 7, 15));
    framebuffer.draw_rect(x, y, width, height, edge);
    let travel = ((1.0 - ratio) * (width.saturating_sub(36)) as f32).round() as i32;
    framebuffer.draw_line(x + 12 + travel, y + 2, x + 26 + travel, y + 2, edge);

    if let Some(name) = synergy {
        vc_visual_draw_centered_text(framebuffer, y + 8, "SYNERGY AWAKENED", 1, SYNERGY_LIGHT);
        vc_visual_draw_centered_text(framebuffer, y + 24, name, 2, SYNERGY_GOLD);
    } else {
        vc_visual_draw_centered_text(framebuffer, y + 9, &format!("LOCKED / {label}"), 1, accent);
    }
}

#[cfg(test)]
mod synergy_showcase_tests {
    use super::*;

    #[test]
    fn upgrade_preview_uses_real_synergy_rules() {
        let build = BuildState::default();
        let mut mutations = MutationBuild::default();
        mutations.split_volley = 1;
        assert_eq!(
            synergy_after_upgrade(build, mutations, UpgradeKind::RapidFire),
            Some("CANTOR STORM")
        );
        assert_eq!(
            synergy_after_upgrade(build, mutations, UpgradeKind::VitalSpark),
            None
        );
    }

    #[test]
    fn mutation_preview_uses_real_synergy_rules() {
        let mut build = BuildState::default();
        build.stellar_power = 1;
        let mutations = MutationBuild::default();
        assert_eq!(
            synergy_after_mutation(build, mutations, MutationKind::PiercingLance),
            Some("TWIN REQUIEM")
        );
        assert_eq!(
            synergy_after_mutation(build, mutations, MutationKind::Orbitals),
            None
        );
    }

    #[test]
    fn already_active_synergy_is_not_previewed_as_new() {
        let mut build = BuildState::default();
        build.magnet_field = 1;
        let mut mutations = MutationBuild::default();
        mutations.death_nova = 1;
        assert_eq!(
            synergy_after_upgrade(build, mutations, UpgradeKind::MagnetField),
            None
        );
    }
}
