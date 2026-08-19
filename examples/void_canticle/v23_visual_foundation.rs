#[cfg(not(target_arch = "wasm32"))]
const VC_VISUAL_PRESENTATION_SCALE: u32 = 2;
#[cfg(target_arch = "wasm32")]
const VC_VISUAL_PRESENTATION_SCALE: u32 = 1;

const VC_VISUAL_PRESENTATION_WIDTH: u32 = FRAMEBUFFER_WIDTH * VC_VISUAL_PRESENTATION_SCALE;
const VC_VISUAL_PRESENTATION_HEIGHT: u32 = FRAMEBUFFER_HEIGHT * VC_VISUAL_PRESENTATION_SCALE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VcVisualMode {
    Combat,
    Pause,
    LevelChoice,
    MutationChoice,
    SupportChoice,
    Death,
    StageClear,
}

struct VoidCanticleVisualFoundation {
    game: VoidCanticleV23Sustain,
    simulation_framebuffer: Framebuffer,
    background_framebuffer: Framebuffer,
    player_overlay: Framebuffer,
}

impl VoidCanticleVisualFoundation {
    fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        eprintln!(
            "Void Canticle visual foundation: simulation {}x{} -> presentation {}x{}",
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
            VC_VISUAL_PRESENTATION_WIDTH,
            VC_VISUAL_PRESENTATION_HEIGHT
        );

