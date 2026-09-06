# GPE.UI P6 — Multi-resolution presentation spike v0.1

Status: OPEN / CAPABILITY GAP IDENTIFIED

## Trigger

P6.1 migrated the real Arcade catalog to the converged GPE.UI stack and exercised responsive cards, styles/themes, outline fonts, fuzzy search, filters and spatial input.

Human runtime review exposed a presentation limit that cannot be fixed honestly inside the Arcade consumer:

- the Arcade logical framebuffer is 560×320 in Native mode;
- outline fonts are rasterized into that framebuffer;
- the renderer presents the framebuffer texture with nearest-neighbour sampling;
- the Native Arcade window is normally 3× the framebuffer dimensions;
- therefore anti-aliased outline glyphs are magnified as low-resolution framebuffer pixels.

The result is functionally correct outline text but visibly coarse/soft typography at desktop window size.

The renderer already uses a nearest sampler. Changing font family or glyph size does not remove this structural limit.

## Capability gap

GPE currently couples these concerns into one resolution:

1. game simulation/render framebuffer resolution;
2. UI rasterization resolution;
3. presentation texture resolution.

This is appropriate for pure pixel-art games, but insufficient for a consumer that wants both:

- low-resolution pixel-perfect gameplay;
- high-resolution modern UI / outline typography.

P6 therefore supplies consumer evidence for a new engine-level capability boundary.

## Required property

A GPE application should be able to compose:

> pixel-low-res gameplay + high-res UI/presentation

without forcing the consumer to:

- duplicate rendering code;
- manually upscale/copy every framebuffer;
- rebuild input transforms;
- bypass GPE.UI;
- choose between blurry UI and blurry pixel art.

## Candidate models

### A. High-resolution host + low-resolution game surface

The engine owns a high-resolution presentation surface. Pixel gameplay renders to a lower-resolution offscreen surface that is composited with nearest filtering. GPE.UI renders directly at presentation resolution.

Advantages:

- crisp outline/UI text;
- pixel-perfect gameplay remains nearest-scaled;
- clean conceptual separation.

Risks:

- requires explicit surface/input coordinate ownership;
- additional framebuffer/composition API;
- must remain Web/WASM compatible.

### B. Independent UI overlay surface

Keep the current game framebuffer and add a second high-resolution UI surface composited by the renderer.

Advantages:

- minimal disruption to existing games;
- natural fit for overlays, HUD, menus and tooling.

Risks:

- renderer now owns multi-layer composition;
- alpha/color-space semantics must be defined;
- input coordinate mapping must be unambiguous.

### C. Per-frame presentation filter only

Allow switching nearest/linear sampling per frame.

This may improve visual smoothness but does **not** solve the fundamental resolution problem: outline glyphs are still rasterized at low logical resolution. It would also risk blurring pixel gameplay.

**Not sufficient as the P6 solution.**

## Recommended direction

Prototype **B first** as the smallest falsifiable engine capability:

- preserve the existing game framebuffer contract unchanged;
- add one optional presentation-resolution UI overlay;
- composite game framebuffer nearest-first, UI overlay alpha-over at native presentation resolution;
- expose deterministic coordinate mapping for pointer/touch;
- keep the overlay opt-in and zero-cost when unused.

If B forces duplicated viewport/input ownership, compare directly against A before stabilizing any API.

## Falsifiable MFE

Create a minimal example with:

- 320×180 pixel-art game framebuffer;
- approximately 1280×720 presentation surface;
- nearest-scaled checker/sprite content;
- one outline-font heading rendered at presentation resolution;
- pointer hit target in the high-resolution UI layer;
- Native and WASM compile gates.

PASS requires simultaneously:

- visibly pixel-sharp game content;
- visibly smooth outline typography;
- correct pointer coordinates;
- no consumer-side manual framebuffer copy loop;
- optional capability disabled by default;
- no regression to current single-framebuffer consumers.

## Non-goals

This spike does not define:

- a scene graph;
- retained-mode UI;
- arbitrary compositor layers;
- vector rendering beyond the existing outline-font capability;
- a general render graph;
- post-processing.

## P6 consequence

P6.1 should not hide this finding by adding consumer-local scaling hacks. The Arcade consumer is doing its job: it has demonstrated that the current single-resolution presentation boundary is too restrictive for one real GPE.UI use case.

Until this capability is resolved, P6.1 visual polish remains OPEN even if its behavior/input/performance gates pass.
