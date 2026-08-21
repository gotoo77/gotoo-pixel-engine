const CONTROL_ROWS: [(&str, &str, &str, Pixel); 6] = [
    ("MOVE", "ARROWS / WASD", "STICK / DPAD", ART_CYAN_LIGHT),
    ("FIRE", "SPACE", "SOUTH", BOLT_EDGE),
    ("FOCUS", "SHIFT", "LB", PILGRIM_VIOLET),
    ("CANTICLE", "X", "EAST", CANTICLE_COLOR),
    ("EMP", "C", "WEST", PRESENTATION_ARMOR_LIGHT),
    ("PAUSE", "ESC", "START", POWER_RELIC_LIGHT),
];

fn render_controls_reference(framebuffer: &mut Framebuffer, time: f32) {
    choice_header(
        framebuffer,
        "CONTROLS",
        "PILGRIM INPUT / COMBAT REFERENCE",
        ART_CYAN_LIGHT,
        time,
    );

    framebuffer.draw_text(24, 96, "ACTION", WRECK_LIGHT);
    framebuffer.draw_text(145, 96, "KEYBOARD", WRECK_LIGHT);
    framebuffer.draw_text(264, 96, "GAMEPAD", WRECK_LIGHT);
    framebuffer.draw_line(20, 112, 340, 112, WRECK_MID);

    for (index, (action, keyboard, gamepad, accent)) in CONTROL_ROWS.into_iter().enumerate() {
        let y = 128 + index as i32 * 58;
        render_control_row(
            framebuffer,
            action,
            keyboard,
            gamepad,
            accent,
            18,
            y,
            324,
            46,
            time,
            index,
        );
    }

    framebuffer.draw_line(36, 494, 324, 494, WRECK_MID);
    vc_visual_draw_centered_text(
        framebuffer,
        514,
        "ESC / START / SOUTH  BACK",
        1,
        POWER_RELIC_LIGHT,
    );
    vc_visual_draw_centered_text(
        framebuffer,
        554,
        "FOCUS = PRECISION MOVEMENT",
        1,
        WRECK_LIGHT,
    );
    vc_visual_draw_centered_text(
        framebuffer,
        574,
        "EMP = SHIELD DISRUPTION",
        1,
        PRESENTATION_ARMOR_LIGHT,
    );
    vc_visual_draw_centered_text(
        framebuffer,
        594,
        "CANTICLE = CORE RELEASE",
        1,
        CANTICLE_COLOR,
    );
}

#[allow(clippy::too_many_arguments)]
fn render_control_row(
    framebuffer: &mut Framebuffer,
    action: &str,
    keyboard: &str,
    gamepad: &str,
    accent: Pixel,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    time: f32,
    index: usize,
) {
    let pulse = ((time * 2.0 + index as f32 * 0.7).sin().abs() * 0.5) > 0.42;
    let edge = if pulse { accent } else { WRECK_MID };
    framebuffer.fill_rect(x, y, width, height, Pixel::rgb(7, 10, 18));
    framebuffer.draw_rect(x, y, width, height, edge);
    choice_card_corner(framebuffer, x + 5, y + 5, 1, 1, edge);
    choice_card_corner(
        framebuffer,
        x + width as i32 - 6,
        y + height as i32 - 6,
        -1,
        -1,
        edge,
    );

    framebuffer.fill_rect(x + 12, y + 12, 7, 22, accent);
    framebuffer.draw_text_scaled(x + 30, y + 12, action, 2, accent);
    framebuffer.draw_text(x + 128, y + 17, keyboard, TEXT);
    framebuffer.draw_text(x + 246, y + 17, gamepad, TEXT);
}

#[cfg(test)]
mod controls_modal_tests {
    use super::*;

    #[test]
    fn controls_reference_includes_emp_binding() {
        assert!(CONTROL_ROWS.iter().any(|(action, keyboard, gamepad, _)| {
            *action == "EMP" && *keyboard == "C" && *gamepad == "WEST"
        }));
    }

    #[test]
    fn six_control_rows_fit_inside_presentation() {
        let last_y = 128 + 5 * 58;
        assert!(last_y + 46 < 494);
        assert!(594 < VC_VISUAL_PRESENTATION_HEIGHT as i32);
    }
}
