fn upgrade_profile(upgrade: UpgradeKind) -> ChoiceProfile<'static> {
    let (category, accent, renderer): (&'static str, Pixel, ChoiceArtRenderer) = match upgrade {
        UpgradeKind::RapidFire => ("WEAPON", BOLT_EDGE, upgrade_rapid_fire_art),
        UpgradeKind::MagnetField => ("UTILITY", ART_CYAN_LIGHT, upgrade_magnet_field_art),
        UpgradeKind::StellarPower => ("POWER", CANTICLE_COLOR, upgrade_stellar_power_art),
        UpgradeKind::XpHunger => ("GROWTH", XP_ORB_CORE, upgrade_xp_hunger_art),
        UpgradeKind::VitalSpark => ("SURVIVAL", PRESENTATION_HULL_COLOR, upgrade_vital_spark_art),
        UpgradeKind::CoreSurge => ("CANTICLE", PILGRIM_VIOLET, upgrade_core_surge_art),
    };
    ChoiceProfile::new(
        upgrade_name(upgrade),
        category,
        accent,
        ChoiceAssets::procedural(renderer),
    )
}

fn upgrade_accent(upgrade: UpgradeKind) -> Pixel {
    upgrade_profile(upgrade).accent()
}

fn upgrade_assets(upgrade: UpgradeKind) -> ChoiceAssets<'static> {
    upgrade_profile(upgrade).assets()
}

fn upgrade_stack(build: &BuildState, upgrade: UpgradeKind) -> u32 {
    match upgrade {
        UpgradeKind::RapidFire => build.rapid_fire,
        UpgradeKind::MagnetField => build.magnet_field,
        UpgradeKind::StellarPower => build.stellar_power,
        UpgradeKind::XpHunger => build.xp_hunger,
        UpgradeKind::VitalSpark => build.vital_spark,
        UpgradeKind::CoreSurge => build.core_surge,
    }
}

fn upgrade_effect(upgrade: UpgradeKind) -> String {
    match upgrade {
        UpgradeKind::RapidFire => "FIRE RATE +18 PCT".to_owned(),
        UpgradeKind::MagnetField => "PICKUP RANGE +22".to_owned(),
        UpgradeKind::StellarPower => "POWER LEVEL +1".to_owned(),
        UpgradeKind::XpHunger => "XP VALUE +25 PCT".to_owned(),
        UpgradeKind::VitalSpark => format!("MAX HULL +{}", VITAL_SPARK_HULL_BONUS as u32),
        UpgradeKind::CoreSurge => "CORE CHARGE +30".to_owned(),
    }
}

fn upgrade_detail(upgrade: UpgradeKind) -> &'static str {
    match upgrade {
        UpgradeKind::RapidFire => "FASTER CONTINUOUS FIRE",
        UpgradeKind::MagnetField => "ECHOES PULL FROM FARTHER",
        UpgradeKind::StellarPower => "STRONGER PRIMARY BOLTS",
        UpgradeKind::XpHunger => "MORE ECHO PER PICKUP",
        UpgradeKind::VitalSpark => "RAISE CAP AND REPAIR HULL",
        UpgradeKind::CoreSurge => "IMMEDIATE CANTICLE CHARGE",
    }
}

fn render_upgrade_showcase(
    framebuffer: &mut Framebuffer,
    progression: &VoidCanticleV13,
    mutations: &MutationBuild,
    choice: &LevelChoice,
    time: f32,
) {
    let accent = echo_level_color(progression.level);
    choice_header(
        framebuffer,
        "LEVEL UP",
        &format!("ECHO LEVEL {} / CHOOSE AN AUGMENT", progression.level),
        accent,
        time,
    );

    let card_x = 12_i32;
    let card_width = VC_VISUAL_PRESENTATION_WIDTH.saturating_sub(24);
    let card_height = 130_u32;
    let card_start_y = 94_i32;
    let card_gap = 10_i32;

    for (index, upgrade) in choice.offers.iter().copied().enumerate() {
        let y = card_start_y + index as i32 * (card_height as i32 + card_gap);
        let selected = choice.menu.selected() == Some(index);
        render_upgrade_card(
            framebuffer,
            upgrade,
            &progression.build,
            mutations,
            selected,
            card_x,
            y,
            card_width,
            card_height,
            time,
        );
    }

    choice_footer(
        framebuffer,
        accent,
        "BUILD THE PILGRIM / SURVIVE THE CANTICLE",
    );
}