        Self {
            game: VoidCanticleV23Sustain::new(),
            simulation_framebuffer: Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT),
            background_framebuffer: Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT),
            player_overlay: Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT),
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

    fn modal_bottom_start(mode: VcVisualMode) -> i32 {
        match mode {
            VcVisualMode::Combat => 265,
            VcVisualMode::Pause => 272,
            VcVisualMode::LevelChoice | VcVisualMode::MutationChoice => 268,
            VcVisualMode::SupportChoice => 248,
            VcVisualMode::Death => 186,
            VcVisualMode::StageClear => 263,
        }
    }

    fn prepare_background_frame(&mut self) {
        let (scroll, color) = {
            let base = self.game.game.base();
            (
                base.scroll,
                if base.canticle_timer > 0.46 {
                    BG_CANTICLE
                } else {
                    BG
                },
            )
        };
        self.background_framebuffer.clear(color);
        render_grave_orbit_background(&mut self.background_framebuffer, scroll);
    }

    fn clean_simulation_legacy_hud(&mut self, mode: VcVisualMode) {
        // Restore the actual Grave Orbit background underneath pixels that
        // belong to historical HUD layers. This is deliberately different
        // from painting opaque BG rectangles: gameplay keeps visually flowing
        // behind the new presentation layer instead of being cut by black bands.
        const META_COLORS: &[Pixel] = &[
            BG,
            TEXT,
            ACCENT,
            WRECK_LIGHT,
            PILGRIM_CORE,
            CANTICLE_COLOR,
        ];
        const SURVIVAL_COLORS: &[Pixel] = &[
            BG,
            TEXT,
            CORE_BG,
            VC20_HULL,
            VC20_ARMOR,
            VC20_ARMOR_LIGHT,
            VC20_ARMOR_BG,
            DANGER,
            WRECK_LIGHT,
        ];
        const BOTTOM_COLORS: &[Pixel] = &[
            BG,
            TEXT,
            WRECK_LIGHT,
            CORE_BG,
            CINDER,
            CANTICLE_COLOR,
            XP_ORB_CORE,
            XP_BAR_BG,
            XP_BAR_FILL,
            VOID_DIM,
            VOID_GLOW,
            VOID_DANGER,
            VOID_CATA,
            VOID_LIGHT,
        ];

        vc_visual_restore_matching_colors(
            &mut self.simulation_framebuffer,
            &self.background_framebuffer,
            gotoo_pixel_engine::Rect {
                x: 0,
                y: 0,
                width: 180,
                height: 24,
            },
            META_COLORS,
        );
        vc_visual_restore_matching_colors(
            &mut self.simulation_framebuffer,
            &self.background_framebuffer,
            gotoo_pixel_engine::Rect {
                x: 0,
                y: 24,
                width: 108,
                height: 17,
            },
            SURVIVAL_COLORS,
        );

        if mode == VcVisualMode::Combat {
            const EVENT_COLORS: &[Pixel] = &[
                Pixel::rgb(6, 5, 16),
                Pixel::rgb(7, 6, 16),
                Pixel::rgb(7, 15, 20),
                Pixel::rgb(8, 7, 18),
                Pixel::rgb(10, 7, 12),
                TEXT,
                WRECK_LIGHT,
                VOID_DIM,
                VOID_GLOW,
                VOID_DANGER,
                VOID_CATA,
                VOID_LIGHT,
                SYNERGY_COLOR,
                SYNERGY_LIGHT,
                SYNERGY_GOLD,
                BELL_LIGHT,
                CANTICLE_COLOR,
            ];
            for rect in [
                gotoo_pixel_engine::Rect {
                    x: 0,
                    y: 0,
                    width: 180,
                    height: 29,
                },
                gotoo_pixel_engine::Rect {
                    x: 39,
                    y: 30,
                    width: 103,
                    height: 23,
                },
                gotoo_pixel_engine::Rect {
                    x: 10,
                    y: 33,
                    width: 160,
                    height: 37,
                },
                gotoo_pixel_engine::Rect {
                    x: 21,
                    y: 72,
                    width: 138,
                    height: 35,
                },
            ] {
                vc_visual_restore_matching_colors(
                    &mut self.simulation_framebuffer,
                    &self.background_framebuffer,
                    rect,
                    EVENT_COLORS,
                );
            }
        }

        let bottom_start = Self::modal_bottom_start(mode);
        for (start, end) in [(267, 290), (288, 315), (315, 320)] {
            let y = start.max(bottom_start);
            if y >= end {
                continue;
            }
            vc_visual_restore_matching_colors(
                &mut self.simulation_framebuffer,
                &self.background_framebuffer,
                gotoo_pixel_engine::Rect {
                    x: 0,
                    y,
                    width: FRAMEBUFFER_WIDTH,
                    height: (end - y) as u32,
                },
                BOTTOM_COLORS,
            );
        }
    }

    fn render_presentation(&mut self, framebuffer: &mut Framebuffer, mode: VcVisualMode) {
        // Non-combat screens are already fully rendered by the simulation
        // layer. Do not crop/recompose them: that was the source of the pause
        // and upgrade-screen desynchronisation. Combat alone receives the HD HUD.
        if mode == VcVisualMode::Combat {
            self.render_clean_combat_presentation(framebuffer);
        }
    }

    fn render_clean_combat_presentation(&mut self, framebuffer: &mut Framebuffer) {
        if !self.game.game.active_combat() {
            return;
        }

        self.render_top_survival_console(framebuffer);
        self.render_boss_status(framebuffer);
        self.render_compact_core(framebuffer);
        self.render_compact_xp(framebuffer);
        self.render_event_announcement(framebuffer);
        self.render_player_foreground(framebuffer);
    }

    fn render_top_survival_console(&self, framebuffer: &mut Framebuffer) {
        let scale = VC_VISUAL_PRESENTATION_SCALE;
        let model = self.game.combat_model();
        let hull_cap = self.game.hull_cap();
        let shield_cap = self.game.shield_cap().max(1.0);
        let hull_segments = ((hull_cap / 10.0).ceil() as u32).clamp(4, 12);
        let shield_segments = ((shield_cap / 5.0).ceil() as u32).clamp(3, 10);

        // No opaque top console. Only the gauges themselves occupy pixels;
        // Grave Orbit remains visible between and underneath them.
        let margin = 13 * scale;
        let gap = 18 * scale;
        let bar_width = (VC_VISUAL_PRESENTATION_WIDTH - margin * 2 - gap) / 2;
        let bar_height = 5 * scale;
        let y = 8 * scale;
        let hull_x = margin;
        let shield_x = margin + bar_width + gap;

        vc_visual_segmented_bar(
            framebuffer,
            hull_x as i32,
            y as i32,
            bar_width,
            bar_height,
            model.player_hull,
            hull_cap,
            hull_segments,
            if model.player_hull_flash_timer > 0.0 {
                DANGER
            } else {
                VC20_HULL
            },
            CORE_BG,
        );
        vc_visual_segmented_bar(
            framebuffer,
            shield_x as i32,
            y as i32,
            bar_width,
            bar_height,
            model.player_shield,
            shield_cap,
            shield_segments,
            if model.player_shield_flash_timer > 0.0 {
                VC20_ARMOR_LIGHT
            } else {
                VC20_ARMOR
            },
            VC20_ARMOR_BG,
        );

        let hull_icon_x = (margin / 2) as i32;
        let icon_y = (y + bar_height / 2) as i32;
        framebuffer.draw_line(hull_icon_x - 4, icon_y, hull_icon_x, icon_y - 4, VC20_HULL);
        framebuffer.draw_line(hull_icon_x, icon_y - 4, hull_icon_x + 4, icon_y, VC20_HULL);
        framebuffer.draw_line(hull_icon_x + 4, icon_y, hull_icon_x, icon_y + 4, VC20_HULL);
        framebuffer.draw_line(hull_icon_x, icon_y + 4, hull_icon_x - 4, icon_y, VC20_HULL);

        let shield_icon_x = (shield_x - gap / 2) as i32;
        framebuffer.draw_circle(shield_icon_x, icon_y, 4 * scale, VC20_ARMOR_LIGHT);

        self.render_sustain_into_survival_console(
            framebuffer,
            hull_x,
            shield_x,
            bar_width,
            y,
            bar_height,
        );
    }

    fn render_sustain_into_survival_console(
        &self,
        framebuffer: &mut Framebuffer,
        hull_x: u32,
        shield_x: u32,
        bar_width: u32,
        bar_y: u32,
        bar_height: u32,
    ) {
        let Some(augment) = self.game.augment else {
            return;
        };

        let (x, delay, color) = match augment {
            Vc23SustainAugment::NaniteRepair => (hull_x, VC23_NANITE_DELAY, CANTICLE_COLOR),
            Vc23SustainAugment::ShieldCapacitor => {
                (shield_x, VC23_CAPACITOR_DELAY, ART_CYAN_LIGHT)
            }
        };
        let scale = VC_VISUAL_PRESENTATION_SCALE;
        let y = bar_y + bar_height + 2 * scale;
        let progress = (self.game.quiet_timer / delay).clamp(0.0, 1.0);
        framebuffer.draw_line(
            x as i32,
            y as i32,
            (x + bar_width - 1) as i32,
            y as i32,
            WRECK_MID,
        );
        let filled = (bar_width as f32 * progress).round() as u32;
        if filled > 0 {
            framebuffer.draw_line(
                x as i32,
                y as i32,
                (x + filled.min(bar_width) - 1) as i32,
                y as i32,
                color,
            );
        }

        if self.game.quiet_timer >= delay || self.game.sustain_flash_timer > 0.0 {
            framebuffer.draw_rect(
                x as i32 - 2,
                bar_y as i32 - 2,
                bar_width + 4,
                bar_height + 4,
                color,
            );
        }
    }

    fn render_boss_status(&self, framebuffer: &mut Framebuffer) {
        let base = self.game.game.base();
        if base.encounter_phase != EncounterPhase::BossFight {
            return;
        }
        let Some(boss) = base.boss else {
            return;
        };

        let scale = VC_VISUAL_PRESENTATION_SCALE;
        let width = 82 * scale;
        let height = 2 * scale;
        let x = (VC_VISUAL_PRESENTATION_WIDTH - width) / 2;
        let y = 20 * scale;
        let ratio = boss.hp as f32 / BELLKEEPER_MAX_HP as f32;
        framebuffer.draw_rect(
            x as i32 - 1,
            y as i32 - 1,
            width + 2,
            height + 2,
            BELL_DARK,
        );
        let filled = (width as f32 * ratio).round() as u32;
        if filled > 0 {
            framebuffer.fill_rect(x as i32, y as i32, filled.min(width), height, DANGER);
        }
    }

    fn render_compact_core(&self, framebuffer: &mut Framebuffer) {
        let scale = VC_VISUAL_PRESENTATION_SCALE;
        let base = self.game.game.base();
        let x = 10 * scale;
        let y = 306 * scale;
        let width = 62 * scale;
        let height = 5 * scale;
        let ratio = base.core_charge.min(CORE_MAX) as f32 / CORE_MAX as f32;
        let ready = base.core_charge >= CORE_MAX;
        let color = if ready { CANTICLE_COLOR } else { CINDER };

        // The empty part is intentionally not filled: the game remains visible
        // through the gauge. Only outline, charge and segment ticks are drawn.
        framebuffer.draw_rect(x as i32 - 1, y as i32 - 1, width + 2, height + 2, WRECK_MID);
        let fill = (width as f32 * ratio).round() as u32;
        if fill > 0 {
            framebuffer.fill_rect(x as i32, y as i32, fill.min(width), height, color);
        }
        for segment in 1..5 {
            let separator = x + width * segment / 5;
            framebuffer.draw_line(
                separator as i32,
                y as i32,
                separator as i32,
                (y + height - 1) as i32,
                WRECK_MID,
            );
        }

        let icon_x = x as i32 - 7;
        let icon_y = (y + height / 2) as i32;
        framebuffer.draw_line(icon_x, icon_y - 4, icon_x + 4, icon_y, color);
        framebuffer.draw_line(icon_x + 4, icon_y, icon_x, icon_y + 4, color);
        framebuffer.draw_line(icon_x, icon_y + 4, icon_x - 4, icon_y, color);
        framebuffer.draw_line(icon_x - 4, icon_y, icon_x, icon_y - 4, color);

        if ready {
            let (label_width, label_height) = Framebuffer::text_size("READY", 1);
            let label_x = x as i32 + (width.saturating_sub(label_width) / 2) as i32;
            let label_y = y as i32 + (height.saturating_sub(label_height) / 2) as i32;
            framebuffer.draw_text(label_x, label_y, "READY", BG);
        }
    }

    fn render_compact_xp(&self, framebuffer: &mut Framebuffer) {
        let progression = &self.game.game.v20().game.v14().progression;
        let scale = VC_VISUAL_PRESENTATION_SCALE;
        let width = 154 * scale;
        let x = (VC_VISUAL_PRESENTATION_WIDTH - width) / 2;
        let y = VC_VISUAL_PRESENTATION_HEIGHT.saturating_sub(2 * scale);
        let ratio = progression.xp.min(progression.xp_next) as f32 / progression.xp_next.max(1) as f32;
        let fill = (width as f32 * ratio).round() as u32;

        framebuffer.draw_line(
            x as i32,
            y as i32,
            (x + width - 1) as i32,
            y as i32,
            WRECK_MID,
        );
        if fill > 0 {
            framebuffer.draw_line(
                x as i32,
                y as i32,
                (x + fill.min(width) - 1) as i32,
                y as i32,
                XP_BAR_FILL,
            );
        }
    }

    fn v16b(&self) -> &VoidCanticleV16B {
        &self.game.game.v20().game.ui.game.combat
    }

    fn render_event_announcement(&self, framebuffer: &mut Framebuffer) {
        let v16b = self.v16b();

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

    fn render_player_foreground(&mut self, framebuffer: &mut Framebuffer) {
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
        let game = &self.game;
        let overlay = &mut self.player_overlay;
        let base = game.game.base();
        base.visuals.render_pilgrim(
            overlay,
            base.player_x.round() as i32,
            base.player_y.round() as i32,
            focused,
            base.invulnerability,
            base.animation_time,
        );
        vc_visual_blit_nearest(
            overlay,
            framebuffer,
            VC_VISUAL_PRESENTATION_SCALE,
            true,
        );
    }
}

