#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DamageState {
    Intact,
    Damaged,
    Critical,
}

pub(super) fn damage_state(value: u32, max_value: u32) -> DamageState {
    if max_value == 0 {
        return DamageState::Intact;
    }

    let ratio = value.min(max_value) as f32 / max_value as f32;
    if ratio > 0.66 {
        DamageState::Intact
    } else if ratio > 0.33 {
        DamageState::Damaged
    } else {
        DamageState::Critical
    }
}

pub(super) fn render_damage_marks(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    state: DamageState,
    age: f32,
    spread: i32,
) {
    if state == DamageState::Intact {
        return;
    }

    let crack = if state == DamageState::Critical {
        DANGER
    } else {
        ART_RUST
    };
    let side = if ((age * 2.7) as i32 & 1) == 0 { -1 } else { 1 };
    let reach = (spread / 3).max(6);

    framebuffer.draw_line(
        x + side * reach,
        y - reach / 2,
        x + side * (reach / 2),
        y,
        crack,
    );
    framebuffer.draw_line(
        x + side * (reach / 2),
        y,
        x + side * reach,
        y + reach / 2,
        ART_METAL_LIGHT,
    );
    framebuffer.draw(x - side * (reach + 2), y + reach / 3, ART_RUST);

    if state == DamageState::Critical {
        framebuffer.draw_line(
            x - side * (reach / 2),
            y - reach,
            x,
            y - reach / 3,
            DANGER,
        );
        framebuffer.draw_line(
            x,
            y + reach / 3,
            x - side * reach,
            y + reach,
            ART_RUST,
        );

        if ((age * 10.0) as i32 % 5) <= 1 {
            let spark_x = x + side * (spread / 2).max(8);
            let spark_y = y - spread / 4;
            framebuffer.draw_line(spark_x - 2, spark_y, spark_x + 2, spark_y, ART_GOLD);
            framebuffer.draw_line(spark_x, spark_y - 2, spark_x, spark_y + 2, CANTICLE_COLOR);
        }
    }
}

pub(super) fn presentation_coord(value: f32) -> i32 {
    (value * VC_VISUAL_PRESENTATION_SCALE as f32).round() as i32
}

pub(super) fn render_carrion(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    age: f32,
    phase: f32,
) {
    let flap = ((age * 7.0 + phase).sin() * 4.0).round() as i32;
    let bob = ((age * 4.2 + phase).sin() * 2.0).round() as i32;
    let y = y + bob;
    let eye_hot = ((age * 11.0 + phase).sin() * 0.5 + 0.5) > 0.42;

    for dy in -9_i32..=9 {
        let taper = dy.unsigned_abs() as i32 * 2;
        let half_width = (33 - taper).max(13);
        framebuffer.draw_line(x - half_width, y + dy, x + half_width, y + dy, ART_SHADOW);
    }

    framebuffer.draw_line(x - 33, y - 1 - flap, x - 9, y - 7, ART_RUST);
    framebuffer.draw_line(x + 33, y - 1 - flap, x + 9, y - 7, ART_RUST);
    framebuffer.draw_line(x - 31, y + 2 + flap, x - 8, y + 7, ART_BONE);
    framebuffer.draw_line(x + 31, y + 2 + flap, x + 8, y + 7, ART_BONE);

    for rib in [12, 18, 24, 30] {
        framebuffer.draw_line(x - 5, y - 3, x - rib, y - 6 + rib / 8 - flap / 2, ART_BONE);
        framebuffer.draw_line(x + 5, y - 3, x + rib, y - 6 + rib / 8 - flap / 2, ART_BONE);
    }

    framebuffer.draw_line(x - 9, y - 7, x, y - 15, ART_BONE);
    framebuffer.draw_line(x + 9, y - 7, x, y - 15, ART_BONE);
    framebuffer.draw_line(x - 9, y + 7, x, y + 15, ART_RUST);
    framebuffer.draw_line(x + 9, y + 7, x, y + 15, ART_RUST);
    framebuffer.fill_circle(x, y, 7, ART_VOID);
    framebuffer.draw_circle(x, y, 8, ART_BONE);
    framebuffer.fill_circle(x, y, 3, if eye_hot { ENEMY_EYE } else { ART_RUST });
    framebuffer.draw(x, y, CANTICLE_COLOR);

    framebuffer.draw(x - 14, y + 3, ART_GOLD);
    framebuffer.draw(x + 17, y - 2, ART_GOLD);
    framebuffer.draw(x - 25, y - 2 - flap, ART_RUST);
    framebuffer.draw(x + 27, y + flap / 2, ART_BONE);
}

