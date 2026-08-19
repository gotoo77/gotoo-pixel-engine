const VC25_PRESENTATION_VERSION: &str = "VC2.5";

struct VoidCanticleV25HudReset {
    inner: VoidCanticleV24HudPolish,
    presentation_background: Framebuffer,
}

impl VoidCanticleV25HudReset {
    fn new() -> Self {
        Self {
            inner: VoidCanticleV24HudPolish::new(),
            presentation_background: Framebuffer::new(
                VC_VISUAL_PRESENTATION_WIDTH,
                VC_VISUAL_PRESENTATION_HEIGHT,
            ),
        }
    }

    fn rebuild_presentation_background(&mut self) {
        let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
        for y in 0..VC_VISUAL_PRESENTATION_HEIGHT {
            for x in 0..VC_VISUAL_PRESENTATION_WIDTH {
                let source_x = (x / scale) as i32;
                let source_y = (y / scale) as i32;
                if let Some(pixel) = self
                    .inner
                    .inner
                    .background_framebuffer
                    .pixel(source_x, source_y)
                {
                    self.presentation_background.set_pixel_in_bounds(x, y, pixel);
                }
            }
        }
    }

    fn strip_legacy_ui_from_final_frame(&self, framebuffer: &mut Framebuffer) {
        let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
        let legacy_ui_colors = [
            BG,
            TEXT,
            ACCENT,
            WRECK_LIGHT,
            WRECK_MID,
            PILGRIM_CORE,
            CANTICLE_COLOR,
            CORE_BG,
            VC20_HULL,
            VC20_ARMOR,
            VC20_ARMOR_LIGHT,
            VC20_ARMOR_BG,
            DANGER,
            CINDER,
            POWER_RELIC,
            POWER_RELIC_LIGHT,
            XP_ORB_CORE,
            XP_BAR_BG,
            XP_BAR_FILL,
            ART_CYAN,
            ART_CYAN_LIGHT,
            ART_GOLD,
            VOID_DIM,
            VOID_GLOW,
            VOID_DANGER,
            VOID_CATA,
            VOID_LIGHT,
            BELL_LIGHT,
            Pixel::rgb(9, 8, 15),
            Pixel::rgb(6, 8, 17),
        ];

        // Persistent legacy meta + Hull/Shield/sustain area. R2 only cleaned
        // x=0..108 in simulation space even though Shield reaches x~176.
        vc25_restore_matching_pixels(
            framebuffer,
            &self.presentation_background,
            0,
            0,
            VC_VISUAL_PRESENTATION_WIDTH,
            41 * scale,
            &legacy_ui_colors,
        );

        // Historical transient panels are removed here as well. VC2.5 redraws
        // only the authored announcement layer afterwards.
        for (x, y, width, height) in [
            (102, 34, 74, 31),
            (105, 76, 71, 31),
            (4, 90, 92, 45),
        ] {
            vc25_restore_matching_pixels(
                framebuffer,
                &self.presentation_background,
                x * scale,
                y * scale,
                width * scale,
                height * scale,
                &legacy_ui_colors,
            );
        }

        // Kill the whole family of historical lower HUD fragments: LV, CORE,
        // power pips, XP, VOID status and the previous VC2.4 bars. The current
        // Hull/Shield pair is redrawn after this pass.
        vc25_restore_matching_pixels(
            framebuffer,
            &self.presentation_background,
            0,
            286 * scale,
            VC_VISUAL_PRESENTATION_WIDTH,
            34 * scale,
            &legacy_ui_colors,
        );
    }

    fn redraw_v25_presentation(&mut self, framebuffer: &mut Framebuffer) {
        // The top of the screen is announcement-only. No permanent gauge lives
        // there anymore.
        self.inner.inner.render_event_announcement(framebuffer);

        // If the player enters the lower presentation reserve, keep the ship
        // visible above the cleaned background before drawing the gauges.
        self.inner.inner.render_player_foreground(framebuffer);

        // Exactly two permanent gauges, one physical row in simulation scale
        // (2 px on the native 2x presentation): Hull left, Shield right.
        self.inner.render_bottom_survival_bars(framebuffer);
    }
}

impl Game for VoidCanticleV25HudReset {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let result = self.inner.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        if self.inner.inner.visual_mode() != VcVisualMode::Combat {
            return result;
        }

        self.rebuild_presentation_background();
        self.strip_legacy_ui_from_final_frame(frame.framebuffer);
        self.redraw_v25_presentation(frame.framebuffer);
        GameResult::Continue
    }
}

fn vc25_restore_matching_pixels(
    framebuffer: &mut Framebuffer,
    background: &Framebuffer,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    colors: &[Pixel],
) {
    let max_x = x.saturating_add(width).min(framebuffer.width());
    let max_y = y.saturating_add(height).min(framebuffer.height());

    for py in y.min(framebuffer.height())..max_y {
        for px in x.min(framebuffer.width())..max_x {
            let Some(pixel) = framebuffer.pixel(px as i32, py as i32) else {
                continue;
            };
            if !colors.contains(&pixel) {
                continue;
            }
            if let Some(replacement) = background.pixel(px as i32, py as i32) {
                framebuffer.set_pixel_in_bounds(px, py, replacement);
            }
        }
    }
}

pub fn run_v25_hud_reset_with_obs_mirror() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: format!(
                "Void Canticle {VC25_PRESENTATION_VERSION} HUD Reset - Gotoo Pixel Engine"
            ),
            framebuffer_width: VC_VISUAL_PRESENTATION_WIDTH,
            framebuffer_height: VC_VISUAL_PRESENTATION_HEIGHT,
            window_width,
            window_height,
        },
        gotoo_pixel_engine::ObsMirrorGame::from_env(
            VoidCanticleV25HudReset::new(),
            VC_VISUAL_PRESENTATION_WIDTH,
            VC_VISUAL_PRESENTATION_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v25_tests {
    use super::*;

    #[test]
    fn presentation_version_is_unambiguous() {
        assert_eq!(VC25_PRESENTATION_VERSION, "VC2.5");
    }

    #[test]
    fn final_cleanup_restores_matching_legacy_pixel_only() {
        let mut framebuffer = Framebuffer::new(8, 4);
        let mut background = Framebuffer::new(8, 4);
        framebuffer.clear(Pixel::RED);
        background.clear(Pixel::BLUE);
        framebuffer.draw(2, 1, ART_CYAN_LIGHT);

        vc25_restore_matching_pixels(
            &mut framebuffer,
            &background,
            0,
            0,
            8,
            4,
            &[ART_CYAN_LIGHT],
        );

        assert_eq!(framebuffer.pixel(2, 1), Some(Pixel::BLUE));
        assert_eq!(framebuffer.pixel(3, 1), Some(Pixel::RED));
    }
}
