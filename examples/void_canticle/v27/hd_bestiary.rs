use super::*;

pub(super) fn vc27_hd_render_bell_wraith(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    age: f32,
    phase: f32,
) {
    let breathe = ((age * 4.6 + phase).sin() * 3.0).round() as i32;
    let pulse = ((age * 8.0 + phase).sin() * 0.5 + 0.5) > 0.48;

    framebuffer.fill_rect(x - 21, y - 19, 43, 32, ART_SHADOW);
    framebuffer.fill_circle(x, y - 7, 17, ART_SHADOW);

    framebuffer.draw_circle(x, y - 5, (23 + breathe).max(18) as u32, ART_VOID);
    framebuffer.draw_circle(x, y - 5, 18, WRAITH_GLOW);
    framebuffer.draw_line(x - 15, y - 8, x - 9, y - 22, WRAITH_CORE);
    framebuffer.draw_line(x + 15, y - 8, x + 9, y - 22, WRAITH_CORE);
    framebuffer.draw_line(x - 9, y - 22, x + 9, y - 22, ART_METAL_LIGHT);
    framebuffer.draw_line(x - 17, y - 8, x - 20, y + 7, WRAITH_GLOW);
    framebuffer.draw_line(x + 17, y - 8, x + 20, y + 7, WRAITH_GLOW);
    framebuffer.draw_line(x - 20, y + 7, x + 20, y + 7, WRAITH_CORE);

    framebuffer.draw_circle(x, y - 5, 8, WRAITH_CORE);
    framebuffer.fill_circle(x, y - 5, 5, ART_SHADOW);
    framebuffer.fill_circle(
        x,
        y - 5,
        2,
        if pulse { CANTICLE_COLOR } else { ART_VOID },
    );

    framebuffer.draw_line(x - 13, y + 8, x - 18 - breathe, y + 27, WRAITH_GLOW);
    framebuffer.draw_line(x, y + 8, x + breathe, y + 31, WRAITH_CORE);
    framebuffer.draw_line(x + 13, y + 8, x + 18 + breathe, y + 27, WRAITH_GLOW);
    framebuffer.draw(x - 11, y - 16, ART_GOLD);
    framebuffer.draw(x + 12, y - 15, ART_GOLD);
}

pub(super) fn vc27_hd_render_relic_carrier(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    age: f32,
    phase: f32,
    direction: f32,
) {
    let flutter = ((age * 7.2 + phase).sin() * 4.0).round() as i32;
    let pulse = ((age * 10.0 + phase).sin() * 0.5 + 0.5) > 0.45;
    let wake = if direction >= 0.0 { -1 } else { 1 };

    framebuffer.fill_rect(x - 18, y - 15, 37, 31, CARRIER_VOID);
    framebuffer.draw_rect(x - 18, y - 15, 37, 31, CARRIER_GOLD);

    framebuffer.draw_line(x - 18, y - 10, x - 37, y - 18 - flutter, CARRIER_GOLD);
    framebuffer.draw_line(x - 18, y + 10, x - 37, y + 18 + flutter, CARRIER_GOLD);
    framebuffer.draw_line(x + 18, y - 10, x + 37, y - 18 - flutter, CARRIER_GOLD);
    framebuffer.draw_line(x + 18, y + 10, x + 37, y + 18 + flutter, CARRIER_GOLD);
    framebuffer.draw_line(x - 37, y - 18 - flutter, x - 31, y + flutter, ART_METAL_LIGHT);
    framebuffer.draw_line(x + 37, y - 18 - flutter, x + 31, y + flutter, ART_METAL_LIGHT);

    framebuffer.draw_circle(x, y, 11, POWER_RELIC_LIGHT);
    framebuffer.fill_circle(x, y, 8, POWER_RELIC);
    framebuffer.draw_circle(x, y, 5, CARRIER_GOLD);
    framebuffer.fill_circle(
        x,
        y,
        2,
        if pulse { CANTICLE_COLOR } else { PILGRIM_VIOLET },
    );

    framebuffer.draw_line(x - 13, y - 12, x + 13, y - 12, ART_GOLD);
    framebuffer.draw_line(x - 13, y + 12, x + 13, y + 12, ART_GOLD);
    framebuffer.draw(x - 15, y, ART_METAL_LIGHT);
    framebuffer.draw(x + 15, y, ART_METAL_LIGHT);

    framebuffer.draw_line(x + wake * 22, y - 5, x + wake * 42, y - 5, ART_GOLD);
    framebuffer.draw_line(x + wake * 24, y, x + wake * 48, y, WRECK_LIGHT);
    framebuffer.draw_line(x + wake * 22, y + 5, x + wake * 38, y + 5, ART_METAL);
}

pub(super) fn vc27_hd_render_choir_node(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    age: f32,
    phase: f32,
) {
    let breathe = ((age * 3.5 + phase).sin() * 3.0).round() as i32;
    let pulse = ((age * 7.0 + phase).sin() * 0.5 + 0.5) > 0.52;

    framebuffer.fill_circle(x, y, 24, ART_SHADOW);
    framebuffer.draw_circle(x, y, (27 + breathe).max(22) as u32, ART_CYAN);
    framebuffer.draw_circle(x, y, 20, ART_CYAN_LIGHT);

    for (dx, dy, color) in [
        (0, -30, ART_CYAN_LIGHT),
        (21, -21, ART_GOLD),
        (30, 0, ART_CYAN_LIGHT),
        (21, 21, ART_GOLD),
        (0, 30, ART_CYAN_LIGHT),
        (-21, 21, ART_GOLD),
        (-30, 0, ART_CYAN_LIGHT),
        (-21, -21, ART_GOLD),
    ] {
        framebuffer.draw_line(x, y, x + dx, y + dy, color);
        framebuffer.fill_circle(x + dx, y + dy, 1, color);
    }

    framebuffer.draw_circle(x, y, 11, CHOIR_GLOW);
    framebuffer.fill_circle(x, y, 7, BG);
    framebuffer.draw_circle(x, y, 5, CHOIR_CORE);
    framebuffer.fill_circle(x, y, 2, if pulse { CANTICLE_COLOR } else { CHOIR_CORE });
    framebuffer.draw(x - 12, y - 7, ART_GOLD);
    framebuffer.draw(x + 13, y + 6, ART_GOLD);
}

