fn build_synergy_names(build: BuildState, mutations: MutationBuild) -> Vec<&'static str> {
    let mask = synergy_mask(build, mutations);
    let mut names = Vec::new();
    for (bit, name) in [
        (SYNERGY_CANTOR_STORM, "CANTOR STORM"),
        (SYNERGY_TWIN_REQUIEM, "TWIN REQUIEM"),
        (SYNERGY_GRAVITY_WAKE, "GRAVITY WAKE"),
        (SYNERGY_CANTICLE_CHOIR, "CANTICLE CHOIR"),
    ] {
        if mask & bit != 0 {
            names.push(name);
        }
    }
    names
}

fn render_build_overview(
    framebuffer: &mut Framebuffer,
    sustain: &GameplayRuntime,
    time: f32,
) {
    let progression = sustain.presentation_progression();
    let build = progression.progression.build;
    let mutations = progression.mutations;
    let chassis = sustain
        .presentation_selected_chassis()
        .unwrap_or(ExosuitChassis::Pilgrim);
    let accent = chassis_accent(chassis);

    choice_header(
        framebuffer,
        "BUILD",
        &format!("ECHO LEVEL {} / RUN LOADOUT", progression.progression.level),
        accent,
        time,
    );

    render_build_chassis_panel(framebuffer, sustain, chassis, 12, 82, 336, 86, time);

    framebuffer.draw_text(16, 180, "AUGMENTS", ART_CYAN_LIGHT);
    for (index, upgrade) in UPGRADE_POOL.into_iter().enumerate() {
        let column = index % 2;
        let row = index / 2;
        let x = 12 + column as i32 * 174;
        let y = 194 + row as i32 * 62;
        render_build_upgrade_slot(framebuffer, upgrade, &build, x, y, 162, 54, time);
    }

    framebuffer.draw_text(16, 386, "MUTATIONS", MUTATION_LIGHT);
    for (index, mutation) in MUTATION_POOL.into_iter().enumerate() {
        let column = index % 2;
        let row = index / 2;
        let x = 12 + column as i32 * 174;
        let y = 400 + row as i32 * 62;
        render_build_mutation_slot(
            framebuffer,
            mutation,
            &mutations,
            x,
            y,
            162,
            54,
            time,
        );
    }

    render_build_footer_panel(framebuffer, sustain, build, mutations, 12, 530, 336, 72);
    vc_visual_draw_centered_text(framebuffer, 617, "ESC / START / SOUTH  BACK", 1, WRECK_LIGHT);
}

#[allow(clippy::too_many_arguments)]
fn render_build_chassis_panel(
    framebuffer: &mut Framebuffer,
    sustain: &GameplayRuntime,
    chassis: ExosuitChassis,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    time: f32,
) {
    let accent = chassis_accent(chassis);
    choice_card_frame(
        framebuffer,
        x,
        y,
        width,
        height,
        true,
        ChoiceCardStyle::new(accent, Pixel::rgb(7, 9, 18), Pixel::rgb(11, 14, 27)),
        time,
    );

    let ship_x = x + 47;
    let ship_y = y + 43;
    render_chassis_ship(framebuffer, chassis, ship_x, ship_y, false, time);

    let info_x = x + 94;
    framebuffer.draw_text_scaled(info_x, y + 10, chassis.name(), 2, accent);
    framebuffer.draw_text(info_x, y + 31, chassis.passive_name(), CANTICLE_COLOR);
    framebuffer.draw_text(info_x, y + 44, chassis.passive_description(), WRECK_LIGHT);

    let combat = sustain.combat_model();
    framebuffer.draw_text(
        info_x,
        y + 63,
        &format!(
            "HULL {:03}   SHLD {:03}",
            combat.player_hull.max(0.0).round() as u32,
            combat.player_shield.max(0.0).round() as u32
        ),
        TEXT,
    );
}

