fn vc27_render_stage_clear_presentation(
    framebuffer: &mut Framebuffer,
    sustain: &VoidCanticleV23Sustain,
    time: f32,
) {
    vc27_choice_header(
        framebuffer,
        "STAGE CLEAR",
        "GRAVE ORBIT / PATH OPENS",
        CANTICLE_COLOR,
        time,
    );

    let pulse = ((time * 4.0).sin().abs() * 4.0).round() as u32;
    framebuffer.draw_circle(180, 136, 34 + pulse, ART_GOLD);
    framebuffer.draw_circle(180, 136, 25, CANTICLE_COLOR);
    framebuffer.draw_line(156, 136, 174, 154, SYNERGY_COLOR);
    framebuffer.draw_line(174, 154, 207, 116, CANTICLE_COLOR);
    framebuffer.fill_circle(180, 136, 5, POWER_RELIC_LIGHT);

    let summary = vc27_run_summary(sustain);
    vc27_render_run_summary(
        framebuffer,
        &summary,
        20,
        190,
        320,
        CANTICLE_COLOR,
        time,
    );

    framebuffer.draw_line(42, 470, 318, 470, WRECK_MID);
    vc_visual_draw_centered_text(framebuffer, 492, "SPACE / SOUTH  RESTART", 1, CANTICLE_COLOR);
    vc_visual_draw_centered_text(framebuffer, 516, "ESC / START  MENU", 1, TEXT);
    vc_visual_draw_centered_text(framebuffer, 558, "THE CANTICLE RELEASES YOU", 1, ART_GOLD);
}

#[cfg(test)]
mod stage_clear_presentation_tests {
    use super::*;

    #[test]
    fn stage_clear_actions_match_stabilized_runtime_bindings() {
        assert_eq!(VC21_STAGE_RESTART.as_str(), "void_canticle.stage.restart");
        assert_eq!(VC21_STAGE_MENU.as_str(), "void_canticle.stage.menu");
    }

    #[test]
    fn stage_clear_footer_fits_presentation_space() {
        assert!(558 < VC_VISUAL_PRESENTATION_HEIGHT as i32);
    }
}
