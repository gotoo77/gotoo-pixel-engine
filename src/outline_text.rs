//! Experimental CPU outline text; enabled only by `outline-fonts`.
use std::collections::HashMap;

use crate::{Framebuffer, Pixel, Rect, Size};
use fontdue::{
    Font, FontSettings, Metrics,
    layout::{CoordinateSystem, GlyphRasterConfig, Layout, LayoutSettings, TextStyle},
};

/// Maximum supersampling factor accepted by [`OutlineFont::draw_supersampled`].
///
/// Keeping this bounded prevents accidental large temporary framebuffers while
/// still providing enough headroom for dense glyphs such as CJK at small UI sizes.
pub const MAX_TEXT_RASTER_SCALE: u32 = 4;

/// One owned font and its bounded glyph cache. Multiple instances can coexist.
/// Uses fontdue's basic layout, not complex-script shaping or automatic fallback.
pub struct OutlineFont {
    font: Font,
    layout: Layout,
    cache: HashMap<GlyphRasterConfig, (Metrics, Vec<u8>)>,
}

impl OutlineFont {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        Ok(Self {
            font: Font::from_bytes(bytes, FontSettings::default())?,
            layout: Layout::new(CoordinateSystem::PositiveYDown),
            cache: HashMap::new(),
        })
    }

    pub fn has_glyph(&self, character: char) -> bool {
        self.font.has_glyph(character)
    }

    /// Returns true when every non-whitespace character is available in this font.
    ///
    /// This is intentionally a coverage check, not a shaping decision. It lets UI
    /// code select a fallback face before measuring or drawing text.
    pub fn supports_text(&self, text: &str) -> bool {
        text.chars()
            .all(|character| character.is_whitespace() || self.has_glyph(character))
    }

    fn arrange(&mut self, text: &str, px: f32, bounds: Rect) -> Size {
        self.layout.reset(&LayoutSettings {
            x: bounds.x as f32,
            y: bounds.y as f32,
            max_width: Some(bounds.width as f32),
            ..LayoutSettings::default()
        });
        if !px.is_finite() || px <= 0.0 || bounds.width == 0 || bounds.height == 0 {
            return Size {
                width: 0,
                height: 0,
            };
        }
        self.layout
            .append(&[&self.font], &TextStyle::new(text, px.min(256.0), 0));
        let width = self
            .layout
            .glyphs()
            .iter()
            .filter(|g| !g.parent.is_control())
            .map(|g| (g.x - bounds.x as f32 + g.width as f32).ceil() as u32)
            .max()
            .unwrap_or(0);
        Size {
            width,
            height: self.layout.height().ceil() as u32,
        }
    }

    /// Measures with the same wrapped layout used for painting.
    pub fn measure(&mut self, text: &str, px: f32, bounds: Rect) -> Size {
        self.arrange(text, px, bounds)
    }

    /// Paints antialiased glyphs, clipped to bounds and the framebuffer.
    pub fn draw(
        &mut self,
        framebuffer: &mut Framebuffer,
        text: &str,
        px: f32,
        bounds: Rect,
        color: Pixel,
    ) -> Size {
        let measured = self.arrange(text, px, bounds);
        let right =
            (i64::from(bounds.x) + i64::from(bounds.width)).min(i64::from(framebuffer.width()));
        let bottom =
            (i64::from(bounds.y) + i64::from(bounds.height)).min(i64::from(framebuffer.height()));
        for glyph in self.layout.glyphs() {
            if glyph.parent.is_control() {
                continue;
            }
            if self.cache.len() >= 1024 {
                self.cache.clear();
            }
            let (metrics, bitmap) = self
                .cache
                .entry(glyph.key)
                .or_insert_with(|| self.font.rasterize_config(glyph.key));
            for row in 0..metrics.height {
                let y = glyph.y.floor() as i64 + row as i64;
                if y < i64::from(bounds.y).max(0) || y >= bottom {
                    continue;
                }
                for column in 0..metrics.width {
                    let x = glyph.x.floor() as i64 + column as i64;
                    if x < i64::from(bounds.x).max(0) || x >= right {
                        continue;
                    }
                    let alpha = (u16::from(bitmap[row * metrics.width + column])
                        * u16::from(color.a)
                        / 255) as u8;
                    framebuffer.blend_rgba8(
                        x as u32,
                        y as u32,
                        &[color.r, color.g, color.b, alpha],
                    );
                }
            }
        }
        measured
    }

    /// Paints outline text through a higher-resolution temporary surface and
    /// box-downsamples it back to the logical UI size.
    ///
    /// This keeps layout coordinates and apparent text size unchanged while
    /// increasing raster precision. It is primarily useful for dense glyphs
    /// (notably CJK) that lose too much detail at small pixel sizes.
    ///
    /// `raster_scale` is clamped to `1..=MAX_TEXT_RASTER_SCALE`. A scale of 1 is
    /// exactly equivalent to [`OutlineFont::draw`]. If temporary dimensions
    /// overflow, rendering safely falls back to the normal path.
    pub fn draw_supersampled(
        &mut self,
        framebuffer: &mut Framebuffer,
        text: &str,
        px: f32,
        bounds: Rect,
        color: Pixel,
        raster_scale: u32,
    ) -> Size {
        let raster_scale = raster_scale.clamp(1, MAX_TEXT_RASTER_SCALE);
        if raster_scale == 1 || bounds.width == 0 || bounds.height == 0 {
            return self.draw(framebuffer, text, px, bounds, color);
        }

        let Some(temp_width) = bounds.width.checked_mul(raster_scale) else {
            return self.draw(framebuffer, text, px, bounds, color);
        };
        let Some(temp_height) = bounds.height.checked_mul(raster_scale) else {
            return self.draw(framebuffer, text, px, bounds, color);
        };

        let mut temp = Framebuffer::new(temp_width, temp_height);
        temp.clear(Pixel::rgba(0, 0, 0, 0));
        let temp_bounds = Rect {
            x: 0,
            y: 0,
            width: temp_width,
            height: temp_height,
        };
        let measured = self.draw(
            &mut temp,
            text,
            px * raster_scale as f32,
            temp_bounds,
            color,
        );

        let pixels = temp.as_rgba8();
        let samples_per_pixel = raster_scale.saturating_mul(raster_scale);
        for logical_y in 0..bounds.height {
            let dest_y = i64::from(bounds.y) + i64::from(logical_y);
            if dest_y < 0 || dest_y >= i64::from(framebuffer.height()) {
                continue;
            }
            for logical_x in 0..bounds.width {
                let dest_x = i64::from(bounds.x) + i64::from(logical_x);
                if dest_x < 0 || dest_x >= i64::from(framebuffer.width()) {
                    continue;
                }

                let mut alpha_sum = 0_u32;
                let source_x = logical_x * raster_scale;
                let source_y = logical_y * raster_scale;
                for sample_y in 0..raster_scale {
                    for sample_x in 0..raster_scale {
                        let x = source_x + sample_x;
                        let y = source_y + sample_y;
                        let index = ((y * temp_width + x) * 4 + 3) as usize;
                        alpha_sum = alpha_sum.saturating_add(u32::from(pixels[index]));
                    }
                }

                let alpha = ((alpha_sum + samples_per_pixel / 2) / samples_per_pixel) as u8;
                if alpha == 0 {
                    continue;
                }
                framebuffer.blend_rgba8(
                    dest_x as u32,
                    dest_y as u32,
                    &[color.r, color.g, color.b, alpha],
                );
            }
        }

        Size {
            width: div_ceil_u32(measured.width, raster_scale),
            height: div_ceil_u32(measured.height, raster_scale),
        }
    }
}

