fn vc27_upgrade_accent(upgrade: UpgradeKind) -> Pixel {
    match upgrade {
        UpgradeKind::RapidFire => BOLT_EDGE,
        UpgradeKind::MagnetField => ART_CYAN_LIGHT,
        UpgradeKind::StellarPower => CANTICLE_COLOR,
        UpgradeKind::XpHunger => XP_ORB_CORE,
        UpgradeKind::VitalSpark => VC20_HULL,
        UpgradeKind::CoreSurge => PILGRIM_VIOLET,
    }
}

fn vc27_upgrade_category(upgrade: UpgradeKind) -> &'static str {
    match upgrade {
        UpgradeKind::RapidFire => "WEAPON",
        UpgradeKind::MagnetField => "UTILITY",
        UpgradeKind::StellarPower => "POWER",
        UpgradeKind::XpHunger => "GROWTH",
        UpgradeKind::VitalSpark => "SURVIVAL",
        UpgradeKind::CoreSurge => "CANTICLE",
    }
}

fn vc27_upgrade_stack(build: &BuildState, upgrade: UpgradeKind) -> u32 {
    match upgrade {
        UpgradeKind::RapidFire => build.rapid_fire,
        UpgradeKind::MagnetField => build.magnet_field,
        UpgradeKind::StellarPower => build.stellar_power,
        UpgradeKind::XpHunger => build.xp_hunger,
        UpgradeKind::VitalSpark => build.vital_spark,
        UpgradeKind::CoreSurge => build.core_surge,
    }
}

fn vc27_upgrade_effect(upgrade: UpgradeKind) -> String {
    match upgrade {
        UpgradeKind::RapidFire => "FIRE RATE +18 PCT".to_owned(),
        UpgradeKind::MagnetField => "PICKUP RANGE +22".to_owned(),
        UpgradeKind::StellarPower => "POWER LEVEL +1".to_owned(),
        UpgradeKind::XpHunger => "XP VALUE +25 PCT".to_owned(),
        UpgradeKind::VitalSpark => format!("MAX HULL +{}", VC21_VITAL_SPARK_HULL_BONUS as u32),
        UpgradeKind::CoreSurge => "CORE CHARGE +30".to_owned(),
    }
}

fn vc27_upgrade_detail(upgrade: UpgradeKind) -> &'static str {
    match upgrade {
        UpgradeKind::RapidFire => "FASTER CONTINUOUS FIRE",
        UpgradeKind::MagnetField => "ECHOES PULL FROM FARTHER",
        UpgradeKind::StellarPower => "STRONGER PRIMARY BOLTS",
        UpgradeKind::XpHunger => "MORE ECHO PER PICKUP",
        UpgradeKind::VitalSpark => "RAISE CAP AND REPAIR HULL",
        UpgradeKind::CoreSurge => "IMMEDIATE CANTICLE CHARGE",
    }
}

fn vc27_render_upgrade_showcase(
    framebuffer: &mut Framebuffer,
    progression: &VoidCanticleV13,
    choice: &LevelChoice,
    time: f32,
) {
    let accent = vc27_echo_level_color(progression.level);
    vc_visual_draw_centered_text(framebuffer, 24, "LEVEL UP", 3, accent);
    vc_visual_draw_centered_text(
        framebuffer,
        57,
        &format!("ECHO LEVEL {} / CHOOSE AN AUGMENT", progression.level),
        1,
        WRECK_LIGHT,
    );

    framebuffer.draw_line(36, 78, 324, 78, WRECK_MID);
    let spark = ((time * 55.0) as i32).rem_euclid(272);
    framebuffer.draw_line(44 + spark, 78, 54 + spark, 78, accent);

    let card_x = 12_i32;
    let card_width = VC_VISUAL_PRESENTATION_WIDTH.saturating_sub(24);
    let card_height = 130_u32;
    let card_start_y = 94_i32;
    let card_gap = 10_i32;

    for (index, upgrade) in choice.offers.iter().copied().enumerate() {
        let y = card_start_y + index as i32 * (card_height as i32 + card_gap);
        let selected = choice.menu.selected() == Some(index);
        vc27_render_upgrade_card(
            framebuffer,
            upgrade,
            &progression.build,
            selected,
            card_x,
            y,
            card_width,
            card_height,
            time,
        );
    }

    framebuffer.draw_line(36, 505, 324, 505, WRECK_MID);
    vc_visual_draw_centered_text(framebuffer, 523, "UP DOWN / DPAD", 1, TEXT);
    vc_visual_draw_centered_text(
        framebuffer,
        544,
        "SPACE / SOUTH  SELECT",
        1,
        accent,
    );
    vc_visual_draw_centered_text(
        framebuffer,
        581,
        "BUILD THE PILGRIM / SURVIVE THE CANTICLE",
        1,
        WRECK_LIGHT,
    );
}

