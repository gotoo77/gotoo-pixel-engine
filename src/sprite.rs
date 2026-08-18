use std::fmt;

use crate::{Framebuffer, Pixel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sprite {
    width: u32,
    height: u32,
    pixels: Vec<Pixel>,
}

impl Sprite {
    pub fn new(width: u32, height: u32, pixels: Vec<Pixel>) -> Result<Self, SpriteError> {
        let expected = width as usize * height as usize;
        if pixels.len() != expected {
            return Err(SpriteError::PixelCountMismatch {
                expected,
                actual: pixels.len(),
            });
        }

        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[Pixel] {
        &self.pixels
    }

    pub fn draw(&self, framebuffer: &mut Framebuffer, x: i32, y: i32) {
        for sprite_y in 0..self.height {
            for sprite_x in 0..self.width {
                let index = (sprite_y * self.width + sprite_x) as usize;
                let pixel = self.pixels[index];
                if pixel.a == 0 {
                    continue;
                }

                let Some(target_x) = i32::try_from(sprite_x)
                    .ok()
                    .and_then(|offset| x.checked_add(offset))
                else {
                    continue;
                };
                let Some(target_y) = i32::try_from(sprite_y)
                    .ok()
                    .and_then(|offset| y.checked_add(offset))
                else {
                    continue;
                };

                framebuffer.draw(target_x, target_y, pixel);
            }
        }
    }

    pub fn draw_centered(&self, framebuffer: &mut Framebuffer, center_x: i32, center_y: i32) {
        let half_width = i32::try_from(self.width / 2).unwrap_or(i32::MAX);
        let half_height = i32::try_from(self.height / 2).unwrap_or(i32::MAX);
        self.draw(
            framebuffer,
            center_x.saturating_sub(half_width),
            center_y.saturating_sub(half_height),
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteError {
    PixelCountMismatch { expected: usize, actual: usize },
}

impl fmt::Display for SpriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PixelCountMismatch { expected, actual } => write!(
                f,
                "sprite pixel count mismatch: expected {expected}, got {actual}"
            ),
        }
    }
}

impl std::error::Error for SpriteError {}

#[cfg(test)]
mod tests {
    use super::{Sprite, SpriteError};
    use crate::{Framebuffer, Pixel};

    #[test]
    fn rejects_invalid_pixel_count() {
        assert_eq!(
            Sprite::new(2, 2, vec![Pixel::WHITE]).unwrap_err(),
            SpriteError::PixelCountMismatch {
                expected: 4,
                actual: 1,
            }
        );
    }

    #[test]
    fn transparent_pixels_leave_framebuffer_untouched() {
        let sprite = Sprite::new(
            2,
            1,
            vec![Pixel::TRANSPARENT, Pixel::rgb(10, 20, 30)],
        )
        .unwrap();
        let mut framebuffer = Framebuffer::new(3, 1);
        framebuffer.clear(Pixel::rgb(1, 2, 3));

        sprite.draw(&mut framebuffer, 0, 0);

        assert_eq!(framebuffer.pixel(0, 0), Some(Pixel::rgb(1, 2, 3)));
        assert_eq!(framebuffer.pixel(1, 0), Some(Pixel::rgb(10, 20, 30)));
    }

    #[test]
    fn drawing_is_clipped_by_framebuffer() {
        let sprite = Sprite::new(2, 2, vec![Pixel::WHITE; 4]).unwrap();
        let mut framebuffer = Framebuffer::new(2, 2);
        framebuffer.clear(Pixel::BLACK);

        sprite.draw(&mut framebuffer, 1, 1);

        assert_eq!(framebuffer.pixel(1, 1), Some(Pixel::WHITE));
        assert_eq!(framebuffer.pixel(0, 0), Some(Pixel::BLACK));
    }
}
