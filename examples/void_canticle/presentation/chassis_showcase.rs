fn chassis_accent(chassis: ExosuitChassis) -> Pixel {
    chassis_profile(chassis).accent()
}

fn render_chassis_showcase(
    clean_background: &mut Framebuffer,
    framebuffer: &mut Framebuffer,
    selector: &VoidCanticleV22,
    time: f32,
) {
    clean_background.clear(BG);
    render_grave_orbit_background(clean_background, time * 9.0);
    vc_visual_blit_nearest(
        clean_background,
        framebuffer,
        VC_VISUAL_PRESENTATION_SCALE,
        false,
    );

    vc_visual_draw_centered_text(
        framebuffer,
        26,
        "EXOSUIT",
        3,
        POWER_RELIC_LIGHT,
    );
    vc_visual_draw_centered_text(
        framebuffer,
        58,
        "CHOOSE YOUR VESSEL",
        1,
        WRECK_LIGHT,
    );

    let card_x = 12;
    let card_width = VC_VISUAL_PRESENTATION_WIDTH.saturating_sub(24);
    let card_height = 136_u32;
    let card_start_y = 88_i32;
    let card_gap = 9_i32;

    for (index, chassis) in VC22_CHASSIS.iter().copied().enumerate() {
        let y = card_start_y + index as i32 * (card_height as i32 + card_gap);
        let selected = selector.menu.selected() == Some(index);
        let (hull, shield) = selector.chassis_limits(chassis);
        render_chassis_card(
            framebuffer,
            chassis,
            selected,
            card_x,
            y,
            card_width,
            card_height,
            hull,
            shield,
            time,
        );
    }

    framebuffer.draw_line(36, 534, 324, 534, WRECK_MID);
    vc_visual_draw_centered_text(framebuffer, 549, "UP DOWN / DPAD", 1, TEXT);
    vc_visual_draw_centered_text(
        framebuffer,
        568,
        "SPACE / SOUTH  SELECT",
        1,
        POWER_RELIC_LIGHT,
    );
    vc_visual_draw_centered_text(
        framebuffer,
        600,
        "IDENTITY BEFORE LAUNCH",
        1,
        WRECK_LIGHT,
    );
}

#[allow(clippy::too_many_arguments)]
fn render_chassis_card(
    framebuffer: &mut Framebuffer,
    chassis: ExosuitChassis,
    selected: bool,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    hull: f32,
    shield: f32,
    time: f32,
) {
    let choice_profile = chassis_profile(chassis);
    let accent = choice_profile.accent();
    let panel = if selected {
        Pixel::rgb(11, 14, 27)
    } else {
        Pixel::rgb(7, 9, 18)
    };
    let edge = if selected { accent } else { WRECK_MID };

    framebuffer.fill_rect(x, y, width, height, panel);
    framebuffer.draw_rect(x, y, width, height, edge);
    if selected {
        framebuffer.draw_rect(x + 3, y + 3, width - 6, height - 6, WRECK_LIGHT);
        let travel = ((time * 72.0) as i32).rem_euclid(width.saturating_sub(30) as i32);
        framebuffer.draw_line(x + 15 + travel, y + 1, x + 29 + travel, y + 1, accent);
    }

    render_card_corner(framebuffer, x + 5, y + 5, 1, 1, edge);
    render_card_corner(
        framebuffer,
        x + width as i32 - 6,
        y + 5,
        -1,
        1,
        edge,
    );
    render_card_corner(
        framebuffer,
        x + 5,
        y + height as i32 - 6,
        1,
        -1,
        edge,
    );
    render_card_corner(
        framebuffer,
        x + width as i32 - 6,
        y + height as i32 - 6,
        -1,
        -1,
        edge,
    );

    let ship_x = x + 54;
    let ship_y = y + 67;
    choice_profile.render_art(framebuffer, ship_x, ship_y, selected, time);

    let info_x = x + 108;
    framebuffer.draw_text_scaled(info_x, y + 12, choice_profile.label(), 2, accent);
    framebuffer.draw_text(info_x, y + 33, choice_profile.category(), WRECK_LIGHT);
    if selected {
        framebuffer.draw_text(x + width as i32 - 58, y + 13, "ACTIVE", accent);
    }

    let profile = chassis.profile();
    render_chassis_stat(
        framebuffer,
        info_x,
        y + 52,
        "HULL",
        hull.round() as u32,
        profile.hull_multiplier / 1.50,
        VC20_HULL,
    );
    render_chassis_stat(
        framebuffer,
        info_x,
        y + 70,
        "SHLD",
        shield.round() as u32,
        profile.shield_multiplier / 1.60,
        VC20_ARMOR,
    );
    render_chassis_stat(
        framebuffer,
        info_x,
        y + 88,
        "MOVE",
        (profile.move_multiplier * 100.0).round() as u32,
        profile.move_multiplier / 1.28,
        ART_CYAN_LIGHT,
    );

    framebuffer.draw_text(info_x, y + 108, chassis.passive_name(), CANTICLE_COLOR);
    framebuffer.draw_text(info_x, y + 121, chassis.passive_description(), WRECK_LIGHT);
}

fn render_card_corner(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    sx: i32,
    sy: i32,
    color: Pixel,
) {
    framebuffer.draw_line(x, y, x + sx * 10, y, color);
    framebuffer.draw_line(x, y, x, y + sy * 10, color);
}

fn render_chassis_stat(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    label: &str,
    value: u32,
    ratio: f32,
    color: Pixel,
) {
    framebuffer.draw_text(x, y, label, TEXT);
    framebuffer.draw_text(x + 35, y, &value.to_string(), color);
    render_chassis_stat_bar(framebuffer, x + 77, y + 1, 132, ratio, color);
}