pub(super) fn vc27_hd_render_void_leech(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    age: f32,
    phase: f32,
    charge: u32,
) {
    let writhe = ((age * 5.4 + phase).sin() * 4.0).round() as i32;
    let charged = charge.min(LEECH_PULSE_CHARGE);

    framebuffer.fill_circle(x, y, 26, ART_SHADOW);
    framebuffer.draw_circle(x, y, 27, ART_VOID);
    framebuffer.draw_circle(x, y, 20, LEECH_GLOW);

    for index in 0..8 {
        let t = index as f32 / 8.0 * std::f32::consts::TAU + age * 0.65;
        let radius = 20.0 + ((index as f32 + age * 3.0).sin() * 2.0);
        let sx = x + (t.cos() * radius).round() as i32;
        let sy = y + (t.sin() * radius).round() as i32;
        let color = if index < charged as usize {
            LEECH_CORE
        } else {
            ART_VOID
        };
        framebuffer.fill_circle(sx, sy, 3, color);
        framebuffer.draw(sx, sy, CANTICLE_COLOR);
    }

    framebuffer.fill_circle(x, y, 10, BG);
    framebuffer.draw_circle(x, y, 9, LEECH_CORE);
    framebuffer.fill_circle(x, y, 3, DANGER);

    framebuffer.draw_line(x - 18, y - 12, x - 34 - writhe, y - 23, LEECH_GLOW);
    framebuffer.draw_line(x + 17, y - 13, x + 33 + writhe, y - 25, LEECH_GLOW);
    framebuffer.draw_line(x - 16, y + 13, x - 29 + writhe, y + 29, ART_VOID);
    framebuffer.draw_line(x + 16, y + 13, x + 31 - writhe, y + 27, ART_VOID);
}

pub(super) fn vc27_hd_render_bellkeeper(framebuffer: &mut Framebuffer, boss: Bellkeeper) {
    let x = vc27_present(boss.x);
    let y = vc27_present(boss.y);
    let pulse = ((boss.age * 5.0).sin().abs() * 4.0) as u32;
    let sway = ((boss.age * 2.2).sin() * 4.0).round() as i32;

    framebuffer.fill_circle(x, y - 5, 36, ART_SHADOW);
    framebuffer.fill_rect(x - 28, y - 20, 57, 50, ART_SHADOW);

    let outer = match boss.phase() {
        BellPhase::Procession => 45 + pulse,
        BellPhase::Resonance => 51 + pulse,
        BellPhase::FinalToll => 57 + pulse,
    };
    let phase_color = match boss.phase() {
        BellPhase::Procession => BELL_METAL,
        BellPhase::Resonance => ART_VOID,
        BellPhase::FinalToll => DANGER,
    };

    framebuffer.draw_circle(x, y - 6, outer, phase_color);
    framebuffer.draw_circle(x, y - 6, 39, BELL_LIGHT);
    framebuffer.draw_line(x - 24, y - 30, x, y - 48, ART_GOLD);
    framebuffer.draw_line(x + 24, y - 30, x, y - 48, ART_GOLD);
    framebuffer.draw_line(x - 24, y - 30, x - 31, y + 17, BELL_LIGHT);
    framebuffer.draw_line(x + 24, y - 30, x + 31, y + 17, BELL_LIGHT);
    framebuffer.draw_line(x - 31, y + 17, x + 31, y + 17, ART_GOLD);
    framebuffer.draw_line(x - 27, y + 23, x + 27, y + 23, BELL_METAL);

    framebuffer.draw_circle(x, y - 9, 15, BELL_LIGHT);
    framebuffer.fill_circle(x, y - 9, 10, BG);
    framebuffer.draw_circle(x, y - 9, 6, ART_GOLD);
    framebuffer.fill_circle(x, y - 9, 2, DANGER);

    framebuffer.draw_line(x - 22, y - 23, x - 53, y - 33 + sway, BELL_METAL);
    framebuffer.draw_line(x + 22, y - 23, x + 53, y - 33 - sway, BELL_METAL);
    framebuffer.draw_circle(x - 56, y - 33 + sway, 7, BELL_LIGHT);
    framebuffer.draw_circle(x + 56, y - 33 - sway, 7, BELL_LIGHT);
    framebuffer.fill_circle(x - 56, y - 33 + sway, 3, BG);
    framebuffer.fill_circle(x + 56, y - 33 - sway, 3, BG);

    framebuffer.draw_line(x, y + 16, x + sway / 2, y + 42, BELL_LIGHT);
    framebuffer.fill_circle(x + sway / 2, y + 46, 5, phase_color);
    framebuffer.draw(x + sway / 2, y + 46, CANTICLE_COLOR);

    if boss.phase() == BellPhase::Resonance {
        framebuffer.draw_circle(x, y - 6, 62 + pulse, WRAITH_GLOW);
    } else if boss.phase() == BellPhase::FinalToll {
        framebuffer.draw_circle(x, y - 6, 64 + pulse, ART_GOLD);
        framebuffer.draw_line(x - 65, y - 6, x + 65, y - 6, DANGER);
    }
}
