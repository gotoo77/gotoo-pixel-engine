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

fn vc_visual_announcement(
    framebuffer: &mut Framebuffer,
    headline: &str,
    detail: &str,
    accent: Pixel,
    text: Pixel,
) {
    let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
    let y = (11 * scale) as i32;

    // Announcements deliberately have no opaque panel. Gameplay remains
    // visible and projectiles are never hidden by presentation furniture.
    vc_visual_draw_centered_text(framebuffer, y, headline, scale, text);
    if !detail.is_empty() {
        vc_visual_draw_centered_text(
            framebuffer,
            y + 10 * scale as i32,
            detail,
            1,
            accent,
        );
    }

    let line_width = 48 * scale;
    let line_y = y + 19 * scale as i32;
    let line_x = ((VC_VISUAL_PRESENTATION_WIDTH - line_width) / 2) as i32;
    framebuffer.draw_line(
        line_x,
        line_y,
        line_x + line_width as i32 - 1,
        line_y,
        accent,
    );
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
    fn transparent_blit_preserves_destination() {
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
