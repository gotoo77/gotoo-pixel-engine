const ABYSSAL_BASE: Pixel = Pixel::rgb(2, 3, 9);

fn render_abyssal_void_background(framebuffer: &mut Framebuffer, time: f32) {
    debug_assert_eq!(framebuffer.width(), FRONT_WIDTH);
    debug_assert_eq!(framebuffer.height(), FRONT_HEIGHT);

    framebuffer.clear(ABYSSAL_BASE);
    render_abyssal_haze(framebuffer, time);
    render_dead_celestial_mass(framebuffer, time);
    render_star_layer(framebuffer, time, 0x41A7, 46, 4.5, 1, 31);
    render_star_layer(framebuffer, time, 0x8F31, 30, 10.0, 1, 54);
    render_star_layer(framebuffer, time, 0xD20B, 18, 18.0, 2, 82);
    render_singularity_event(framebuffer, time);
}

fn render_abyssal_haze(framebuffer: &mut Framebuffer, time: f32) {
    let drift = (time * 0.09).sin() * 18.0;
    let center_x = (-34.0 + drift).round() as i32;
    let center_y = 672 + ((time * 0.07).cos() * 12.0).round() as i32;

    for (radius, color) in [
        (300, Pixel::rgb(5, 4, 15)),
        (252, Pixel::rgb(7, 5, 19)),
        (204, Pixel::rgb(9, 6, 23)),
        (158, Pixel::rgb(10, 7, 27)),
    ] {
        framebuffer.draw_circle(center_x, center_y, radius, color);
        framebuffer.draw_circle(center_x + 1, center_y, radius.saturating_sub(1), color);
    }

    let upper_drift = (time * 0.06).cos() * 24.0;
    let upper_x = (FRONT_WIDTH as f32 * 0.72 + upper_drift).round() as i32;
    for (radius, color) in [
        (236, Pixel::rgb(4, 7, 17)),
        (188, Pixel::rgb(5, 9, 22)),
        (142, Pixel::rgb(7, 10, 26)),
    ] {
        framebuffer.draw_circle(upper_x, 156, radius, color);
    }
}

fn render_dead_celestial_mass(framebuffer: &mut Framebuffer, time: f32) {
    let x = FRONT_WIDTH as i32 + 118 + ((time * 0.035).sin() * 8.0).round() as i32;
    let y = 344 + ((time * 0.027).cos() * 6.0).round() as i32;
    let radius = 286;

    framebuffer.fill_circle(x, y, radius, Pixel::rgb(3, 4, 10));
    framebuffer.draw_circle(x - 4, y, radius, Pixel::rgb(17, 10, 29));
    framebuffer.draw_circle(x - 7, y, radius.saturating_sub(2), Pixel::rgb(8, 16, 31));
}

fn render_star_layer(
    framebuffer: &mut Framebuffer,
    time: f32,
    salt: u32,
    count: u32,
    speed: f32,
    size: u32,
    base_luma: u8,
) {
    let height = FRONT_HEIGHT as f32;
    for index in 0..count {
        let seed = background_hash(index.wrapping_add(salt));
        let x = seed % FRONT_WIDTH;
        let base_y = background_hash(seed ^ 0x9E37_79B9) % FRONT_HEIGHT;
        let y = (base_y as f32 + time.max(0.0) * speed).rem_euclid(height) as i32;
        let twinkle_seed = background_hash(seed ^ ((time.max(0.0) * 3.0) as u32));
        let twinkle = (twinkle_seed % 24) as u8;
        let luma = base_luma.saturating_add(twinkle);
        let color = if seed & 1 == 0 {
            Pixel::rgb(luma.saturating_sub(8), luma, luma.saturating_add(14))
        } else {
            Pixel::rgb(luma.saturating_add(10), luma.saturating_sub(5), luma.saturating_add(8))
        };
        framebuffer.fill_rect(x as i32, y, size, size, color);
    }
}

