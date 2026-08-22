use crate::{Font, Framebuffer, Pixel};

/// Lightweight text renderer layered on top of the built-in bitmap fonts.
///
/// The historical `Framebuffer::draw_text*` API remains available and keeps its
/// deterministic placeholder behavior for unknown glyphs. `TextRenderer` is the
/// managed path for UI/i18n text: it adds a small set of punctuation and Latin
/// glyphs that are required by real game interfaces while preserving the same
/// fixed-width metrics as the selected built-in font.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextRenderer {
    font: Font,
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new(Font::default())
    }
}

impl TextRenderer {
    pub const fn new(font: Font) -> Self {
        Self { font }
    }

    pub const fn font(self) -> Font {
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
        let mut cursor_x = x;
        let advance = glyph_advance(self.font, scale);

        for character in text.chars() {
            if let Some(glyph) = managed_glyph(self.font, character) {
                draw_managed_glyph(framebuffer, self.font, cursor_x, y, glyph, scale, pixel);
            } else {
                let normalized = normalize_latin_character(character);
                let mut encoded = [0_u8; 4];
                let text = normalized.encode_utf8(&mut encoded);
                framebuffer.draw_text_scaled_with_font(self.font, cursor_x, y, text, scale, pixel);
            }
            cursor_x = cursor_x.saturating_add(advance);
        }
    }

    pub fn text_size(self, text: &str, scale: u32) -> (u32, u32) {
        Framebuffer::text_size_with_font(self.font, text, scale)
    }
}

fn glyph_advance(font: Font, scale: u32) -> i32 {
    let scale = scale.max(1);
    let (single, _) = Framebuffer::text_size_with_font(font, "A", scale);
    let (pair, _) = Framebuffer::text_size_with_font(font, "AA", scale);
    i32::try_from(pair.saturating_sub(single)).unwrap_or(i32::MAX)
}

fn font_dimensions(font: Font) -> (u32, usize) {
    match font {
        Font::Pixel5x7 => (5, 7),
        Font::Mini3x5 => (3, 5),
    }
}

fn draw_managed_glyph(
    framebuffer: &mut Framebuffer,
    font: Font,
    x: i32,
    y: i32,
    glyph: &'static [u8],
    scale: u32,
    pixel: Pixel,
) {
    let scale = scale.max(1);
    let (width, height) = font_dimensions(font);

    for (row_index, row) in glyph.iter().copied().take(height).enumerate() {
        for column in 0..width {
            let mask = 1 << (width - 1 - column);
            if row & mask == 0 {
                continue;
            }

            let pixel_x = i64::from(x) + i64::from(column.saturating_mul(scale));
            let pixel_y = i64::from(y) + row_index as i64 * i64::from(scale);
            let (Ok(pixel_x), Ok(pixel_y)) = (i32::try_from(pixel_x), i32::try_from(pixel_y))
            else {
                continue;
            };
            framebuffer.fill_rect(pixel_x, pixel_y, scale, scale, pixel);
        }
    }
}

fn normalize_latin_character(character: char) -> char {
    match character {
        'à' | 'á' | 'â' | 'ä' | 'ã' | 'å' | 'À' | 'Á' | 'Â' | 'Ä' | 'Ã' | 'Å' => 'A',
        'ç' | 'Ç' => 'C',
        'è' | 'é' | 'ê' | 'ë' | 'È' | 'É' | 'Ê' | 'Ë' => 'E',
        'ì' | 'í' | 'î' | 'ï' | 'Ì' | 'Í' | 'Î' | 'Ï' => 'I',
        'ñ' | 'Ñ' => 'N',
        'ò' | 'ó' | 'ô' | 'ö' | 'õ' | 'Ò' | 'Ó' | 'Ô' | 'Ö' | 'Õ' => 'O',
        'ù' | 'ú' | 'û' | 'ü' | 'Ù' | 'Ú' | 'Û' | 'Ü' => 'U',
        'ý' | 'ÿ' | 'Ý' | 'Ÿ' => 'Y',
        _ => character.to_ascii_uppercase(),
    }
}