#[allow(clippy::too_many_arguments)]
fn render_build_upgrade_slot(
    framebuffer: &mut Framebuffer,
    upgrade: UpgradeKind,
    build: &BuildState,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    time: f32,
) {
    let stack = upgrade_stack(build, upgrade);
    let accent = upgrade_accent(upgrade);
    let active = stack > 0;
    let edge = if active { accent } else { WRECK_MID };
    framebuffer.fill_rect(x, y, width, height, Pixel::rgb(7, 10, 18));
    framebuffer.draw_rect(x, y, width, height, edge);

    if active {
        upgrade_assets(upgrade).render(framebuffer, x + 27, y + 27, false, time);
    } else {
        framebuffer.draw_circle(x + 27, y + 27, 11, WRECK_MID);
    }

    framebuffer.draw_text(
        x + 52,
        y + 10,
        upgrade_name(upgrade),
        if active { accent } else { WRECK_LIGHT },
    );
    framebuffer.draw_text(
        x + 52,
        y + 29,
        &format!("STACK {:02}", stack),
        if active { TEXT } else { WRECK_MID },
    );
}

#[allow(clippy::too_many_arguments)]
fn render_build_mutation_slot(
    framebuffer: &mut Framebuffer,
    mutation: MutationKind,
    build: &MutationBuild,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    time: f32,
) {
    let stack = mutation_stack(build, mutation);
    let max = mutation_max(mutation);
    let accent = mutation_accent(mutation);
    let active = stack > 0;
    let edge = if active { accent } else { WRECK_MID };
    framebuffer.fill_rect(x, y, width, height, Pixel::rgb(12, 6, 17));
    framebuffer.draw_rect(x, y, width, height, edge);

    if active {
        mutation_assets(mutation).render(framebuffer, x + 27, y + 27, false, time);
    } else {
        framebuffer.draw_circle(x + 27, y + 27, 11, WRECK_MID);
    }

    framebuffer.draw_text(
        x + 52,
        y + 10,
        mutation_name(mutation),
        if active { accent } else { WRECK_LIGHT },
    );
    framebuffer.draw_text(
        x + 52,
        y + 29,
        &format!("STACK {:02}/{:02}", stack, max),
        if active { TEXT } else { WRECK_MID },
    );
}

fn render_build_footer_panel(
    framebuffer: &mut Framebuffer,
    sustain: &GameplayRuntime,
    build: BuildState,
    mutations: MutationBuild,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) {
    framebuffer.fill_rect(x, y, width, height, Pixel::rgb(7, 9, 17));
    framebuffer.draw_rect(x, y, width, height, PILGRIM_VIOLET);

    framebuffer.draw_text(x + 9, y + 9, "SUPPORT", ART_CYAN_LIGHT);
    match sustain.augment {
        Some(augment) => {
            let accent = support_accent(augment);
            framebuffer.draw_text(x + 66, y + 9, augment.name(), accent);
            framebuffer.draw_text(x + 66, y + 23, &support_effect(augment), WRECK_LIGHT);
        }
        None => framebuffer.draw_text(x + 66, y + 9, "NOT INSTALLED", WRECK_MID),
    }

    framebuffer.draw_text(x + 9, y + 43, "SYNERGY", SYNERGY_COLOR);
    let synergies = build_synergy_names(build, mutations);
    if synergies.is_empty() {
        framebuffer.draw_text(x + 66, y + 43, "NONE AWAKENED", WRECK_MID);
    } else {
        for (index, name) in synergies.into_iter().take(2).enumerate() {
            framebuffer.draw_text(x + 66 + index as i32 * 125, y + 43, name, SYNERGY_GOLD);
        }
    }
}

#[cfg(test)]
mod build_overview_tests {
    use super::*;

    #[test]
    fn synergy_list_tracks_all_real_synergy_bits() {
        let mut build = BuildState::default();
        build.rapid_fire = 1;
        build.stellar_power = 1;
        build.magnet_field = 1;
        build.core_surge = 1;
        let mut mutations = MutationBuild::default();
        mutations.split_volley = 1;
        mutations.piercing_lance = 1;
        mutations.death_nova = 1;
        mutations.orbitals = 1;
        assert_eq!(build_synergy_names(build, mutations).len(), 4);
    }

    #[test]
    fn build_overview_layout_fits_presentation_space() {
        assert!(12 + 336 <= VC_VISUAL_PRESENTATION_WIDTH as i32);
        assert!(530 + 72 < 617);
        assert!(617 < VC_VISUAL_PRESENTATION_HEIGHT as i32);
    }
}
