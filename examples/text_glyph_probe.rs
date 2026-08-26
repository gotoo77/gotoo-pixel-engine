use gotoo_pixel_engine::{
    EngineConfig, EngineError, Font, Frame, Framebuffer, Game, GameResult, Key, Pixel, Rect, run,
};

const FRAMEBUFFER_WIDTH: u32 = 320;
const FRAMEBUFFER_HEIGHT: u32 = 180;
const ASCII_FIRST: u8 = 0x20;
const ASCII_LAST: u8 = 0x7E;
const PAGE_SIZE: u8 = 16;
const PAGE_COUNT: usize = 6;

const BG: Pixel = Pixel::rgb(8, 10, 14);
const FG: Pixel = Pixel::rgb(236, 238, 240);
const MUTED: Pixel = Pixel::rgb(150, 158, 170);
const ACCENT: Pixel = Pixel::rgb(116, 170, 255);
const PASS: Pixel = Pixel::rgb(120, 225, 150);
const FAIL: Pixel = Pixel::rgb(255, 110, 110);
const BORDER: Pixel = Pixel::rgb(64, 76, 94);

struct GlyphProbe {
    font: Font,
    page: usize,
}

impl GlyphProbe {
    fn new() -> Self {
        Self {
            font: Font::Pixel5x7,
            page: 0,
        }
    }

    fn update_input(&mut self, frame: &Frame<'_>) -> GameResult {
        if frame.input.key(Key::Escape).pressed() {
            return GameResult::Exit;
        }

        if frame.input.key(Key::Left).pressed() {
            self.page = self.page.saturating_sub(1);
        }
        if frame.input.key(Key::Right).pressed() {
            self.page = (self.page + 1).min(PAGE_COUNT - 1);
        }
        if frame.input.key(Key::F).pressed() {
            self.font = match self.font {
                Font::Pixel5x7 => Font::Mini3x5,
                Font::Mini3x5 => Font::Pixel5x7,
            };
        }

        GameResult::Continue
    }

    fn draw(&self, framebuffer: &mut Framebuffer) {
        framebuffer.clear(BG);

        let fallback_count = printable_ascii_fallback_count(self.font);
        let coverage = if fallback_count == 0 { "PASS" } else { "FAIL" };
        framebuffer.draw_text(
            8,
            6,
            &format!(
                "TEXT GLYPH PROBE  FONT {}  PAGE {} OF {}",
                safe_font_name(self.font),
                self.page + 1,
                PAGE_COUNT
            ),
            FG,
        );
        framebuffer.draw_text(
            8,
            16,
            &format!("PRINTABLE ASCII  FALLBACKS {}  {}", fallback_count, coverage),
            if fallback_count == 0 { PASS } else { FAIL },
        );

        let first = ASCII_FIRST.saturating_add((self.page as u8).saturating_mul(PAGE_SIZE));
        for slot in 0..PAGE_SIZE {
            let code = first.saturating_add(slot);
            if code > ASCII_LAST {
                break;
            }

            let column = i32::from(slot % 4);
            let row = i32::from(slot / 4);
            let x = column * 80;
            let y = 32 + row * 31;
            draw_cell(
                framebuffer,
                self.font,
                code,
                Rect {
                    x,
                    y,
                    width: 80,
                    height: 30,
                },
            );
        }

        framebuffer.draw_text(8, 163, "LEFT RIGHT PAGE   F FONT   ESC QUIT", MUTED);
    }
}

impl Game for GlyphProbe {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let result = self.update_input(frame);
        self.draw(frame.framebuffer);
        result
    }
}

fn draw_cell(framebuffer: &mut Framebuffer, font: Font, code: u8, rect: Rect) {
    framebuffer.draw_rect(rect.x, rect.y, rect.width, rect.height, BORDER);

    let character = char::from(code);
    let fallback = is_accidental_fallback(font, character);

    framebuffer.draw_text(
        rect.x + 3,
        rect.y + 3,
        &format!("{:02X} {}", code, ascii_name(code)),
        MUTED,
    );

    framebuffer.draw_text_scaled_with_font(
        font,
        rect.x + 5,
        rect.y + 13,
        &character.to_string(),
        2,
        ACCENT,
    );

    if fallback {
        draw_fallback_marker(framebuffer, rect.x + 22, rect.y + 15);
        framebuffer.draw_text(rect.x + 32, rect.y + 17, "MISS", FAIL);
    } else {
        framebuffer.draw_text(rect.x + 22, rect.y + 17, "OK", PASS);
    }
}