pub(super) fn render_grave_knight(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    age: f32,
) {
    let pulse = ((age * 9.0).sin() * 0.5 + 0.5) > 0.5;
    let stride = ((age * 3.0).sin() * 2.0).round() as i32;
    let kick = if (age * 1.7).fract() > 0.88 { 3 } else { 0 };
    let y = y + stride;

    framebuffer.fill_rect(x - 17, y - 23 + kick, 35, 46, ART_SHADOW);
    framebuffer.fill_rect(x - 31, y - 10 + kick, 14, 22, ART_SHADOW);
    framebuffer.fill_rect(x + 18, y - 10 + kick, 14, 22, ART_SHADOW);
    framebuffer.draw_rect(x - 17, y - 23 + kick, 35, 46, ART_METAL);
    framebuffer.draw_rect(x - 31, y - 10 + kick, 14, 22, ART_METAL);
    framebuffer.draw_rect(x + 18, y - 10 + kick, 14, 22, ART_METAL);

    framebuffer.draw_line(x - 16, y - 22 + kick, x, y - 35 + kick, ART_GOLD);
    framebuffer.draw_line(x + 16, y - 22 + kick, x, y - 35 + kick, ART_GOLD);
    framebuffer.draw_line(x - 31, y - 10 + kick, x - 38, y + 2 + kick, ART_GOLD);
    framebuffer.draw_line(x + 31, y - 10 + kick, x + 38, y + 2 + kick, ART_GOLD);
    framebuffer.draw_line(x, y + 22 + kick, x, y + 34 + kick, ART_METAL_LIGHT);

    framebuffer.fill_rect(x - 7, y - 11 + kick, 15, 15, DANGER);
    framebuffer.draw_rect(x - 8, y - 12 + kick, 17, 17, ART_GOLD);
    framebuffer.fill_rect(x - 3, y - 7 + kick, 7, 7, ART_SHADOW);
    framebuffer.draw(
        x,
        y - 4 + kick,
        if pulse { CANTICLE_COLOR } else { ENEMY_EYE },
    );

    for offset in [-11, -6, 6, 11] {
        framebuffer.draw_line(
            x + offset,
            y + 7 + kick,
            x + offset,
            y + 18 + kick,
            ART_METAL_LIGHT,
        );
    }
    framebuffer.draw_line(x - 14, y + 14 + kick, x + 14, y + 14 + kick, ART_GOLD);

    let thrust = if kick > 0 { 46 } else { 41 };
    framebuffer.draw_line(x - 3, y + 35, x - 5, y + thrust, PILGRIM_THRUSTER);
    framebuffer.draw_line(x + 3, y + 35, x + 5, y + thrust, PILGRIM_THRUSTER);
}

