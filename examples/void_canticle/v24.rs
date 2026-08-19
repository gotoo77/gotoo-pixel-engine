const VC24_PRESENTATION_VERSION: &str = "VC2.4";

struct VoidCanticleV24HudPolish {
    inner: VoidCanticleVisualFoundation,
}

impl VoidCanticleV24HudPolish {
    fn new() -> Self {
        Self {
            inner: VoidCanticleVisualFoundation::new(),
        }
    }

    fn clean_residual_legacy_hud(&mut self, mode: VcVisualMode) {
        if mode != VcVisualMode::Combat {
            return;
        }

        // The VC2 presentation owns combat UI now. Remove the remaining
        // historical wave-toast glyphs so the upper screen is reserved for
        // the authored VC2 announcement layer only.
        vc_visual_restore_matching_colors(
            &mut self.inner.simulation_framebuffer,
            &self.inner.background_framebuffer,
            gotoo_pixel_engine::Rect {
                x: 92,
                y: 28,
                width: 88,
                height: 43,
            },
            &[POWER_RELIC_LIGHT, ART_GOLD, XP_ORB_CORE],
        );

        // Explicitly kill legacy textual HUD in the lower-left presentation
        // zone (LV, old status/meta copy, etc.). The bars drawn afterwards are
        // label-free and therefore cannot inherit these remnants.
        vc_visual_restore_matching_colors(
            &mut self.inner.simulation_framebuffer,
            &self.inner.background_framebuffer,
            gotoo_pixel_engine::Rect {
                x: 0,
                y: 276,
                width: 92,
                height: 30,
            },
            &[
                XP_ORB_CORE,
                TEXT,
                WRECK_LIGHT,
                POWER_RELIC_LIGHT,
                ART_GOLD,
                ACCENT,
            ],
        );
    }

    fn render_combat_presentation(&mut self, framebuffer: &mut Framebuffer) {
        if !self.inner.game.game.active_combat() {
            return;
        }

        // Upper screen: announcements only. All persistent gauges live below.
        self.inner.render_event_announcement(framebuffer);
        self.inner.render_player_foreground(framebuffer);

        self.render_bottom_boss_status(framebuffer);
        self.render_bottom_survival_bars(framebuffer);
        self.render_textless_core(framebuffer);
        self.inner.render_compact_xp(framebuffer);
    }

    fn render_bottom_survival_bars(&self, framebuffer: &mut Framebuffer) {
        let scale = VC_VISUAL_PRESENTATION_SCALE;
        let model = self.inner.game.combat_model();
        let hull_cap = self.inner.game.hull_cap();
        let shield_cap = self.inner.game.shield_cap().max(1.0);

        let margin = 10 * scale;
        let gap = 14 * scale;
        let bar_width = (VC_VISUAL_PRESENTATION_WIDTH - margin * 2 - gap) / 2;
        let bar_height = 2 * scale;
        let y = VC_VISUAL_PRESENTATION_HEIGHT.saturating_sub(30 * scale);
        let hull_x = margin;
        let shield_x = margin + bar_width + gap;

        vc24_thin_transparent_bar(
            framebuffer,
            hull_x as i32,
            y as i32,
            bar_width,
            bar_height,
            model.player_hull,
            hull_cap,
            if model.player_hull_flash_timer > 0.0 {
                DANGER
            } else {
                VC20_HULL
            },
        );
        vc24_thin_transparent_bar(
            framebuffer,
            shield_x as i32,
            y as i32,
            bar_width,
            bar_height,
            model.player_shield,
            shield_cap,
            if model.player_shield_flash_timer > 0.0 {
                VC20_ARMOR_LIGHT
            } else {
                VC20_ARMOR
            },
        );

        // Sustain is intentionally not represented by a second progress line.
        // A very brief outline flash is enough feedback without creating the
        // ugly persistent yellow/cyan underline from VC2.3.
        if self.inner.game.sustain_flash_timer > 0.0 {
            let (x, color) = match self.inner.game.augment {
                Some(Vc23SustainAugment::NaniteRepair) => (hull_x, CANTICLE_COLOR),
                Some(Vc23SustainAugment::ShieldCapacitor) => (shield_x, ART_CYAN_LIGHT),
                None => return,
            };
            framebuffer.draw_rect(
                x as i32 - 1,
                y as i32 - 1,
                bar_width + 2,
                bar_height + 2,
                color,
            );
        }
    }

    fn render_bottom_boss_status(&self, framebuffer: &mut Framebuffer) {
        let base = self.inner.game.game.base();
        if base.encounter_phase != EncounterPhase::BossFight {
            return;
        }
        let Some(boss) = base.boss else {
            return;
        };

        let scale = VC_VISUAL_PRESENTATION_SCALE;
        let width = 82 * scale;
        let height = scale;
        let x = (VC_VISUAL_PRESENTATION_WIDTH - width) / 2;
        let y = VC_VISUAL_PRESENTATION_HEIGHT.saturating_sub(38 * scale);
        let ratio = boss.hp as f32 / BELLKEEPER_MAX_HP as f32;
        let filled = (width as f32 * ratio).round() as u32;

        framebuffer.draw_rect(
            x as i32 - 1,
            y as i32 - 1,
            width + 2,
            height + 2,
            BELL_DARK,
        );
        if filled > 0 {
            framebuffer.fill_rect(x as i32, y as i32, filled.min(width), height, DANGER);
        }
    }

