const VC26_PRESENTATION_VERSION: &str = "VC2.6";

struct VoidCanticleV26HudOwnership {
    inner: VoidCanticleV25HudReset,
    presentation_background: Framebuffer,
}

impl VoidCanticleV26HudOwnership {
    fn new() -> Self {
        Self {
            inner: VoidCanticleV25HudReset::new(),
            presentation_background: Framebuffer::new(
                VC_VISUAL_PRESENTATION_WIDTH,
                VC_VISUAL_PRESENTATION_HEIGHT,
            ),
        }
    }

    fn rebuild_presentation_background(&mut self) {
        let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
        let background = &self.inner.inner.inner.background_framebuffer;
        for y in 0..VC_VISUAL_PRESENTATION_HEIGHT {
            for x in 0..VC_VISUAL_PRESENTATION_WIDTH {
                if let Some(pixel) = background.pixel((x / scale) as i32, (y / scale) as i32) {
                    self.presentation_background.set_pixel_in_bounds(x, y, pixel);
                }
            }
        }
    }

    fn restore_rect_from_background(
        &self,
        framebuffer: &mut Framebuffer,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) {
        let max_x = x.saturating_add(width).min(framebuffer.width());
        let max_y = y.saturating_add(height).min(framebuffer.height());
        for py in y.min(framebuffer.height())..max_y {
            for px in x.min(framebuffer.width())..max_x {
                if let Some(pixel) = self.presentation_background.pixel(px as i32, py as i32) {
                    framebuffer.set_pixel_in_bounds(px, py, pixel);
                }
            }
        }
    }

    fn remove_historical_hud_regions(&self, framebuffer: &mut Framebuffer) {
        let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);

        // Upper screen belongs to VC2 announcements only. This is a hard
        // geometry reset, not a colour heuristic: legacy lives/score/boss,
        // Hull/Shield and sustain cannot leak through anymore.
        self.restore_rect_from_background(
            framebuffer,
            0,
            0,
            VC_VISUAL_PRESENTATION_WIDTH,
            41 * scale,
        );

        // Old notification panels are also removed geometrically; the current
        // authored announcement layer is redrawn afterwards.
        for (x, y, width, height) in [
            (102, 34, 74, 31),
            (105, 76, 71, 31),
            (4, 90, 92, 45),
        ] {
            self.restore_rect_from_background(
                framebuffer,
                x * scale,
                y * scale,
                width * scale,
                height * scale,
            );
        }

        // Historical lower-left LV text and compact VOID status.
        self.restore_rect_from_background(framebuffer, 0, 286 * scale, 72 * scale, 16 * scale);
        self.restore_rect_from_background(framebuffer, 110 * scale, 286 * scale, 70 * scale, 16 * scale);

        // Historical CORE/power strip and XP strip. The Canticle gauge is
        // reconstructed below at its familiar lower-left placement.
        self.restore_rect_from_background(
            framebuffer,
            0,
            300 * scale,
            VC_VISUAL_PRESENTATION_WIDTH,
            12 * scale,
        );
        self.restore_rect_from_background(
            framebuffer,
            0,
            314 * scale,
            VC_VISUAL_PRESENTATION_WIDTH,
            6 * scale,
        );
    }

    fn render_canticle_charge(&self, framebuffer: &mut Framebuffer) {
        let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
        let base = self.inner.inner.inner.game.game.base();

        // Preserve the visual position of the VC2.3 Canticle/Core charge:
        // compact, segmented, lower-left, with no CORE/READY text.
        let x = 10 * scale;
        let y = 306 * scale;
        let width = 62 * scale;
        let height = 3 * scale;
        let ratio = base.core_charge.min(CORE_MAX) as f32 / CORE_MAX as f32;
        let filled = (width as f32 * ratio).round() as u32;
        let color = if base.core_charge >= CORE_MAX {
            CANTICLE_COLOR
        } else {
            CINDER
        };

        // Transparent empty space: scene remains visible behind the gauge.
        framebuffer.draw_rect(
            x as i32 - 1,
            y as i32 - 1,
            width + 2,
            height + 2,
            WRECK_MID,
        );
        if filled > 0 {
            framebuffer.fill_rect(x as i32, y as i32, filled.min(width), height, color);
        }

        for segment in 1..5 {
            let sx = x + width * segment / 5;
            framebuffer.draw_line(
                sx as i32,
                y as i32,
                sx as i32,
                (y + height - 1) as i32,
                WRECK_MID,
            );
        }
    }

    fn redraw_owned_hud(&mut self, framebuffer: &mut Framebuffer) {
        // Upper screen: text announcements only.
        self.inner.inner.inner.render_event_announcement(framebuffer);

        // Bottom: Canticle charge plus thin Hull/Shield.
        self.render_canticle_charge(framebuffer);
        self.inner.inner.render_bottom_survival_bars(framebuffer);

        // Critical z-order rule: player is last, so gauges never slice through
        // the ship when it enters the lower screen area.
        self.inner.inner.inner.render_player_foreground(framebuffer);
    }
}

impl Game for VoidCanticleV26HudOwnership {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let result = self.inner.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        if self.inner.inner.inner.visual_mode() != VcVisualMode::Combat {
            return result;
        }

        self.rebuild_presentation_background();
        self.remove_historical_hud_regions(frame.framebuffer);
        self.redraw_owned_hud(frame.framebuffer);
        GameResult::Continue
    }
}

pub fn run_v26_hud_ownership_with_obs_mirror() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: format!(
                "Void Canticle {VC26_PRESENTATION_VERSION} HUD Ownership - Gotoo Pixel Engine"
            ),
            framebuffer_width: VC_VISUAL_PRESENTATION_WIDTH,
            framebuffer_height: VC_VISUAL_PRESENTATION_HEIGHT,
            window_width,
            window_height,
        },
        gotoo_pixel_engine::ObsMirrorGame::from_env(
            VoidCanticleV26HudOwnership::new(),
            VC_VISUAL_PRESENTATION_WIDTH,
            VC_VISUAL_PRESENTATION_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v26_tests {
    use super::*;

    #[test]
    fn version_is_explicit() {
        assert_eq!(VC26_PRESENTATION_VERSION, "VC2.6");
    }

    #[test]
    fn hard_restore_ignores_source_pixel_colour() {
        let mut game = VoidCanticleV26HudOwnership::new();
        game.presentation_background.clear(Pixel::BLUE);
        let mut framebuffer = Framebuffer::new(8, 8);
        framebuffer.clear(Pixel::RED);
        framebuffer.draw(3, 3, Pixel::GREEN);

        game.restore_rect_from_background(&mut framebuffer, 2, 2, 3, 3);

        assert_eq!(framebuffer.pixel(3, 3), Some(Pixel::BLUE));
        assert_eq!(framebuffer.pixel(1, 1), Some(Pixel::RED));
    }
}
