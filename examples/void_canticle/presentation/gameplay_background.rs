const GAMEPLAY_BACKGROUND_PNG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/assets/backgrounds/void_abyss.png"
));

fn load_authored_gameplay_background() -> Framebuffer {
    let image = gotoo_pixel_engine::Image::decode_png(GAMEPLAY_BACKGROUND_PNG)
        .expect("checked-in Void Canticle gameplay background should decode");
    let source_width = image.width();
    let source_height = image.height();
    assert!(
        source_width >= 2 && source_height >= 2,
        "Void Canticle gameplay background must have at least two pixels per axis"
    );

    let rgba = image.as_rgba8();
    let mut framebuffer = Framebuffer::new(FRONT_WIDTH, FRONT_HEIGHT);

    let source_max_x = (source_width - 1) as f32;
    let source_max_y = (source_height - 1) as f32;
    let destination_max_x = (FRONT_WIDTH - 1) as f32;
    let destination_max_y = (FRONT_HEIGHT - 1) as f32;

    for y in 0..FRONT_HEIGHT {
        let source_y = y as f32 * source_max_y / destination_max_y;
        let y0 = source_y.floor() as u32;
        let y1 = (y0 + 1).min(source_height - 1);
        let ty = source_y - y0 as f32;

        for x in 0..FRONT_WIDTH {
            let source_x = x as f32 * source_max_x / destination_max_x;
            let x0 = source_x.floor() as u32;
            let x1 = (x0 + 1).min(source_width - 1);
            let tx = source_x - x0 as f32;

            let channel = |channel: usize| -> u8 {
                let sample = |sx: u32, sy: u32| -> f32 {
                    rgba[((sy * source_width + sx) * 4) as usize + channel] as f32
                };
                let top = sample(x0, y0) + (sample(x1, y0) - sample(x0, y0)) * tx;
                let bottom = sample(x0, y1) + (sample(x1, y1) - sample(x0, y1)) * tx;
                (top + (bottom - top) * ty).round().clamp(0.0, 255.0) as u8
            };

            framebuffer.set_pixel_in_bounds(
                x,
                y,
                Pixel::rgba(channel(0), channel(1), channel(2), channel(3)),
            );
        }
    }

    framebuffer
}

fn composite_over(background: Pixel, foreground: Pixel) -> Pixel {
    if foreground.a == 0 {
        return background;
    }
    if foreground.a == 255 {
        return foreground;
    }

    let source_alpha = u32::from(foreground.a);
    let destination_alpha = u32::from(background.a);
    let inverse_source = 255 - source_alpha;
    let output_alpha = source_alpha + (destination_alpha * inverse_source + 127) / 255;
    if output_alpha == 0 {
        return Pixel::TRANSPARENT;
    }

    let blend_channel = |source: u8, destination: u8| -> u8 {
        let source_premultiplied = u32::from(source) * source_alpha;
        let destination_premultiplied =
            (u32::from(destination) * destination_alpha * inverse_source + 127) / 255;
        ((source_premultiplied + destination_premultiplied + output_alpha / 2) / output_alpha)
            .min(255) as u8
    };

    Pixel::rgba(
        blend_channel(foreground.r, background.r),
        blend_channel(foreground.g, background.g),
        blend_channel(foreground.b, background.b),
        output_alpha.min(255) as u8,
    )
}

#[cfg(test)]
mod gameplay_background_tests {
    use super::*;

    #[test]
    fn checked_in_background_uses_compact_authored_source_size() {
        let image = gotoo_pixel_engine::Image::decode_png(GAMEPLAY_BACKGROUND_PNG).unwrap();
        assert_eq!((image.width(), image.height()), (180, 320));
    }

    #[test]
    fn authored_background_is_scaled_to_hd_front_size() {
        let framebuffer = load_authored_gameplay_background();
        assert_eq!((framebuffer.width(), framebuffer.height()), (FRONT_WIDTH, FRONT_HEIGHT));
        for (x, y) in [
            (0, 0),
            (FRONT_WIDTH as i32 / 2, FRONT_HEIGHT as i32 / 2),
            (FRONT_WIDTH as i32 - 1, FRONT_HEIGHT as i32 - 1),
        ] {
            assert_eq!(framebuffer.pixel(x, y).unwrap().a, 255);
        }
    }

    #[test]
    fn transparent_foreground_preserves_background() {
        let background = Pixel::rgb(4, 8, 16);
        assert_eq!(composite_over(background, Pixel::TRANSPARENT), background);
    }

    #[test]
    fn opaque_foreground_replaces_background() {
        let foreground = Pixel::rgb(240, 48, 200);
        assert_eq!(
            composite_over(Pixel::rgb(4, 8, 16), foreground),
            foreground
        );
    }
}
