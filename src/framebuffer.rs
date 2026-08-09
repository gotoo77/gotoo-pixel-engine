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
}

#[cfg(test)]
mod tests {
    use super::{Framebuffer, Pixel};

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
}
