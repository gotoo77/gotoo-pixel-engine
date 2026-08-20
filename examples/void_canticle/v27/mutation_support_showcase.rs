fn vc27_mutation_accent(mutation: MutationKind) -> Pixel {
    match mutation {
        MutationKind::PiercingLance => MUTATION_LIGHT,
        MutationKind::SplitVolley => MUTATION_COLOR,
        MutationKind::DeathNova => DANGER,
        MutationKind::Orbitals => VOID_LIGHT,
    }
}

fn vc27_mutation_category(mutation: MutationKind) -> &'static str {
    match mutation {
        MutationKind::PiercingLance => "WEAPON FORM",
        MutationKind::SplitVolley => "WEAPON SPREAD",
        MutationKind::DeathNova => "DEATH FIELD",
        MutationKind::Orbitals => "RELIC SWARM",
    }
}

fn vc27_mutation_stack(build: &MutationBuild, mutation: MutationKind) -> u32 {
    match mutation {
        MutationKind::PiercingLance => build.piercing_lance,
        MutationKind::SplitVolley => build.split_volley,
        MutationKind::DeathNova => build.death_nova,
        MutationKind::Orbitals => build.orbitals,
    }
}

fn vc27_mutation_max(mutation: MutationKind) -> u32 {
    match mutation {
        MutationKind::PiercingLance | MutationKind::DeathNova => 4,
        MutationKind::SplitVolley | MutationKind::Orbitals => 3,
    }
}

fn vc27_mutation_effect(mutation: MutationKind) -> &'static str {
    match mutation {
        MutationKind::PiercingLance => "PIERCING LINE BEAM",
        MutationKind::SplitVolley => "ADD SIDE SHOT PAIR",
        MutationKind::DeathNova => "KILLS CLEAR BULLETS",
        MutationKind::Orbitals => "ADD FIRING ORBITAL",
    }
}

fn vc27_mutation_detail(mutation: MutationKind) -> &'static str {
    match mutation {
        MutationKind::PiercingLance => "STACKS IMPROVE RATE / DAMAGE",
        MutationKind::SplitVolley => "STACKS ADD WIDER VOLLEYS",
        MutationKind::DeathNova => "STACKS EXPAND CLEAR RADIUS",
        MutationKind::Orbitals => "MAX 3 RELIC SATELLITES",
    }
}

