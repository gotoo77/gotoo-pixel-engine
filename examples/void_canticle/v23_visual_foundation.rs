#[cfg(not(target_arch = "wasm32"))]
const VC_VISUAL_PRESENTATION_SCALE: u32 = 2;
#[cfg(target_arch = "wasm32")]
const VC_VISUAL_PRESENTATION_SCALE: u32 = 1;

const VC_VISUAL_PRESENTATION_WIDTH: u32 = FRAMEBUFFER_WIDTH * VC_VISUAL_PRESENTATION_SCALE;
const VC_VISUAL_PRESENTATION_HEIGHT: u32 = FRAMEBUFFER_HEIGHT * VC_VISUAL_PRESENTATION_SCALE;

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

    fn render_clean_combat_presentation(&mut self, framebuffer: &mut Framebuffer) {
        if !self.game.game.active_combat() || self.game.choosing_support {
            return;
        }

        self.render_top_survival_console(framebuffer);
        self.remove_obsolete_bottom_text(framebuffer);
        self.render_core_state(framebuffer);
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

        // Tiny iconography instead of permanent labels.
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

    fn remove_obsolete_bottom_text(&self, framebuffer: &mut Framebuffer) {
        // Keep the useful graphical Core and XP gauges from the legacy stack,
        // but erase text-only metadata and permanent control hints.
        for (x, y, width, height) in [
            (0, 267, 108, 11), // CINDERS > CORE
            (108, 267, 72, 11), // old READY label
            (0, 277, 34, 11),   // CORE label
            (0, 288, 44, 11),   // LV n
            (0, 292, 74, 11),   // SPACE FIRE
            (74, 292, 106, 11), // SHIFT FOCUS
            (0, 303, 84, 12),   // X CANTICLE
            (100, 303, 80, 12), // ECHOES n
        ] {
            vc_visual_mask_logical_rect(framebuffer, x, y, width, height);
        }
    }

    fn render_core_state(&self, framebuffer: &mut Framebuffer) {
        let scale = VC_VISUAL_PRESENTATION_SCALE;
        let base = self.game.game.base();
        let color = if base.core_charge >= CORE_MAX {
            CANTICLE_COLOR
        } else {
            WRECK_MID
        };

        // The old horizontal Core meter is still useful information; strip its
        // label and make the meter itself carry the READY state through color.
        framebuffer.draw_rect(
            (34 * scale) as i32 - 1,
            (280 * scale) as i32 - 1,
            138 * scale + 2,
            6 * scale + 2,
            color,
        );
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
        self.render_clean_combat_presentation(frame.framebuffer);
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
    fn transparent_overlay_does_not_erase_destination() {
        let source = Framebuffer::new(1, 1);
        let mut destination = Framebuffer::new(2, 2);
        destination.clear(Pixel::GREEN);

        vc_visual_blit_nearest(&source, &mut destination, 2, true);

        assert!(destination.as_rgba8().chunks_exact(4).all(|rgba| rgba == Pixel::GREEN.to_rgba8()));
    }
}