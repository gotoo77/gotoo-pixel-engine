# GPE.UI - P4 Typography: First Slice

Status: P4 ACTIVE / BITMAP AND EXPERIMENTAL 20-FONT GALLERY IMPLEMENTED

Start authorized by maintainer: "go P4", 2026-09-05.
This records authorization to proceed locally, not a P3 merge or a CI result.

Source: https://github.com/gotoo77/gotoo-pixel-engine/issues/69

## Verified Audit

- `src/framebuffer.rs`: Pixel5x7 and Mini3x5 already have deterministic missing
  glyphs distinct from the literal question mark. Existing tests cover both.
- `src/text.rs`: managed bitmap rendering adds selected French accents and
  punctuation, with fixed-width measurement delegated to Framebuffer.
  Some Latin accents intentionally normalize to base letters; this is not full
  Unicode coverage, lowercase typography, or text shaping.
- Managed punctuation/accent tests previously compared against `?`, not the
  actual missing glyph. They now use unsupported U+10FFFF as their reference.
- `src/bitmap_font.rs`: custom bitmap fonts retain their declared fallback.
  This slice does not replace consumer-authored fallback policy.
- Searches of src, tests, examples and Cargo.toml found no TTF/OTF renderer or
  outline-font dependency in this checkout. Older follow-up wording about an
  existing/evolving outline path must not be treated as implementation evidence.

## Implemented

TextRenderer supports U+2013/U+2014 dashes, U+2022 bullet, and U+2190 through
U+2193 arrows in both built-in fonts. The 3x5 font intentionally uses the same
three-pixel stroke for both dashes. Left/right arrows remain distinct.

The legacy Framebuffer glyph tables, custom bitmap font fallback, font metrics,
public API and dependency set remain unchanged.

`examples/gpe_ui_p4_typography_probe.rs` renders a comparison of both fonts at
integer scales 1 and 2, with UI text, French accents, punctuation, literal `?`
and the actual missing-glyph symbol.

```powershell
rtk cargo run --example gpe_ui_p4_typography_probe
rtk cargo run --example gpe_ui_p4_typography_probe -- target/p4-typography.png
```

## Validation

- `rtk cargo test text::tests --lib`: 5 passed.
- `rtk cargo check --target wasm32-unknown-unknown --lib`: PASS.
- `rtk cargo clippy --example gpe_ui_p4_typography_probe -- -D warnings`: PASS.
- PNG export command above: PASS; export visually inspected, text nonblank,
  columns separate, missing symbol distinguishable from question mark.

Native window readability and browser runtime have not been human-validated.
Web compilation is not a browser rendering gate. P4 is not complete.

## Outline Gallery Follow-up

The maintainer explicitly requested at least twenty distinct fonts navigable
through GPE.UI, authorizing the implementation beyond the initial small slice.
The previous dependency/integration hold is superseded for this experiment.

- Optional `outline-fonts` feature uses fontdue 0.9.4; default builds keep the
  bitmap path without this dependency.
- `src/outline_text.rs` owns each loaded font, reusable layout and a bounded
  glyph cache. Measurement and painting use the same layout; coverage is alpha
  blended and clipped to the caller's rectangle and framebuffer.
- `examples/gpe_ui_p4_font_gallery.rs` loads twenty separate TTF families at
  once. Its GPE.UI menu shows font-authored labels, paginates in groups of ten,
  cycles via previous/next and keyboard, and exposes a mouse-editable size slider.
- `assets/fonts/p4` contains the original fonts, individual OFL licences and
  revision/hash provenance. No system font installation or runtime download.
- This is not yet a font field in every UiStyleSheet/component. The gallery
  composes existing GPE.UI interactions with outline painting. Automatic font
  fallback, complex-script shaping and variable-axis controls remain deferred.

```powershell
rtk cargo run --features outline-fonts --example gpe_ui_p4_font_gallery
rtk cargo test --features outline-fonts --example gpe_ui_p4_font_gallery
rtk cargo test --features outline-fonts --lib
rtk cargo check --target wasm32-unknown-unknown --features outline-fonts --lib
rtk cargo clippy --features outline-fonts --lib --example gpe_ui_p4_font_gallery -- -D warnings
```

The gallery tests check twenty distinct antialiased outputs, measurement/paint
agreement, clipping, control characters, selection wrap, mouse traversal of all
twenty entries, size editing and maximum-size preview fit. PNGs are exported for
all families; representative output is inspected visually. Library tests: 319
passed. Web compilation passed; browser runtime is still not verified.

Primary implementation references:
https://docs.rs/fontdue/0.9.4/fontdue/layout/index.html
https://docs.rs/fontdue/0.9.4/fontdue/struct.Font.html

## Initial Decision Record (Superseded for the Gallery)

P4 outline typography requires architecture review under
`docs/agents/SWE_SMALL_TASK_DELEGATION_CONTRACT.md` (new dependency strategy and
public text measurement/painting integration exceed a small local fix).

The concrete decision is which optional outline rasterizer and font asset to
evaluate, and how to route its matching measurement and painting into the UI.
The comparison must preserve the bitmap option and measure readability,
Native/Web behavior, binary size and runtime cost before generalizing the API.
No rasterizer or new text abstraction is selected by this slice.
