fn vc27_render_death_screen(
    framebuffer: &mut Framebuffer,
    sustain: &VoidCanticleV23Sustain,
    time: f32,
) {
    vc27_choice_header(
        framebuffer,
        "PILGRIM FALLEN",
        "THE GRAVE ORBIT KEEPS THE RECORD",
        DANGER,
        time,
    );

    let pulse = ((time * 5.0).sin().abs() * 3.0).round() as u32;
    framebuffer.draw_circle(180, 136, 35 + pulse, DANGER);
    framebuffer.draw_circle(180, 136, 27, ART_VOID);
    framebuffer.draw_line(151, 112, 173, 132, WRECK_LIGHT);
    framebuffer.draw_line(173, 132, 160, 159, DANGER);
    framebuffer.draw_line(209, 111, 187, 133, WRECK_LIGHT);
    framebuffer.draw_line(187, 133, 200, 160, DANGER);
    framebuffer.draw_line(180, 106, 180, 124, ART_GOLD);
    framebuffer.draw_line(180, 148, 180, 166, WRECK_MID);
    framebuffer.fill_circle(180, 136, 4, VOID_DANGER);

    let summary = vc27_run_summary(sustain);
    vc27_render_run_summary(framebuffer, &summary, 20, 190, 320, DANGER, time);

    framebuffer.draw_line(42, 470, 318, 470, WRECK_MID);
    vc_visual_draw_centered_text(framebuffer, 497, "SPACE / SOUTH  REENTER", 1, DANGER);
    vc_visual_draw_centered_text(framebuffer, 535, "THE BUILD ENDS / THE CANTICLE REMAINS", 1, WRECK_LIGHT);
}

#[cfg(test)]
mod death_presentation_tests {
    use super::*;

    #[test]
    fn death_footer_fits_presentation_space() {
        assert!(535 < VC_VISUAL_PRESENTATION_HEIGHT as i32);
    }

    #[test]
    fn death_retry_uses_the_existing_fire_action() {
        assert_eq!(FIRE, ActionId::new("void_canticle.fire"));
    }
}
