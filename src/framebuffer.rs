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

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn set_pixel_unchecked(&mut self, x: u32, y: u32, rgba: [u8; 4]) {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);

        let index = ((y * self.width + x) * 4) as usize;
        self.pixels[index..index + 4].copy_from_slice(&rgba);
    }
}

#[cfg(test)]
mod tests {
    use super::Framebuffer;

    #[test]
    fn allocates_rgba8_storage() {
        let framebuffer = Framebuffer::new(3, 2);

        assert_eq!(framebuffer.width(), 3);
        assert_eq!(framebuffer.height(), 2);
        assert_eq!(framebuffer.pixels().len(), 3 * 2 * 4);
    }

    #[test]
    fn set_pixel_writes_rgba8_at_expected_offset() {
        let mut framebuffer = Framebuffer::new(2, 2);

        framebuffer.set_pixel_unchecked(1, 0, [1, 2, 3, 255]);

        assert_eq!(
            framebuffer.pixels(),
            &[0, 0, 0, 0, 1, 2, 3, 255, 0, 0, 0, 0, 0, 0, 0, 0]
        );
    }
}
