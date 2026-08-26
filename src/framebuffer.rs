use crate::{Image, ImageRegion, Pixel};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Font {
    #[default]
    Pixel5x7,
    Mini3x5,
}

impl Font {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "builtin" | "default" | "pixel" | "pixel_5x7" | "5x7" => Some(Self::Pixel5x7),
            "mini" | "mini_3x5" | "tiny" | "tiny_3x5" | "3x5" => Some(Self::Mini3x5),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Pixel5x7 => "pixel_5x7",
            Self::Mini3x5 => "mini_3x5",
        }
    }

    fn metrics(self) -> FontMetrics {
        match self {
            Self::Pixel5x7 => FontMetrics {
                glyph_width: 5,
                glyph_height: 7,
                glyph_spacing: 1,
            },
            Self::Mini3x5 => FontMetrics {
                glyph_width: 3,
                glyph_height: 5,
                glyph_spacing: 1,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Framebuffer {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let len = width as usize * height as usize * 4;

        Self {
            width,
            height,
            pixels: vec![0; len],
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn as_rgba8(&self) -> &[u8] {
        &self.pixels
    }

    pub fn clear(&mut self, pixel: Pixel) {
        let rgba = pixel.to_rgba8();

        for chunk in self.pixels.chunks_exact_mut(4) {
            chunk.copy_from_slice(&rgba);
        }
    }

    pub fn draw(&mut self, x: i32, y: i32, pixel: Pixel) -> bool {
        let Some(index) = self.pixel_index(x, y) else {
            return false;
        };

        self.pixels[index..index + 4].copy_from_slice(&pixel.to_rgba8());
        true
    }

    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, pixel: Pixel) {
        if self.line_is_outside_framebuffer(x0, y0, x1, y1) {
            return;
        }

        let x0 = i64::from(x0);
        let y0 = i64::from(y0);
        let x1 = i64::from(x1);
        let y1 = i64::from(y1);
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();

        if dx == 0 && dy == 0 {
            self.draw_i64(x0, y0, pixel);
            return;
        }

        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };

        if dx >= dy {
            let Some((first_step, last_step)) = visible_major_steps(x0, x1, self.width) else {
                return;
            };

            for step in first_step..=last_step {
                let x = x0 + sx * step;
                let y = y0 + sy * rounded_minor_step(dy, step, dx);
                self.draw_i64(x, y, pixel);
            }
        } else {
            let Some((first_step, last_step)) = visible_major_steps(y0, y1, self.height) else {
                return;
            };

            for step in first_step..=last_step {
                let x = x0 + sx * rounded_minor_step(dx, step, dy);
                let y = y0 + sy * step;
                self.draw_i64(x, y, pixel);
            }
        }
    }

    pub fn draw_rect(&mut self, x: i32, y: i32, width: u32, height: u32, pixel: Pixel) {
        let Some((min_x, min_y, max_x, max_y)) = rect_bounds(x, y, width, height) else {
            return;
        };

        if !self.rect_intersects_framebuffer(min_x, min_y, max_x, max_y) {
            return;
        }

        self.draw_horizontal_span(min_x, max_x, min_y, pixel);
        self.draw_horizontal_span(min_x, max_x, max_y, pixel);

        let Some((_, visible_min_y, _, visible_max_y)) =
            self.clipped_rect_bounds(min_x, min_y + 1, max_x, max_y - 1)
        else {
            return;
        };

        for y in visible_min_y..=visible_max_y {
            self.draw_i64(min_x, y, pixel);
            if max_x != min_x {
                self.draw_i64(max_x, y, pixel);
            }
        }
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, width: u32, height: u32, pixel: Pixel) {
        let Some((min_x, min_y, max_x, max_y)) = rect_bounds(x, y, width, height) else {
            return;
        };
        let Some((visible_min_x, visible_min_y, visible_max_x, visible_max_y)) =
            self.clipped_rect_bounds(min_x, min_y, max_x, max_y)
        else {
            return;
        };

        for y in visible_min_y..=visible_max_y {
            self.draw_horizontal_span(visible_min_x, visible_max_x, y, pixel);
        }
    }

    pub fn draw_image(&mut self, x: i32, y: i32, image: &Image) {
        self.draw_image_region(
            x,
            y,
            image,
            ImageRegion::new(0, 0, image.width(), image.height()),
        );
    }

    pub fn draw_image_region(&mut self, x: i32, y: i32, image: &Image, region: ImageRegion) {
        let Some(region) = clipped_image_region(image, region) else {
            return;
        };
        let Some((min_x, min_y, max_x, max_y)) = rect_bounds(x, y, region.width, region.height)
        else {
            return;
        };
        let Some((visible_min_x, visible_min_y, visible_max_x, visible_max_y)) =
            self.clipped_rect_bounds(min_x, min_y, max_x, max_y)
        else {
            return;
        };

        for dest_y in visible_min_y..=visible_max_y {
            let source_y = region.y as usize + (dest_y - min_y) as usize;
            for dest_x in visible_min_x..=visible_max_x {
                let source_x = region.x as usize + (dest_x - min_x) as usize;
                let source_index = (source_y * image.width() as usize + source_x) * 4;
                let source = &image.as_rgba8()[source_index..source_index + 4];
                self.blend_rgba8(dest_x as u32, dest_y as u32, source);
            }
        }
    }

    pub fn draw_circle(&mut self, center_x: i32, center_y: i32, radius: u32, pixel: Pixel) {
        if radius == 0 {
            self.draw(center_x, center_y, pixel);
            return;
        }

        if self.circle_is_outside_framebuffer(center_x, center_y, radius)
            || self.circle_outline_is_far_from_framebuffer(center_x, center_y, radius)
        {
            return;
        }

        let center_x = i64::from(center_x);
        let center_y = i64::from(center_y);
        let mut x = i64::from(radius);
        let mut y = 0_i64;
        let mut error = 1 - x;

        while x >= y {
            self.draw_circle_points(center_x, center_y, x, y, pixel);
            y += 1;

            if error < 0 {
                error += 2 * y + 1;
            } else {
                x -= 1;
                error += 2 * (y - x) + 1;
            }
        }
    }

    pub fn fill_circle(&mut self, center_x: i32, center_y: i32, radius: u32, pixel: Pixel) {
        if radius == 0 {
            self.draw(center_x, center_y, pixel);
            return;
        }

        if self.circle_is_outside_framebuffer(center_x, center_y, radius) {
            return;
        }
        if self.circle_covers_framebuffer(center_x, center_y, radius) {
            self.fill_rect(0, 0, self.width, self.height, pixel);
            return;
        }

        let center_x = i64::from(center_x);
        let center_y = i64::from(center_y);
        let mut x = i64::from(radius);
        let mut y = 0_i64;
        let mut error = 1 - x;

        while x >= y {
            self.draw_filled_circle_spans(center_x, center_y, x, y, pixel);
            y += 1;

            if error < 0 {
                error += 2 * y + 1;
            } else {
                x -= 1;
                error += 2 * (y - x) + 1;
            }
        }
    }

    pub fn draw_text(&mut self, x: i32, y: i32, text: &str, pixel: Pixel) {
        self.draw_text_scaled(x, y, text, 1, pixel);
    }

    pub fn draw_text_scaled(&mut self, x: i32, y: i32, text: &str, scale: u32, pixel: Pixel) {
        self.draw_text_scaled_with_font(Font::default(), x, y, text, scale, pixel);
    }

    pub fn draw_text_with_font(&mut self, font: Font, x: i32, y: i32, text: &str, pixel: Pixel) {
        self.draw_text_scaled_with_font(font, x, y, text, 1, pixel);
    }

    pub fn draw_text_scaled_with_font(
        &mut self,
        font: Font,
        x: i32,
        y: i32,
        text: &str,
        scale: u32,
        pixel: Pixel,
    ) {
        let scale = scale.max(1);
        let mut cursor_x = i64::from(x);
        let y = i64::from(y);
        let metrics = font.metrics();
        let advance =
            i64::from((metrics.glyph_width + metrics.glyph_spacing).saturating_mul(scale));

        for character in text.chars() {
            self.draw_glyph_scaled(cursor_x, y, font, glyph_for(font, character), scale, pixel);
            cursor_x += advance;
        }
    }

    pub fn text_size(text: &str, scale: u32) -> (u32, u32) {
        Self::text_size_with_font(Font::default(), text, scale)
    }

    pub fn text_size_with_font(font: Font, text: &str, scale: u32) -> (u32, u32) {
        let scale = scale.max(1);
        let character_count = u32::try_from(text.chars().count()).unwrap_or(u32::MAX);

        if character_count == 0 {
            return (0, 0);
        }

        let metrics = font.metrics();
        let glyph_width = metrics.glyph_width.saturating_mul(scale);
        let glyph_height = metrics.glyph_height.saturating_mul(scale);
        let spacing = metrics.glyph_spacing.saturating_mul(scale);
        let text_width = glyph_width
            .saturating_mul(character_count)
            .saturating_add(spacing.saturating_mul(character_count.saturating_sub(1)));

        (text_width, glyph_height)
    }

    pub fn pixel(&self, x: i32, y: i32) -> Option<Pixel> {
        let index = self.pixel_index(x, y)?;
        let rgba = &self.pixels[index..index + 4];

        Some(Pixel::rgba(rgba[0], rgba[1], rgba[2], rgba[3]))
    }

    pub fn set_pixel_in_bounds(&mut self, x: u32, y: u32, pixel: Pixel) {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);

        let index = ((y * self.width + x) * 4) as usize;
        self.pixels[index..index + 4].copy_from_slice(&pixel.to_rgba8());
    }

    fn draw_glyph_scaled(
        &mut self,
        x: i64,
        y: i64,
        font: Font,
        glyph: Glyph,
        scale: u32,
        pixel: Pixel,
    ) {
        let metrics = font.metrics();
        for (row_index, row) in glyph
            .iter()
            .copied()
            .take(metrics.glyph_height as usize)
            .enumerate()
        {
            for column in 0..metrics.glyph_width {
                let mask = 1 << (metrics.glyph_width - 1 - column);
                if row & mask == 0 {
                    continue;
                }

                let pixel_x = x + i64::from(column * scale);
                let pixel_y = y + row_index as i64 * i64::from(scale);
                let Ok(pixel_x) = i32::try_from(pixel_x) else {
                    continue;
                };
                let Ok(pixel_y) = i32::try_from(pixel_y) else {
                    continue;
                };

                self.fill_rect(pixel_x, pixel_y, scale, scale, pixel);
            }
        }
    }

    fn pixel_index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 {
            return None;
        }

        let x = x as u32;
        let y = y as u32;

        if x >= self.width || y >= self.height {
            return None;
        }

        Some(((y * self.width + x) * 4) as usize)
    }

    fn line_is_outside_framebuffer(&self, x0: i32, y0: i32, x1: i32, y1: i32) -> bool {
        if self.is_empty() {
            return true;
        }

        let min_x = i64::from(x0.min(x1));
        let max_x = i64::from(x0.max(x1));
        let min_y = i64::from(y0.min(y1));
        let max_y = i64::from(y0.max(y1));

        !self.rect_intersects_framebuffer(min_x, min_y, max_x, max_y)
    }

    fn draw_i64(&mut self, x: i64, y: i64, pixel: Pixel) -> bool {
        let Ok(x) = i32::try_from(x) else {
            return false;
        };
        let Ok(y) = i32::try_from(y) else {
            return false;
        };

        self.draw(x, y, pixel)
    }

    fn draw_horizontal_span(&mut self, start_x: i64, end_x: i64, y: i64, pixel: Pixel) {
        let Some((start_x, y, end_x, _)) = self.clipped_rect_bounds(start_x, y, end_x, y) else {
            return;
        };

        let y = y as u32;
        for x in start_x..=end_x {
            self.set_pixel_in_bounds(x as u32, y, pixel);
        }
    }

    fn blend_rgba8(&mut self, x: u32, y: u32, source: &[u8]) {
        debug_assert_eq!(source.len(), 4);
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);

        let source_alpha = source[3];
        if source_alpha == 0 {
            return;
        }

        let index = ((y * self.width + x) * 4) as usize;
        if source_alpha == 255 {
            self.pixels[index..index + 4].copy_from_slice(source);
            return;
        }

        let inverse_alpha = 255 - source_alpha as u16;
        let source_alpha = source_alpha as u16;
        for (channel_offset, &source_channel) in source.iter().take(3).enumerate() {
            let dest_channel = self.pixels[index + channel_offset] as u16;
            self.pixels[index + channel_offset] = blend_channel(
                source_channel as u16,
                source_alpha,
                dest_channel,
                inverse_alpha,
            );
        }

        let dest_alpha = self.pixels[index + 3] as u16;
        let out_alpha = source_alpha + ((dest_alpha * inverse_alpha + 127) / 255);
        self.pixels[index + 3] = out_alpha.min(255) as u8;
    }

    fn draw_circle_points(&mut self, center_x: i64, center_y: i64, x: i64, y: i64, pixel: Pixel) {
        self.draw_i64(center_x + x, center_y + y, pixel);
        self.draw_i64(center_x + y, center_y + x, pixel);
        self.draw_i64(center_x - y, center_y + x, pixel);
        self.draw_i64(center_x - x, center_y + y, pixel);
        self.draw_i64(center_x - x, center_y - y, pixel);
        self.draw_i64(center_x - y, center_y - x, pixel);
        self.draw_i64(center_x + y, center_y - x, pixel);
        self.draw_i64(center_x + x, center_y - y, pixel);
    }

    fn draw_filled_circle_spans(
        &mut self,
        center_x: i64,
        center_y: i64,
        x: i64,
        y: i64,
        pixel: Pixel,
    ) {
        self.draw_horizontal_span(center_x - x, center_x + x, center_y + y, pixel);
        self.draw_horizontal_span(center_x - x, center_x + x, center_y - y, pixel);
        self.draw_horizontal_span(center_x - y, center_x + y, center_y + x, pixel);
        self.draw_horizontal_span(center_x - y, center_x + y, center_y - x, pixel);
    }

    fn circle_is_outside_framebuffer(&self, center_x: i32, center_y: i32, radius: u32) -> bool {
        if self.is_empty() {
            return true;
        }

        let center_x = i64::from(center_x);
        let center_y = i64::from(center_y);
        let radius = i64::from(radius);

        !self.rect_intersects_framebuffer(
            center_x - radius,
            center_y - radius,
            center_x + radius,
            center_y + radius,
        )
    }

    fn circle_covers_framebuffer(&self, center_x: i32, center_y: i32, radius: u32) -> bool {
        if self.is_empty() {
            return false;
        }

        let center_x = i128::from(center_x);
        let center_y = i128::from(center_y);
        let radius = i128::from(radius);
        let radius_squared = radius * radius;
        let max_x = i128::from(self.width - 1);
        let max_y = i128::from(self.height - 1);

        [(0_i128, 0_i128), (max_x, 0), (0, max_y), (max_x, max_y)]
            .into_iter()
            .all(|(x, y)| {
                let dx = x - center_x;
                let dy = y - center_y;
                dx * dx + dy * dy <= radius_squared
            })
    }

    fn circle_outline_is_far_from_framebuffer(
        &self,
        center_x: i32,
        center_y: i32,
        radius: u32,
    ) -> bool {
        if self.is_empty() {
            return true;
        }

        let center_x = i128::from(center_x);
        let center_y = i128::from(center_y);
        let max_x = i128::from(self.width - 1);
        let max_y = i128::from(self.height - 1);
        let max_visible_distance = [(0_i128, 0_i128), (max_x, 0), (0, max_y), (max_x, max_y)]
            .into_iter()
            .map(|(x, y)| (x - center_x).abs().max((y - center_y).abs()))
            .max()
            .unwrap_or(0);

        max_visible_distance * 2 < i128::from(radius)
    }

    fn clipped_rect_bounds(
        &self,
        min_x: i64,
        min_y: i64,
        max_x: i64,
        max_y: i64,
    ) -> Option<(i64, i64, i64, i64)> {
        if !self.rect_intersects_framebuffer(min_x, min_y, max_x, max_y) {
            return None;
        }

        Some((
            min_x.max(0),
            min_y.max(0),
            max_x.min(i64::from(self.width) - 1),
            max_y.min(i64::from(self.height) - 1),
        ))
    }

    fn rect_intersects_framebuffer(&self, min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> bool {
        if self.is_empty() || min_x > max_x || min_y > max_y {
            return false;
        }

        max_x >= 0 && max_y >= 0 && min_x < i64::from(self.width) && min_y < i64::from(self.height)
    }

    fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}