impl Game for VoidCanticleVisualFoundation {
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
        self.prepare_background_frame();
        self.clean_simulation_legacy_hud(mode);

        vc_visual_blit_nearest(
            &self.simulation_framebuffer,
            frame.framebuffer,
            VC_VISUAL_PRESENTATION_SCALE,
            false,
        );
        self.render_presentation(frame.framebuffer, mode);
        GameResult::Continue
    }
}

fn vc_visual_segmented_bar(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    value: f32,
    max_value: f32,
    segments: u32,
    fill: Pixel,
    background: Pixel,
) {
    let ratio = (value / max_value.max(1.0)).clamp(0.0, 1.0);
    framebuffer.fill_rect(x, y, width, height, background);
    let filled = (width as f32 * ratio).round() as u32;
    if filled > 0 {
        framebuffer.fill_rect(x, y, filled.min(width), height, fill);
    }

    let segments = segments.max(1);
    for segment in 1..segments {
        let separator_x = x + (width.saturating_mul(segment) / segments) as i32;
        framebuffer.fill_rect(separator_x, y, 1, height, WRECK_MID);
    }
    framebuffer.draw_rect(x - 1, y - 1, width + 2, height + 2, WRECK_LIGHT);
}

fn vc_visual_announcement(
    framebuffer: &mut Framebuffer,
    headline: &str,
    detail: &str,
    border: Pixel,
    text: Pixel,
) {
    let scale = VC_VISUAL_PRESENTATION_SCALE;
    let width = (146 * scale).min(VC_VISUAL_PRESENTATION_WIDTH.saturating_sub(16 * scale));
    let height = 23 * scale;
    let x = ((VC_VISUAL_PRESENTATION_WIDTH - width) / 2) as i32;
    let y = (27 * scale) as i32;

    framebuffer.fill_rect(x, y, width, height, Pixel::rgb(7, 6, 16));
    framebuffer.draw_rect(x, y, width, height, border);
    framebuffer.draw_line(
        x + 6,
        y + height as i32 - 3,
        x + width as i32 - 7,
        y + height as i32 - 3,
        WRECK_MID,
    );

    vc_visual_draw_centered_text(framebuffer, y + 4, headline, 2.min(scale), text);
    if !detail.is_empty() {
        vc_visual_draw_centered_text(
            framebuffer,
            y + height as i32 - 11,
            detail,
            1,
            border,
        );
    }
}