fn render_singularity_event(framebuffer: &mut Framebuffer, time: f32) {
    let visibility = singularity_visibility(time);
    if visibility <= 0.0 {
        return;
    }

    let center_x = (FRONT_WIDTH as f32 * 0.34 + (time * 0.11).sin() * 9.0).round() as i32;
    let center_y = 214 + ((time * 0.08).cos() * 8.0).round() as i32;
    let violet = scaled_rgb(72, 29, 102, visibility);
    let cyan = scaled_rgb(24, 73, 94, visibility * 0.82);
    let pale = scaled_rgb(105, 86, 124, visibility * 0.72);

    for offset in 0..4 {
        framebuffer.draw_circle(center_x, center_y, 94 + offset * 5, violet);
    }
    framebuffer.draw_circle(center_x, center_y, 118, cyan);
    framebuffer.draw_circle(center_x, center_y, 124, pale);

    let disk_half = (108.0 * visibility).round() as i32;
    for row in -2..=2 {
        let taper = 10 - row_i32_abs(row) * 2;
        framebuffer.draw_line(
            center_x - disk_half - taper,
            center_y + row,
            center_x + disk_half + taper,
            center_y + row,
            if row == 0 { pale } else { violet },
        );
    }

    framebuffer.fill_circle(center_x, center_y, 52, Pixel::rgb(0, 0, 2));
    framebuffer.draw_circle(center_x, center_y, 54, Pixel::rgb(14, 8, 23));
}

fn singularity_visibility(time: f32) -> f32 {
    let phase = time.max(0.0).rem_euclid(52.0);
    if !(16.0..=30.0).contains(&phase) {
        return 0.0;
    }
    if phase < 20.0 {
        return ((phase - 16.0) / 4.0).clamp(0.0, 1.0);
    }
    if phase > 26.0 {
        return ((30.0 - phase) / 4.0).clamp(0.0, 1.0);
    }
    1.0
}

fn scaled_rgb(red: u8, green: u8, blue: u8, intensity: f32) -> Pixel {
    let intensity = intensity.clamp(0.0, 1.0);
    Pixel::rgb(
        (red as f32 * intensity).round() as u8,
        (green as f32 * intensity).round() as u8,
        (blue as f32 * intensity).round() as u8,
    )
}

fn background_hash(mut value: u32) -> u32 {
    value ^= value >> 16;
    value = value.wrapping_mul(0x7FEB_352D);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846C_A68B);
    value ^ (value >> 16)
}

fn row_i32_abs(value: i32) -> i32 {
    value.abs()
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
    fn abyssal_background_is_full_size_and_opaque() {
        let mut framebuffer = Framebuffer::new(FRONT_WIDTH, FRONT_HEIGHT);
        render_abyssal_void_background(&mut framebuffer, 0.0);

        for (x, y) in [(0, 0), (FRONT_WIDTH as i32 / 2, FRONT_HEIGHT as i32 / 2), (FRONT_WIDTH as i32 - 1, FRONT_HEIGHT as i32 - 1)] {
            assert_eq!(framebuffer.pixel(x, y).unwrap().a, 255);
        }
    }

    #[test]
    fn singularity_is_a_rare_timed_signature() {
        assert_eq!(singularity_visibility(0.0), 0.0);
        assert_eq!(singularity_visibility(15.9), 0.0);
        assert!(singularity_visibility(18.0) > 0.0);
        assert_eq!(singularity_visibility(22.0), 1.0);
        assert_eq!(singularity_visibility(30.1), 0.0);
    }

    #[test]
    fn transparent_foreground_preserves_background() {
        let background = Pixel::rgb(4, 8, 16);
        assert_eq!(composite_over(background, Pixel::TRANSPARENT), background);
    }

    #[test]
    fn opaque_foreground_replaces_background() {
        let foreground = Pixel::rgb(240, 48, 200);
        assert_eq!(composite_over(Pixel::rgb(4, 8, 16), foreground), foreground);
    }
}