fn visible_major_steps(start: i64, end: i64, limit: u32) -> Option<(i64, i64)> {
    if limit == 0 {
        return None;
    }

    let total_steps = (end - start).abs();
    let max_coordinate = i64::from(limit) - 1;

    let (first_step, last_step) = if start <= end {
        ((-start).max(0), (max_coordinate - start).min(total_steps))
    } else {
        ((start - max_coordinate).max(0), start.min(total_steps))
    };

    (first_step <= last_step).then_some((first_step, last_step))
}

fn rounded_minor_step(minor_delta: i64, major_step: i64, major_delta: i64) -> i64 {
    debug_assert!(major_delta > 0);
    debug_assert!(major_step >= 0 && major_step <= major_delta);
    debug_assert!(minor_delta >= 0 && minor_delta <= major_delta);

    let numerator = 2 * i128::from(minor_delta) * i128::from(major_step) + i128::from(major_delta);
    let denominator = 2 * i128::from(major_delta);

    (numerator / denominator) as i64
}

fn rect_bounds(x: i32, y: i32, width: u32, height: u32) -> Option<(i64, i64, i64, i64)> {
    if width == 0 || height == 0 {
        return None;
    }

    let min_x = i64::from(x);
    let min_y = i64::from(y);

    Some((
        min_x,
        min_y,
        min_x + i64::from(width) - 1,
        min_y + i64::from(height) - 1,
    ))
}

