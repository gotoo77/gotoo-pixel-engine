use crate::Pixel;

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

        let mut x0 = i64::from(x0);
        let mut y0 = i64::from(y0);
        let x1 = i64::from(x1);
        let y1 = i64::from(y1);

        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut error = dx + dy;

        loop {
            self.draw_i64(x0, y0, pixel);

            if x0 == x1 && y0 == y1 {
                break;
            }

            let doubled_error = error * 2;
            if doubled_error >= dy {
                error += dy;
                x0 += sx;
            }
            if doubled_error <= dx {
                error += dx;
                y0 += sy;
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

#[cfg(test)]
mod tests {
    use super::{Framebuffer, Pixel};

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
}
