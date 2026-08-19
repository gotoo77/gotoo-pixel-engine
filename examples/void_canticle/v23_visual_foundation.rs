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

    fn modal_rect(mode: VcVisualMode) -> Option<gotoo_pixel_engine::Rect> {
        match mode {
            VcVisualMode::Pause => Some(gotoo_pixel_engine::Rect {
                x: 8,
                y: 44,
                width: 164,
                height: 228,
            }),
            VcVisualMode::LevelChoice | VcVisualMode::MutationChoice => {
                Some(gotoo_pixel_engine::Rect {
                    x: 8,
                    y: 48,
                    width: 164,
                    height: 220,
                })
            }
            VcVisualMode::SupportChoice => Some(gotoo_pixel_engine::Rect {
                x: 8,
                y: 72,
                width: 164,
                height: 176,
            }),
            VcVisualMode::Death => Some(gotoo_pixel_engine::Rect {
                x: 18,
                y: 128,
                width: 144,
                height: 58,
            }),
            VcVisualMode::StageClear => Some(gotoo_pixel_engine::Rect {
                x: 10,
                y: 53,
                width: 160,
                height: 210,
            }),
            VcVisualMode::Combat => None,
        }
    }

    fn render_presentation(&mut self, framebuffer: &mut Framebuffer) {
        let mode = self.visual_mode();
        if mode == VcVisualMode::Combat {
            self.render_clean_combat_presentation(framebuffer);
        } else if let Some(rect) = Self::modal_rect(mode) {
            self.render_modal_focus(framebuffer, rect);
        }
    }

    fn render_modal_focus(&self, framebuffer: &mut Framebuffer, rect: gotoo_pixel_engine::Rect) {
        // Modal screens own the complete presentation. Do not leave a stale
        // gameplay frame, player redraw or HUD component around the panel.
        framebuffer.clear(BG);
        vc_visual_blit_region_nearest(
            &self.simulation_framebuffer,
            framebuffer,
            rect,
            VC_VISUAL_PRESENTATION_SCALE,
        );
    }

    fn render_clean_combat_presentation(&mut self, framebuffer: &mut Framebuffer) {
        if !self.game.game.active_combat() {
            return;
        }

        self.render_top_survival_console(framebuffer);
        self.remove_obsolete_bottom_hud(framebuffer);
        self.render_compact_core(framebuffer);
        self.render_event_announcement(framebuffer);
        self.render_player_foreground(framebuffer);
    }

    fn render_top_survival_console(&self, framebuffer: &mut Framebuffer) {
        let scale = VC_VISUAL_PRESENTATION_SCALE;
        let top_height = 42 * scale;
        framebuffer.fill_rect(0, 0, VC_VISUAL_PRESENTATION_WIDTH, top_height, BG);
        framebuffer.draw_line(
            0,
            top_height as i32 - 1,
            VC_VISUAL_PRESENTATION_WIDTH as i32 - 1,
            top_height as i32 - 1,
            WRECK_MID,
        );

        let model = self.game.combat_model();
        let hull_cap = self.game.hull_cap();
        let shield_cap = self.game.shield_cap().max(1.0);
        let hull_segments = ((hull_cap / 10.0).ceil() as u32).clamp(4, 12);
        let shield_segments = ((shield_cap / 5.0).ceil() as u32).clamp(3, 10);

        let margin = 12 * scale;
        let gap = 18 * scale;
        let bar_width = (VC_VISUAL_PRESENTATION_WIDTH - margin * 2 - gap) / 2;
        let bar_height = 6 * scale;
        let y = 14 * scale;
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

        // Tiny iconography instead of permanent HULL/SHIELD labels.
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
            y + bar_height + 3 * scale,
        );
    }

    fn render_sustain_into_survival_console(
        &self,
        framebuffer: &mut Framebuffer,
        hull_x: u32,
        shield_x: u32,
        bar_width: u32,
        y: u32,
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
        let progress = (self.game.quiet_timer / delay).clamp(0.0, 1.0);
        framebuffer.fill_rect(x as i32, y as i32, bar_width, 2, WRECK_MID);
        let filled = (bar_width as f32 * progress).round() as u32;
        if filled > 0 {
            framebuffer.fill_rect(x as i32, y as i32, filled.min(bar_width), 2, color);
        }

        if self.game.quiet_timer >= delay || self.game.sustain_flash_timer > 0.0 {
            let scale = VC_VISUAL_PRESENTATION_SCALE;
            framebuffer.draw_rect(
                x as i32 - 2,
                14 * scale as i32 - 2,
                bar_width + 4,
                6 * scale + 4,
                color,
            );
        }
    }

    fn remove_obsolete_bottom_hud(&self, framebuffer: &mut Framebuffer) {
        // These are legacy text/debug components. The central strip also
        // removes the old oversized Canticle READY bar; the player is redrawn
        // afterwards so HUD cleanup never cuts through the ship.
        for (x, y, width, height) in [
            (0, 267, 180, 23),  // CINDERS/CORE + oversized READY/Core meter
            (0, 288, 62, 15),   // LV n / CORE text
            (0, 292, 180, 22),  // permanent control hints + ECHOES text
        ] {
            vc_visual_mask_logical_rect(framebuffer, x, y, width, height);
        }
    }

    fn render_compact_core(&self, framebuffer: &mut Framebuffer) {
        let scale = VC_VISUAL_PRESENTATION_SCALE;
        let base = self.game.game.base();
        let x = 9 * scale;
        let y = 302 * scale;
        let width = 58 * scale;
        let height = 4 * scale;
        let ratio = base.core_charge.min(CORE_MAX) as f32 / CORE_MAX as f32;
        let ready = base.core_charge >= CORE_MAX;
        let color = if ready { CANTICLE_COLOR } else { CINDER };

        framebuffer.fill_rect(x as i32, y as i32, width, height, CORE_BG);
        let fill = (width as f32 * ratio).round() as u32;
        if fill > 0 {
            framebuffer.fill_rect(x as i32, y as i32, fill.min(width), height, color);
        }
        for segment in 1..5 {
            let separator = x + width * segment / 5;
            framebuffer.fill_rect(separator as i32, y as i32, 1, height, BG);
        }
        framebuffer.draw_rect(x as i32 - 1, y as i32 - 1, width + 2, height + 2, color);

        let icon_x = x as i32 - 7;
        let icon_y = (y + height / 2) as i32;
        framebuffer.draw_line(icon_x, icon_y - 4, icon_x + 4, icon_y, color);
        framebuffer.draw_line(icon_x + 4, icon_y, icon_x, icon_y + 4, color);
        framebuffer.draw_line(icon_x, icon_y + 4, icon_x - 4, icon_y, color);
        framebuffer.draw_line(icon_x - 4, icon_y, icon_x, icon_y - 4, color);

        if ready {
            let label_x = x as i32 + width as i32 - 34;
            let label_y = y as i32 + (height as i32 - 7) / 2;
            framebuffer.fill_rect(label_x - 2, label_y - 1, 34, 9, BG);
            framebuffer.draw_text(label_x, label_y, "READY", CANTICLE_COLOR);
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
            // Replace the legacy mid-screen warning panel with one compact,
            // consistently positioned event banner.
            vc_visual_mask_logical_rect(framebuffer, 39, 30, 102, 22);
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

        vc_visual_blit_nearest(
            &self.simulation_framebuffer,
            frame.framebuffer,
            VC_VISUAL_PRESENTATION_SCALE,
            false,
        );
        self.render_presentation(frame.framebuffer);
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
        framebuffer.fill_rect(separator_x, y, 1, height, BG);
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
    let width = (150 * scale).min(VC_VISUAL_PRESENTATION_WIDTH.saturating_sub(16 * scale));
    let height = 25 * scale;
    let x = ((VC_VISUAL_PRESENTATION_WIDTH - width) / 2) as i32;
    let y = (47 * scale) as i32;

    framebuffer.fill_rect(x, y, width, height, Pixel::rgb(7, 6, 16));
    framebuffer.draw_rect(x, y, width, height, border);
    framebuffer.draw_line(x + 6, y + height as i32 - 3, x + width as i32 - 7, y + height as i32 - 3, WRECK_MID);

    vc_visual_draw_centered_text(framebuffer, y + 5, headline, 2.min(scale), text);
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

fn vc_visual_mask_logical_rect(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) {
    let scale = VC_VISUAL_PRESENTATION_SCALE;
    framebuffer.fill_rect(
        x * scale as i32,
        y * scale as i32,
        width.saturating_mul(scale),
        height.saturating_mul(scale),
        BG,
    );
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

fn vc_visual_blit_region_nearest(
    source: &Framebuffer,
    destination: &mut Framebuffer,
    rect: gotoo_pixel_engine::Rect,
    scale: u32,
) {
    let scale = scale.max(1);
    let start_x = rect.x.max(0) as u32;
    let start_y = rect.y.max(0) as u32;
    let end_x = start_x.saturating_add(rect.width).min(source.width());
    let end_y = start_y.saturating_add(rect.height).min(source.height());

    for y in start_y..end_y {
        for x in start_x..end_x {
            let Some(pixel) = source.pixel(x as i32, y as i32) else {
                continue;
            };
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
    fn all_non_combat_modes_have_modal_focus_rects() {
        for mode in [
            VcVisualMode::Pause,
            VcVisualMode::LevelChoice,
            VcVisualMode::MutationChoice,
            VcVisualMode::SupportChoice,
            VcVisualMode::Death,
            VcVisualMode::StageClear,
        ] {
            assert!(VoidCanticleVisualFoundation::modal_rect(mode).is_some());
        }
        assert!(VoidCanticleVisualFoundation::modal_rect(VcVisualMode::Combat).is_none());
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
    fn modal_region_blit_does_not_copy_gameplay_outside_panel() {
        let mut source = Framebuffer::new(4, 4);
        source.clear(Pixel::RED);
        source.fill_rect(1, 1, 2, 2, Pixel::BLUE);
        let mut destination = Framebuffer::new(8, 8);
        destination.clear(Pixel::BLACK);

        vc_visual_blit_region_nearest(
            &source,
            &mut destination,
            gotoo_pixel_engine::Rect {
                x: 1,
                y: 1,
                width: 2,
                height: 2,
            },
            2,
        );

        assert_eq!(destination.pixel(0, 0), Some(Pixel::BLACK));
        assert_eq!(destination.pixel(2, 2), Some(Pixel::BLUE));
        assert_eq!(destination.pixel(5, 5), Some(Pixel::BLUE));
        assert_eq!(destination.pixel(7, 7), Some(Pixel::BLACK));
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