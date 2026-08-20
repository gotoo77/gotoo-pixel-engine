const VC27_PRESENTATION_VERSION: &str = "VC2.7";

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/v27/hd_bestiary.rs"
));

struct VoidCanticleV27DirectPresentation {
    game: VoidCanticleV23Sustain,
    simulation_framebuffer: Framebuffer,
    player_overlay: Framebuffer,
    clean_background: Framebuffer,
    pilgrim_art: PilgrimV07Visuals,
}

impl VoidCanticleV27DirectPresentation {
    fn new() -> Self {
        Self {
            game: VoidCanticleV23Sustain::new(),
            simulation_framebuffer: Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT),
            player_overlay: Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT),
            clean_background: Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT),
            pilgrim_art: PilgrimV07Visuals::new(),
        }
    }

    fn visual_mode(&self) -> VcVisualMode {
        if self.game.choosing_support {
            return VcVisualMode::SupportChoice;
        }

        let v14 = self.game.game.v20().game.v14();
        if v14.progression.level_choice.is_some() {
            return VcVisualMode::LevelChoice;
        }
        if v14.mutation_choice.is_some() {
            return VcVisualMode::MutationChoice;
        }
        if self.game.game.base().game_over {
            return VcVisualMode::Death;
        }

        let stabilized = &self.game.survival_model().game;
        if stabilized.stage_clear_visible() {
            return VcVisualMode::StageClear;
        }
        if !matches!(&stabilized.pause_ui().state, VcPauseState::Running) {
            return VcVisualMode::Pause;
        }

        VcVisualMode::Combat
    }

    fn render_event_announcement(&self, framebuffer: &mut Framebuffer) {
        let v16b = &self.game.game.v20().game.ui.game.combat;

        if v16b.pressure_reveal_timer > 0.0 && v16b.pressure_reveal != VoidPressure::Dormant {
            let (headline, consequence) = pressure_transition_copy(v16b.pressure_reveal);
            vc_visual_announcement(
                framebuffer,
                headline,
                consequence,
                void_pressure_color(v16b.pressure_reveal),
                VOID_LIGHT,
            );
            return;
        }

        if v16b.boss_phase_banner_timer > 0.0
            && let Some(phase) = v16b.boss_phase_banner
        {
            vc_visual_announcement(
                framebuffer,
                "BELLKEEPER",
                bell_phase_name(phase),
                BELL_LIGHT,
                CANTICLE_COLOR,
            );
            return;
        }

        let v15 = &v16b.combat.combat;
        if v15.synergy_banner_timer > 0.0
            && let Some(name) = v15.synergy_banner_name
        {
            vc_visual_announcement(
                framebuffer,
                "SYNERGY",
                name,
                SYNERGY_COLOR,
                SYNERGY_GOLD,
            );
            return;
        }

        if self.game.game.base().canticle_timer > 0.0 {
            vc_visual_announcement(
                framebuffer,
                "FULL WIPE",
                "CANTICLE",
                CANTICLE_COLOR,
                CANTICLE_COLOR,
            );
            return;
        }

        if let Some(attack) = v16b.combat.pending_attack {
            vc_visual_announcement(
                framebuffer,
                void_attack_name(attack.kind),
                "VOID ATTACK",
                void_pressure_color(v16b.combat.pressure),
                VOID_LIGHT,
            );
        }
    }

    fn render_presentation_bestiary(&self, framebuffer: &mut Framebuffer) {
        for enemy in &self.game.game.base().enemies {
            vc27_hd_render_carrion(
                framebuffer,
                vc27_present(enemy.x),
                vc27_present(enemy.y),
                enemy.age,
                enemy.phase,
            );
        }

        let v12 = &self.game.game.v20().game.v14().progression.combat;
        for enemy in &v12.combat.specials {
            let x = vc27_present(enemy.x);
            let y = vc27_present(enemy.y);
            match enemy.kind {
                SpecialKind::GraveKnight => {
                    vc27_hd_render_grave_knight(framebuffer, x, y, enemy.age)
                }
                SpecialKind::BellWraith => {
                    vc27_hd_render_bell_wraith(framebuffer, x, y, enemy.age, enemy.phase)
                }
                SpecialKind::RelicCarrier => vc27_hd_render_relic_carrier(
                    framebuffer,
                    x,
                    y,
                    enemy.age,
                    enemy.phase,
                    enemy.direction,
                ),
            }
        }

        for threat in &v12.threats {
            let x = vc27_present(threat.x);
            let y = vc27_present(threat.y);
            match threat.kind {
                ThreatKind::ChoirNode => {
                    vc27_hd_render_choir_node(framebuffer, x, y, threat.age, threat.phase)
                }
                ThreatKind::VoidLeech => vc27_hd_render_void_leech(
                    framebuffer,
                    x,
                    y,
                    threat.age,
                    threat.phase,
                    threat.charge,
                ),
            }
        }

        let base = self.game.game.base();
        if base.encounter_phase != EncounterPhase::Cleared
            && let Some(boss) = base.boss
        {
            vc27_hd_render_bellkeeper(framebuffer, boss);
        }
    }

    fn render_canticle_charge(&self, framebuffer: &mut Framebuffer) {
        let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
        let base = self.game.game.base();
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

        vc27_segmented_bar(
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
        vc27_segmented_bar(
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
        let pressure = self.game.game.v20().game.ui.game.combat.combat.pressure;
        let active = vc27_pressure_level(pressure);
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
        let progression = &self.game.game.v20().game.v14().progression;
        let ratio = if progression.xp_next == 0 {
            1.0
        } else {
            progression.xp.min(progression.xp_next) as f32 / progression.xp_next as f32
        };
        let center_x = (18 * scale) as i32;
        let center_y = (VC_VISUAL_PRESENTATION_HEIGHT.saturating_sub(35 * scale)) as i32;
        let radius = 11 * scale;
        let level = progression.level;
        let tier_color = vc27_echo_level_color(level);

        vc27_echo_shell(
            framebuffer,
            center_x,
            center_y,
            radius,
            ratio,
            level,
            tier_color,
        );
    }

    fn render_player_last(&mut self, framebuffer: &mut Framebuffer) {
        if self.game.combat_model().player_hull <= 0.0 {
            return;
        }

        self.player_overlay.clear(Pixel::TRANSPARENT);
        let focused = self
            .game
            .game
            .game
            .game
            .movement_controls
            .action(FOCUS)
            .held();
        let base = self.game.game.base();
        self.pilgrim_art.render(
            &mut self.player_overlay,
            base.player_x.round() as i32,
            base.player_y.round() as i32,
            focused,
            base.invulnerability,
            base.animation_time,
        );
        vc_visual_blit_nearest(
            &self.player_overlay,
            framebuffer,
            VC_VISUAL_PRESENTATION_SCALE,
            true,
        );
        vc27_hd_render_pilgrim(
            framebuffer,
            vc27_present(base.player_x),
            vc27_present(base.player_y),
            focused,
            base.invulnerability,
            base.animation_time,
        );
    }

    fn render_combat_presentation(&mut self, framebuffer: &mut Framebuffer) {
        if !self.game.game.active_combat() {
            return;
        }

        self.render_presentation_bestiary(framebuffer);
        self.render_event_announcement(framebuffer);
        self.render_threat_meter(framebuffer);
        self.render_echo_shell(framebuffer);
        self.render_canticle_charge(framebuffer);
        self.render_survival_bars(framebuffer);
        self.render_player_last(framebuffer);
    }

    fn render_death_presentation(&mut self, framebuffer: &mut Framebuffer) {
        let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
        self.clean_background.clear(BG);
        render_grave_orbit_background(&mut self.clean_background, self.game.game.base().scroll);
        vc_visual_blit_nearest(
            &self.clean_background,
            framebuffer,
            VC_VISUAL_PRESENTATION_SCALE,
            false,
        );

        let panel_width = 150 * scale;
        let panel_height = 62 * scale;
        let panel_x = ((VC_VISUAL_PRESENTATION_WIDTH - panel_width) / 2) as i32;
        let panel_y = (118 * scale) as i32;
        framebuffer.fill_rect(
            panel_x,
            panel_y,
            panel_width,
            panel_height,
            Pixel::rgb(9, 8, 15),
        );
        framebuffer.draw_rect(panel_x, panel_y, panel_width, panel_height, DANGER);
        vc_visual_draw_centered_text(
            framebuffer,
            panel_y + 15 * scale as i32,
            "PILGRIM FALLEN",
            scale,
            DANGER,
        );
        vc_visual_draw_centered_text(
            framebuffer,
            panel_y + 39 * scale as i32,
            "SPACE TO RETURN",
            scale,
            TEXT,
        );
    }
}

impl Game for VoidCanticleV27DirectPresentation {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let result = {
            let mut simulation_frame = Frame {
                framebuffer: &mut self.simulation_framebuffer,
                input: frame.input,
                delta_time: frame.delta_time,
                storage: &mut *frame.storage,
                audio: &mut *frame.audio,
                surface_size: frame.surface_size,
                viewport: gotoo_pixel_engine::Viewport::new(
                    frame.surface_size,
                    Size {
                        width: FRAMEBUFFER_WIDTH,
                        height: FRAMEBUFFER_HEIGHT,
                    },
                ),
            };
            self.game.update(&mut simulation_frame)
        };
        if result == GameResult::Exit {
            return result;
        }

        let mode = self.visual_mode();
        if mode == VcVisualMode::Death {
            self.render_death_presentation(frame.framebuffer);
            return GameResult::Continue;
        }

        vc_visual_blit_nearest(
            &self.simulation_framebuffer,
            frame.framebuffer,
            VC_VISUAL_PRESENTATION_SCALE,
            false,
        );

        if mode == VcVisualMode::Combat {
            self.render_combat_presentation(frame.framebuffer);
        }

        GameResult::Continue
    }
}