fn div_ceil_u32(value: u32, divisor: u32) -> u32 {
    value.div_ceil(divisor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::fonts::EXO_2;

    fn opaque_pixel_count(framebuffer: &Framebuffer) -> usize {
        framebuffer
            .as_rgba8()
            .chunks_exact(4)
            .filter(|pixel| pixel[3] != 0)
            .count()
    }

    #[test]
    fn bundled_font_reports_text_coverage() {
        let font = OutlineFont::from_bytes(EXO_2).expect("bundled font");
        assert!(font.supports_text("GPE Français"));
        assert!(!font.supports_text("日本語"));
    }

    #[test]
    fn supersampled_scale_one_matches_regular_draw() {
        let mut font = OutlineFont::from_bytes(EXO_2).expect("bundled font");
        let bounds = Rect {
            x: 2,
            y: 2,
            width: 80,
            height: 24,
        };
        let mut regular = Framebuffer::new(96, 32);
        let mut supersampled = Framebuffer::new(96, 32);
        regular.clear(Pixel::rgba(0, 0, 0, 0));
        supersampled.clear(Pixel::rgba(0, 0, 0, 0));

        let regular_size = font.draw(&mut regular, "GPE", 14.0, bounds, Pixel::WHITE);
        let supersampled_size =
            font.draw_supersampled(&mut supersampled, "GPE", 14.0, bounds, Pixel::WHITE, 1);

        assert_eq!(regular_size, supersampled_size);
        assert_eq!(regular.as_rgba8(), supersampled.as_rgba8());
    }

    #[test]
    fn supersampled_draw_keeps_logical_size_and_produces_coverage() {
        let mut font = OutlineFont::from_bytes(EXO_2).expect("bundled font");
        let bounds = Rect {
            x: 0,
            y: 0,
            width: 160,
            height: 48,
        };
        let mut framebuffer = Framebuffer::new(160, 48);
        framebuffer.clear(Pixel::rgba(0, 0, 0, 0));

        let size = font.draw_supersampled(&mut framebuffer, "GPE", 18.0, bounds, Pixel::WHITE, 3);

        assert!(size.width > 0);
        assert!(size.height > 0);
        assert!(opaque_pixel_count(&framebuffer) > 0);
    }
}
