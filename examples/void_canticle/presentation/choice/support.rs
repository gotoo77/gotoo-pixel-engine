fn support_profile(augment: SustainAugment) -> ChoiceProfile<'static> {
    let (category, accent, renderer): (&'static str, Pixel, ChoiceArtRenderer) = match augment {
        SustainAugment::NaniteRepair => (
            "HULL SUSTAIN",
            PRESENTATION_HULL_COLOR,
            support_nanite_art,
        ),
        SustainAugment::ShieldCapacitor => (
            "SHIELD SUSTAIN",
            ART_CYAN_LIGHT,
            support_capacitor_art,
        ),
    };
    ChoiceProfile::new(
        augment.name(),
        category,
        accent,
        ChoiceAssets::procedural(renderer),
    )
}

fn support_accent(augment: SustainAugment) -> Pixel {
    support_profile(augment).accent()
}

fn support_trigger(augment: SustainAugment) -> String {
    match augment {
        SustainAugment::NaniteRepair => format!("AFTER {:.1}S WITHOUT DAMAGE", NANITE_DELAY),
        SustainAugment::ShieldCapacitor => {
            format!("AFTER {:.1}S WITHOUT DAMAGE", CAPACITOR_DELAY)
        }
    }
}

fn support_effect(augment: SustainAugment) -> String {
    match augment {
        SustainAugment::NaniteRepair => format!("HULL +{:.2} / SEC", NANITE_REPAIR_PER_SECOND),
        SustainAugment::ShieldCapacitor => {
            format!("SHIELD +{:.0} / SEC", CAPACITOR_REGEN_PER_SECOND)
        }
    }
}

fn support_detail(augment: SustainAugment) -> &'static str {
    match augment {
        SustainAugment::NaniteRepair => "AUTONOMOUS STRUCTURAL REPAIR",
        SustainAugment::ShieldCapacitor => "FAST DEFENSIVE RECHARGE",
    }
}

fn render_support_showcase(
    framebuffer: &mut Framebuffer,
    sustain: &GameplayRuntime,
    time: f32,
) {
    choice_header(
        framebuffer,
        "SUPPORT",
        "CHOOSE A SUSTAIN MODULE",
        ART_CYAN_LIGHT,
        time,
    );

    let progression = sustain.presentation_progression();
    if let Some(name) = primary_active_synergy(progression.progression.build, progression.mutations) {
        render_active_synergy_strip(framebuffer, name, time);
    }

    let card_x = 12_i32;
    let card_width = VC_VISUAL_PRESENTATION_WIDTH.saturating_sub(24);
    let card_height = 170_u32;
    let card_start_y = 112_i32;
    let card_gap = 16_i32;

    for (index, augment) in SUSTAIN_OPTIONS.iter().copied().enumerate() {
        let y = card_start_y + index as i32 * (card_height as i32 + card_gap);
        let selected = sustain.menu.selected() == Some(index);
        render_support_card(
            framebuffer,
            augment,
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
        ART_CYAN_LIGHT,
        "SUSTAIN THE PILGRIM / ENDURE THE ORBIT",
    );
}

#[allow(clippy::too_many_arguments)]
fn render_support_card(
    framebuffer: &mut Framebuffer,
    augment: SustainAugment,
    selected: bool,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    time: f32,
) {
    let profile = support_profile(augment);
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
            Pixel::rgb(6, 12, 18),
            Pixel::rgb(7, 20, 28),
        ),
        time,
    );

    let icon_x = x + 62;
    let icon_y = y + 84;
    choice_icon_shell(framebuffer, icon_x, icon_y, accent, selected, time);
    profile.render_art(framebuffer, icon_x, icon_y, selected, time);

    let info_x = x + 122;
    framebuffer.draw_text(info_x, y + 18, profile.category(), WRECK_LIGHT);
    framebuffer.draw_text_scaled(info_x, y + 37, profile.label(), 2, accent);
    framebuffer.draw_text(info_x, y + 70, &support_trigger(augment), TEXT);
    framebuffer.draw_text(info_x, y + 91, &support_effect(augment), accent);
    framebuffer.draw_text(info_x, y + 114, support_detail(augment), WRECK_LIGHT);
    framebuffer.draw_text(info_x, y + 139, "ONE MODULE / RUN", WRECK_LIGHT);
    if selected {
        framebuffer.draw_text(x + width as i32 - 53, y + 19, "INSTALL", accent);
    }
}