fn vc_visual_draw_centered_text(
    framebuffer: &mut Framebuffer,
    y: i32,
    text: &str,
    scale: u32,
    color: Pixel,
) {
    let scale = scale.max(1);
    let (width, _) = Framebuffer::text_size(text, scale);
    let x = ((VC_VISUAL_PRESENTATION_WIDTH.saturating_sub(width)) / 2) as i32;
    framebuffer.draw_text_scaled(x, y, text, scale, color);
}

fn vc_visual_restore_matching_colors(
    framebuffer: &mut Framebuffer,
    background: &Framebuffer,
    rect: gotoo_pixel_engine::Rect,
    colors: &[Pixel],
) {
    let start_x = rect.x.max(0) as u32;
    let start_y = rect.y.max(0) as u32;
    let end_x = start_x.saturating_add(rect.width).min(framebuffer.width());
    let end_y = start_y.saturating_add(rect.height).min(framebuffer.height());

    for y in start_y..end_y {
        for x in start_x..end_x {
            let Some(pixel) = framebuffer.pixel(x as i32, y as i32) else {
                continue;
            };
            if !colors.contains(&pixel) {
                continue;
            }
            if let Some(replacement) = background.pixel(x as i32, y as i32) {
                framebuffer.set_pixel_in_bounds(x, y, replacement);
            }
        }
    }
}

