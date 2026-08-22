use crate::{Framebuffer, Pixel};

/// One glyph in a user-defined bitmap font.
///
/// Each row stores the left-to-right pixels in the low `width` bits of a `u16`.
/// A set bit draws one pixel; an unset bit stays transparent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitmapGlyph {
    character: char,
    rows: &'static [u16],
}

impl BitmapGlyph {
    pub const fn new(character: char, rows: &'static [u16]) -> Self {
        Self { character, rows }
    }

    pub const fn character(self) -> char {
        self.character
    }

    pub const fn rows(self) -> &'static [u16] {
        self.rows
    }
}

/// Immutable descriptor for a small fixed-width bitmap font authored by a game.
///
/// GPE owns the renderer and metrics while the game owns the actual glyph data.
/// This keeps game-specific visual identity out of the engine without forcing the
/// game to reimplement text layout or framebuffer rasterization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitmapFont {
    name: &'static str,
    glyph_width: u32,
    glyph_height: u32,
    glyph_spacing: u32,
    glyphs: &'static [BitmapGlyph],
    fallback: char,
}

impl BitmapFont {
    pub const fn new(
        name: &'static str,
        glyph_width: u32,
        glyph_height: u32,
        glyph_spacing: u32,
        glyphs: &'static [BitmapGlyph],
        fallback: char,
    ) -> Self {
        Self {
            name,
            glyph_width,
            glyph_height,
            glyph_spacing,
            glyphs,
            fallback,
        }
    }

    pub const fn name(self) -> &'static str {
        self.name
    }

    pub const fn glyph_width(self) -> u32 {
        self.glyph_width
    }

    pub const fn glyph_height(self) -> u32 {
        self.glyph_height
    }

    pub const fn glyph_spacing(self) -> u32 {
        self.glyph_spacing
    }

    pub const fn fallback(self) -> char {
        self.fallback
    }

    fn glyph(self, character: char) -> Option<BitmapGlyph> {
        self.glyphs
            .iter()
            .copied()
            .find(|glyph| glyph.character == character)
            .or_else(|| {
                self.glyphs
                    .iter()
                    .copied()
                    .find(|glyph| glyph.character == self.fallback)
            })
    }
}

/// Renderer for a game-authored fixed-width bitmap font.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitmapTextRenderer {
    font: &'static BitmapFont,
}

impl BitmapTextRenderer {
    pub const fn new(font: &'static BitmapFont) -> Self {
        Self { font }
    }

    pub const fn font(self) -> &'static BitmapFont {
        self.font
    }

    pub fn draw(self, framebuffer: &mut Framebuffer, x: i32, y: i32, text: &str, pixel: Pixel) {
        self.draw_scaled(framebuffer, x, y, text, 1, pixel);
    }

    pub fn draw_scaled(
        self,
        framebuffer: &mut Framebuffer,
        x: i32,
        y: i32,
        text: &str,
        scale: u32,
        pixel: Pixel,
    ) {
        let scale = scale.max(1);
        let advance = self
            .font
            .glyph_width
            .saturating_add(self.font.glyph_spacing)
            .saturating_mul(scale);
        let mut cursor_x = i64::from(x);

        for character in text.chars() {
            if let Some(glyph) = self.font.glyph(character) {
                draw_glyph(
                    framebuffer,
                    self.font,
                    glyph,
                    cursor_x,
                    i64::from(y),
                    scale,
                    pixel,
                );
            }
            cursor_x = cursor_x.saturating_add(i64::from(advance));
        }
    }

    pub fn text_size(self, text: &str, scale: u32) -> (u32, u32) {
        let scale = scale.max(1);
        let count = u32::try_from(text.chars().count()).unwrap_or(u32::MAX);
        if count == 0 {
            return (0, 0);
        }

        let glyph_width = self.font.glyph_width.saturating_mul(scale);
        let spacing = self.font.glyph_spacing.saturating_mul(scale);
        let width = glyph_width
            .saturating_mul(count)
            .saturating_add(spacing.saturating_mul(count.saturating_sub(1)));
        let height = self.font.glyph_height.saturating_mul(scale);
        (width, height)
    }
}

fn draw_glyph(
    framebuffer: &mut Framebuffer,
    font: &BitmapFont,
    glyph: BitmapGlyph,
    x: i64,
    y: i64,
    scale: u32,
    pixel: Pixel,
) {
    let width = font.glyph_width.min(16);
    let height = usize::try_from(font.glyph_height).unwrap_or(usize::MAX);

    for (row_index, row) in glyph.rows.iter().copied().take(height).enumerate() {
        for column in 0..width {
            let mask = 1_u16 << (width - 1 - column);
            if row & mask == 0 {
                continue;
            }

            let pixel_x = x.saturating_add(i64::from(column.saturating_mul(scale)));
            let pixel_y = y.saturating_add(row_index as i64 * i64::from(scale));
            let (Ok(pixel_x), Ok(pixel_y)) = (i32::try_from(pixel_x), i32::try_from(pixel_y))
            else {
                continue;
            };
            framebuffer.fill_rect(pixel_x, pixel_y, scale, scale, pixel);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_GLYPHS: &[BitmapGlyph] = &[
        BitmapGlyph::new('A', &[0b010, 0b101, 0b111, 0b101, 0b101]),
        BitmapGlyph::new('=', &[0b000, 0b111, 0b000, 0b111, 0b000]),
        BitmapGlyph::new('?', &[0b110, 0b001, 0b010, 0b000, 0b010]),
        BitmapGlyph::new(' ', &[0, 0, 0, 0, 0]),
    ];
    static TEST_FONT: BitmapFont = BitmapFont::new("test_3x5", 3, 5, 1, TEST_GLYPHS, '?');

    fn rendered(text: &str) -> Vec<u8> {
        let renderer = BitmapTextRenderer::new(&TEST_FONT);
        let (width, height) = renderer.text_size(text, 1);
        let mut framebuffer = Framebuffer::new(width.max(1), height.max(1));
        renderer.draw(&mut framebuffer, 0, 0, text, Pixel::WHITE);
        framebuffer.as_rgba8().to_vec()
    }

    #[test]
    fn custom_font_renders_authored_punctuation_without_placeholder() {
        assert_ne!(rendered("="), rendered("?"));
    }

    #[test]
    fn custom_font_uses_declared_fallback_for_unknown_glyphs() {
        assert_eq!(rendered("@"), rendered("?"));
    }

    #[test]
    fn custom_font_metrics_are_fixed_width_and_scaled() {
        let renderer = BitmapTextRenderer::new(&TEST_FONT);
        assert_eq!(renderer.text_size("AA", 1), (7, 5));
        assert_eq!(renderer.text_size("AA", 2), (14, 10));
    }
}
