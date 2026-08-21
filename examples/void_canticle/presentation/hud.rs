impl VoidCanticlePresentation {
    fn render_event_announcement(&self, framebuffer: &mut Framebuffer) {
        let Some(announcement) = self.game.presentation_announcement() else {
            return;
        };

        match announcement {
            PresentationAnnouncement::Pressure(pressure) => {
                let (headline, consequence) = pressure_transition_copy(pressure);
                vc_visual_announcement(
                    framebuffer,
                    headline,
                    consequence,
                    void_pressure_color(pressure),
                    VOID_LIGHT,
                );
            }
            PresentationAnnouncement::BossPhase(phase) => {
                vc_visual_announcement(
                    framebuffer,
                    "BELLKEEPER",
                    bell_phase_name(phase),
                    BELL_LIGHT,
                    CANTICLE_COLOR,
                );
            }
            PresentationAnnouncement::Synergy(name) => {
                vc_visual_announcement(
                    framebuffer,
                    "SYNERGY",
                    name,
                    SYNERGY_COLOR,
                    SYNERGY_GOLD,
                );
            }
            PresentationAnnouncement::Canticle => {
                vc_visual_announcement(
                    framebuffer,
                    "FULL WIPE",
                    "CANTICLE",
                    CANTICLE_COLOR,
                    CANTICLE_COLOR,
                );
            }
            PresentationAnnouncement::VoidAttack(kind) => {
                vc_visual_announcement(
                    framebuffer,
                    void_attack_name(kind),
                    "VOID ATTACK",
                    void_pressure_color(self.game.presentation_void_pressure()),
                    VOID_LIGHT,
                );
            }
        }
    }

    fn render_canticle_charge(&self, framebuffer: &mut Framebuffer) {
        let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
        let base = self.game.presentation_base();
        let width = 88 * scale;
        let height = 4 * scale;
        let x = (VC_VISUAL_PRESENTATION_WIDTH - width) / 2;
        let y = VC_VISUAL_PRESENTATION_HEIGHT.saturating_sub(18 * scale);
        let ratio = base.core_charge.min(CORE_MAX) as f32 / CORE_MAX as f32;
        let filled = (width as f32 * ratio).round() as u32;
        let ready = base.core_charge >= CORE_MAX;
        let color = if ready { CANTICLE_COLOR } else { CINDER };

        framebuffer.draw_text_scaled(8 * scale as i32, y as i32 - 1, "CORE", scale, TEXT);
        framebuffer.fill_rect(x as i32, y as i32, width, height, WRECK_MID);
        if filled > 0 {
            framebuffer.fill_rect(x as i32, y as i32, filled.min(width), height, color);
        }
        for segment in 1..5 {
            let separator = x + width * segment / 5;
            framebuffer.fill_rect(separator as i32, y as i32, 1, height, BG);
        }
        framebuffer.draw_rect(
            x as i32 - 1,
            y as i32 - 1,
            width + 2,
            height + 2,
            WRECK_LIGHT,
        );

        if ready && ((base.animation_time * 6.0) as i32 & 1) == 0 {
            let (text_width, text_height) = Framebuffer::text_size("READY", 1);
            let text_x = x + width.saturating_sub(text_width) / 2;
            let text_y = y + height.saturating_sub(text_height) / 2;
            framebuffer.draw_text(text_x as i32, text_y as i32, "READY", BG);
        }
    }

    fn render_survival_bars(&self, framebuffer: &mut Framebuffer) {
        let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
        let model = self.game.combat_model();
        let hull_cap = self.game.hull_cap().max(1.0);
        let shield_cap = self.game.shield_cap().max(1.0);

        let margin = 8 * scale;
        let width = 72 * scale;
        let height = 4 * scale;
        let y = VC_VISUAL_PRESENTATION_HEIGHT.saturating_sub(7 * scale);
        let hull_x = margin;
        let shield_x = VC_VISUAL_PRESENTATION_WIDTH
            .saturating_sub(margin)
            .saturating_sub(width);
        let hull_segments = ((hull_cap / 10.0).ceil() as u32).clamp(4, 12);
        let shield_segments = ((shield_cap / 5.0).ceil() as u32).clamp(3, 10);

        segmented_bar(
            framebuffer,
            hull_x as i32,
            y as i32,
            width,
            height,
            model.player_hull,
            hull_cap,
            hull_segments,
            if model.player_hull_flash_timer > 0.0 {
                DANGER
            } else {
                VC20_HULL
            },
        );
        segmented_bar(
            framebuffer,
            shield_x as i32,
            y as i32,
            width,
            height,
            model.player_shield,
            shield_cap,
            shield_segments,
            if model.player_shield_flash_timer > 0.0 {
                VC20_ARMOR_LIGHT
            } else {
                VC20_ARMOR
            },
        );
    }

    fn render_threat_meter(&self, framebuffer: &mut Framebuffer) {
        let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
        let pressure = self.game.presentation_void_pressure();
        let active = pressure_level(pressure);
        let brick_width = 4 * scale;
        let brick_height = 6 * scale;
        let gap = 2 * scale;
        let x = VC_VISUAL_PRESENTATION_WIDTH.saturating_sub(8 * scale);
        let base_y = 196 * scale;
        let states = [
            VoidPressure::Dormant,
            VoidPressure::Stirring,
            VoidPressure::Awake,
            VoidPressure::Hostile,
            VoidPressure::Cataclysmic,
        ];

        for (index, state) in states.into_iter().enumerate() {
            let level = index as u32 + 1;
            let inverted_index = 4_u32.saturating_sub(index as u32);
            let y = base_y + inverted_index * (brick_height + gap);
            let lit = level <= active;
            let fill = if lit {
                void_pressure_color(state)
            } else {
                WRECK_MID
            };

            if lit {
                framebuffer.fill_rect(x as i32, y as i32, brick_width, brick_height, fill);
            }
            framebuffer.draw_rect(
                x as i32 - 1,
                y as i32 - 1,
                brick_width + 2,
                brick_height + 2,
                fill,
            );
        }
    }

    fn render_echo_shell(&self, framebuffer: &mut Framebuffer) {
        let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
        let progression = &self.game.presentation_progression().progression;
        let ratio = if progression.xp_next == 0 {
            1.0
        } else {
            progression.xp.min(progression.xp_next) as f32 / progression.xp_next as f32
        };
        let center_x = (18 * scale) as i32;
        let center_y = (VC_VISUAL_PRESENTATION_HEIGHT.saturating_sub(35 * scale)) as i32;
        let radius = 11 * scale;
        let level = progression.level;
        let tier_color = echo_level_color(level);

        echo_shell(
            framebuffer,
            center_x,
            center_y,
            radius,
            ratio,
            level,
            tier_color,
        );
    }
}