fn clipped_image_region(image: &Image, region: ImageRegion) -> Option<ImageRegion> {
    let max_x = region.x.checked_add(region.width)?.min(image.width());
    let max_y = region.y.checked_add(region.height)?.min(image.height());

    if region.x >= max_x || region.y >= max_y {
        return None;
    }

    Some(ImageRegion::new(
        region.x,
        region.y,
        max_x - region.x,
        max_y - region.y,
    ))
}

fn blend_channel(source: u16, source_alpha: u16, dest: u16, inverse_alpha: u16) -> u8 {
    ((source * source_alpha + dest * inverse_alpha + 127) / 255) as u8
}

const MAX_FONT_GLYPH_HEIGHT: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FontMetrics {
    glyph_width: u32,
    glyph_height: u32,
    glyph_spacing: u32,
}

type Glyph = [u8; MAX_FONT_GLYPH_HEIGHT];

fn glyph_for(font: Font, character: char) -> Glyph {
    let character = character.to_ascii_uppercase();
    match font {
        Font::Pixel5x7 => pixel_5x7_glyph_for(character),
        Font::Mini3x5 => mini_3x5_glyph_for(character),
    }
}

fn pixel_5x7_glyph_for(character: char) -> Glyph {
    match character {
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'G' => [
            0b01111, 0b10000, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
        ],
        'J' => [
            0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010,
        ],
        'X' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
        ],
        'Y' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'Z' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
        ],
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '3' => [
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110,
        ],
        '6' => [
            0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110,
        ],
        '"' => [
            0b01010, 0b01010, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '#' => [
            0b01010, 0b11111, 0b01010, 0b11111, 0b01010, 0b00000, 0b00000,
        ],
        '$' => [
            0b00100, 0b01111, 0b10100, 0b01110, 0b00101, 0b11110, 0b00100,
        ],
        '%' => [
            0b11001, 0b11010, 0b00100, 0b01011, 0b10011, 0b00000, 0b00000,
        ],
        '&' => [
            0b01100, 0b10010, 0b10100, 0b01000, 0b10101, 0b10010, 0b01101,
        ],
        '\'' => [
            0b00100, 0b00100, 0b00010, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '(' => [
            0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010,
        ],
        ')' => [
            0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000,
        ],
        '*' => [
            0b00000, 0b10101, 0b01110, 0b11111, 0b01110, 0b10101, 0b00000,
        ],
        ';' => [
            0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b01000,
        ],
        '<' => [
            0b00001, 0b00010, 0b00100, 0b01000, 0b00100, 0b00010, 0b00001,
        ],
        '=' => [
            0b00000, 0b11111, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ],
        '@' => [
            0b01110, 0b10001, 0b10111, 0b10101, 0b10111, 0b10000, 0b01110,
        ],
        '[' => [
            0b01110, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01110,
        ],
        '\\' => [
            0b10000, 0b01000, 0b01000, 0b00100, 0b00010, 0b00010, 0b00001,
        ],
        ']' => [
            0b01110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b01110,
        ],
        '^' => [
            0b00100, 0b01010, 0b10001, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '_' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111,
        ],
        '`' => [
            0b01000, 0b00100, 0b00010, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '{' => [
            0b00011, 0b00100, 0b00100, 0b01000, 0b00100, 0b00100, 0b00011,
        ],
        '|' => [
            0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        '}' => [
            0b11000, 0b00100, 0b00100, 0b00010, 0b00100, 0b00100, 0b11000,
        ],
        '~' => [
            0b00000, 0b00000, 0b01001, 0b10110, 0b00000, 0b00000, 0b00000,
        ],
        '>' => [
            0b10000, 0b01000, 0b00100, 0b00010, 0b00100, 0b01000, 0b10000,
        ],
        '/' => [
            0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000,
        ],
        '+' => [
            0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000,
        ],
        ':' => [
            0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b00000,
        ],
        '-' => [
            0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ],
        '.' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100,
        ],
        ',' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00100, 0b01000,
        ],
        '!' => [
            0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100,
        ],
        '?' => [
            0b11111, 0b10001, 0b00001, 0b00010, 0b00100, 0b00000, 0b00100,
        ],
        ' ' => [0; MAX_FONT_GLYPH_HEIGHT],
        _ => [
            0b11111, 0b10001, 0b00001, 0b00010, 0b00100, 0b00000, 0b00100,
        ],
    }
}