pub(super) fn render_bell_wraith(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    age: f32,
    phase: f32,
) {
    let breathe = ((age * 4.6 + phase).sin() * 3.0).round() as i32;
    let sway = ((age * 2.8 + phase).sin() * 5.0).round() as i32;
    let pulse = ((age * 8.0 + phase).sin() * 0.5 + 0.5) > 0.48;

    framebuffer.fill_rect(x - 21, y - 19, 43, 32, ART_SHADOW);
    framebuffer.fill_circle(x, y - 7, 17, ART_SHADOW);
    framebuffer.draw_circle(x, y - 5, (23 + breathe).max(18) as u32, ART_VOID);
    framebuffer.draw_circle(x, y - 5, 18, WRAITH_GLOW);
    framebuffer.draw_line(x - 15, y - 8, x - 9 + sway / 3, y - 22, WRAITH_CORE);
    framebuffer.draw_line(x + 15, y - 8, x + 9 + sway / 3, y - 22, WRAITH_CORE);
    framebuffer.draw_line(x - 9 + sway / 3, y - 22, x + 9 + sway / 3, y - 22, ART_METAL_LIGHT);
    framebuffer.draw_line(x - 17, y - 8, x - 20, y + 7, WRAITH_GLOW);
    framebuffer.draw_line(x + 17, y - 8, x + 20, y + 7, WRAITH_GLOW);
    framebuffer.draw_line(x - 20, y + 7, x + 20, y + 7, WRAITH_CORE);

    framebuffer.draw_circle(x, y - 5, 8, WRAITH_CORE);
    framebuffer.fill_circle(x, y - 5, 5, ART_SHADOW);
    framebuffer.fill_circle(x, y - 5, 2, if pulse { CANTICLE_COLOR } else { ART_VOID });
    framebuffer.draw_line(x - 13, y + 8, x - 18 - breathe + sway, y + 27, WRAITH_GLOW);
    framebuffer.draw_line(x, y + 8, x + sway, y + 31, WRAITH_CORE);
    framebuffer.draw_line(x + 13, y + 8, x + 18 + breathe + sway, y + 27, WRAITH_GLOW);
    framebuffer.draw_line(x + sway / 2, y + 9, x + sway, y + 17, ART_GOLD);
    framebuffer.fill_circle(x + sway, y + 19, 2, WRAITH_CORE);
    framebuffer.draw(x - 11, y - 16, ART_GOLD);
    framebuffer.draw(x + 12, y - 15, ART_GOLD);
}

pub(super) fn render_relic_carrier(
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
    let tilt = ((age * 3.7 + phase).sin() * 2.0).round() as i32;

    framebuffer.fill_rect(x - 18, y - 15 + tilt, 37, 31, CARRIER_VOID);
    framebuffer.draw_rect(x - 18, y - 15 + tilt, 37, 31, CARRIER_GOLD);
    framebuffer.draw_line(x - 18, y - 10 + tilt, x - 37, y - 18 - flutter, CARRIER_GOLD);
    framebuffer.draw_line(x - 18, y + 10 + tilt, x - 37, y + 18 + flutter, CARRIER_GOLD);
    framebuffer.draw_line(x + 18, y - 10 + tilt, x + 37, y - 18 - flutter, CARRIER_GOLD);
    framebuffer.draw_line(x + 18, y + 10 + tilt, x + 37, y + 18 + flutter, CARRIER_GOLD);
    framebuffer.draw_line(x - 37, y - 18 - flutter, x - 31, y + flutter, ART_METAL_LIGHT);
    framebuffer.draw_line(x + 37, y - 18 - flutter, x + 31, y + flutter, ART_METAL_LIGHT);

    let relic_radius = if pulse { 9 } else { 8 };
    framebuffer.draw_circle(x, y + tilt, 11, POWER_RELIC_LIGHT);
    framebuffer.fill_circle(x, y + tilt, relic_radius, POWER_RELIC);
    framebuffer.draw_circle(x, y + tilt, 5, CARRIER_GOLD);
    framebuffer.fill_circle(
        x,
        y + tilt,
        2,
        if pulse { CANTICLE_COLOR } else { PILGRIM_VIOLET },
    );
    framebuffer.draw_line(x - 13, y - 12 + tilt, x + 13, y - 12 + tilt, ART_GOLD);
    framebuffer.draw_line(x - 13, y + 12 + tilt, x + 13, y + 12 + tilt, ART_GOLD);

    framebuffer.draw_line(x + wake * 22, y - 5, x + wake * 42, y - 5, ART_GOLD);
    framebuffer.draw_line(x + wake * 24, y, x + wake * 48, y, WRECK_LIGHT);
    framebuffer.draw_line(x + wake * 22, y + 5, x + wake * 38, y + 5, ART_METAL);
}