fn pressure_level(pressure: VoidPressure) -> u32 {
    match pressure {
        VoidPressure::Dormant => 1,
        VoidPressure::Stirring => 2,
        VoidPressure::Awake => 3,
        VoidPressure::Hostile => 4,
        VoidPressure::Cataclysmic => 5,
    }
}

fn echo_level_color(level: u32) -> Pixel {
    match (level / 10) % 6 {
        0 => XP_ORB_CORE,
        1 => ART_CYAN_LIGHT,
        2 => SYNERGY_GOLD,
        3 => VOID_GLOW,
        4 => CANTICLE_COLOR,
        _ => DANGER,
    }
}

fn echo_shell(
    framebuffer: &mut Framebuffer,
    center_x: i32,
    center_y: i32,
    radius: u32,
    ratio: f32,
    level: u32,
    tier_color: Pixel,
) {
    let ratio = ratio.clamp(0.0, 1.0);
    let radius = radius.max(4);
    let steps = 48_u32;
    let active_steps = (steps as f32 * ratio).round() as u32;

    framebuffer.draw_circle(center_x, center_y, radius, tier_color);
    framebuffer.draw_circle(center_x, center_y, radius.saturating_sub(3), XP_BAR_BG);

    let mut previous = None;
    let mut active_tip = (center_x, center_y);
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let angle = -std::f32::consts::FRAC_PI_2 + t * std::f32::consts::TAU * 2.20;
        let spiral_radius = 1.5 + t * (radius as f32 - 3.0);
        let x = center_x + (angle.cos() * spiral_radius).round() as i32;
        let y = center_y + (angle.sin() * spiral_radius).round() as i32;

        if let Some((px, py)) = previous {
            let color = if step <= active_steps {
                tier_color
            } else {
                WRECK_MID
            };
            framebuffer.draw_line(px, py, x, y, color);
        }
        if step <= active_steps {
            active_tip = (x, y);
        }
        previous = Some((x, y));
    }

    if active_steps > 0 {
        framebuffer.fill_circle(active_tip.0, active_tip.1, 1, tier_color);
    }

    let lip_x = center_x + radius as i32 - 2;
    framebuffer.draw_line(
        center_x + radius as i32 / 2,
        center_y + radius as i32 / 3,
        lip_x,
        center_y + radius as i32 / 2,
        tier_color,
    );

    let badge_radius = (radius / 3).max(4);
    framebuffer.fill_circle(center_x, center_y, badge_radius, BG);
    framebuffer.draw_circle(center_x, center_y, badge_radius, tier_color);

    let label = level.to_string();
    let preferred_scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
    let text_scale = if label.len() <= 2 { preferred_scale } else { 1 };
    let (text_width, text_height) = Framebuffer::text_size(&label, text_scale);
    framebuffer.draw_text_scaled(
        center_x - text_width as i32 / 2,
        center_y - text_height as i32 / 2,
        &label,
        text_scale,
        tier_color,
    );
}