    fn render_textless_core(&self, framebuffer: &mut Framebuffer) {
        let scale = VC_VISUAL_PRESENTATION_SCALE;
        let base = self.inner.game.game.base();
        let x = 10 * scale;
        let y = VC_VISUAL_PRESENTATION_HEIGHT.saturating_sub(14 * scale);
        let width = 62 * scale;
        let height = 3 * scale;
        let ratio = base.core_charge.min(CORE_MAX) as f32 / CORE_MAX as f32;
        let ready = base.core_charge >= CORE_MAX;
        let color = if ready { CANTICLE_COLOR } else { CINDER };

        framebuffer.draw_rect(
            x as i32 - 1,
            y as i32 - 1,
            width + 2,
            height + 2,
            WRECK_MID,
        );
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

        // Ready state is conveyed by color only: no READY/LV/status text in
        // the lower-left corner.
        if ready {
            framebuffer.draw_rect(
                x as i32 - 2,
                y as i32 - 2,
                width + 4,
                height + 4,
                CANTICLE_COLOR,
            );
        }
    }
}

impl Game for VoidCanticleV24HudPolish {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let result = {
            let mut simulation_frame = Frame {
                framebuffer: &mut self.inner.simulation_framebuffer,
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
            self.inner.game.update(&mut simulation_frame)
        };
        if result == GameResult::Exit {
            return result;
        }

        let mode = self.inner.visual_mode();
        self.inner.prepare_background_frame();
        self.inner.clean_simulation_legacy_hud(mode);
        self.clean_residual_legacy_hud(mode);

        vc_visual_blit_nearest(
            &self.inner.simulation_framebuffer,
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

fn vc24_thin_transparent_bar(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    value: f32,
    max_value: f32,
    fill: Pixel,
) {
    let ratio = (value / max_value.max(1.0)).clamp(0.0, 1.0);
    let filled = (width as f32 * ratio).round() as u32;

    // Deliberately no opaque background fill: unfilled gauge space is the
    // live Grave Orbit scene.
    framebuffer.draw_rect(x - 1, y - 1, width + 2, height + 2, WRECK_MID);
    if filled > 0 {
        framebuffer.fill_rect(x, y, filled.min(width), height, fill);
    }
}

pub fn run_hud_polish_with_obs_mirror() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: format!(
                "Void Canticle {VC24_PRESENTATION_VERSION} HUD Layout - Gotoo Pixel Engine"
            ),
            framebuffer_width: VC_VISUAL_PRESENTATION_WIDTH,
            framebuffer_height: VC_VISUAL_PRESENTATION_HEIGHT,
            window_width,
            window_height,
        },
        gotoo_pixel_engine::ObsMirrorGame::from_env(
            VoidCanticleV24HudPolish::new(),
            VC_VISUAL_PRESENTATION_WIDTH,
            VC_VISUAL_PRESENTATION_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v24_tests {
    use super::*;

    #[test]
    fn presentation_version_is_unambiguous() {
        assert_eq!(VC24_PRESENTATION_VERSION, "VC2.4");
    }

    #[test]
    fn wave_toast_gold_ghost_is_restored_from_background() {
        let mut game = VoidCanticleV24HudPolish::new();
        game.inner.background_framebuffer.clear(Pixel::BLUE);
        game.inner.simulation_framebuffer.clear(Pixel::RED);
        game.inner
            .simulation_framebuffer
            .draw(110, 40, POWER_RELIC_LIGHT);

        game.clean_residual_legacy_hud(VcVisualMode::Combat);

        assert_eq!(
            game.inner.simulation_framebuffer.pixel(110, 40),
            Some(Pixel::BLUE)
        );
    }

    #[test]
    fn legacy_level_label_colors_are_removed_from_bottom_left() {
        let mut game = VoidCanticleV24HudPolish::new();
        game.inner.background_framebuffer.clear(Pixel::BLUE);
        game.inner.simulation_framebuffer.clear(Pixel::RED);
        game.inner.simulation_framebuffer.draw(8, 292, XP_ORB_CORE);
        game.inner.simulation_framebuffer.draw(12, 292, TEXT);

        game.clean_residual_legacy_hud(VcVisualMode::Combat);

        assert_eq!(
            game.inner.simulation_framebuffer.pixel(8, 292),
            Some(Pixel::BLUE)
        );
        assert_eq!(
            game.inner.simulation_framebuffer.pixel(12, 292),
            Some(Pixel::BLUE)
        );
    }

    #[test]
    fn thin_bar_keeps_unfilled_area_transparent_to_scene() {
        let mut framebuffer = Framebuffer::new(20, 8);
        framebuffer.clear(Pixel::BLUE);

        vc24_thin_transparent_bar(
            &mut framebuffer,
            2,
            3,
            12,
            2,
            3.0,
            12.0,
            Pixel::RED,
        );

        assert_eq!(framebuffer.pixel(4, 3), Some(Pixel::RED));
        assert_eq!(framebuffer.pixel(12, 3), Some(Pixel::BLUE));
    }
}