fn vc27_render_mutation_showcase(
    framebuffer: &mut Framebuffer,
    game: &VoidCanticleV14,
    choice: &MutationChoice,
    time: f32,
) {
    vc27_choice_header(
        framebuffer,
        "MUTATION",
        "BUILD EVOLVES / EMBRACE THE VOID",
        MUTATION_LIGHT,
        time,
    );

    let card_x = 12_i32;
    let card_width = VC_VISUAL_PRESENTATION_WIDTH.saturating_sub(24);
    let card_height = 130_u32;
    let card_start_y = 94_i32;
    let card_gap = 10_i32;

    for (index, mutation) in choice.offers.iter().copied().enumerate() {
        let y = card_start_y + index as i32 * (card_height as i32 + card_gap);
        let selected = choice.menu.selected() == Some(index);
        vc27_render_mutation_card(
            framebuffer,
            mutation,
            &game.mutations,
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
        MUTATION_LIGHT,
        "EVOLVE THE BUILD / ACCEPT THE CONSEQUENCE",
    );
}

#[allow(clippy::too_many_arguments)]
fn vc27_render_mutation_card(
    framebuffer: &mut Framebuffer,
    mutation: MutationKind,
    build: &MutationBuild,
    selected: bool,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    time: f32,
) {
    let accent = vc27_mutation_accent(mutation);
    vc27_choice_card_frame(
        framebuffer,
        x,
        y,
        width,
        height,
        selected,
        Vc27ChoiceCardStyle::new(
            accent,
            Pixel::rgb(13, 6, 18),
            Pixel::rgb(24, 8, 26),
        ),
        time,
    );

    let icon_x = x + 58;
    let icon_y = y + 64;
    vc27_choice_icon_shell(framebuffer, icon_x, icon_y, accent, selected, time);
    vc27_render_mutation_icon(framebuffer, mutation, icon_x, icon_y, time);

    let info_x = x + 112;
    framebuffer.draw_text(info_x, y + 12, vc27_mutation_category(mutation), WRECK_LIGHT);
    framebuffer.draw_text_scaled(info_x, y + 28, mutation_name(mutation), 2, accent);
    framebuffer.draw_text(info_x, y + 54, vc27_mutation_effect(mutation), TEXT);
    framebuffer.draw_text(info_x, y + 70, vc27_mutation_detail(mutation), WRECK_LIGHT);

    let stack = vc27_mutation_stack(build, mutation);
    let max = vc27_mutation_max(mutation);
    let next = stack.saturating_add(1).min(max);
    framebuffer.draw_text(
        info_x,
        y + 94,
        &format!("STACK {:02}  >  {:02} / {:02}", stack, next, max),
        if selected { accent } else { WRECK_LIGHT },
    );
    if selected {
        framebuffer.draw_text(x + width as i32 - 53, y + 13, "EVOLVE", accent);
    }
    vc27_choice_stack_nodes(framebuffer, info_x, y + 112, stack, max, accent);
}

fn vc27_render_mutation_icon(
    framebuffer: &mut Framebuffer,
    mutation: MutationKind,
    x: i32,
    y: i32,
    time: f32,
) {
    match mutation {
        MutationKind::PiercingLance => {
            framebuffer.draw_line(x, y + 25, x, y - 26, MUTATION_LIGHT);
            framebuffer.draw_line(x - 3, y + 20, x - 1, y - 22, MUTATION_COLOR);
            framebuffer.draw_line(x + 3, y + 20, x + 1, y - 22, MUTATION_COLOR);
            framebuffer.draw_line(x - 12, y + 15, x, y + 25, VOID_LIGHT);
            framebuffer.draw_line(x + 12, y + 15, x, y + 25, VOID_LIGHT);
            framebuffer.fill_circle(x, y - 25, 3, CANTICLE_COLOR);
        }
        MutationKind::SplitVolley => {
            for (dx, top_x) in [(-18, -27), (-7, -13), (7, 13), (18, 27)] {
                framebuffer.draw_line(x + dx, y + 22, x + top_x, y - 23, MUTATION_COLOR);
                framebuffer.fill_circle(x + top_x, y - 23, 2, MUTATION_LIGHT);
            }
            framebuffer.draw_line(x, y + 25, x, y - 12, CANTICLE_COLOR);
        }
        MutationKind::DeathNova => {
            let pulse = ((time * 6.0).sin().abs() * 3.0).round() as u32;
            framebuffer.draw_circle(x, y, 23 + pulse, DANGER);
            framebuffer.draw_circle(x, y, 15, MUTATION_COLOR);
            framebuffer.fill_circle(x, y, 5, VOID_DANGER);
            for (dx, dy) in [(0, -28), (28, 0), (0, 28), (-28, 0)] {
                framebuffer.draw_line(x, y, x + dx, y + dy, MUTATION_LIGHT);
            }
        }
        MutationKind::Orbitals => {
            framebuffer.draw_circle(x, y, 23, VOID_LIGHT);
            framebuffer.fill_circle(x, y, 5, PILGRIM_VIOLET);
            for (dx, dy) in [(0, -24), (21, 12), (-21, 12)] {
                framebuffer.fill_circle(x + dx, y + dy, 4, MUTATION_COLOR);
                framebuffer.draw_circle(x + dx, y + dy, 6, MUTATION_LIGHT);
            }
        }
    }
}

fn vc27_support_accent(augment: Vc23SustainAugment) -> Pixel {
    match augment {
        Vc23SustainAugment::NaniteRepair => VC20_HULL,
        Vc23SustainAugment::ShieldCapacitor => ART_CYAN_LIGHT,
    }
}

fn vc27_support_category(augment: Vc23SustainAugment) -> &'static str {
    match augment {
        Vc23SustainAugment::NaniteRepair => "HULL SUSTAIN",
        Vc23SustainAugment::ShieldCapacitor => "SHIELD SUSTAIN",
    }
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
    let accent = vc27_support_accent(augment);
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
    vc27_render_support_icon(framebuffer, augment, icon_x, icon_y, selected, time);

    let info_x = x + 122;
    framebuffer.draw_text(info_x, y + 18, vc27_support_category(augment), WRECK_LIGHT);
    framebuffer.draw_text_scaled(info_x, y + 37, augment.name(), 2, accent);
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

#[cfg(test)]
mod mutation_support_showcase_tests {
    use super::*;

    #[test]
    fn mutation_stack_caps_match_gameplay_caps() {
        assert_eq!(vc27_mutation_max(MutationKind::PiercingLance), 4);
        assert_eq!(vc27_mutation_max(MutationKind::SplitVolley), 3);
        assert_eq!(vc27_mutation_max(MutationKind::DeathNova), 4);
        assert_eq!(vc27_mutation_max(MutationKind::Orbitals), 3);
    }

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
    fn two_support_cards_fit_above_common_footer() {
        let start = 112_i32;
        let height = 170_i32;
        let gap = 16_i32;
        let bottom = start + height + gap + height;
        assert!(bottom < 505);
    }
}