#[allow(clippy::too_many_arguments)]
fn segmented_bar(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    value: f32,
    max_value: f32,
    segments: u32,
    fill: Pixel,
) {
    let ratio = (value / max_value.max(1.0)).clamp(0.0, 1.0);
    let filled = (width as f32 * ratio).round() as u32;
    framebuffer.fill_rect(x, y, width, height, WRECK_MID);
    if filled > 0 {
        framebuffer.fill_rect(x, y, filled.min(width), height, fill);
    }

    for segment in 1..segments.max(1) {
        let separator = width.saturating_mul(segment) / segments.max(1);
        framebuffer.fill_rect(x + separator as i32, y, 1, height, BG);
    }
    framebuffer.draw_rect(x - 1, y - 1, width + 2, height + 2, WRECK_LIGHT);
}

#[cfg(test)]
mod hud_tests {
    use super::*;

    #[test]
    fn pressure_maps_to_five_visual_bricks() {
        assert_eq!(pressure_level(VoidPressure::Dormant), 1);
        assert_eq!(pressure_level(VoidPressure::Stirring), 2);
        assert_eq!(pressure_level(VoidPressure::Awake), 3);
        assert_eq!(pressure_level(VoidPressure::Hostile), 4);
        assert_eq!(pressure_level(VoidPressure::Cataclysmic), 5);
    }

    #[test]
    fn echo_shell_changes_color_at_ten_level_boundaries() {
        assert_eq!(echo_level_color(1), echo_level_color(9));
        assert_ne!(echo_level_color(9), echo_level_color(10));
        assert_ne!(echo_level_color(19), echo_level_color(20));
        assert_ne!(echo_level_color(29), echo_level_color(30));
    }

    #[test]
    fn echo_shell_stays_local_to_its_glyph() {
        let mut framebuffer = Framebuffer::new(40, 40);
        framebuffer.clear(Pixel::BLUE);
        echo_shell(
            &mut framebuffer,
            20,
            20,
            8,
            0.5,
            12,
            echo_level_color(12),
        );
        assert_eq!(framebuffer.pixel(0, 0), Some(Pixel::BLUE));
        assert_eq!(framebuffer.pixel(39, 39), Some(Pixel::BLUE));
    }

    #[test]
    fn segmented_bar_only_touches_its_own_rows() {
        let mut framebuffer = Framebuffer::new(20, 8);
        framebuffer.clear(Pixel::BLUE);
        segmented_bar(&mut framebuffer, 2, 3, 12, 2, 3.0, 12.0, 4, Pixel::RED);
        assert_eq!(framebuffer.pixel(2, 1), Some(Pixel::BLUE));
        assert_eq!(framebuffer.pixel(2, 6), Some(Pixel::BLUE));
    }
}
