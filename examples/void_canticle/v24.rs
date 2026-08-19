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

        // VC1.x wave notifications used POWER_RELIC_LIGHT for the compact
        // "W x/15" label. The VC2.3 event cleanup already removed its panel
        // and WRECK_LIGHT text, leaving only these pale-gold glyph pixels behind.
        // Clean that exact historical toast area before presentation upscaling.
        vc_visual_restore_matching_colors(
            &mut self.inner.simulation_framebuffer,
            &self.inner.background_framebuffer,
            gotoo_pixel_engine::Rect {
                x: 100,
                y: 32,
                width: 78,
                height: 35,
            },
            &[POWER_RELIC_LIGHT],
        );

        // Keep the old progression label out of the new HUD even if a legacy
        // layer changes drawing order later. The compact XP line is the only
        // progression presentation that belongs in combat now.
        vc_visual_restore_matching_colors(
            &mut self.inner.simulation_framebuffer,
            &self.inner.background_framebuffer,
            gotoo_pixel_engine::Rect {
                x: 0,
                y: 286,
                width: 46,
                height: 18,
            },
            &[XP_ORB_CORE],
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
        self.inner.clean_simulation_legacy_hud(mode);
        self.clean_residual_legacy_hud(mode);

        vc_visual_blit_nearest(
            &self.inner.simulation_framebuffer,
            frame.framebuffer,
            VC_VISUAL_PRESENTATION_SCALE,
            false,
        );
        self.inner.render_presentation(frame.framebuffer, mode);
        GameResult::Continue
    }
}

pub fn run_hud_polish_with_obs_mirror() -> Result<(), EngineError> {
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
    fn wave_toast_gold_ghost_is_restored_from_background() {
        let mut game = VoidCanticleV24HudPolish::new();
        game.inner.background_framebuffer.clear(Pixel::BLUE);
        game.inner.simulation_framebuffer.clear(Pixel::RED);
        game.inner
            .simulation_framebuffer
            .draw(110, 40, POWER_RELIC_LIGHT);

        game.clean_residual_legacy_hud(VcVisualMode::Combat);

        assert_eq!(game.inner.simulation_framebuffer.pixel(110, 40), Some(Pixel::BLUE));
    }

    #[test]
    fn legacy_level_label_color_is_removed_from_bottom_left() {
        let mut game = VoidCanticleV24HudPolish::new();
        game.inner.background_framebuffer.clear(Pixel::BLUE);
        game.inner.simulation_framebuffer.clear(Pixel::RED);
        game.inner
            .simulation_framebuffer
            .draw(8, 292, XP_ORB_CORE);

        game.clean_residual_legacy_hud(VcVisualMode::Combat);

        assert_eq!(game.inner.simulation_framebuffer.pixel(8, 292), Some(Pixel::BLUE));
    }
}