fn render_chassis_stat_bar(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    width: u32,
    ratio: f32,
    color: Pixel,
) {
    let ratio = ratio.clamp(0.0, 1.0);
    framebuffer.fill_rect(x, y, width, 5, WRECK_MID);
    let filled = (width as f32 * ratio).round() as u32;
    if filled > 0 {
        framebuffer.fill_rect(x, y, filled.min(width), 5, color);
    }
    for tick in 1..4 {
        let tx = x + (width * tick / 4) as i32;
        framebuffer.draw_line(tx, y, tx, y + 4, BG);
    }
    framebuffer.draw_rect(x - 1, y - 1, width + 2, 7, WRECK_LIGHT);
}

fn render_chassis_ship(
    framebuffer: &mut Framebuffer,
    chassis: ExosuitChassis,
    x: i32,
    y: i32,
    selected: bool,
    time: f32,
) {
    let accent = chassis_accent(chassis);
    if selected {
        let pulse = ((time * 4.0).sin().abs() * 3.0).round() as u32;
        framebuffer.draw_circle(x, y, 38 + pulse, accent);
        framebuffer.draw_circle(x, y, 32 + pulse / 2, WRECK_MID);
    }

    match chassis {
        ExosuitChassis::Bulwark => {
            framebuffer.fill_rect(x - 23, y - 24, 46, 43, BELL_DARK);
            framebuffer.draw_rect(x - 23, y - 24, 46, 43, BELL_METAL);
            framebuffer.fill_rect(x - 35, y - 15, 12, 30, BELL_METAL);
            framebuffer.fill_rect(x + 24, y - 15, 12, 30, BELL_METAL);
            framebuffer.draw_rect(x - 35, y - 15, 12, 30, ART_GOLD);
            framebuffer.draw_rect(x + 24, y - 15, 12, 30, ART_GOLD);
            framebuffer.draw_line(x - 22, y - 25, x, y - 36, BELL_LIGHT);
            framebuffer.draw_line(x, y - 36, x + 22, y - 25, BELL_LIGHT);
            framebuffer.fill_rect(x - 9, y - 18, 18, 25, KNIGHT_SHADOW);
            framebuffer.draw_rect(x - 9, y - 18, 18, 25, KNIGHT_GOLD);
            framebuffer.fill_circle(x, y - 5, 5, VC20_ARMOR);
            framebuffer.fill_circle(x, y - 5, 2, VC20_ARMOR_LIGHT);
            framebuffer.draw_line(x - 16, y + 20, x - 20, y + 31, THRUSTER);
            framebuffer.draw_line(x + 16, y + 20, x + 20, y + 31, THRUSTER);
            framebuffer.draw_line(x - 28, y + 3, x - 39, y + 12, ART_GOLD);
            framebuffer.draw_line(x + 28, y + 3, x + 39, y + 12, ART_GOLD);
        }
        ExosuitChassis::Pilgrim => {
            render_pilgrim(framebuffer, x, y, false, 0.0, time);
            framebuffer.draw_circle(x, y - 2, 30, PILGRIM_VIOLET);
            framebuffer.draw(x - 30, y - 2, CANTICLE_COLOR);
            framebuffer.draw(x + 30, y - 2, CANTICLE_COLOR);
        }
        ExosuitChassis::Wraith => {
            framebuffer.draw_line(x, y - 38, x, y + 27, WRAITH_CORE);
            framebuffer.draw_line(x - 2, y - 30, x - 2, y + 20, ART_VOID);
            framebuffer.draw_line(x + 2, y - 30, x + 2, y + 20, ART_VOID);
            framebuffer.draw_line(x, y - 29, x - 31, y + 10, WRAITH_GLOW);
            framebuffer.draw_line(x, y - 29, x + 31, y + 10, WRAITH_GLOW);
            framebuffer.draw_line(x - 31, y + 10, x - 11, y + 4, ART_CYAN_LIGHT);
            framebuffer.draw_line(x + 31, y + 10, x + 11, y + 4, ART_CYAN_LIGHT);
            framebuffer.draw_line(x - 24, y + 12, x - 7, y + 25, WRAITH_GLOW);
            framebuffer.draw_line(x + 24, y + 12, x + 7, y + 25, WRAITH_GLOW);
            framebuffer.fill_circle(x, y - 3, 5, WRAITH_GLOW);
            framebuffer.fill_circle(x, y - 3, 2, WRAITH_CORE);
            framebuffer.draw_line(x - 6, y + 24, x - 10, y + 35, ART_CYAN_LIGHT);
            framebuffer.draw_line(x + 6, y + 24, x + 10, y + 35, ART_CYAN_LIGHT);
            framebuffer.draw(x, y - 38, VOID_LIGHT);
        }
    }
}

#[cfg(test)]
mod chassis_showcase_tests {
    use super::*;

    #[test]
    fn chassis_showcase_has_three_distinct_identity_accents() {
        let bulwark = chassis_profile(ExosuitChassis::Bulwark).accent();
        let pilgrim = chassis_profile(ExosuitChassis::Pilgrim).accent();
        let wraith = chassis_profile(ExosuitChassis::Wraith).accent();
        assert_ne!(bulwark, pilgrim);
        assert_ne!(bulwark, wraith);
        assert_ne!(pilgrim, wraith);
    }

    #[test]
    fn three_showcase_cards_fit_above_navigation_footer() {
        let card_height = 136_i32;
        let start = 88_i32;
        let gap = 9_i32;
        let last_bottom = start + 2 * (card_height + gap) + card_height;
        assert!(last_bottom < 534);
        assert!(534 < VC_VISUAL_PRESENTATION_HEIGHT as i32);
    }
}
