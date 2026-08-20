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

fn vc27_mutation_assets(mutation: MutationKind) -> Vc27ChoiceAssets<'static> {
    let renderer = match mutation {
        MutationKind::PiercingLance => vc27_mutation_piercing_lance_art as Vc27ChoiceArtRenderer,
        MutationKind::SplitVolley => vc27_mutation_split_volley_art,
        MutationKind::DeathNova => vc27_mutation_death_nova_art,
        MutationKind::Orbitals => vc27_mutation_orbitals_art,
    };
    Vc27ChoiceAssets::procedural(renderer)
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
    vc27_mutation_assets(mutation).render(framebuffer, icon_x, icon_y, selected, time);

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

fn vc27_mutation_piercing_lance_art(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    _selected: bool,
    time: f32,
) {
    vc27_render_mutation_icon(framebuffer, MutationKind::PiercingLance, x, y, time);
}

fn vc27_mutation_split_volley_art(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    _selected: bool,
    time: f32,
) {
    vc27_render_mutation_icon(framebuffer, MutationKind::SplitVolley, x, y, time);
}

fn vc27_mutation_death_nova_art(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    _selected: bool,
    time: f32,
) {
    vc27_render_mutation_icon(framebuffer, MutationKind::DeathNova, x, y, time);
}

fn vc27_mutation_orbitals_art(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    _selected: bool,
    time: f32,
) {
    vc27_render_mutation_icon(framebuffer, MutationKind::Orbitals, x, y, time);
}

#[cfg(test)]
mod mutation_showcase_tests {
    use super::*;

    #[test]
    fn mutation_stack_caps_match_gameplay_caps() {
        assert_eq!(vc27_mutation_max(MutationKind::PiercingLance), 4);
        assert_eq!(vc27_mutation_max(MutationKind::SplitVolley), 3);
        assert_eq!(vc27_mutation_max(MutationKind::DeathNova), 4);
        assert_eq!(vc27_mutation_max(MutationKind::Orbitals), 3);
    }

    #[test]
    fn mutations_expose_choice_assets_and_hover_audio() {
        for mutation in MUTATION_POOL {
            assert_eq!(
                vc27_mutation_assets(mutation).hover_sound(),
                Some(VC27_CHOICE_HOVER_SOUND)
            );
        }
    }
}