#[allow(clippy::too_many_arguments)]
fn render_upgrade_card(
    framebuffer: &mut Framebuffer,
    upgrade: UpgradeKind,
    build: &BuildState,
    mutations: &MutationBuild,
    selected: bool,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    time: f32,
) {
    let profile = upgrade_profile(upgrade);
    let accent = profile.accent();
    choice_card_frame(
        framebuffer,
        x,
        y,
        width,
        height,
        selected,
        ChoiceCardStyle::new(
            accent,
            Pixel::rgb(7, 10, 19),
            Pixel::rgb(10, 17, 29),
        ),
        time,
    );

    let icon_x = x + 58;
    let icon_y = y + 64;
    choice_icon_shell(framebuffer, icon_x, icon_y, accent, selected, time);
    profile.render_art(framebuffer, icon_x, icon_y, selected, time);

    let info_x = x + 112;
    framebuffer.draw_text(info_x, y + 12, profile.category(), WRECK_LIGHT);
    framebuffer.draw_text_scaled(info_x, y + 28, profile.label(), 2, accent);
    framebuffer.draw_text(info_x, y + 54, &upgrade_effect(upgrade), TEXT);
    framebuffer.draw_text(info_x, y + 70, upgrade_detail(upgrade), WRECK_LIGHT);

    if let Some(name) = synergy_after_upgrade(*build, *mutations, upgrade) {
        render_synergy_hint(framebuffer, info_x, y + 87, name, selected, time);
    }

    let stack = upgrade_stack(build, upgrade);
    let next = stack.saturating_add(1);
    framebuffer.draw_text(
        info_x,
        y + 98,
        &format!("STACK {:02}  >  {:02}", stack, next),
        if selected { accent } else { WRECK_LIGHT },
    );
    if selected {
        framebuffer.draw_text(x + width as i32 - 53, y + 13, "SELECT", accent);
    }
    choice_stack_nodes(framebuffer, info_x, y + 114, next, 8, accent);
}

fn render_upgrade_icon(
    framebuffer: &mut Framebuffer,
    upgrade: UpgradeKind,
    x: i32,
    y: i32,
) {
    match upgrade {
        UpgradeKind::RapidFire => {
            for dx in [-12, 0, 12] {
                framebuffer.draw_line(x + dx, y + 18, x + dx, y - 19, BOLT_EDGE);
                framebuffer.draw_line(x + dx - 2, y + 12, x + dx, y - 23, BOLT_CORE);
            }
            framebuffer.draw_line(x - 19, y + 21, x + 19, y + 21, BOLT_RELIC);
        }
        UpgradeKind::MagnetField => {
            framebuffer.draw_line(x - 17, y - 19, x - 17, y + 11, ART_CYAN_LIGHT);
            framebuffer.draw_line(x + 17, y - 19, x + 17, y + 11, ART_CYAN_LIGHT);
            framebuffer.draw_line(x - 17, y + 11, x - 8, y + 21, ART_CYAN_LIGHT);
            framebuffer.draw_line(x + 17, y + 11, x + 8, y + 21, ART_CYAN_LIGHT);
            framebuffer.draw_line(x - 8, y + 21, x + 8, y + 21, XP_ORB_CORE);
            framebuffer.fill_circle(x, y - 3, 3, XP_ORB_CORE);
            framebuffer.draw_circle(x, y - 3, 11, ART_CYAN);
        }
        UpgradeKind::StellarPower => {
            framebuffer.draw_line(x, y - 25, x, y + 25, CANTICLE_COLOR);
            framebuffer.draw_line(x - 25, y, x + 25, y, CANTICLE_COLOR);
            framebuffer.draw_line(x - 18, y - 18, x + 18, y + 18, ART_GOLD);
            framebuffer.draw_line(x + 18, y - 18, x - 18, y + 18, ART_GOLD);
            framebuffer.fill_circle(x, y, 7, BOLT_CORE);
            framebuffer.fill_circle(x, y, 3, CANTICLE_COLOR);
        }
        UpgradeKind::XpHunger => {
            framebuffer.draw_line(x, y - 25, x + 19, y, XP_ORB_CORE);
            framebuffer.draw_line(x + 19, y, x, y + 25, XP_ORB_CORE);
            framebuffer.draw_line(x, y + 25, x - 19, y, XP_ORB_CORE);
            framebuffer.draw_line(x - 19, y, x, y - 25, XP_ORB_CORE);
            framebuffer.fill_circle(x, y, 4, XP_ORB);
            for (dx, dy) in [(-24, -14), (24, -14), (-25, 15), (25, 15)] {
                framebuffer.fill_circle(x + dx, y + dy, 2, ART_CYAN_LIGHT);
            }
        }
        UpgradeKind::VitalSpark => {
            framebuffer.draw_line(x, y + 24, x - 22, y - 3, PRESENTATION_HULL_COLOR);
            framebuffer.draw_line(x - 22, y - 3, x - 13, y - 20, PRESENTATION_HULL_COLOR);
            framebuffer.draw_line(x - 13, y - 20, x, y - 10, CANTICLE_COLOR);
            framebuffer.draw_line(x, y - 10, x + 13, y - 20, CANTICLE_COLOR);
            framebuffer.draw_line(x + 13, y - 20, x + 22, y - 3, PRESENTATION_HULL_COLOR);
            framebuffer.draw_line(x + 22, y - 3, x, y + 24, PRESENTATION_HULL_COLOR);
            framebuffer.fill_circle(x, y - 1, 5, CANTICLE_COLOR);
            framebuffer.draw_circle(x, y - 1, 10, PRESENTATION_HULL_COLOR);
        }
        UpgradeKind::CoreSurge => {
            framebuffer.draw_circle(x, y, 22, PILGRIM_VIOLET);
            framebuffer.draw_circle(x, y, 14, CANTICLE_COLOR);
            framebuffer.fill_circle(x, y, 5, CINDER);
            framebuffer.draw_line(x, y - 29, x, y - 17, CANTICLE_COLOR);
            framebuffer.draw_line(x + 29, y, x + 17, y, CANTICLE_COLOR);
            framebuffer.draw_line(x, y + 29, x, y + 17, CANTICLE_COLOR);
            framebuffer.draw_line(x - 29, y, x - 17, y, CANTICLE_COLOR);
        }
    }
}