fn mini_3x5_glyph_for(character: char) -> Glyph {
    match character {
        'A' => [0b111, 0b101, 0b111, 0b101, 0b101, 0, 0],
        'B' => [0b110, 0b101, 0b110, 0b101, 0b110, 0, 0],
        'C' => [0b111, 0b100, 0b100, 0b100, 0b111, 0, 0],
        'D' => [0b110, 0b101, 0b101, 0b101, 0b110, 0, 0],
        'E' => [0b111, 0b100, 0b110, 0b100, 0b111, 0, 0],
        'F' => [0b111, 0b100, 0b110, 0b100, 0b100, 0, 0],
        'G' => [0b111, 0b100, 0b101, 0b101, 0b111, 0, 0],
        'H' => [0b101, 0b101, 0b111, 0b101, 0b101, 0, 0],
        'I' => [0b111, 0b010, 0b010, 0b010, 0b111, 0, 0],
        'J' => [0b001, 0b001, 0b001, 0b101, 0b111, 0, 0],
        'K' => [0b101, 0b101, 0b110, 0b101, 0b101, 0, 0],
        'L' => [0b100, 0b100, 0b100, 0b100, 0b111, 0, 0],
        'M' => [0b101, 0b111, 0b111, 0b101, 0b101, 0, 0],
        'N' => [0b101, 0b111, 0b111, 0b111, 0b101, 0, 0],
        'O' => [0b111, 0b101, 0b101, 0b101, 0b111, 0, 0],
        'P' => [0b111, 0b101, 0b111, 0b100, 0b100, 0, 0],
        'Q' => [0b111, 0b101, 0b101, 0b111, 0b001, 0, 0],
        'R' => [0b110, 0b101, 0b110, 0b101, 0b101, 0, 0],
        'S' => [0b111, 0b100, 0b111, 0b001, 0b111, 0, 0],
        'T' => [0b111, 0b010, 0b010, 0b010, 0b010, 0, 0],
        'U' => [0b101, 0b101, 0b101, 0b101, 0b111, 0, 0],
        'V' => [0b101, 0b101, 0b101, 0b101, 0b010, 0, 0],
        'W' => [0b101, 0b101, 0b111, 0b111, 0b101, 0, 0],
        'X' => [0b101, 0b101, 0b010, 0b101, 0b101, 0, 0],
        'Y' => [0b101, 0b101, 0b010, 0b010, 0b010, 0, 0],
        'Z' => [0b111, 0b001, 0b010, 0b100, 0b111, 0, 0],
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111, 0, 0],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111, 0, 0],
        '2' => [0b111, 0b001, 0b111, 0b100, 0b111, 0, 0],
        '3' => [0b111, 0b001, 0b111, 0b001, 0b111, 0, 0],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001, 0, 0],
        '5' => [0b111, 0b100, 0b111, 0b001, 0b111, 0, 0],
        '6' => [0b111, 0b100, 0b111, 0b101, 0b111, 0, 0],
        '7' => [0b111, 0b001, 0b010, 0b010, 0b010, 0, 0],
        '8' => [0b111, 0b101, 0b111, 0b101, 0b111, 0, 0],
        '9' => [0b111, 0b101, 0b111, 0b001, 0b111, 0, 0],
        '"' => [0b101, 0b101, 0b000, 0b000, 0b000, 0, 0],
        '#' => [0b101, 0b111, 0b101, 0b111, 0b101, 0, 0],
        '$' => [0b010, 0b111, 0b110, 0b011, 0b111, 0, 0],
        '%' => [0b101, 0b001, 0b010, 0b100, 0b101, 0, 0],
        '&' => [0b010, 0b101, 0b010, 0b101, 0b011, 0, 0],
        '\'' => [0b010, 0b010, 0b000, 0b000, 0b000, 0, 0],
        '(' => [0b010, 0b100, 0b100, 0b100, 0b010, 0, 0],
        ')' => [0b010, 0b001, 0b001, 0b001, 0b010, 0, 0],
        '*' => [0b000, 0b101, 0b010, 0b101, 0b000, 0, 0],
        ';' => [0b000, 0b010, 0b000, 0b010, 0b100, 0, 0],
        '<' => [0b001, 0b010, 0b100, 0b010, 0b001, 0, 0],
        '=' => [0b000, 0b111, 0b000, 0b111, 0b000, 0, 0],
        '@' => [0b111, 0b101, 0b111, 0b100, 0b111, 0, 0],
        '[' => [0b110, 0b100, 0b100, 0b100, 0b110, 0, 0],
        '\\' => [0b100, 0b100, 0b010, 0b001, 0b001, 0, 0],
        ']' => [0b011, 0b001, 0b001, 0b001, 0b011, 0, 0],
        '^' => [0b010, 0b101, 0b000, 0b000, 0b000, 0, 0],
        '_' => [0b000, 0b000, 0b000, 0b000, 0b111, 0, 0],
        '`' => [0b100, 0b010, 0b000, 0b000, 0b000, 0, 0],
        '{' => [0b011, 0b010, 0b100, 0b010, 0b011, 0, 0],
        '|' => [0b010, 0b010, 0b010, 0b010, 0b010, 0, 0],
        '}' => [0b110, 0b010, 0b001, 0b010, 0b110, 0, 0],
        '~' => [0b000, 0b000, 0b101, 0b010, 0b000, 0, 0],
        '>' => [0b100, 0b010, 0b001, 0b010, 0b100, 0, 0],
        '/' => [0b001, 0b001, 0b010, 0b100, 0b100, 0, 0],
        '+' => [0b000, 0b010, 0b111, 0b010, 0b000, 0, 0],
        ':' => [0b000, 0b010, 0b000, 0b010, 0b000, 0, 0],
        '-' => [0b000, 0b000, 0b111, 0b000, 0b000, 0, 0],
        '.' => [0b000, 0b000, 0b000, 0b000, 0b010, 0, 0],
        ',' => [0b000, 0b000, 0b000, 0b010, 0b100, 0, 0],
        '!' => [0b010, 0b010, 0b010, 0b000, 0b010, 0, 0],
        '?' => [0b111, 0b001, 0b010, 0b000, 0b010, 0, 0],
        ' ' => [0; MAX_FONT_GLYPH_HEIGHT],
        _ => [0b111, 0b001, 0b010, 0b000, 0b010, 0, 0],
    }
}

