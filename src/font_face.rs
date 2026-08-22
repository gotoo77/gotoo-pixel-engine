use std::fmt;

use ab_glyph::{point, Font as _, FontArc, GlyphId, PxScale, ScaleFont};

use crate::{Framebuffer, Pixel};

/// An authored TrueType/OpenType font owned by the game.
///
/// `FontFace` keeps font choice outside the engine: games can construct as many
/// faces as they need from embedded bytes and select the appropriate face for
/// each draw call. The same CPU rasterization path is used by native and Web/WASM.
#[derive(Clone, Debug)]
pub struct FontFace {
    font: FontArc,
}

/// Error returned when bytes do not contain a supported TrueType/OpenType face.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FontFaceError;

impl fmt::Display for FontFaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid or unsupported font data")
    }
}

impl std::error::Error for FontFaceError {}

impl FontFace {
    /// Parse a font from arbitrary bytes, taking an owned copy of the data.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, FontFaceError> {
        let font = FontArc::try_from_vec(bytes.to_vec()).map_err(|_| FontFaceError)?;
        Ok(Self { font })
    }

    /// Parse a font from static bytes without copying them.
    ///
    /// This is the preferred path for `include_bytes!("...")` assets.
    pub fn from_static_bytes(bytes: &'static [u8]) -> Result<Self, FontFaceError> {
        let font = FontArc::try_from_slice(bytes).map_err(|_| FontFaceError)?;
        Ok(Self { font })
    }

    /// Return layout dimensions for `text` at the requested pixel height.
    ///
    /// The result uses font advances and line metrics; it is suitable for UI
    /// layout and alignment rather than exact painted-pixel bounds.
    pub fn text_size(&self, text: &str, pixel_height: f32) -> (u32, u32) {
        if text.is_empty() {
            return (0, 0);
        }

        let scale = PxScale::from(normalize_pixel_height(pixel_height));
        let scaled = self.font.as_scaled(scale);
        let line_advance = scaled.height() + scaled.line_gap();
        let mut line_width = 0.0_f32;
        let mut max_width = 0.0_f32;
        let mut previous = None;
        let mut line_count = 1_u32;

        for character in text.chars() {
            if character == '\n' {
                max_width = max_width.max(line_width);
                line_width = 0.0;
                previous = None;
                line_count = line_count.saturating_add(1);
                continue;
            }
            if character == '\r' {
                continue;
            }

            let glyph_id = scaled.glyph_id(character);
            if let Some(previous_id) = previous {
                line_width += scaled.kern(previous_id, glyph_id);
            }
            line_width += scaled.h_advance(glyph_id);
            previous = Some(glyph_id);
        }
        max_width = max_width.max(line_width);

        let height = scaled.height() + line_advance * line_count.saturating_sub(1) as f32;
        (ceil_to_u32(max_width), ceil_to_u32(height))
    }

    /// Rasterize `text` directly into the CPU framebuffer.
    ///
    /// `x`/`y` identify the top-left of the first line. `pixel_height` is the
    /// font's rendered pixel height, not a bitmap integer scale factor.
    pub fn draw_text(
        &self,
        framebuffer: &mut Framebuffer,
        x: i32,
        y: i32,
        text: &str,
        pixel_height: f32,
        pixel: Pixel,
    ) {
        if text.is_empty() || pixel.a == 0 {
            return;
        }

        let scale = PxScale::from(normalize_pixel_height(pixel_height));
        let scaled = self.font.as_scaled(scale);
        let line_advance = scaled.height() + scaled.line_gap();
        let mut baseline = y as f32 + scaled.ascent();
        let mut cursor_x = x as f32;
        let mut previous: Option<GlyphId> = None;

        for character in text.chars() {
            if character == '\n' {
                cursor_x = x as f32;
                baseline += line_advance;
                previous = None;
                continue;
            }
            if character == '\r' {
                continue;
            }

            let glyph_id = scaled.glyph_id(character);
            if let Some(previous_id) = previous {
                cursor_x += scaled.kern(previous_id, glyph_id);
            }

            let glyph = glyph_id.with_scale_and_position(scale, point(cursor_x, baseline));
            if let Some(outlined) = self.font.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                let origin_x = bounds.min.x.floor() as i32;
                let origin_y = bounds.min.y.floor() as i32;
                outlined.draw(|glyph_x, glyph_y, coverage| {
                    let Ok(glyph_x) = i32::try_from(glyph_x) else {
                        return;
                    };
                    let Ok(glyph_y) = i32::try_from(glyph_y) else {
                        return;
                    };
                    blend_coverage(
                        framebuffer,
                        origin_x.saturating_add(glyph_x),
                        origin_y.saturating_add(glyph_y),
                        pixel,
                        coverage,
                    );
                });
            }

            cursor_x += scaled.h_advance(glyph_id);
            previous = Some(glyph_id);
        }
    }
}

fn normalize_pixel_height(pixel_height: f32) -> f32 {
    if pixel_height.is_finite() {
        pixel_height.max(1.0)
    } else {
        1.0
    }
}

fn ceil_to_u32(value: f32) -> u32 {
    if !value.is_finite() || value <= 0.0 {
        return 0;
    }
    value.ceil().min(u32::MAX as f32) as u32
}

fn blend_coverage(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    source: Pixel,
    coverage: f32,
) {
    let Some(destination) = framebuffer.pixel(x, y) else {
        return;
    };

    let coverage = coverage.clamp(0.0, 1.0);
    let source_alpha = ((source.a as f32 * coverage).round() as u16).min(255);
    if source_alpha == 0 {
        return;
    }

    let inverse_alpha = 255 - source_alpha;
    let blend_channel = |source_channel: u8, destination_channel: u8| -> u8 {
        ((source_channel as u16 * source_alpha
            + destination_channel as u16 * inverse_alpha
            + 127)
            / 255) as u8
    };
    let output_alpha = source_alpha + ((destination.a as u16 * inverse_alpha + 127) / 255);

    framebuffer.set_pixel_in_bounds(
        x as u32,
        y as u32,
        Pixel::rgba(
            blend_channel(source.r, destination.r),
            blend_channel(source.g, destination.g),
            blend_channel(source.b, destination.b),
            output_alpha.min(255) as u8,
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_font_bytes() {
        assert_eq!(
            FontFace::from_bytes(b"not a font").unwrap_err(),
            FontFaceError
        );
    }

    #[test]
    fn normalizes_invalid_pixel_heights() {
        assert_eq!(normalize_pixel_height(0.0), 1.0);
        assert_eq!(normalize_pixel_height(-4.0), 1.0);
        assert_eq!(normalize_pixel_height(f32::NAN), 1.0);
        assert_eq!(normalize_pixel_height(18.5), 18.5);
    }
}