fn managed_glyph(font: Font, character: char) -> Option<&'static [u8]> {
    let character = match character {
        'à' => 'À',
        'â' => 'Â',
        'ç' => 'Ç',
        'è' => 'È',
        'é' => 'É',
        'ê' => 'Ê',
        'ù' => 'Ù',
        'û' => 'Û',
        'ü' => 'Ü',
        other => other,
    };

    match font {
        Font::Pixel5x7 => match character {
            '%' => Some(&[
                0b11001, 0b11010, 0b00100, 0b01011, 0b10011, 0b00000, 0b00000,
            ]),
            '*' => Some(&[
                0b00000, 0b10101, 0b01110, 0b11111, 0b01110, 0b10101, 0b00000,
            ]),
            '\'' => Some(&[
                0b00100, 0b00100, 0b00010, 0b00000, 0b00000, 0b00000, 0b00000,
            ]),
            'À' => Some(&[
                0b01000, 0b01110, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
            ]),
            'Â' => Some(&[
                0b01010, 0b01110, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
            ]),
            'Ç' => Some(&[
                0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111, 0b00100,
            ]),
            'È' => Some(&[
                0b01000, 0b11111, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
            ]),
            'É' => Some(&[
                0b00100, 0b11111, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
            ]),
            'Ê' => Some(&[
                0b01010, 0b11111, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
            ]),
            'Ù' => Some(&[
                0b01000, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
            ]),
            'Û' => Some(&[
                0b01010, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
            ]),
            'Ü' => Some(&[
                0b10001, 0b00000, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
            ]),
            _ => None,
        },
        Font::Mini3x5 => match character {
            '%' => Some(&[0b101, 0b001, 0b010, 0b100, 0b101]),
            '*' => Some(&[0b000, 0b101, 0b010, 0b101, 0b000]),
            '\'' => Some(&[0b010, 0b010, 0b000, 0b000, 0b000]),
            'À' => Some(&[0b100, 0b111, 0b101, 0b111, 0b101]),
            'Â' => Some(&[0b101, 0b111, 0b101, 0b111, 0b101]),
            'Ç' => Some(&[0b111, 0b100, 0b100, 0b111, 0b010]),
            'È' => Some(&[0b100, 0b111, 0b110, 0b100, 0b111]),
            'É' => Some(&[0b010, 0b111, 0b110, 0b100, 0b111]),
            'Ê' => Some(&[0b101, 0b111, 0b110, 0b100, 0b111]),
            'Ù' => Some(&[0b100, 0b101, 0b101, 0b101, 0b111]),
            'Û' => Some(&[0b101, 0b101, 0b101, 0b101, 0b111]),
            'Ü' => Some(&[0b101, 0b000, 0b101, 0b101, 0b111]),
            _ => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rendered(font: Font, text: &str) -> Vec<u8> {
        let renderer = TextRenderer::new(font);
        let (width, height) = renderer.text_size(text, 1);
        let mut framebuffer = Framebuffer::new(width.max(1), height.max(1));
        renderer.draw(&mut framebuffer, 0, 0, text, Pixel::WHITE);
        framebuffer.as_rgba8().to_vec()
    }

    #[test]
    fn managed_renderer_covers_ui_punctuation_without_placeholder() {
        for font in [Font::Pixel5x7, Font::Mini3x5] {
            let placeholder = rendered(font, "?");
            for character in ["%", "*", "'"] {
                assert_ne!(
                    rendered(font, character),
                    placeholder,
                    "{font:?} {character}"
                );
            }
        }
    }

    #[test]
    fn managed_renderer_preserves_common_french_diacritics() {
        for font in [Font::Pixel5x7, Font::Mini3x5] {
            let placeholder = rendered(font, "?");
            assert_ne!(rendered(font, "É"), placeholder);
            assert_ne!(rendered(font, "È"), placeholder);
            assert_ne!(rendered(font, "Ç"), placeholder);
            assert_ne!(rendered(font, "À"), placeholder);
        }
    }

    #[test]
    fn unsupported_latin_diacritics_fall_back_to_readable_base_letters() {
        for font in [Font::Pixel5x7, Font::Mini3x5] {
            assert_eq!(rendered(font, "Ï"), rendered(font, "I"));
            assert_eq!(rendered(font, "Ö"), rendered(font, "O"));
        }
    }

    #[test]
    fn managed_renderer_keeps_builtin_fixed_width_metrics() {
        for font in [Font::Pixel5x7, Font::Mini3x5] {
            let renderer = TextRenderer::new(font);
            assert_eq!(
                renderer.text_size("100% ÉCHO", 2),
                Framebuffer::text_size_with_font(font, "100% ÉCHO", 2)
            );
        }
    }
}