fn render_support_icon(
    framebuffer: &mut Framebuffer,
    augment: SustainAugment,
    x: i32,
    y: i32,
    selected: bool,
    time: f32,
) {
    match augment {
        SustainAugment::NaniteRepair => {
            framebuffer.draw_rect(x - 15, y - 18, 30, 36, PRESENTATION_HULL_COLOR);
            framebuffer.fill_rect(x - 3, y - 12, 6, 24, CANTICLE_COLOR);
            framebuffer.fill_rect(x - 12, y - 3, 24, 6, CANTICLE_COLOR);
            let orbit = if selected {
                ((time * 8.0).sin() * 3.0).round() as i32
            } else {
                0
            };
            for (dx, dy) in [(-24, -16), (24, -14), (-23, 17), (23, 18)] {
                framebuffer.fill_circle(x + dx + orbit, y + dy, 2, MUTATION_LIGHT);
            }
        }
        SustainAugment::ShieldCapacitor => {
            framebuffer.draw_circle(x, y, 23, ART_CYAN_LIGHT);
            framebuffer.draw_circle(x, y, 16, PRESENTATION_ARMOR_COLOR);
            framebuffer.fill_circle(x, y, 6, PRESENTATION_ARMOR_LIGHT);
            framebuffer.draw_line(x - 26, y, x - 15, y, ART_CYAN_LIGHT);
            framebuffer.draw_line(x + 15, y, x + 26, y, ART_CYAN_LIGHT);
            framebuffer.draw_line(x, y - 26, x, y - 15, ART_CYAN_LIGHT);
            framebuffer.draw_line(x, y + 15, x, y + 26, ART_CYAN_LIGHT);
        }
    }
}

fn support_nanite_art(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    selected: bool,
    time: f32,
) {
    render_support_icon(
        framebuffer,
        SustainAugment::NaniteRepair,
        x,
        y,
        selected,
        time,
    );
}

fn support_capacitor_art(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    selected: bool,
    time: f32,
) {
    render_support_icon(
        framebuffer,
        SustainAugment::ShieldCapacitor,
        x,
        y,
        selected,
        time,
    );
}

#[cfg(test)]
mod support_showcase_tests {
    use super::*;

    #[test]
    fn support_copy_tracks_real_sustain_constants() {
        assert!(support_trigger(SustainAugment::NaniteRepair)
            .contains(&format!("{:.1}", NANITE_DELAY)));
        assert!(support_trigger(SustainAugment::ShieldCapacitor)
            .contains(&format!("{:.1}", CAPACITOR_DELAY)));
        assert!(support_effect(SustainAugment::NaniteRepair)
            .contains(&format!("{:.2}", NANITE_REPAIR_PER_SECOND)));
        assert!(support_effect(SustainAugment::ShieldCapacitor)
            .contains(&format!("{:.0}", CAPACITOR_REGEN_PER_SECOND)));
    }

    #[test]
    fn support_modules_expose_choice_profile_assets_and_audio() {
        for augment in SUSTAIN_OPTIONS {
            let profile = support_profile(augment);
            let (expected_hover, expected_confirm) = match augment {
                SustainAugment::NaniteRepair => (
                    Some(ChoiceArtId::NaniteRepair.hover_override_sound()),
                    Some(ChoiceArtId::NaniteRepair.confirm_override_sound()),
                ),
                SustainAugment::ShieldCapacitor => {
                    (Some(CHOICE_HOVER_SOUND), Some(CHOICE_CONFIRM_SOUND))
                }
            };
            assert_eq!(profile.label(), augment.name());
            assert_eq!(profile.hover_sound(), expected_hover);
            assert_eq!(profile.confirm_sound(), expected_confirm);
        }
    }

    #[test]
    fn two_support_cards_fit_above_common_footer() {
        let start = 112_i32;
        let height = 170_i32;
        let gap = 16_i32;
        let bottom = start + height + gap + height;
        assert!(bottom < 505);
    }
}