fn upgrade_rapid_fire_art(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    _selected: bool,
    _time: f32,
) {
    render_upgrade_icon(framebuffer, UpgradeKind::RapidFire, x, y);
}

fn upgrade_magnet_field_art(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    _selected: bool,
    _time: f32,
) {
    render_upgrade_icon(framebuffer, UpgradeKind::MagnetField, x, y);
}

fn upgrade_stellar_power_art(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    _selected: bool,
    _time: f32,
) {
    render_upgrade_icon(framebuffer, UpgradeKind::StellarPower, x, y);
}

fn upgrade_xp_hunger_art(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    _selected: bool,
    _time: f32,
) {
    render_upgrade_icon(framebuffer, UpgradeKind::XpHunger, x, y);
}

fn upgrade_vital_spark_art(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    _selected: bool,
    _time: f32,
) {
    render_upgrade_icon(framebuffer, UpgradeKind::VitalSpark, x, y);
}

fn upgrade_core_surge_art(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    _selected: bool,
    _time: f32,
) {
    render_upgrade_icon(framebuffer, UpgradeKind::CoreSurge, x, y);
}

#[cfg(test)]
mod upgrade_showcase_tests {
    use super::*;

    #[test]
    fn modern_vital_spark_copy_tracks_survival_bonus() {
        assert_eq!(
            upgrade_effect(UpgradeKind::VitalSpark),
            format!("MAX HULL +{}", VITAL_SPARK_HULL_BONUS as u32)
        );
    }

    #[test]
    fn all_upgrade_kinds_have_distinct_semantic_categories() {
        let categories = UPGRADE_POOL.map(|upgrade| upgrade_profile(upgrade).category());
        for index in 0..categories.len() {
            for other in index + 1..categories.len() {
                assert_ne!(categories[index], categories[other]);
            }
        }
    }

    #[test]
    fn upgrades_expose_choice_profile_assets_and_audio() {
        for upgrade in UPGRADE_POOL {
            let profile = upgrade_profile(upgrade);
            let (expected_hover, expected_confirm) = match upgrade {
                UpgradeKind::RapidFire => (
                    Some(ChoiceArtId::RapidFire.hover_override_sound()),
                    Some(ChoiceArtId::RapidFire.confirm_override_sound()),
                ),
                _ => (Some(CHOICE_HOVER_SOUND), Some(CHOICE_CONFIRM_SOUND)),
            };
            assert_eq!(profile.label(), upgrade_name(upgrade));
            assert_eq!(profile.hover_sound(), expected_hover);
            assert_eq!(profile.confirm_sound(), expected_confirm);
        }
    }

    #[test]
    fn three_upgrade_cards_fit_above_footer() {
        let start = 94_i32;
        let height = 130_i32;
        let gap = 10_i32;
        let bottom = start + 2 * (height + gap) + height;
        assert!(bottom < 505);
        assert!(505 < VC_VISUAL_PRESENTATION_HEIGHT as i32);
    }
}
