//! Generic outline-font fallback selection.
//!
//! This module keeps fallback policy in GPE core while allowing each consumer
//! to provide its own font assets. It deliberately does not hard-code a locale
//! or a country: resolution is based on actual glyph coverage.

use crate::{Framebuffer, Pixel, Rect, Size, outline_text::OutlineFont};

/// One contiguous UTF-8 byte range resolved to one face in an [`OutlineFontStack`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutlineFontRun {
    pub start: usize,
    pub end: usize,
    pub font_index: usize,
}

impl OutlineFontRun {
    pub fn text(self, source: &str) -> &str {
        &source[self.start..self.end]
    }
}

/// Ordered outline-font stack used for glyph-coverage fallback.
///
/// The first font is the primary UI face. Additional fonts are tried in order.
/// Whole-string resolution remains available for simple consumers, while
/// [`OutlineFontStack::resolve_runs`] supports mixed-script strings by assigning
/// each contiguous glyph run to the first face that covers it.
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

    /// Returns the first face that covers one character.
    pub fn resolve_glyph_font_index(&self, character: char) -> Option<usize> {
        self.fonts.iter().position(|font| font.has_glyph(character))
    }

    /// Resolves a mixed-script string into contiguous font runs.
    ///
    /// Resolution is deterministic: each character uses the earliest face in
    /// the stack that contains its glyph. If any non-control character is
    /// missing from every face, `None` is returned rather than drawing a silent
    /// placeholder. Control characters stay on the primary face so callers can
    /// preserve line/control semantics without requiring every fallback font to
    /// contain them.
    pub fn resolve_runs(&self, text: &str) -> Option<Vec<OutlineFontRun>> {
        if text.is_empty() {
            return Some(Vec::new());
        }

        let mut runs = Vec::new();
        let mut current_start = 0;
        let mut current_font = None;

        for (byte_index, character) in text.char_indices() {
            let font_index = if character.is_control() {
                0
            } else {
                self.resolve_glyph_font_index(character)?
            };

            match current_font {
                None => {
                    current_start = byte_index;
                    current_font = Some(font_index);
                }
                Some(active) if active != font_index => {
                    runs.push(OutlineFontRun {
                        start: current_start,
                        end: byte_index,
                        font_index: active,
                    });
                    current_start = byte_index;
                    current_font = Some(font_index);
                }
                Some(_) => {}
            }
        }

        if let Some(font_index) = current_font {
            runs.push(OutlineFontRun {
                start: current_start,
                end: text.len(),
                font_index,
            });
        }

        Some(runs)
    }

    /// True when at least one face can render the complete string.
    pub fn supports_text(&self, text: &str) -> bool {
        self.resolve_font_index(text).is_some()
    }

    /// True when every glyph can be resolved across the stack, even if no one
    /// face contains the entire mixed-script string.
    pub fn supports_text_by_runs(&self, text: &str) -> bool {
        self.resolve_runs(text).is_some()
    }

    /// Measures using the same face that would be selected for whole-string drawing.
    pub fn measure(&mut self, text: &str, px: f32, bounds: Rect) -> Option<Size> {
        let index = self.resolve_font_index(text)?;
        Some(self.fonts[index].measure(text, px, bounds))
    }

    /// Draws using the first face with complete glyph coverage.
    ///
    /// Returns `None` rather than substituting placeholder glyphs when no face
    /// can render the full string. Use [`OutlineFontStack::draw_runs`] for
    /// mixed-script strings whose glyphs are covered by different faces.
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

    /// Draws a mixed-script, single-line string as contiguous fallback runs.
    ///
    /// Each run is measured and painted by its resolved face, then the cursor is
    /// advanced by the measured logical width. Cross-face kerning/shaping is not
    /// attempted; this is intended for UI labels and metadata, not complex text
    /// shaping. Newlines should be laid out by the caller as separate lines.
    pub fn draw_runs(
        &mut self,
        framebuffer: &mut Framebuffer,
        text: &str,
        px: f32,
        bounds: Rect,
        color: Pixel,
        raster_scale: u32,
    ) -> Option<Size> {
        if text.contains('\n') || text.contains('\r') {
            return None;
        }

        let runs = self.resolve_runs(text)?;
        let mut cursor_x = bounds.x;
        let mut total_width = 0_u32;
        let mut max_height = 0_u32;

        for run in runs {
            let remaining_width = bounds.width.saturating_sub(total_width);
            if remaining_width == 0 {
                break;
            }
            let run_bounds = Rect {
                x: cursor_x,
                y: bounds.y,
                width: remaining_width,
                height: bounds.height,
            };
            let size = self.fonts[run.font_index].draw_supersampled(
                framebuffer,
                run.text(text),
                px,
                run_bounds,
                color,
                raster_scale,
            );
            total_width = total_width.saturating_add(size.width).min(bounds.width);
            cursor_x = bounds
                .x
                .saturating_add(i32::try_from(total_width).unwrap_or(i32::MAX));
            max_height = max_height.max(size.height);
        }

        Some(Size {
            width: total_width,
            height: max_height.min(bounds.height),
        })
    }

    /// Draws through GPE's higher-resolution raster path using one selected face.
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
        assert_eq!(
            stack.resolve_runs("GPE UI"),
            Some(vec![OutlineFontRun {
                start: 0,
                end: 6,
                font_index: 0,
            }])
        );
    }

    #[test]
    fn missing_cjk_is_reported_instead_of_silently_replaced() {
        let primary = OutlineFont::from_bytes(EXO_2).expect("bundled font");
        let fallback = OutlineFont::from_bytes(UNBOUNDED).expect("bundled font");
        let mut stack = OutlineFontStack::new(primary);
        stack.push_fallback(fallback);

        assert_eq!(stack.resolve_font_index("日本語"), None);
        assert!(!stack.supports_text("ゲーム開始"));
        assert!(!stack.supports_text_by_runs("START / 日本語 / F1"));
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
            stack.draw(&mut framebuffer, "日本語", 20.0, bounds, Pixel::WHITE,),
            None
        );
        assert_eq!(
            stack.draw_runs(&mut framebuffer, "日本語", 20.0, bounds, Pixel::WHITE, 3,),
            None
        );
    }
}
