#[derive(Debug, Clone, Copy)]
struct Vc27ChoiceCardStyle {
    accent: Pixel,
    panel_idle: Pixel,
    panel_selected: Pixel,
}

impl Vc27ChoiceCardStyle {
    const fn new(accent: Pixel, panel_idle: Pixel, panel_selected: Pixel) -> Self {
        Self {
            accent,
            panel_idle,
            panel_selected,
        }
    }
}

fn vc27_choice_header(
    framebuffer: &mut Framebuffer,
    title: &str,
    subtitle: &str,
    accent: Pixel,
    time: f32,
) {
    vc_visual_draw_centered_text(framebuffer, 24, title, 3, accent);
    vc_visual_draw_centered_text(framebuffer, 57, subtitle, 1, WRECK_LIGHT);
    framebuffer.draw_line(36, 78, 324, 78, WRECK_MID);
    let spark = ((time * 55.0) as i32).rem_euclid(272);
    framebuffer.draw_line(44 + spark, 78, 54 + spark, 78, accent);
}

#[allow(clippy::too_many_arguments)]
fn vc27_choice_card_frame(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    selected: bool,
    style: Vc27ChoiceCardStyle,
    time: f32,
) {
    let panel = if selected {
        style.panel_selected
    } else {
        style.panel_idle
    };
    let edge = if selected { style.accent } else { WRECK_MID };

    framebuffer.fill_rect(x, y, width, height, panel);
    framebuffer.draw_rect(x, y, width, height, edge);
    vc27_choice_card_corner(framebuffer, x + 5, y + 5, 1, 1, edge);
    vc27_choice_card_corner(
        framebuffer,
        x + width as i32 - 6,
        y + 5,
        -1,
        1,
        edge,
    );
    vc27_choice_card_corner(
        framebuffer,
        x + 5,
        y + height as i32 - 6,
        1,
        -1,
        edge,
    );
    vc27_choice_card_corner(
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
        framebuffer.draw_line(
            x + 20 + travel,
            y + 1,
            x + 36 + travel,
            y + 1,
            style.accent,
        );
    }
}

fn vc27_choice_card_corner(
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

fn vc27_choice_icon_shell(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    accent: Pixel,
    selected: bool,
    time: f32,
) {
    let pulse = if selected {
        ((time * 5.0).sin().abs() * 3.0).round() as u32
    } else {
        0
    };
    framebuffer.draw_circle(
        x,
        y,
        34 + pulse,
        if selected { accent } else { WRECK_MID },
    );
    framebuffer.draw_circle(x, y, 28, WRECK_LIGHT);
}

fn vc27_choice_stack_nodes(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    current: u32,
    max: u32,
    accent: Pixel,
) {
    let max = max.max(1).min(8);
    for node in 0..max as i32 {
        let color = if node < current.min(max) as i32 {
            accent
        } else {
            WRECK_MID
        };
        framebuffer.fill_rect(x + node * 15, y, 9, 3, color);
    }
}

fn vc27_choice_footer(framebuffer: &mut Framebuffer, accent: Pixel, motto: &str) {
    framebuffer.draw_line(36, 505, 324, 505, WRECK_MID);
    vc_visual_draw_centered_text(framebuffer, 523, "UP DOWN / DPAD", 1, TEXT);
    vc_visual_draw_centered_text(
        framebuffer,
        544,
        "SPACE / SOUTH  SELECT",
        1,
        accent,
    );
    vc_visual_draw_centered_text(framebuffer, 581, motto, 1, WRECK_LIGHT);
}

#[cfg(test)]
mod choice_showcase_tests {
    use super::*;

    #[test]
    fn common_three_card_layout_stays_above_footer() {
        let start = 94_i32;
        let height = 130_i32;
        let gap = 10_i32;
        let bottom = start + 2 * (height + gap) + height;
        assert!(bottom < 505);
    }

    #[test]
    fn common_footer_stays_inside_presentation_space() {
        assert!(581 < VC_VISUAL_PRESENTATION_HEIGHT as i32);
    }
}
