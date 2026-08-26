from pathlib import Path

path = Path("src/framebuffer.rs")
text = path.read_text()

pixel_anchor = """        '>' => [
            0b10000, 0b01000, 0b00100, 0b00010, 0b00100, 0b01000, 0b10000,
        ],
"""
pixel_insert = r"""        '"' => [
            0b01010, 0b01010, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '#' => [
            0b01010, 0b11111, 0b01010, 0b11111, 0b01010, 0b00000, 0b00000,
        ],
        '$' => [
            0b00100, 0b01111, 0b10100, 0b01110, 0b00101, 0b11110, 0b00100,
        ],
        '%' => [
            0b11001, 0b11010, 0b00100, 0b01011, 0b10011, 0b00000, 0b00000,
        ],
        '&' => [
            0b01100, 0b10010, 0b10100, 0b01000, 0b10101, 0b10010, 0b01101,
        ],
        '\'' => [
            0b00100, 0b00100, 0b00010, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '(' => [
            0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010,
        ],
        ')' => [
            0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000,
        ],
        '*' => [
            0b00000, 0b10101, 0b01110, 0b11111, 0b01110, 0b10101, 0b00000,
        ],
        ';' => [
            0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b01000,
        ],
        '<' => [
            0b00001, 0b00010, 0b00100, 0b01000, 0b00100, 0b00010, 0b00001,
        ],
        '=' => [
            0b00000, 0b11111, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ],
        '@' => [
            0b01110, 0b10001, 0b10111, 0b10101, 0b10111, 0b10000, 0b01110,
        ],
        '[' => [
            0b01110, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01110,
        ],
        '\\' => [
            0b10000, 0b01000, 0b01000, 0b00100, 0b00010, 0b00010, 0b00001,
        ],
        ']' => [
            0b01110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b01110,
        ],
        '^' => [
            0b00100, 0b01010, 0b10001, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '_' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111,
        ],
        '`' => [
            0b01000, 0b00100, 0b00010, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '{' => [
            0b00011, 0b00100, 0b00100, 0b01000, 0b00100, 0b00100, 0b00011,
        ],
        '|' => [
            0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        '}' => [
            0b11000, 0b00100, 0b00100, 0b00010, 0b00100, 0b00100, 0b11000,
        ],
        '~' => [
            0b00000, 0b00000, 0b01001, 0b10110, 0b00000, 0b00000, 0b00000,
        ],
"""
if text.count(pixel_anchor) != 1:
    raise SystemExit("Pixel5x7 anchor mismatch")
text = text.replace(pixel_anchor, pixel_insert + pixel_anchor, 1)

mini_anchor = """        '>' => [0b100, 0b010, 0b001, 0b010, 0b100, 0, 0],
"""
mini_insert = r"""        '"' => [0b101, 0b101, 0b000, 0b000, 0b000, 0, 0],
        '#' => [0b101, 0b111, 0b101, 0b111, 0b101, 0, 0],
        '$' => [0b010, 0b111, 0b110, 0b011, 0b111, 0, 0],
        '%' => [0b101, 0b001, 0b010, 0b100, 0b101, 0, 0],
        '&' => [0b010, 0b101, 0b010, 0b101, 0b011, 0, 0],
        '\'' => [0b010, 0b010, 0b000, 0b000, 0b000, 0, 0],
        '(' => [0b010, 0b100, 0b100, 0b100, 0b010, 0, 0],
        ')' => [0b010, 0b001, 0b001, 0b001, 0b010, 0, 0],
        '*' => [0b000, 0b101, 0b010, 0b101, 0b000, 0, 0],
        ';' => [0b000, 0b010, 0b000, 0b010, 0b100, 0, 0],
        '<' => [0b001, 0b010, 0b100, 0b010, 0b001, 0, 0],
        '=' => [0b000, 0b111, 0b000, 0b111, 0b000, 0, 0],
        '@' => [0b111, 0b101, 0b111, 0b100, 0b111, 0, 0],
        '[' => [0b110, 0b100, 0b100, 0b100, 0b110, 0, 0],
        '\\' => [0b100, 0b100, 0b010, 0b001, 0b001, 0, 0],
        ']' => [0b011, 0b001, 0b001, 0b001, 0b011, 0, 0],
        '^' => [0b010, 0b101, 0b000, 0b000, 0b000, 0, 0],
        '_' => [0b000, 0b000, 0b000, 0b000, 0b111, 0, 0],
        '`' => [0b100, 0b010, 0b000, 0b000, 0b000, 0, 0],
        '{' => [0b011, 0b010, 0b100, 0b010, 0b011, 0, 0],
        '|' => [0b010, 0b010, 0b010, 0b010, 0b010, 0, 0],
        '}' => [0b110, 0b010, 0b001, 0b010, 0b110, 0, 0],
        '~' => [0b000, 0b000, 0b101, 0b010, 0b000, 0, 0],
"""
if text.count(mini_anchor) != 1:
    raise SystemExit("Mini3x5 anchor mismatch")
text = text.replace(mini_anchor, mini_insert + mini_anchor, 1)

old_test = """    #[test]
    fn menu_punctuation_has_explicit_glyphs() {
        for font in [Font::Pixel5x7, Font::Mini3x5] {
            assert_ne!(glyph_for(font, '>'), glyph_for(font, '?'));
            assert_ne!(glyph_for(font, '/'), glyph_for(font, '?'));
            assert_ne!(glyph_for(font, '+'), glyph_for(font, '?'));
            assert_ne!(glyph_for(font, ':'), glyph_for(font, '?'));
            assert_ne!(glyph_for(font, '-'), glyph_for(font, '?'));
            assert_ne!(glyph_for(font, '.'), glyph_for(font, '?'));
            assert_ne!(glyph_for(font, ','), glyph_for(font, '?'));
            assert_ne!(glyph_for(font, '!'), glyph_for(font, '?'));
        }
    }
"""
new_test = """    #[test]
    fn printable_ascii_has_explicit_glyphs() {
        for font in [Font::Pixel5x7, Font::Mini3x5] {
            let fallback = glyph_for(font, '?');
            for code in 0x20_u8..=0x7E {
                let character = char::from(code);
                if character == '?' {
                    continue;
                }
                assert_ne!(
                    glyph_for(font, character),
                    fallback,
                    "{font:?} U+{code:04X} {character:?} unexpectedly uses fallback"
                );
            }
        }
    }
"""
if text.count(old_test) != 1:
    raise SystemExit("glyph coverage test anchor mismatch")
text = text.replace(old_test, new_test, 1)

path.write_text(text)