#[allow(clippy::too_many_arguments)]
fn vc27_render_upgrade_card(
    framebuffer: &mut Framebuffer,
    upgrade: UpgradeKind,
    build: &BuildState,
    selected: bool,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    time: f32,
) {
    let accent = vc27_upgrade_accent(upgrade);
    let panel = if selected {
        Pixel::rgb(10, 17, 29)
    } else {
        Pixel::rgb(7, 10, 19)
    };
    let edge = if selected { accent } else { WRECK_MID };

    framebuffer.fill_rect(x, y, width, height, panel);
    framebuffer.draw_rect(x, y, width, height, edge);
    vc27_render_card_corner(framebuffer, x + 5, y + 5, 1, 1, edge);
    vc27_render_card_corner(
        framebuffer,
        x + width as i32 - 6,
        y + 5,
        -1,
        1,
        edge,
    );
    vc27_render_card_corner(
        framebuffer,
        x + 5,
        y + height as i32 - 6,
        1,
        -1,
        edge,
    );
    vc27_render_card_corner(
        framebuffer,
        x + width as i32 - 6,
        y + height as i32 - 6,
        -1,
        -1,
        edge,
    );

    if selected {
        framebuffer.draw_rect(x + 3, y + 3, width - 6, height - 6, WRECK_LIGHT);
        let travel = ((time * 76.0) as i32).rem_euclid(width.saturating_sub(42) as i32);
        framebuffer.draw_line(x + 20 + travel, y + 1, x + 36 + travel, y + 1, accent);
    }

    let icon_x = x + 58;
    let icon_y = y + 64;
    vc27_render_upgrade_icon(framebuffer, upgrade, icon_x, icon_y, selected, time);

    let info_x = x + 112;
    framebuffer.draw_text(info_x, y + 12, vc27_upgrade_category(upgrade), WRECK_LIGHT);
    framebuffer.draw_text_scaled(info_x, y + 28, upgrade_name(upgrade), 2, accent);
    framebuffer.draw_text(info_x, y + 54, &vc27_upgrade_effect(upgrade), TEXT);
    framebuffer.draw_text(info_x, y + 70, vc27_upgrade_detail(upgrade), WRECK_LIGHT);

    let stack = vc27_upgrade_stack(build, upgrade);
    framebuffer.draw_text(
        info_x,
        y + 94,
        &format!("STACK {:02}  >  {:02}", stack, stack.saturating_add(1)),
        if selected { accent } else { WRECK_LIGHT },
    );
    if selected {
        framebuffer.draw_text(x + width as i32 - 53, y + 13, "SELECT", accent);
    }

    let nodes = (stack.min(8) + 1) as i32;
    for node in 0..8_i32 {
        let color = if node < nodes { accent } else { WRECK_MID };
        let nx = info_x + node * 15;
        framebuffer.fill_rect(nx, y + 112, 9, 3, color);
    }
}

fn vc27_render_upgrade_icon(
    framebuffer: &mut Framebuffer,
    upgrade: UpgradeKind,
    x: i32,
    y: i32,
    selected: bool,
    time: f32,
) {
    let accent = vc27_upgrade_accent(upgrade);
    let pulse = if selected {
        ((time * 5.0).sin().abs() * 3.0).round() as u32
    } else {
        0
    };
    framebuffer.draw_circle(x, y, 34 + pulse, if selected { accent } else { WRECK_MID });
    framebuffer.draw_circle(x, y, 28, WRECK_LIGHT);

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
            framebuffer.draw_line(x, y + 24, x - 22, y - 3, VC20_HULL);
            framebuffer.draw_line(x - 22, y - 3, x - 13, y - 20, VC20_HULL);
            framebuffer.draw_line(x - 13, y - 20, x, y - 10, CANTICLE_COLOR);
            framebuffer.draw_line(x, y - 10, x + 13, y - 20, CANTICLE_COLOR);
            framebuffer.draw_line(x + 13, y - 20, x + 22, y - 3, VC20_HULL);
            framebuffer.draw_line(x + 22, y - 3, x, y + 24, VC20_HULL);
            framebuffer.fill_circle(x, y - 1, 5, CANTICLE_COLOR);
            framebuffer.draw_circle(x, y - 1, 10, VC20_HULL);
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

#[cfg(test)]
mod upgrade_showcase_tests {
    use super::*;

    #[test]
    fn modern_vital_spark_copy_tracks_survival_bonus() {
        assert_eq!(
            vc27_upgrade_effect(UpgradeKind::VitalSpark),
            format!("MAX HULL +{}", VC21_VITAL_SPARK_HULL_BONUS as u32)
        );
    }

    #[test]
    fn all_upgrade_kinds_have_distinct_semantic_categories() {
        let categories = UPGRADE_POOL.map(vc27_upgrade_category);
        for index in 0..categories.len() {
            for other in index + 1..categories.len() {
                assert_ne!(categories[index], categories[other]);
            }
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
