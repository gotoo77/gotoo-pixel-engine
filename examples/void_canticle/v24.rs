const VC24_PRESENTATION_VERSION: &str = "VC2.4-R2";

struct VoidCanticleV24HudPolish {
    inner: VoidCanticleVisualFoundation,
    combat_snapshot: Framebuffer,
    has_combat_snapshot: bool,
}

impl VoidCanticleV24HudPolish {
    fn new() -> Self {
        Self {
            inner: VoidCanticleVisualFoundation::new(),
            combat_snapshot: Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT),
            has_combat_snapshot: false,
        }
    }

    fn clean_residual_legacy_hud(&mut self, mode: VcVisualMode) {
        if mode != VcVisualMode::Combat {
            return;
        }

        // The upper presentation area belongs to authored VC2 announcements,
        // not to old wave/status HUD fragments.
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

        // Remove legacy lower-left copy (LV/status) before the VC2 presentation
        // is upscaled. Persistent combat state is rendered by VC2.4-R2 only.
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

    fn remember_clean_combat_frame(&mut self) {
        vc24_copy_framebuffer(
            &self.inner.simulation_framebuffer,
            &mut self.combat_snapshot,
        );
        self.has_combat_snapshot = true;
    }

    fn restore_modal_base(&mut self) {
        if self.has_combat_snapshot {
            vc24_copy_framebuffer(
                &self.combat_snapshot,
                &mut self.inner.simulation_framebuffer,
            );
        } else {
            vc24_copy_framebuffer(
                &self.inner.background_framebuffer,
                &mut self.inner.simulation_framebuffer,
            );
        }
    }

    fn redraw_modal_last(&mut self, mode: VcVisualMode) {
        match mode {
            VcVisualMode::SupportChoice => {
                let game = &self.inner.game;
                let framebuffer = &mut self.inner.simulation_framebuffer;
                game.render_support_choice(framebuffer);
            }
            VcVisualMode::LevelChoice => {
                let v14 = self.inner.game.game.v20().game.v14();
                let framebuffer = &mut self.inner.simulation_framebuffer;
                if let Some(choice) = &v14.progression.level_choice {
                    v14.progression.render_level_choice(framebuffer, choice);
                }
            }
            VcVisualMode::MutationChoice => {
                let v14 = self.inner.game.game.v20().game.v14();
                let framebuffer = &mut self.inner.simulation_framebuffer;
                if let Some(choice) = &v14.mutation_choice {
                    v14.render_mutation_choice(framebuffer, choice);
                }
            }
            VcVisualMode::Pause => {
                let pause = self.inner.game.survival_model().game.pause_ui();
                let framebuffer = &mut self.inner.simulation_framebuffer;
                match &pause.state {
                    VcPauseState::Controls => pause.render_controls(framebuffer),
                    VcPauseState::BuildInfo => pause.render_build_info(framebuffer),
                    _ => pause.render_menu(framebuffer),
                }
            }
            VcVisualMode::StageClear => {
                let stabilized = &self.inner.game.survival_model().game;
                let framebuffer = &mut self.inner.simulation_framebuffer;
                stabilized.render_stage_clear(framebuffer);
            }
            VcVisualMode::Combat | VcVisualMode::Death => {}
        }
    }

    fn render_combat_presentation(&mut self, framebuffer: &mut Framebuffer) {
        if !self.inner.game.game.active_combat() {
            return;
        }

        // Upper screen: announcements only. No persistent Hull/Shield/Core/XP
        // furniture is allowed here.
        self.inner.render_event_announcement(framebuffer);
        self.inner.render_player_foreground(framebuffer);

        // Persistent combat HUD is intentionally reduced to exactly two thin
        // gauges at the bottom edge.
        self.render_bottom_survival_bars(framebuffer);
    }

    fn render_bottom_survival_bars(&self, framebuffer: &mut Framebuffer) {
        let scale = VC_VISUAL_PRESENTATION_SCALE;
        let model = self.inner.game.combat_model();
        let hull_cap = self.inner.game.hull_cap();
        let shield_cap = self.inner.game.shield_cap().max(1.0);

        let margin = 8 * scale;
        let bar_width = 72 * scale;
        let bar_height = scale;
        let y = VC_VISUAL_PRESENTATION_HEIGHT.saturating_sub(4 * scale);
        let hull_x = margin;
        let shield_x = VC_VISUAL_PRESENTATION_WIDTH
            .saturating_sub(margin)
            .saturating_sub(bar_width);

        let hull_fill = if model.player_hull_flash_timer > 0.0 {
            DANGER
        } else if self.inner.game.sustain_flash_timer > 0.0
            && self.inner.game.augment == Some(Vc23SustainAugment::NaniteRepair)
        {
            CANTICLE_COLOR
        } else {
            VC20_HULL
        };
        let shield_fill = if model.player_shield_flash_timer > 0.0 {
            VC20_ARMOR_LIGHT
        } else if self.inner.game.sustain_flash_timer > 0.0
            && self.inner.game.augment == Some(Vc23SustainAugment::ShieldCapacitor)
        {
            ART_CYAN_LIGHT
        } else {
            VC20_ARMOR
        };

        vc24_thin_bar(
            framebuffer,
            hull_x as i32,
            y as i32,
            bar_width,
            bar_height,
            model.player_hull,
            hull_cap,
            hull_fill,
        );
        vc24_thin_bar(
            framebuffer,
            shield_x as i32,
            y as i32,
            bar_width,
            bar_height,
            model.player_shield,
            shield_cap,
            shield_fill,
        );
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

        match mode {
            VcVisualMode::Combat => {
                self.inner.clean_simulation_legacy_hud(mode);
                self.clean_residual_legacy_hud(mode);
                self.remember_clean_combat_frame();
            }
            VcVisualMode::Pause
            | VcVisualMode::LevelChoice
            | VcVisualMode::MutationChoice
            | VcVisualMode::SupportChoice => {
                // The historical wrappers keep rendering gameplay/FX after
                // their modal panel. Restore a clean frozen combat frame and
                // redraw the modal last so bullets/orbs can never cover it.
                self.restore_modal_base();
                self.redraw_modal_last(mode);
            }
            VcVisualMode::StageClear => {
                self.redraw_modal_last(mode);
            }
            VcVisualMode::Death => {
                self.inner.clean_simulation_legacy_hud(mode);
            }
        }

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

fn vc24_copy_framebuffer(source: &Framebuffer, destination: &mut Framebuffer) {
    for y in 0..source.height() {
        for x in 0..source.width() {
            if let Some(pixel) = source.pixel(x as i32, y as i32) {
                destination.set_pixel_in_bounds(x, y, pixel);
            }
        }
    }
}

fn vc24_thin_bar(
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

    // One dim baseline + one coloured fill. No frame, no opaque backing, no
    // secondary sustain line: at presentation scale 2 this is only 2 px high.
    framebuffer.fill_rect(x, y, width, height, WRECK_MID);
    if filled > 0 {
        framebuffer.fill_rect(x, y, filled.min(width), height, fill);
    }
}

pub fn run_hud_polish_with_obs_mirror() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: format!(
                "Void Canticle {VC24_PRESENTATION_VERSION} HUD R2 - Gotoo Pixel Engine"
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
        assert_eq!(VC24_PRESENTATION_VERSION, "VC2.4-R2");
    }

    #[test]
    fn framebuffer_copy_preserves_snapshot_pixels() {
        let mut source = Framebuffer::new(3, 2);
        source.clear(Pixel::BLUE);
        source.draw(1, 1, Pixel::RED);
        let mut destination = Framebuffer::new(3, 2);
        destination.clear(Pixel::GREEN);

        vc24_copy_framebuffer(&source, &mut destination);

        assert_eq!(destination.pixel(0, 0), Some(Pixel::BLUE));
        assert_eq!(destination.pixel(1, 1), Some(Pixel::RED));
    }

    #[test]
    fn thin_bar_stays_one_row_high_and_has_no_frame() {
        let mut framebuffer = Framebuffer::new(20, 8);
        framebuffer.clear(Pixel::BLUE);

        vc24_thin_bar(
            &mut framebuffer,
            2,
            3,
            12,
            1,
            3.0,
            12.0,
            Pixel::RED,
        );

        assert_eq!(framebuffer.pixel(2, 3), Some(Pixel::RED));
        assert_eq!(framebuffer.pixel(12, 3), Some(WRECK_MID));
        assert_eq!(framebuffer.pixel(2, 2), Some(Pixel::BLUE));
        assert_eq!(framebuffer.pixel(2, 4), Some(Pixel::BLUE));
    }
}
