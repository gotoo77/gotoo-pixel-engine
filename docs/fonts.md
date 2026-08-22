# Authored fonts

GPE keeps its built-in bitmap fonts (`Font::Pixel5x7` and `Font::Mini3x5`) for small pixel-first UI, and also supports game-owned TrueType/OpenType faces through `FontFace`.

The engine does not own a global font registry. A game owns the faces it needs, just like any other game asset, and can keep several faces alive at the same time.

```rust
use gotoo_pixel_engine::{FontFace, Framebuffer, Pixel};

static TITLE_FONT_BYTES: &[u8] = include_bytes!("../assets/fonts/title.ttf");
static UI_FONT_BYTES: &[u8] = include_bytes!("../assets/fonts/ui.ttf");

fn draw_screen(framebuffer: &mut Framebuffer) {
    let title_font = FontFace::from_static_bytes(TITLE_FONT_BYTES)
        .expect("title font should be valid");
    let ui_font = FontFace::from_static_bytes(UI_FONT_BYTES)
        .expect("UI font should be valid");

    title_font.draw_text(framebuffer, 20, 20, "THE CANTICLE", 30.0, Pixel::WHITE);
    ui_font.draw_text(framebuffer, 20, 64, "HULL 100", 12.0, Pixel::WHITE);
}
```

In a real game, parse the faces once during initialization and store the resulting `FontFace` values instead of rebuilding them every frame.

`FontFace::text_size` returns layout dimensions using the font's advance, kerning and line metrics, so authored text can be centered or aligned without hard-coded widths.

The same CPU rasterization path is used on native and Web/WASM. Font files remain assets of the consuming game; their licensing and redistribution terms are therefore the game's responsibility.