#[cfg(test)]
mod tests {
    use super::{Font, Framebuffer, Image, ImageRegion, Pixel, glyph_for};

    fn drawn_pixels(framebuffer: &Framebuffer, pixel: Pixel) -> Vec<(i32, i32)> {
        let mut pixels = Vec::new();

        for y in 0..framebuffer.height() as i32 {
            for x in 0..framebuffer.width() as i32 {
                if framebuffer.pixel(x, y) == Some(pixel) {
                    pixels.push((x, y));
                }
            }
        }

        pixels
    }

    #[test]
    fn allocates_rgba8_storage() {
        let framebuffer = Framebuffer::new(3, 2);

        assert_eq!(framebuffer.width(), 3);
        assert_eq!(framebuffer.height(), 2);
        assert_eq!(framebuffer.as_rgba8().len(), 3 * 2 * 4);
    }

    #[test]
    fn clear_writes_every_pixel() {
        let mut framebuffer = Framebuffer::new(2, 2);

        framebuffer.clear(Pixel::rgba(1, 2, 3, 4));

        assert_eq!(
            framebuffer.as_rgba8(),
            &[1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4]
        );
    }

    #[test]
    fn set_pixel_writes_rgba8_at_expected_offset() {
        let mut framebuffer = Framebuffer::new(2, 2);

        framebuffer.set_pixel_in_bounds(1, 0, Pixel::rgba(1, 2, 3, 255));

        assert_eq!(
            framebuffer.as_rgba8(),
            &[0, 0, 0, 0, 1, 2, 3, 255, 0, 0, 0, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn draw_image_opaque_blit_copies_source_pixels() {
        let mut framebuffer = Framebuffer::new(4, 3);
        let image =
            Image::from_rgba8(2, 1, vec![255, 0, 0, 255, 0, 255, 0, 255]).expect("valid image");

        framebuffer.draw_image(1, 1, &image);

        assert_eq!(framebuffer.pixel(1, 1), Some(Pixel::rgba(255, 0, 0, 255)));
        assert_eq!(framebuffer.pixel(2, 1), Some(Pixel::rgba(0, 255, 0, 255)));
        assert_eq!(framebuffer.pixel(0, 1), Some(Pixel::TRANSPARENT));
    }

    #[test]
    fn draw_image_alpha_blends_over_destination() {
        let mut framebuffer = Framebuffer::new(1, 1);
        framebuffer.clear(Pixel::rgba(0, 0, 255, 255));
        let image = Image::from_rgba8(1, 1, vec![255, 0, 0, 128]).expect("valid translucent image");

        framebuffer.draw_image(0, 0, &image);

        assert_eq!(framebuffer.pixel(0, 0), Some(Pixel::rgba(128, 0, 127, 255)));
    }

    #[test]
    fn draw_image_clips_left_right_top_and_bottom() {
        let image = solid_image(3, 3, Pixel::WHITE);

        let mut left = Framebuffer::new(3, 3);
        left.draw_image(-1, 0, &image);
        assert_eq!(drawn_pixels(&left, Pixel::WHITE).len(), 6);
        assert_eq!(left.pixel(2, 1), Some(Pixel::TRANSPARENT));

        let mut right = Framebuffer::new(3, 3);
        right.draw_image(1, 0, &image);
        assert_eq!(drawn_pixels(&right, Pixel::WHITE).len(), 6);
        assert_eq!(right.pixel(0, 1), Some(Pixel::TRANSPARENT));

        let mut top = Framebuffer::new(3, 3);
        top.draw_image(0, -1, &image);
        assert_eq!(drawn_pixels(&top, Pixel::WHITE).len(), 6);
        assert_eq!(top.pixel(1, 2), Some(Pixel::TRANSPARENT));

        let mut bottom = Framebuffer::new(3, 3);
        bottom.draw_image(0, 1, &image);
        assert_eq!(drawn_pixels(&bottom, Pixel::WHITE).len(), 6);
        assert_eq!(bottom.pixel(1, 0), Some(Pixel::TRANSPARENT));
    }

    #[test]
    fn draw_image_region_blits_only_selected_source_rectangle() {
        let image = Image::from_rgba8(
            3,
            2,
            vec![
                10, 0, 0, 255, 20, 0, 0, 255, 30, 0, 0, 255, 40, 0, 0, 255, 50, 0, 0, 255, 60, 0,
                0, 255,
            ],
        )
        .expect("valid image");
        let mut framebuffer = Framebuffer::new(3, 1);

        framebuffer.draw_image_region(0, 0, &image, ImageRegion::new(1, 0, 2, 1));

        assert_eq!(framebuffer.pixel(0, 0), Some(Pixel::rgba(20, 0, 0, 255)));
        assert_eq!(framebuffer.pixel(1, 0), Some(Pixel::rgba(30, 0, 0, 255)));
        assert_eq!(framebuffer.pixel(2, 0), Some(Pixel::TRANSPARENT));
    }

    #[test]
    fn draw_image_totally_outside_framebuffer_is_noop() {
        let mut framebuffer = Framebuffer::new(2, 2);
        framebuffer.clear(Pixel::rgba(3, 4, 5, 255));
        let before = framebuffer.as_rgba8().to_vec();
        let image = solid_image(2, 2, Pixel::WHITE);

        framebuffer.draw_image(-10, -10, &image);
        framebuffer.draw_image(10, 10, &image);

        assert_eq!(framebuffer.as_rgba8(), before);
    }

    #[test]
    fn draw_returns_true_and_writes_when_in_bounds() {
        let mut framebuffer = Framebuffer::new(2, 2);

        assert!(framebuffer.draw(1, 1, Pixel::WHITE));
        assert_eq!(framebuffer.pixel(1, 1), Some(Pixel::WHITE));
    }

    #[test]
    fn draw_returns_false_and_ignores_out_of_bounds_coordinates() {
        let mut framebuffer = Framebuffer::new(2, 2);

        assert!(!framebuffer.draw(-1, 0, Pixel::WHITE));
        assert!(!framebuffer.draw(2, 0, Pixel::WHITE));
        assert!(!framebuffer.draw(0, 2, Pixel::WHITE));
        assert_eq!(framebuffer.as_rgba8(), &[0; 16]);
    }

    #[test]
    fn pixel_returns_none_out_of_bounds() {
        let framebuffer = Framebuffer::new(2, 2);

        assert_eq!(framebuffer.pixel(-1, 0), None);
        assert_eq!(framebuffer.pixel(0, -1), None);
        assert_eq!(framebuffer.pixel(2, 0), None);
        assert_eq!(framebuffer.pixel(0, 2), None);
    }

    #[test]
    fn draw_line_draws_a_diagonal() {
        let mut framebuffer = Framebuffer::new(5, 5);

        framebuffer.draw_line(0, 0, 4, 4, Pixel::WHITE);

        assert_eq!(
            drawn_pixels(&framebuffer, Pixel::WHITE),
            &[(0, 0), (1, 1), (2, 2), (3, 3), (4, 4)]
        );
    }

    #[test]
    fn draw_line_clips_pixels_outside_framebuffer() {
        let mut framebuffer = Framebuffer::new(3, 1);

        framebuffer.draw_line(-2, 0, 2, 0, Pixel::WHITE);

        assert_eq!(
            drawn_pixels(&framebuffer, Pixel::WHITE),
            &[(0, 0), (1, 0), (2, 0)]
        );
    }

    #[test]
    fn draw_line_clipping_preserves_bresenham_phase() {
        let mut framebuffer = Framebuffer::new(3, 3);

        framebuffer.draw_line(-2, -1, 4, 2, Pixel::WHITE);

        assert_eq!(
            drawn_pixels(&framebuffer, Pixel::WHITE),
            &[(0, 0), (1, 1), (2, 1)]
        );
    }

    #[test]
    fn draw_line_with_extreme_horizontal_span_only_draws_visible_pixels() {
        let mut framebuffer = Framebuffer::new(3, 3);

        framebuffer.draw_line(i32::MIN, 1, i32::MAX, 1, Pixel::WHITE);

        assert_eq!(
            drawn_pixels(&framebuffer, Pixel::WHITE),
            &[(0, 1), (1, 1), (2, 1)]
        );
    }

    #[test]
    fn draw_line_with_extreme_vertical_span_only_draws_visible_pixels() {
        let mut framebuffer = Framebuffer::new(3, 3);

        framebuffer.draw_line(1, i32::MIN, 1, i32::MAX, Pixel::WHITE);

        assert_eq!(
            drawn_pixels(&framebuffer, Pixel::WHITE),
            &[(1, 0), (1, 1), (1, 2)]
        );
    }

    #[test]
    fn draw_line_with_extreme_diagonal_span_only_draws_visible_pixels() {
        let mut framebuffer = Framebuffer::new(3, 3);

        framebuffer.draw_line(i32::MIN, i32::MIN, i32::MAX, i32::MAX, Pixel::WHITE);

        assert_eq!(
            drawn_pixels(&framebuffer, Pixel::WHITE),
            &[(0, 0), (1, 1), (2, 2)]
        );
    }

    #[test]
    fn draw_line_totally_outside_is_noop() {
        let mut framebuffer = Framebuffer::new(3, 3);

        framebuffer.draw_line(-3, -3, -1, -1, Pixel::WHITE);

        assert!(drawn_pixels(&framebuffer, Pixel::WHITE).is_empty());
    }

    #[test]
    fn draw_rect_draws_an_outline() {
        let mut framebuffer = Framebuffer::new(5, 4);

        framebuffer.draw_rect(1, 1, 3, 2, Pixel::WHITE);

        assert_eq!(
            drawn_pixels(&framebuffer, Pixel::WHITE),
            &[(1, 1), (2, 1), (3, 1), (1, 2), (2, 2), (3, 2)]
        );
    }

    #[test]
    fn draw_rect_with_zero_size_is_noop() {
        let mut framebuffer = Framebuffer::new(3, 3);

        framebuffer.draw_rect(0, 0, 0, 2, Pixel::WHITE);
        framebuffer.draw_rect(0, 0, 2, 0, Pixel::WHITE);

        assert!(drawn_pixels(&framebuffer, Pixel::WHITE).is_empty());
    }

    #[test]
    fn draw_rect_clips_pixels_outside_framebuffer() {
        let mut framebuffer = Framebuffer::new(3, 3);

        framebuffer.draw_rect(-1, -1, 3, 3, Pixel::WHITE);

        assert_eq!(
            drawn_pixels(&framebuffer, Pixel::WHITE),
            &[(1, 0), (0, 1), (1, 1)]
        );
    }

    #[test]
    fn fill_rect_draws_every_pixel_in_area() {
        let mut framebuffer = Framebuffer::new(4, 4);

        framebuffer.fill_rect(1, 1, 2, 3, Pixel::WHITE);

        assert_eq!(
            drawn_pixels(&framebuffer, Pixel::WHITE),
            &[(1, 1), (2, 1), (1, 2), (2, 2), (1, 3), (2, 3)]
        );
    }

    #[test]
    fn fill_rect_clips_pixels_outside_framebuffer() {
        let mut framebuffer = Framebuffer::new(3, 3);

        framebuffer.fill_rect(-1, 1, 3, 3, Pixel::WHITE);

        assert_eq!(
            drawn_pixels(&framebuffer, Pixel::WHITE),
            &[(0, 1), (1, 1), (0, 2), (1, 2)]
        );
    }

    #[test]
    fn fill_rect_with_huge_invisible_area_only_draws_visible_pixels() {
        let mut framebuffer = Framebuffer::new(3, 3);

        framebuffer.fill_rect(-1, -1, u32::MAX, u32::MAX, Pixel::WHITE);

        assert_eq!(
            drawn_pixels(&framebuffer, Pixel::WHITE),
            &[
                (0, 0),
                (1, 0),
                (2, 0),
                (0, 1),
                (1, 1),
                (2, 1),
                (0, 2),
                (1, 2),
                (2, 2)
            ]
        );
    }

    #[test]
    fn fill_rect_totally_outside_with_huge_dimensions_is_noop() {
        let mut framebuffer = Framebuffer::new(3, 3);

        framebuffer.fill_rect(i32::MAX, i32::MAX, u32::MAX, u32::MAX, Pixel::WHITE);

        assert!(drawn_pixels(&framebuffer, Pixel::WHITE).is_empty());
    }

    #[test]
    fn draw_circle_with_zero_radius_draws_center_pixel() {
        let mut framebuffer = Framebuffer::new(3, 3);

        framebuffer.draw_circle(1, 1, 0, Pixel::WHITE);

        assert_eq!(drawn_pixels(&framebuffer, Pixel::WHITE), &[(1, 1)]);
    }

    #[test]
    fn draw_circle_draws_radius_one_outline() {
        let mut framebuffer = Framebuffer::new(3, 3);

        framebuffer.draw_circle(1, 1, 1, Pixel::WHITE);

        assert_eq!(
            drawn_pixels(&framebuffer, Pixel::WHITE),
            &[(1, 0), (0, 1), (2, 1), (1, 2)]
        );
    }

    #[test]
    fn draw_circle_clips_pixels_outside_framebuffer() {
        let mut framebuffer = Framebuffer::new(3, 3);

        framebuffer.draw_circle(0, 0, 2, Pixel::WHITE);

        assert_eq!(
            drawn_pixels(&framebuffer, Pixel::WHITE),
            &[(2, 0), (2, 1), (0, 2), (1, 2)]
        );
    }

    #[test]
    fn draw_circle_totally_outside_is_noop() {
        let mut framebuffer = Framebuffer::new(3, 3);

        framebuffer.draw_circle(-10, -10, 1, Pixel::WHITE);

        assert!(drawn_pixels(&framebuffer, Pixel::WHITE).is_empty());
    }

    #[test]
    fn fill_circle_draws_radius_one_disk() {
        let mut framebuffer = Framebuffer::new(3, 3);

        framebuffer.fill_circle(1, 1, 1, Pixel::WHITE);

        assert_eq!(
            drawn_pixels(&framebuffer, Pixel::WHITE),
            &[(1, 0), (0, 1), (1, 1), (2, 1), (1, 2)]
        );
    }

    #[test]
    fn fill_circle_with_zero_radius_draws_center_pixel() {
        let mut framebuffer = Framebuffer::new(3, 3);

        framebuffer.fill_circle(1, 1, 0, Pixel::WHITE);

        assert_eq!(drawn_pixels(&framebuffer, Pixel::WHITE), &[(1, 1)]);
    }

    #[test]
    fn fill_circle_clips_pixels_outside_framebuffer() {
        let mut framebuffer = Framebuffer::new(3, 3);

        framebuffer.fill_circle(0, 0, 1, Pixel::WHITE);

        assert_eq!(
            drawn_pixels(&framebuffer, Pixel::WHITE),
            &[(0, 0), (1, 0), (0, 1)]
        );
    }

    #[test]
    fn fill_circle_totally_outside_is_noop() {
        let mut framebuffer = Framebuffer::new(3, 3);

        framebuffer.fill_circle(-10, -10, 1, Pixel::WHITE);

        assert!(drawn_pixels(&framebuffer, Pixel::WHITE).is_empty());
    }

    #[test]
    fn draw_circle_totally_outside_with_huge_radius_is_noop() {
        let mut framebuffer = Framebuffer::new(3, 3);

        framebuffer.draw_circle(i32::MAX, i32::MAX, 1_000_000, Pixel::WHITE);

        assert!(drawn_pixels(&framebuffer, Pixel::WHITE).is_empty());
    }

    #[test]
    fn draw_circle_with_huge_radius_far_from_visible_pixels_is_noop() {
        let mut framebuffer = Framebuffer::new(3, 3);

        framebuffer.draw_circle(1, 1, u32::MAX, Pixel::WHITE);

        assert!(drawn_pixels(&framebuffer, Pixel::WHITE).is_empty());
    }

    #[test]
    fn fill_circle_covering_framebuffer_fills_visible_pixels_directly() {
        let mut framebuffer = Framebuffer::new(3, 3);

        framebuffer.fill_circle(1, 1, u32::MAX, Pixel::WHITE);

        assert_eq!(
            drawn_pixels(&framebuffer, Pixel::WHITE),
            &[
                (0, 0),
                (1, 0),
                (2, 0),
                (0, 1),
                (1, 1),
                (2, 1),
                (0, 2),
                (1, 2),
                (2, 2)
            ]
        );
    }

    #[test]
    fn draw_text_draws_letters() {
        let mut framebuffer = Framebuffer::new(5, 7);

        framebuffer.draw_text(0, 0, "A", Pixel::WHITE);

        assert_eq!(
            drawn_pixels(&framebuffer, Pixel::WHITE),
            &[
                (1, 0),
                (2, 0),
                (3, 0),
                (0, 1),
                (4, 1),
                (0, 2),
                (4, 2),
                (0, 3),
                (1, 3),
                (2, 3),
                (3, 3),
                (4, 3),
                (0, 4),
                (4, 4),
                (0, 5),
                (4, 5),
                (0, 6),
                (4, 6)
            ]
        );
    }

    #[test]
    fn draw_text_draws_digits() {
        let mut framebuffer = Framebuffer::new(5, 7);

        framebuffer.draw_text(0, 0, "2", Pixel::WHITE);

        assert_eq!(
            drawn_pixels(&framebuffer, Pixel::WHITE),
            &[
                (1, 0),
                (2, 0),
                (3, 0),
                (0, 1),
                (4, 1),
                (4, 2),
                (3, 3),
                (2, 4),
                (1, 5),
                (0, 6),
                (1, 6),
                (2, 6),
                (3, 6),
                (4, 6)
            ]
        );
    }

    #[test]
    fn draw_text_space_advances_without_drawing_pixels() {
        let mut framebuffer = Framebuffer::new(17, 7);

        framebuffer.draw_text(0, 0, "A A", Pixel::WHITE);

        assert_eq!(framebuffer.pixel(6, 0), Some(Pixel::TRANSPARENT));
        assert_eq!(framebuffer.pixel(13, 0), Some(Pixel::WHITE));
    }

    #[test]
    fn draw_text_empty_string_is_noop() {
        let mut framebuffer = Framebuffer::new(5, 7);

        framebuffer.draw_text(0, 0, "", Pixel::WHITE);

        assert!(drawn_pixels(&framebuffer, Pixel::WHITE).is_empty());
        assert_eq!(Framebuffer::text_size("", 1), (0, 0));
    }

    #[test]
    fn draw_text_unknown_character_uses_deterministic_placeholder() {
        let mut framebuffer = Framebuffer::new(5, 7);

        framebuffer.draw_text(0, 0, "?", Pixel::WHITE);

        assert_eq!(framebuffer.pixel(0, 0), Some(Pixel::WHITE));
        assert_eq!(framebuffer.pixel(4, 0), Some(Pixel::WHITE));
        assert_eq!(framebuffer.pixel(2, 6), Some(Pixel::WHITE));
    }

    #[test]
    fn printable_ascii_has_explicit_glyphs() {
        for font in [Font::Pixel5x7, Font::Mini3x5] {
            let fallback = glyph_for(font, '?');
            for code in 0x20_u8..=0x7E {
                let character = char::from(code);
                if character == '?' {
                    continue;
                }
                assert_ne!(
                    glyph_for(font, character),
                    fallback,
                    "{font:?} U+{code:04X} {character:?} unexpectedly uses fallback"
                );
            }
        }
    }

    #[test]
    fn font_names_parse_expected_builtin_fonts() {
        assert_eq!(Font::from_name("builtin"), Some(Font::Pixel5x7));
        assert_eq!(Font::from_name("pixel_5x7"), Some(Font::Pixel5x7));
        assert_eq!(Font::from_name("mini_3x5"), Some(Font::Mini3x5));
        assert_eq!(Font::Mini3x5.name(), "mini_3x5");
        assert_eq!(Font::from_name("serif"), None);
    }

    #[test]
    fn mini_font_has_smaller_text_metrics_and_draws() {
        let mut framebuffer = Framebuffer::new(3, 5);

        framebuffer.draw_text_with_font(Font::Mini3x5, 0, 0, "A", Pixel::WHITE);

        assert_eq!(
            Framebuffer::text_size_with_font(Font::Mini3x5, "A", 1),
            (3, 5)
        );
        assert_eq!(Framebuffer::text_size("A", 1), (5, 7));
        assert_eq!(framebuffer.pixel(0, 0), Some(Pixel::WHITE));
        assert_eq!(framebuffer.pixel(1, 1), Some(Pixel::TRANSPARENT));
    }

    #[test]
    fn lowercase_ascii_reuses_uppercase_glyphs() {
        assert_eq!(
            glyph_for(Font::Pixel5x7, 'a'),
            glyph_for(Font::Pixel5x7, 'A')
        );
        assert_eq!(glyph_for(Font::Mini3x5, 'z'), glyph_for(Font::Mini3x5, 'Z'));
    }

    #[test]
    fn draw_text_scaled_draws_scaled_pixels() {
        let mut framebuffer = Framebuffer::new(10, 14);

        framebuffer.draw_text_scaled(0, 0, "A", 2, Pixel::WHITE);

        assert_eq!(framebuffer.pixel(2, 0), Some(Pixel::WHITE));
        assert_eq!(framebuffer.pixel(3, 1), Some(Pixel::WHITE));
        assert_eq!(framebuffer.pixel(0, 2), Some(Pixel::WHITE));
        assert_eq!(framebuffer.pixel(9, 3), Some(Pixel::WHITE));
        assert_eq!(framebuffer.pixel(2, 2), Some(Pixel::TRANSPARENT));
    }

    #[test]
    fn text_size_reports_scaled_metrics() {
        assert_eq!(Framebuffer::text_size("A", 1), (5, 7));
        assert_eq!(Framebuffer::text_size("A A", 2), (34, 14));
        assert_eq!(Framebuffer::text_size("A", 0), (5, 7));
    }

    #[test]
    fn draw_text_clips_left_and_negative_coordinates() {
        let mut framebuffer = Framebuffer::new(5, 7);

        framebuffer.draw_text(-2, 0, "A", Pixel::WHITE);

        assert_eq!(framebuffer.pixel(0, 0), Some(Pixel::WHITE));
        assert_eq!(framebuffer.pixel(1, 0), Some(Pixel::WHITE));
        assert_eq!(framebuffer.pixel(2, 1), Some(Pixel::WHITE));
    }

    #[test]
    fn draw_text_clips_right() {
        let mut framebuffer = Framebuffer::new(3, 7);

        framebuffer.draw_text(0, 0, "A", Pixel::WHITE);

        assert_eq!(framebuffer.pixel(1, 0), Some(Pixel::WHITE));
        assert_eq!(framebuffer.pixel(2, 0), Some(Pixel::WHITE));
        assert!(drawn_pixels(&framebuffer, Pixel::WHITE).contains(&(0, 3)));
    }

    #[test]
    fn draw_text_clips_top() {
        let mut framebuffer = Framebuffer::new(5, 5);

        framebuffer.draw_text(0, -2, "A", Pixel::WHITE);

        assert_eq!(framebuffer.pixel(0, 1), Some(Pixel::WHITE));
        assert_eq!(framebuffer.pixel(4, 1), Some(Pixel::WHITE));
    }

    #[test]
    fn draw_text_clips_bottom() {
        let mut framebuffer = Framebuffer::new(5, 3);

        framebuffer.draw_text(0, 0, "A", Pixel::WHITE);

        assert_eq!(framebuffer.pixel(1, 0), Some(Pixel::WHITE));
        assert_eq!(framebuffer.pixel(0, 2), Some(Pixel::WHITE));
    }

    #[test]
    fn draw_text_totally_outside_framebuffer_is_noop() {
        let mut framebuffer = Framebuffer::new(5, 7);

        framebuffer.draw_text(20, 20, "A", Pixel::WHITE);
        framebuffer.draw_text(-20, -20, "A", Pixel::WHITE);

        assert!(drawn_pixels(&framebuffer, Pixel::WHITE).is_empty());
    }

    #[test]
    fn draw_text_outside_framebuffer_does_not_panic() {
        let mut framebuffer = Framebuffer::new(5, 7);

        framebuffer.draw_text(i32::MAX, i32::MAX, "SCORE 1", Pixel::WHITE);
        framebuffer.draw_text_scaled(i32::MIN, i32::MIN, "GAME OVER", 2, Pixel::WHITE);

        assert!(drawn_pixels(&framebuffer, Pixel::WHITE).is_empty());
    }

    fn solid_image(width: u32, height: u32, pixel: Pixel) -> Image {
        let mut rgba8 = Vec::new();
        for _ in 0..width * height {
            rgba8.extend_from_slice(&pixel.to_rgba8());
        }
        Image::from_rgba8(width, height, rgba8).expect("valid solid image")
    }
}
