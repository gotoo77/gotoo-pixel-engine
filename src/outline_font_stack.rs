//! Generic outline-font fallback selection.
//!
//! This module keeps fallback policy in GPE core while allowing each consumer
//! to provide its own font assets. It deliberately does not hard-code a locale
//! or a country: resolution is based on actual glyph coverage.

use crate::{Framebuffer, Pixel, Rect, Size, outline_text::OutlineFont};

/// Ordered outline-font stack used for glyph-coverage fallback.
///
/// The first font is the primary UI face. Additional fonts are tried in order
/// and the first font that can render the complete string is selected.
pub struct OutlineFontStack {
    fonts: Vec<OutlineFont>,
}

impl OutlineFontStack {
    /// Creates a stack from a primary font.
    pub fn new(primary: OutlineFont) -> Self {
        Self {
            fonts: vec![primary],
        }
    }

    /// Adds one fallback font after the existing faces.
    pub fn push_fallback(&mut self, font: OutlineFont) {
        self.fonts.push(font);
    }

    /// Number of faces in the stack, including the primary face.
    pub fn len(&self) -> usize {
        self.fonts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fonts.is_empty()
    }

    /// Returns the first font index that can render the whole string.
    pub fn resolve_font_index(&self, text: &str) -> Option<usize> {
        self.fonts.iter().position(|font| font.supports_text(text))
    }

    /// True when at least one face can render the complete string.
    pub fn supports_text(&self, text: &str) -> bool {
        self.resolve_font_index(text).is_some()
    }

    /// Measures using the same face that would be selected for drawing.
    pub fn measure(&mut self, text: &str, px: f32, bounds: Rect) -> Option<Size> {
        let index = self.resolve_font_index(text)?;
        Some(self.fonts[index].measure(text, px, bounds))
    }

    /// Draws using the first face with complete glyph coverage.
    ///
    /// Returns `None` rather than substituting placeholder glyphs when no face
    /// can render the full string. Consumers can then make fallback failure
    /// explicit in diagnostics or UI.
    pub fn draw(
        &mut self,
        framebuffer: &mut Framebuffer,
        text: &str,
        px: f32,
        bounds: Rect,
        color: Pixel,
    ) -> Option<Size> {
        let index = self.resolve_font_index(text)?;
        Some(self.fonts[index].draw(framebuffer, text, px, bounds, color))
    }

    /// Draws through GPE's higher-resolution raster path using the selected face.
    pub fn draw_supersampled(
        &mut self,
        framebuffer: &mut Framebuffer,
        text: &str,
        px: f32,
        bounds: Rect,
        color: Pixel,
        raster_scale: u32,
    ) -> Option<Size> {
        let index = self.resolve_font_index(text)?;
        Some(self.fonts[index].draw_supersampled(
            framebuffer,
            text,
            px,
            bounds,
            color,
            raster_scale,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::fonts::{EXO_2, UNBOUNDED};

    #[test]
    fn primary_face_is_selected_when_it_has_complete_coverage() {
        let primary = OutlineFont::from_bytes(EXO_2).expect("bundled font");
        let fallback = OutlineFont::from_bytes(UNBOUNDED).expect("bundled font");
        let mut stack = OutlineFontStack::new(primary);
        stack.push_fallback(fallback);

        assert_eq!(stack.resolve_font_index("GPE UI"), Some(0));
        assert!(stack.supports_text("Français"));
    }

    #[test]
    fn missing_cjk_is_reported_instead_of_silently_replaced() {
        let primary = OutlineFont::from_bytes(EXO_2).expect("bundled font");
        let fallback = OutlineFont::from_bytes(UNBOUNDED).expect("bundled font");
        let mut stack = OutlineFontStack::new(primary);
        stack.push_fallback(fallback);

        assert_eq!(stack.resolve_font_index("日本語"), None);
        assert!(!stack.supports_text("ゲーム開始"));
    }

    #[test]
    fn draw_returns_none_when_no_face_supports_text() {
        let primary = OutlineFont::from_bytes(EXO_2).expect("bundled font");
        let mut stack = OutlineFontStack::new(primary);
        let mut framebuffer = Framebuffer::new(160, 48);
        let bounds = Rect {
            x: 0,
            y: 0,
            width: 160,
            height: 48,
        };

        assert_eq!(
            stack.draw(
                &mut framebuffer,
                "日本語",
                20.0,
                bounds,
                Pixel::WHITE,
            ),
            None
        );
    }
}
