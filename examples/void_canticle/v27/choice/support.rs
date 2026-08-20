fn vc27_support_profile(augment: Vc23SustainAugment) -> Vc27ChoiceProfile<'static> {
    let (category, accent, renderer): (&'static str, Pixel, Vc27ChoiceArtRenderer) = match augment {
        Vc23SustainAugment::NaniteRepair => (
            "HULL SUSTAIN",
            VC20_HULL,
            vc27_support_nanite_art,
        ),
        Vc23SustainAugment::ShieldCapacitor => (
            "SHIELD SUSTAIN",
            ART_CYAN_LIGHT,
            vc27_support_capacitor_art,
        ),
    };
    Vc27ChoiceProfile::new(
        augment.name(),
        category,
        accent,
        Vc27ChoiceAssets::procedural(renderer),
    )
}

fn vc27_support_accent(augment: Vc23SustainAugment) -> Pixel {
    vc27_support_profile(augment).accent()
}

fn vc27_support_trigger(augment: Vc23SustainAugment) -> String {
    match augment {
        Vc23SustainAugment::NaniteRepair => {
            format!("AFTER {:.1}S WITHOUT DAMAGE", VC23_NANITE_DELAY)
        }
        Vc23SustainAugment::ShieldCapacitor => {
            format!("AFTER {:.1}S WITHOUT DAMAGE", VC23_CAPACITOR_DELAY)
        }
    }
}

fn vc27_support_effect(augment: Vc23SustainAugment) -> String {
    match augment {
        Vc23SustainAugment::NaniteRepair => {
            format!("HULL +{:.2} / SEC", VC23_NANITE_REPAIR_PER_SECOND)
        }
        Vc23SustainAugment::ShieldCapacitor => {
            format!("SHIELD +{:.0} / SEC", VC23_CAPACITOR_REGEN_PER_SECOND)
        }
    }
}

fn vc27_support_detail(augment: Vc23SustainAugment) -> &'static str {
    match augment {
        Vc23SustainAugment::NaniteRepair => "AUTONOMOUS STRUCTURAL REPAIR",
        Vc23SustainAugment::ShieldCapacitor => "FAST DEFENSIVE RECHARGE",
    }
}