fn draw_fallback_marker(framebuffer: &mut Framebuffer, x: i32, y: i32) {
    const SIZE: i32 = 7;
    framebuffer.draw_rect(x, y, SIZE as u32, SIZE as u32, FAIL);
    for offset in 1..(SIZE - 1) {
        framebuffer.fill_rect(x + offset, y + offset, 1, 1, FAIL);
        framebuffer.fill_rect(x + (SIZE - 1 - offset), y + offset, 1, 1, FAIL);
    }
}

fn printable_ascii_fallback_count(font: Font) -> usize {
    (ASCII_FIRST..=ASCII_LAST)
        .map(char::from)
        .filter(|&character| is_accidental_fallback(font, character))
        .count()
}

fn is_accidental_fallback(font: Font, character: char) -> bool {
    character != '?' && glyph_signature(font, character) == glyph_signature(font, '?')
}

fn glyph_signature(font: Font, character: char) -> Vec<u8> {
    let mut framebuffer = Framebuffer::new(5, 7);
    framebuffer.clear(Pixel::BLACK);
    framebuffer.draw_text_with_font(font, 0, 0, &character.to_string(), Pixel::WHITE);
    framebuffer.as_rgba8().to_vec()
}

fn safe_font_name(font: Font) -> &'static str {
    match font {
        Font::Pixel5x7 => "PIXEL5X7",
        Font::Mini3x5 => "MINI3X5",
    }
}

fn ascii_name(code: u8) -> String {
    match code {
        0x20 => "SPACE".into(),
        0x21 => "EXCLAM".into(),
        0x22 => "QUOTE".into(),
        0x23 => "HASH".into(),
        0x24 => "DOLLAR".into(),
        0x25 => "PERCENT".into(),
        0x26 => "AMP".into(),
        0x27 => "APOSTROPHE".into(),
        0x28 => "LPAREN".into(),
        0x29 => "RPAREN".into(),
        0x2A => "ASTERISK".into(),
        0x2B => "PLUS".into(),
        0x2C => "COMMA".into(),
        0x2D => "MINUS".into(),
        0x2E => "DOT".into(),
        0x2F => "SLASH".into(),
        0x30..=0x39 => format!("DIGIT {}", char::from(code)),
        0x3A => "COLON".into(),
        0x3B => "SEMICOLON".into(),
        0x3C => "LESS".into(),
        0x3D => "EQUAL".into(),
        0x3E => "GREATER".into(),
        0x3F => "QUESTION".into(),
        0x40 => "AT".into(),
        0x41..=0x5A => format!("UPPER {}", char::from(code)),
        0x5B => "LBRACKET".into(),
        0x5C => "BACKSLASH".into(),
        0x5D => "RBRACKET".into(),
        0x5E => "CARET".into(),
        0x5F => "UNDERSCORE".into(),
        0x60 => "BACKTICK".into(),
        0x61..=0x7A => format!("LOWER {}", char::from(code).to_ascii_uppercase()),
        0x7B => "LBRACE".into(),
        0x7C => "PIPE".into(),
        0x7D => "RBRACE".into(),
        0x7E => "TILDE".into(),
        _ => "UNKNOWN".into(),
    }
}

fn main() -> Result<(), EngineError> {
    run(
        EngineConfig {
            title: "GPE Text Glyph Conformance Probe".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width: FRAMEBUFFER_WIDTH * 3,
            window_height: FRAMEBUFFER_HEIGHT * 3,
        },
        GlyphProbe::new(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pages_cover_printable_ascii_exactly() {
        assert_eq!(ASCII_FIRST, 0x20);
        assert_eq!(ASCII_LAST, 0x7E);
        assert_eq!(PAGE_COUNT * usize::from(PAGE_SIZE), 96);
        assert_eq!(usize::from(ASCII_LAST - ASCII_FIRST + 1), 95);
    }

    #[test]
    fn bracket_names_are_safe_labels() {
        assert_eq!(ascii_name(0x5B), "LBRACKET");
        assert_eq!(ascii_name(0x5D), "RBRACKET");
    }

    #[test]
    fn question_mark_is_reference_glyph_not_fallback_failure() {
        for font in [Font::Pixel5x7, Font::Mini3x5] {
            assert!(!is_accidental_fallback(font, '?'));
        }
    }
}