fn vc27_pressure_level(pressure: VoidPressure) -> u32 {
    match pressure {
        VoidPressure::Dormant => 1,
        VoidPressure::Stirring => 2,
        VoidPressure::Awake => 3,
        VoidPressure::Hostile => 4,
        VoidPressure::Cataclysmic => 5,
    }
}

fn vc27_echo_level_color(level: u32) -> Pixel {
    match (level / 10) % 6 {
        0 => XP_ORB_CORE,
        1 => ART_CYAN_LIGHT,
        2 => SYNERGY_GOLD,
        3 => VOID_GLOW,
        4 => CANTICLE_COLOR,
        _ => DANGER,
    }
}

fn vc27_echo_shell(
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

fn vc27_segmented_bar(
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

pub fn run_v27_direct_presentation_with_obs_mirror() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: format!(
                "Void Canticle {VC27_PRESENTATION_VERSION} Direct HUD - Gotoo Pixel Engine"
            ),
            framebuffer_width: VC_VISUAL_PRESENTATION_WIDTH,
            framebuffer_height: VC_VISUAL_PRESENTATION_HEIGHT,
            window_width,
            window_height,
        },
        gotoo_pixel_engine::ObsMirrorGame::from_env(
            VoidCanticleV27DirectPresentation::new(),
            VC_VISUAL_PRESENTATION_WIDTH,
            VC_VISUAL_PRESENTATION_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v27_tests {
    use super::*;

    #[test]
    fn version_is_explicit() {
        assert_eq!(VC27_PRESENTATION_VERSION, "VC2.7");
    }

    #[test]
    fn simulation_coordinates_map_to_presentation_space() {
        assert_eq!(vc27_present(0.0), 0);
        assert_eq!(vc27_present(90.0), 180);
        assert_eq!(vc27_present(319.5), 639);
    }

    #[test]
    fn pressure_maps_to_five_visual_bricks() {
        assert_eq!(vc27_pressure_level(VoidPressure::Dormant), 1);
        assert_eq!(vc27_pressure_level(VoidPressure::Stirring), 2);
        assert_eq!(vc27_pressure_level(VoidPressure::Awake), 3);
        assert_eq!(vc27_pressure_level(VoidPressure::Hostile), 4);
        assert_eq!(vc27_pressure_level(VoidPressure::Cataclysmic), 5);
    }

    #[test]
    fn echo_shell_changes_color_at_ten_level_boundaries() {
        assert_eq!(vc27_echo_level_color(1), vc27_echo_level_color(9));
        assert_ne!(vc27_echo_level_color(9), vc27_echo_level_color(10));
        assert_ne!(vc27_echo_level_color(19), vc27_echo_level_color(20));
        assert_ne!(vc27_echo_level_color(29), vc27_echo_level_color(30));
    }

    #[test]
    fn echo_shell_stays_local_to_its_glyph() {
        let mut framebuffer = Framebuffer::new(40, 40);
        framebuffer.clear(Pixel::BLUE);
        vc27_echo_shell(
            &mut framebuffer,
            20,
            20,
            8,
            0.5,
            12,
            vc27_echo_level_color(12),
        );
        assert_eq!(framebuffer.pixel(0, 0), Some(Pixel::BLUE));
        assert_eq!(framebuffer.pixel(39, 39), Some(Pixel::BLUE));
    }

    #[test]
    fn hd_choir_node_keeps_changes_local() {
        let mut framebuffer = Framebuffer::new(96, 96);
        framebuffer.clear(Pixel::BLUE);
        vc27_hd_render_choir_node(&mut framebuffer, 48, 48, 1.0, 0.0);
        assert_ne!(framebuffer.pixel(48, 48), Some(Pixel::BLUE));
        assert_eq!(framebuffer.pixel(0, 0), Some(Pixel::BLUE));
        assert_eq!(framebuffer.pixel(95, 95), Some(Pixel::BLUE));
    }

    #[test]
    fn segmented_bar_only_touches_its_own_rows() {
        let mut framebuffer = Framebuffer::new(20, 8);
        framebuffer.clear(Pixel::BLUE);
        vc27_segmented_bar(&mut framebuffer, 2, 3, 12, 2, 3.0, 12.0, 4, Pixel::RED);
        assert_eq!(framebuffer.pixel(2, 1), Some(Pixel::BLUE));
        assert_eq!(framebuffer.pixel(2, 6), Some(Pixel::BLUE));
    }
}
