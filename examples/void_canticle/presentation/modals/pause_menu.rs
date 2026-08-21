const PAUSE_ITEMS: [(&str, &str); 5] = [
    ("RESUME", "RETURN TO THE CANTICLE"),
    ("RESTART", "BEGIN A NEW PILGRIMAGE"),
    ("CONTROLS", "INPUT / COMBAT REFERENCE"),
    ("BUILD INFO", "INSPECT CURRENT LOADOUT"),
    ("QUIT", "LEAVE THE GRAVE ORBIT"),
];

fn render_pause_menu(
    framebuffer: &mut Framebuffer,
    sustain: &GameplayRuntime,
    pause: &PauseMenuState,
    time: f32,
) {
    choice_header(
        framebuffer,
        "PAUSED",
        "THE GRAVE ORBIT WAITS",
        PILGRIM_VIOLET,
        time,
    );

    let selected = pause.menu.selected();
    for (index, (label, detail)) in PAUSE_ITEMS.into_iter().enumerate() {
        let y = 100 + index as i32 * 70;
        render_pause_item(
            framebuffer,
            label,
            detail,
            selected == Some(index),
            18,
            y,
            214,
            56,
            time,
        );
    }

    render_pause_run_status(framebuffer, sustain, 244, 100, 104, 336, time);

    framebuffer.draw_line(36, 474, 324, 474, WRECK_MID);
    vc_visual_draw_centered_text(framebuffer, 494, "UP DOWN / DPAD", 1, TEXT);
    vc_visual_draw_centered_text(
        framebuffer,
        516,
        "SPACE / SOUTH  SELECT",
        1,
        POWER_RELIC_LIGHT,
    );
    vc_visual_draw_centered_text(
        framebuffer,
        538,
        "ESC / START  RESUME",
        1,
        PILGRIM_VIOLET,
    );
    vc_visual_draw_centered_text(
        framebuffer,
        584,
        "THE CANTICLE CONTINUES WHEN YOU DO",
        1,
        WRECK_LIGHT,
    );
}

#[allow(clippy::too_many_arguments)]
fn render_pause_item(
    framebuffer: &mut Framebuffer,
    label: &str,
    detail: &str,
    selected: bool,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    time: f32,
) {
    let accent = if label == "QUIT" { DANGER } else { PILGRIM_VIOLET };
    choice_card_frame(
        framebuffer,
        x,
        y,
        width,
        height,
        selected,
        ChoiceCardStyle::new(
            accent,
            Pixel::rgb(7, 9, 17),
            Pixel::rgb(12, 10, 25),
        ),
        time,
    );
    framebuffer.draw_text_scaled(
        x + 14,
        y + 9,
        label,
        2,
        if selected { accent } else { TEXT },
    );
    framebuffer.draw_text(x + 14, y + 35, detail, WRECK_LIGHT);
    if selected {
        framebuffer.draw_text(x + width as i32 - 38, y + 10, ">>", accent);
    }
}

fn render_pause_run_status(
    framebuffer: &mut Framebuffer,
    sustain: &GameplayRuntime,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    time: f32,
) {
    let progression = sustain.presentation_progression();
    let chassis = sustain
        .presentation_selected_chassis()
        .unwrap_or(ExosuitChassis::Pilgrim);
    let accent = chassis_accent(chassis);
    let synergy_count = build_synergy_names(progression.progression.build, progression.mutations).len();

    choice_card_frame(
        framebuffer,
        x,
        y,
        width,
        height,
        false,
        ChoiceCardStyle::new(
            accent,
            Pixel::rgb(6, 8, 15),
            Pixel::rgb(6, 8, 15),
        ),
        time,
    );

    vc_visual_draw_centered_text_in_rect(
        framebuffer,
        x,
        width,
        y + 14,
        "RUN",
        1,
        WRECK_LIGHT,
    );
    render_chassis_ship(framebuffer, chassis, x + 52, y + 72, false, time);
    vc_visual_draw_centered_text_in_rect(
        framebuffer,
        x,
        width,
        y + 119,
        chassis.name(),
        1,
        accent,
    );

    framebuffer.draw_line(x + 12, y + 142, x + width as i32 - 12, y + 142, WRECK_MID);
    framebuffer.draw_text(x + 12, y + 158, "ECHO", WRECK_LIGHT);
    framebuffer.draw_text(
        x + 58,
        y + 158,
        &format!("{:02}", progression.progression.level),
        TEXT,
    );
    framebuffer.draw_text(x + 12, y + 184, "SYNR", WRECK_LIGHT);
    framebuffer.draw_text(
        x + 58,
        y + 184,
        &format!("{:02}", synergy_count),
        SYNERGY_GOLD,
    );
    framebuffer.draw_text(x + 12, y + 210, "SUPP", WRECK_LIGHT);

    let support = sustain.augment.map(SustainAugment::name).unwrap_or("NONE");
    let support_label = match support {
        "NANITE REPAIR" => "NANITE",
        "SHIELD CAPACITOR" => "CAPAC",
        _ => "NONE",
    };
    framebuffer.draw_text(x + 49, y + 210, support_label, ART_CYAN_LIGHT);

    framebuffer.draw_line(x + 12, y + 238, x + width as i32 - 12, y + 238, WRECK_MID);
    vc_visual_draw_centered_text_in_rect(
        framebuffer,
        x,
        width,
        y + 258,
        PRESENTATION_VERSION,
        1,
        PILGRIM_VIOLET,
    );
}

fn vc_visual_draw_centered_text_in_rect(
    framebuffer: &mut Framebuffer,
    x: i32,
    width: u32,
    y: i32,
    text: &str,
    scale: u32,
    color: Pixel,
) {
    let glyph_width = 6_i32 * scale as i32;
    let text_width = text.chars().count() as i32 * glyph_width;
    let text_x = x + ((width as i32 - text_width) / 2).max(0);
    framebuffer.draw_text_scaled(text_x, y, text, scale, color);
}

#[cfg(test)]
mod pause_menu_tests {
    use super::*;

    #[test]
    fn pause_has_same_actions_as_gameplay_pause_menu() {
        assert_eq!(PAUSE_ITEMS.len(), 5);
        assert_eq!(PAUSE_ITEMS[0].0, "RESUME");
        assert_eq!(PAUSE_ITEMS[3].0, "BUILD INFO");
        assert_eq!(PAUSE_ITEMS[4].0, "QUIT");
    }

    #[test]
    fn pause_menu_layout_fits_above_navigation_footer() {
        let last_y = 100 + 4 * 70;
        assert!(last_y + 56 < 474);
        assert!(584 < VC_VISUAL_PRESENTATION_HEIGHT as i32);
    }
}
