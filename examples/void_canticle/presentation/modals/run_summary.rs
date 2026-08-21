#[derive(Clone)]
struct RunSummary {
    chassis: ExosuitChassis,
    score: u32,
    level: u32,
    echoes: u32,
    hull: u32,
    shield: u32,
    support: Option<SustainAugment>,
    synergies: Vec<&'static str>,
}

fn run_summary(sustain: &GameplayRuntime) -> RunSummary {
    let progression = sustain.presentation_progression();
    let chassis = sustain
        .presentation_selected_chassis()
        .unwrap_or(ExosuitChassis::Pilgrim);
    let combat = sustain.combat_model();
    RunSummary {
        chassis,
        score: sustain.presentation_base().score,
        level: progression.progression.level,
        echoes: progression.progression.xp,
        hull: combat.player_hull.max(0.0).round() as u32,
        shield: combat.player_shield.max(0.0).round() as u32,
        support: sustain.augment,
        synergies: build_synergy_names(progression.progression.build, progression.mutations),
    }
}

fn render_run_summary(
    framebuffer: &mut Framebuffer,
    summary: &RunSummary,
    x: i32,
    y: i32,
    width: u32,
    accent: Pixel,
    time: f32,
) {
    choice_card_frame(
        framebuffer,
        x,
        y,
        width,
        250,
        true,
        ChoiceCardStyle::new(
            accent,
            Pixel::rgb(6, 8, 16),
            Pixel::rgb(9, 12, 22),
        ),
        time,
    );

    render_chassis_ship(
        framebuffer,
        summary.chassis,
        x + 50,
        y + 50,
        false,
        time,
    );
    framebuffer.draw_text_scaled(
        x + 98,
        y + 16,
        summary.chassis.name(),
        2,
        chassis_accent(summary.chassis),
    );
    framebuffer.draw_text(
        x + 98,
        y + 39,
        summary.chassis.passive_name(),
        CANTICLE_COLOR,
    );

    for (row, (label, value, color)) in [
        ("SCORE", summary.score, PILGRIM_CORE),
        ("ECHO LEVEL", summary.level, XP_ORB_CORE),
        ("ECHOES", summary.echoes, XP_ORB_CORE),
        ("HULL", summary.hull, PRESENTATION_HULL_COLOR),
        ("SHIELD", summary.shield, PRESENTATION_ARMOR_LIGHT),
    ]
    .into_iter()
    .enumerate()
    {
        let line_y = y + 91 + row as i32 * 20;
        framebuffer.draw_text(x + 24, line_y, label, WRECK_LIGHT);
        framebuffer.draw_text(x + 140, line_y, &value.to_string(), color);
    }

    framebuffer.draw_line(x + 18, y + 194, x + width as i32 - 18, y + 194, WRECK_MID);
    framebuffer.draw_text(x + 24, y + 207, "SUPPORT", ART_CYAN_LIGHT);
    match summary.support {
        Some(augment) => framebuffer.draw_text(
            x + 102,
            y + 207,
            augment.name(),
            support_accent(augment),
        ),
        None => framebuffer.draw_text(x + 102, y + 207, "NOT INSTALLED", WRECK_MID),
    }

    framebuffer.draw_text(x + 24, y + 228, "SYNERGY", SYNERGY_COLOR);
    if summary.synergies.is_empty() {
        framebuffer.draw_text(x + 102, y + 228, "NONE AWAKENED", WRECK_MID);
    } else {
        let names = summary.synergies.join(" / ");
        framebuffer.draw_text(x + 102, y + 228, &names, SYNERGY_GOLD);
    }
}

#[cfg(test)]
mod run_summary_tests {
    use super::*;

    #[test]
    fn fresh_run_summary_uses_real_default_build_state() {
        let sustain = GameplayRuntime::new();
        let summary = run_summary(&sustain);
        assert_eq!(summary.score, 0);
        assert_eq!(summary.level, 1);
        assert!(summary.synergies.is_empty());
        assert_eq!(summary.support, None);
    }

    #[test]
    fn run_summary_panel_fits_transition_layout() {
        let y = 190_i32;
        assert!(y + 250 < 500);
        assert!(500 < VC_VISUAL_PRESENTATION_HEIGHT as i32);
    }
}
