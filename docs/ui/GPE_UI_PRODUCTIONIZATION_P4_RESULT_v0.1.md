# GPE.UI - P4 Typography: First Slice

Status: P4 ACTIVE / BITMAP + 52-FONT OUTLINE GALLERY IMPLEMENTED / NATIVE HUMAN RUNTIME PASS / WEB HUMAN RUNTIME PENDING

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
- The outline path is optional and remains isolated behind the `outline-fonts`
  feature; the default bitmap path does not pull `fontdue`.

## Implemented Bitmap Follow-up

TextRenderer supports U+2013/U+2014 dashes, U+2022 bullet, and U+2190 through
U+2193 arrows in both built-in fonts. The 3x5 font intentionally uses the same
three-pixel stroke for both dashes. Left/right arrows remain distinct.

The legacy Framebuffer glyph tables, custom bitmap font fallback, font metrics,
public API and default dependency set remain unchanged.

`examples/gpe_ui_p4_typography_probe.rs` renders a comparison of both fonts at
integer scales 1 and 2, with UI text, French accents, punctuation, literal `?`
and the actual missing-glyph symbol.

```powershell
rtk cargo run --example gpe_ui_p4_typography_probe
rtk cargo run --example gpe_ui_p4_typography_probe -- target/p4-typography.png
```

Recorded validation for that slice:

- `rtk cargo test text::tests --lib`: 5 passed.
- `rtk cargo check --target wasm32-unknown-unknown --lib`: PASS.
- `rtk cargo clippy --example gpe_ui_p4_typography_probe -- -D warnings`: PASS.
- PNG export: PASS; export visually inspected.

## Outline Gallery

The maintainer explicitly requested a broad navigable font gallery and then
human-validated the current Native interaction/rendering behavior on 2026-09-05.

- Optional `outline-fonts` feature uses `fontdue 0.9.4`.
- `src/outline_text.rs` owns each loaded font, reusable layout and a bounded
  glyph cache. Measurement and painting use the same layout; coverage is alpha
  blended and clipped to the caller's rectangle and framebuffer.
- `examples/gpe_ui_p4_font_gallery.rs` currently embeds **52 distinct TTF
  families**, with fuzzy search, paginated navigation, previous/next controls
  and a size slider.
- Slider rendering exposes a distinct current-value thumb.
- Focused slider keyboard left/right uses deterministic repeat: immediate step,
  300 ms initial delay, then 60 ms repeat cadence based on `delta_time`.
- GPE input now exposes frame-scoped mouse-wheel steps. In the gallery, wheel
  changes the slider only when that slider is both hovered and focused.
- `assets/fonts/p4` contains font files, individual OFL licences and provenance.
  No system font installation or runtime download is required.
- This is still an experimental integration surface, not yet a font field in
  every `UiStyleSheet`/component.
- Automatic font fallback, complex-script shaping and variable-axis controls
  remain deferred.

Native runtime gate: **PASS (maintainer, 2026-09-05)** for gallery navigation,
font rendering, slider value marker, pointer editing, keyboard repeat and the
hover+focus wheel rule.

## Web Runtime Probe

A dedicated Web wrapper now reuses the same gallery implementation:

- `examples/gpe_ui_p4_font_gallery_web.rs`
- `web/gpe_ui_p4_font_gallery.html`
- `scripts/build_p4_font_gallery_web.ps1`

Build and run from PowerShell:

```powershell
.\scripts\build_p4_font_gallery_web.ps1
python .\scripts\dev.py serve-web
```

Then open:

```text
http://127.0.0.1:8000/gpe_ui_p4_font_gallery.html
```

Web human-runtime acceptance must verify at least:

1. the same outline fonts render legibly;
2. fuzzy search and previous/next navigation work;
3. the slider is pointer-editable;
4. focused left/right repeat behaves as on Native;
5. wheel changes the slider only while it is both hovered and focused;
6. moving keyboard focus away while leaving the pointer over the slider disables
   wheel changes.

**Browser runtime is still PENDING until the maintainer executes this probe.**
Web compilation alone is not a browser-rendering gate.

## Validation Commands

```powershell
rtk cargo run --features outline-fonts --example gpe_ui_p4_font_gallery
rtk cargo test --features outline-fonts --example gpe_ui_p4_font_gallery
rtk cargo test --features outline-fonts --lib
rtk cargo check --target wasm32-unknown-unknown --features outline-fonts --lib
rtk cargo clippy --features outline-fonts --lib --example gpe_ui_p4_font_gallery -- -D warnings
.\scripts\build_p4_font_gallery_web.ps1
```

## Current P4 Boundary

P4 is not complete yet.

Open decisions after the browser runtime gate:

- whether/how outline font selection becomes component-wide styling rather than
  gallery-local custom painting;
- the smallest explicit fallback policy across bitmap/outline paths;
- whether browser/native metric or performance differences justify further work;
- dependency/binary-size/runtime-cost evidence before any public API freeze.

Non-goals remain browser-grade shaping, CSS typography, a global font manager,
or SDF/MSDF by default.