pub(super) fn render_choir_node(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    age: f32,
    phase: f32,
) {
    let breathe = ((age * 3.5 + phase).sin() * 3.0).round() as i32;
    let pulse = ((age * 7.0 + phase).sin() * 0.5 + 0.5) > 0.52;
    let rotation = age * 0.42 + phase * 0.15;

    framebuffer.fill_circle(x, y, 24, ART_SHADOW);
    framebuffer.draw_circle(x, y, (27 + breathe).max(22) as u32, ART_CYAN);
    framebuffer.draw_circle(x, y, 20, ART_CYAN_LIGHT);
    for index in 0..8 {
        let angle = rotation + index as f32 * std::f32::consts::FRAC_PI_4;
        let radius = if index & 1 == 0 { 30.0 } else { 29.0 };
        let dx = (angle.cos() * radius).round() as i32;
        let dy = (angle.sin() * radius).round() as i32;
        let color = if index & 1 == 0 { ART_CYAN_LIGHT } else { ART_GOLD };
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

pub(super) fn render_void_leech(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    age: f32,
    phase: f32,
    charge: u32,
) {
    let writhe = ((age * 5.4 + phase).sin() * 4.0).round() as i32;
    let charged = charge.min(LEECH_PULSE_CHARGE);
    let contraction = ((age * (3.0 + charged as f32 * 0.35) + phase).sin() * 2.0).round() as i32;
    let ring = (27 + contraction).max(23) as u32;

    framebuffer.fill_circle(x, y, 26, ART_SHADOW);
    framebuffer.draw_circle(x, y, ring, ART_VOID);
    framebuffer.draw_circle(x, y, (20 + contraction / 2).max(17) as u32, LEECH_GLOW);
    for index in 0..8 {
        let t = index as f32 / 8.0 * std::f32::consts::TAU + age * (0.65 + charged as f32 * 0.04);
        let radius = 20.0 + contraction as f32 + ((index as f32 + age * 3.0).sin() * 2.0);
        let sx = x + (t.cos() * radius).round() as i32;
        let sy = y + (t.sin() * radius).round() as i32;
        let color = if index < charged as usize { LEECH_CORE } else { ART_VOID };
        framebuffer.fill_circle(sx, sy, 3, color);
        framebuffer.draw(sx, sy, CANTICLE_COLOR);
    }

    framebuffer.fill_circle(x, y, 10, BG);
    framebuffer.draw_circle(x, y, 9, LEECH_CORE);
    framebuffer.fill_circle(x, y, 3 + (charged / 3), DANGER);
    framebuffer.draw_line(x - 18, y - 12, x - 34 - writhe, y - 23, LEECH_GLOW);
    framebuffer.draw_line(x + 17, y - 13, x + 33 + writhe, y - 25, LEECH_GLOW);
    framebuffer.draw_line(x - 16, y + 13, x - 29 + writhe, y + 29, ART_VOID);
    framebuffer.draw_line(x + 16, y + 13, x + 31 - writhe, y + 27, ART_VOID);
}

pub(super) fn render_bellkeeper(framebuffer: &mut Framebuffer, boss: Bellkeeper) {
    let x = presentation_coord(boss.x);
    let y = presentation_coord(boss.y);
    let pulse = ((boss.age * 5.0).sin().abs() * 4.0) as u32;
    let sway = ((boss.age * 2.2).sin() * 4.0).round() as i32;
    let toll = ((boss.age * 1.6).sin() * 2.0).round() as i32;

    framebuffer.fill_circle(x, y - 5 + toll, 36, ART_SHADOW);
    framebuffer.fill_rect(x - 28, y - 20 + toll, 57, 50, ART_SHADOW);

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

    framebuffer.draw_circle(x, y - 6 + toll, outer, phase_color);
    framebuffer.draw_circle(x, y - 6 + toll, 39, BELL_LIGHT);
    framebuffer.draw_line(x - 24, y - 30 + toll, x, y - 48 + toll, ART_GOLD);
    framebuffer.draw_line(x + 24, y - 30 + toll, x, y - 48 + toll, ART_GOLD);
    framebuffer.draw_line(x - 24, y - 30 + toll, x - 31, y + 17 + toll, BELL_LIGHT);
    framebuffer.draw_line(x + 24, y - 30 + toll, x + 31, y + 17 + toll, BELL_LIGHT);
    framebuffer.draw_line(x - 31, y + 17 + toll, x + 31, y + 17 + toll, ART_GOLD);
    framebuffer.draw_line(x - 27, y + 23 + toll, x + 27, y + 23 + toll, BELL_METAL);
    framebuffer.draw_circle(x, y - 9 + toll, 15, BELL_LIGHT);
    framebuffer.fill_circle(x, y - 9 + toll, 10, BG);
    framebuffer.draw_circle(x, y - 9 + toll, 6, ART_GOLD);
    framebuffer.fill_circle(x, y - 9 + toll, 2, DANGER);

    framebuffer.draw_line(x - 22, y - 23 + toll, x - 53, y - 33 + sway, BELL_METAL);
    framebuffer.draw_line(x + 22, y - 23 + toll, x + 53, y - 33 - sway, BELL_METAL);
    framebuffer.draw_circle(x - 56, y - 33 + sway, 7, BELL_LIGHT);
    framebuffer.draw_circle(x + 56, y - 33 - sway, 7, BELL_LIGHT);
    framebuffer.fill_circle(x - 56, y - 33 + sway, 3, BG);
    framebuffer.fill_circle(x + 56, y - 33 - sway, 3, BG);
    framebuffer.draw_line(x, y + 16 + toll, x + sway / 2, y + 42 + toll, BELL_LIGHT);
    framebuffer.fill_circle(x + sway / 2, y + 46 + toll, 5, phase_color);
    framebuffer.draw(x + sway / 2, y + 46 + toll, CANTICLE_COLOR);

    if boss.phase() == BellPhase::Resonance {
        framebuffer.draw_circle(x, y - 6 + toll, 62 + pulse, WRAITH_GLOW);
    } else if boss.phase() == BellPhase::FinalToll {
        framebuffer.draw_circle(x, y - 6 + toll, 64 + pulse, ART_GOLD);
        framebuffer.draw_line(x - 65, y - 6 + toll, x + 65, y - 6 + toll, DANGER);
    }

    render_damage_marks(
        framebuffer,
        x,
        y + toll,
        damage_state(boss.hp, BELLKEEPER_MAX_HP),
        boss.age,
        64,
    );
}

pub(super) fn render_pilgrim(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    focused: bool,
    invulnerability: f32,
    animation_time: f32,
) {
    if invulnerability > 0.0 && ((invulnerability * 16.0) as i32 & 1) != 0 {
        return;
    }

    let flame = if ((animation_time * 12.0) as i32 & 1) == 0 { 7 } else { 10 };
    let wing = if focused { 15 } else { 22 };
    let lean = if focused {
        0
    } else {
        ((animation_time * 4.5).sin() * 1.5).round() as i32
    };
    framebuffer.draw_line(x, y - 29, x - 9 + lean, y - 16, PILGRIM_GOLD);
    framebuffer.draw_line(x, y - 29, x + 9 + lean, y - 16, PILGRIM_GOLD);
    framebuffer.draw_line(x - 9 + lean, y - 16, x - wing, y + 4, PILGRIM);
    framebuffer.draw_line(x + 9 + lean, y - 16, x + wing, y + 4, PILGRIM);
    framebuffer.draw_line(x - wing, y + 4, x - 7, y + 14, PILGRIM_DARK);
    framebuffer.draw_line(x + wing, y + 4, x + 7, y + 14, PILGRIM_DARK);
    framebuffer.draw_line(x - 6, y - 15, x - 3, y + 18, ART_METAL_LIGHT);
    framebuffer.draw_line(x + 6, y - 15, x + 3, y + 18, ART_METAL_LIGHT);
    framebuffer.draw_line(x - 3, y + 18, x, y + 24, PILGRIM_GOLD);
    framebuffer.draw_line(x + 3, y + 18, x, y + 24, PILGRIM_GOLD);
    framebuffer.fill_circle(x, y - 3, 6, PILGRIM_VIOLET);
    framebuffer.draw_circle(x, y - 3, 8, PILGRIM_GOLD);
    framebuffer.fill_circle(x, y - 3, 2, CANTICLE_COLOR);
    framebuffer.draw_line(x - 5, y + 19, x - 7, y + 19 + flame, PILGRIM_THRUSTER);
    framebuffer.draw_line(x + 5, y + 19, x + 7, y + 19 + flame, PILGRIM_THRUSTER);
    framebuffer.draw(x - 8, y - 9, ART_GOLD);
    framebuffer.draw(x + 8, y - 9, ART_GOLD);
    framebuffer.draw(x, y - 24, ART_METAL_LIGHT);

    if focused {
        framebuffer.draw_circle(x, y - 3, 4, PILGRIM_GOLD);
        framebuffer.fill_circle(x, y - 3, 1, FOCUS_COLOR);
    }
}