fn vc_visual_blit_nearest(
    source: &Framebuffer,
    destination: &mut Framebuffer,
    scale: u32,
    skip_transparent: bool,
) {
    let scale = scale.max(1);
    let bytes = source.as_rgba8();
    let source_width = source.width();

    for y in 0..source.height() {
        for x in 0..source_width {
            let index = ((y * source_width + x) * 4) as usize;
            let pixel = Pixel::rgba(
                bytes[index],
                bytes[index + 1],
                bytes[index + 2],
                bytes[index + 3],
            );
            if skip_transparent && pixel.a == 0 {
                continue;
            }
            destination.fill_rect(
                (x * scale) as i32,
                (y * scale) as i32,
                scale,
                scale,
                pixel,
            );
        }
    }
}

pub fn run_visual_foundation_with_obs_mirror() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: format!("Void Canticle {VC23_VERSION} Visual Foundation - Gotoo Pixel Engine"),
            framebuffer_width: VC_VISUAL_PRESENTATION_WIDTH,
            framebuffer_height: VC_VISUAL_PRESENTATION_HEIGHT,
            window_width,
            window_height,
        },
        gotoo_pixel_engine::ObsMirrorGame::from_env(
            VoidCanticleVisualFoundation::new(),
            VC_VISUAL_PRESENTATION_WIDTH,
            VC_VISUAL_PRESENTATION_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod visual_foundation_tests {
    use super::*;

    #[test]
    fn presentation_keeps_simulation_coordinates_stable() {
        assert_eq!(FRAMEBUFFER_WIDTH, 180);
        assert_eq!(FRAMEBUFFER_HEIGHT, 320);
        assert_eq!(
            VC_VISUAL_PRESENTATION_WIDTH,
            FRAMEBUFFER_WIDTH * VC_VISUAL_PRESENTATION_SCALE
        );
        assert_eq!(
            VC_VISUAL_PRESENTATION_HEIGHT,
            FRAMEBUFFER_HEIGHT * VC_VISUAL_PRESENTATION_SCALE
        );
    }

    #[test]
    fn modal_cleanup_starts_below_each_panel() {
        assert_eq!(VoidCanticleVisualFoundation::modal_bottom_start(VcVisualMode::Pause), 272);
        assert_eq!(
            VoidCanticleVisualFoundation::modal_bottom_start(VcVisualMode::LevelChoice),
            268
        );
        assert_eq!(
            VoidCanticleVisualFoundation::modal_bottom_start(VcVisualMode::SupportChoice),
            248
        );
    }

    #[test]
    fn nearest_blit_expands_each_source_pixel_without_filtering() {
        let mut source = Framebuffer::new(2, 1);
        source.draw(0, 0, Pixel::RED);
        source.draw(1, 0, Pixel::BLUE);
        let mut destination = Framebuffer::new(4, 2);

        vc_visual_blit_nearest(&source, &mut destination, 2, false);

        for y in 0..2 {
            assert_eq!(destination.pixel(0, y), Some(Pixel::RED));
            assert_eq!(destination.pixel(1, y), Some(Pixel::RED));
            assert_eq!(destination.pixel(2, y), Some(Pixel::BLUE));
            assert_eq!(destination.pixel(3, y), Some(Pixel::BLUE));
        }
    }

    #[test]
    fn legacy_hud_cleanup_restores_real_background_pixels() {
        let mut rendered = Framebuffer::new(3, 1);
        rendered.clear(Pixel::RED);
        rendered.draw(1, 0, Pixel::WHITE);
        let mut background = Framebuffer::new(3, 1);
        background.clear(Pixel::BLUE);

        vc_visual_restore_matching_colors(
            &mut rendered,
            &background,
            gotoo_pixel_engine::Rect {
                x: 0,
                y: 0,
                width: 3,
                height: 1,
            },
            &[Pixel::WHITE],
        );

        assert_eq!(rendered.pixel(0, 0), Some(Pixel::RED));
        assert_eq!(rendered.pixel(1, 0), Some(Pixel::BLUE));
        assert_eq!(rendered.pixel(2, 0), Some(Pixel::RED));
    }

    #[test]
    fn transparent_overlay_does_not_erase_destination() {
        let source = Framebuffer::new(1, 1);
        let mut destination = Framebuffer::new(2, 2);
        destination.clear(Pixel::GREEN);

        vc_visual_blit_nearest(&source, &mut destination, 2, true);

        assert!(destination
            .as_rgba8()
            .chunks_exact(4)
            .all(|rgba| rgba == Pixel::GREEN.to_rgba8()));
    }
}
