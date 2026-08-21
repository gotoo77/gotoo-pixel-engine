fn mutation_profile(mutation: MutationKind) -> ChoiceProfile<'static> {
    let (category, accent, renderer): (&'static str, Pixel, ChoiceArtRenderer) = match mutation {
        MutationKind::PiercingLance => (
            "WEAPON FORM",
            MUTATION_LIGHT,
            mutation_piercing_lance_art,
        ),
        MutationKind::SplitVolley => (
            "WEAPON SPREAD",
            MUTATION_COLOR,
            mutation_split_volley_art,
        ),
        MutationKind::DeathNova => ("DEATH FIELD", DANGER, mutation_death_nova_art),
        MutationKind::Orbitals => ("RELIC SWARM", VOID_LIGHT, mutation_orbitals_art),
    };
    ChoiceProfile::new(
        mutation_name(mutation),
        category,
        accent,
        ChoiceAssets::procedural(renderer),
    )
}

fn mutation_accent(mutation: MutationKind) -> Pixel {
    mutation_profile(mutation).accent()
}

fn mutation_assets(mutation: MutationKind) -> ChoiceAssets<'static> {
    mutation_profile(mutation).assets()
}

fn mutation_stack(build: &MutationBuild, mutation: MutationKind) -> u32 {
    match mutation {
        MutationKind::PiercingLance => build.piercing_lance,
        MutationKind::SplitVolley => build.split_volley,
        MutationKind::DeathNova => build.death_nova,
        MutationKind::Orbitals => build.orbitals,
    }
}

fn mutation_max(mutation: MutationKind) -> u32 {
    match mutation {
        MutationKind::PiercingLance | MutationKind::DeathNova => 4,
        MutationKind::SplitVolley | MutationKind::Orbitals => 3,
    }
}

fn mutation_effect(mutation: MutationKind) -> &'static str {
    match mutation {
        MutationKind::PiercingLance => "PIERCING LINE BEAM",
        MutationKind::SplitVolley => "ADD SIDE SHOT PAIR",
        MutationKind::DeathNova => "KILLS CLEAR BULLETS",
        MutationKind::Orbitals => "ADD FIRING ORBITAL",
    }
}

fn mutation_detail(mutation: MutationKind) -> &'static str {
    match mutation {
        MutationKind::PiercingLance => "STACKS IMPROVE RATE / DAMAGE",
        MutationKind::SplitVolley => "STACKS ADD WIDER VOLLEYS",
        MutationKind::DeathNova => "STACKS EXPAND CLEAR RADIUS",
        MutationKind::Orbitals => "MAX 3 RELIC SATELLITES",
    }
}

fn render_mutation_showcase(
    framebuffer: &mut Framebuffer,
    game: &MutationProgression,
    choice: &MutationChoice,
    time: f32,
) {
    choice_header(
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
        render_mutation_card(
            framebuffer,
            mutation,
            &game.progression.build,
            &game.mutations,
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
        MUTATION_LIGHT,
        "EVOLVE THE BUILD / ACCEPT THE CONSEQUENCE",
    );
}

#[allow(clippy::too_many_arguments)]
fn render_mutation_card(
    framebuffer: &mut Framebuffer,
    mutation: MutationKind,
    base_build: &BuildState,
    build: &MutationBuild,
    selected: bool,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    time: f32,
) {
    let profile = mutation_profile(mutation);
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
            Pixel::rgb(13, 6, 18),
            Pixel::rgb(24, 8, 26),
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
    framebuffer.draw_text(info_x, y + 54, mutation_effect(mutation), TEXT);
    framebuffer.draw_text(info_x, y + 70, mutation_detail(mutation), WRECK_LIGHT);

    if let Some(name) = synergy_after_mutation(*base_build, *build, mutation) {
        render_synergy_hint(framebuffer, info_x, y + 87, name, selected, time);
    }

    let stack = mutation_stack(build, mutation);
    let max = mutation_max(mutation);
    let next = stack.saturating_add(1).min(max);
    framebuffer.draw_text(
        info_x,
        y + 98,
        &format!("STACK {:02}  >  {:02} / {:02}", stack, next, max),
        if selected { accent } else { WRECK_LIGHT },
    );
    if selected {
        framebuffer.draw_text(x + width as i32 - 53, y + 13, "EVOLVE", accent);
    }
    choice_stack_nodes(framebuffer, info_x, y + 114, stack, max, accent);
}

fn render_mutation_icon(
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

fn mutation_piercing_lance_art(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    _selected: bool,
    time: f32,
) {
    render_mutation_icon(framebuffer, MutationKind::PiercingLance, x, y, time);
}

fn mutation_split_volley_art(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    _selected: bool,
    time: f32,
) {
    render_mutation_icon(framebuffer, MutationKind::SplitVolley, x, y, time);
}

fn mutation_death_nova_art(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    _selected: bool,
    time: f32,
) {
    render_mutation_icon(framebuffer, MutationKind::DeathNova, x, y, time);
}

fn mutation_orbitals_art(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    _selected: bool,
    time: f32,
) {
    render_mutation_icon(framebuffer, MutationKind::Orbitals, x, y, time);
}

#[cfg(test)]
mod mutation_showcase_tests {
    use super::*;

    #[test]
    fn mutation_stack_caps_match_gameplay_caps() {
        assert_eq!(mutation_max(MutationKind::PiercingLance), 4);
        assert_eq!(mutation_max(MutationKind::SplitVolley), 3);
        assert_eq!(mutation_max(MutationKind::DeathNova), 4);
        assert_eq!(mutation_max(MutationKind::Orbitals), 3);
    }

    #[test]
    fn mutations_expose_choice_profile_assets_and_audio() {
        for mutation in MUTATION_POOL {
            let profile = mutation_profile(mutation);
            let (expected_hover, expected_confirm) = match mutation {
                MutationKind::DeathNova => (
                    Some(ChoiceArtId::DeathNova.hover_override_sound()),
                    Some(ChoiceArtId::DeathNova.confirm_override_sound()),
                ),
                _ => (Some(CHOICE_HOVER_SOUND), Some(CHOICE_CONFIRM_SOUND)),
            };
            assert_eq!(profile.label(), mutation_name(mutation));
            assert_eq!(profile.hover_sound(), expected_hover);
            assert_eq!(profile.confirm_sound(), expected_confirm);
        }
    }
}