fn vc27_render_support_showcase(
    framebuffer: &mut Framebuffer,
    sustain: &VoidCanticleV23Sustain,
    time: f32,
) {
    vc27_choice_header(
        framebuffer,
        "SUPPORT",
        "CHOOSE A SUSTAIN MODULE",
        ART_CYAN_LIGHT,
        time,
    );

    let v14 = sustain.game.v20().game.v14();
    if let Some(name) = vc27_primary_active_synergy(v14.progression.build, v14.mutations) {
        vc27_render_active_synergy_strip(framebuffer, name, time);
    }

    let card_x = 12_i32;
    let card_width = VC_VISUAL_PRESENTATION_WIDTH.saturating_sub(24);
    let card_height = 170_u32;
    let card_start_y = 112_i32;
    let card_gap = 16_i32;

    for (index, augment) in VC23_SUSTAIN_AUGMENTS.iter().copied().enumerate() {
        let y = card_start_y + index as i32 * (card_height as i32 + card_gap);
        let selected = sustain.menu.selected() == Some(index);
        vc27_render_support_card(
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

    vc27_choice_footer(
        framebuffer,
        ART_CYAN_LIGHT,
        "SUSTAIN THE PILGRIM / ENDURE THE ORBIT",
    );
}

#[allow(clippy::too_many_arguments)]
fn vc27_render_support_card(
    framebuffer: &mut Framebuffer,
    augment: Vc23SustainAugment,
    selected: bool,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    time: f32,
) {
    let profile = vc27_support_profile(augment);
    let accent = profile.accent();
    vc27_choice_card_frame(
        framebuffer,
        x,
        y,
        width,
        height,
        selected,
        Vc27ChoiceCardStyle::new(
            accent,
            Pixel::rgb(6, 12, 18),
            Pixel::rgb(7, 20, 28),
        ),
        time,
    );

    let icon_x = x + 62;
    let icon_y = y + 84;
    vc27_choice_icon_shell(framebuffer, icon_x, icon_y, accent, selected, time);
    profile.render_art(framebuffer, icon_x, icon_y, selected, time);

    let info_x = x + 122;
    framebuffer.draw_text(info_x, y + 18, profile.category(), WRECK_LIGHT);
    framebuffer.draw_text_scaled(info_x, y + 37, profile.label(), 2, accent);
    framebuffer.draw_text(info_x, y + 70, &vc27_support_trigger(augment), TEXT);
    framebuffer.draw_text(info_x, y + 91, &vc27_support_effect(augment), accent);
    framebuffer.draw_text(info_x, y + 114, vc27_support_detail(augment), WRECK_LIGHT);
    framebuffer.draw_text(info_x, y + 139, "ONE MODULE / RUN", WRECK_LIGHT);
    if selected {
        framebuffer.draw_text(x + width as i32 - 53, y + 19, "INSTALL", accent);
    }
}

fn vc27_render_support_icon(
    framebuffer: &mut Framebuffer,
    augment: Vc23SustainAugment,
    x: i32,
    y: i32,
    selected: bool,
    time: f32,
) {
    match augment {
        Vc23SustainAugment::NaniteRepair => {
            framebuffer.draw_rect(x - 15, y - 18, 30, 36, VC20_HULL);
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
        Vc23SustainAugment::ShieldCapacitor => {
            framebuffer.draw_circle(x, y, 23, ART_CYAN_LIGHT);
            framebuffer.draw_circle(x, y, 16, VC20_ARMOR);
            framebuffer.fill_circle(x, y, 6, VC20_ARMOR_LIGHT);
            framebuffer.draw_line(x - 26, y, x - 15, y, ART_CYAN_LIGHT);
            framebuffer.draw_line(x + 15, y, x + 26, y, ART_CYAN_LIGHT);
            framebuffer.draw_line(x, y - 26, x, y - 15, ART_CYAN_LIGHT);
            framebuffer.draw_line(x, y + 15, x, y + 26, ART_CYAN_LIGHT);
        }
    }
}

fn vc27_support_nanite_art(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    selected: bool,
    time: f32,
) {
    vc27_render_support_icon(
        framebuffer,
        Vc23SustainAugment::NaniteRepair,
        x,
        y,
        selected,
        time,
    );
}

fn vc27_support_capacitor_art(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    selected: bool,
    time: f32,
) {
    vc27_render_support_icon(
        framebuffer,
        Vc23SustainAugment::ShieldCapacitor,
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
        assert!(vc27_support_trigger(Vc23SustainAugment::NaniteRepair)
            .contains(&format!("{:.1}", VC23_NANITE_DELAY)));
        assert!(vc27_support_trigger(Vc23SustainAugment::ShieldCapacitor)
            .contains(&format!("{:.1}", VC23_CAPACITOR_DELAY)));
        assert!(vc27_support_effect(Vc23SustainAugment::NaniteRepair)
            .contains(&format!("{:.2}", VC23_NANITE_REPAIR_PER_SECOND)));
        assert!(vc27_support_effect(Vc23SustainAugment::ShieldCapacitor)
            .contains(&format!("{:.0}", VC23_CAPACITOR_REGEN_PER_SECOND)));
    }

    #[test]
    fn support_modules_expose_choice_profile_assets_and_audio() {
        for augment in VC23_SUSTAIN_AUGMENTS {
            let profile = vc27_support_profile(augment);
            assert_eq!(profile.label(), augment.name());
            assert_eq!(profile.hover_sound(), Some(VC27_CHOICE_HOVER_SOUND));
            assert_eq!(profile.confirm_sound(), Some(VC27_CHOICE_CONFIRM_SOUND));
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
