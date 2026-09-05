# GPE.UI — P6 / P7 Productionization Roadmap v0.2

Status: **ACTIVE / AUTHORITATIVE FOR P6→P7 SEQUENCING**

This document supersedes the sequencing assumptions in `GPE_UI_PRODUCTIONIZATION_P6_CONSUMER_MIGRATION_v0.1.md` from the point where P6.1 implementation began. The v0.1 document remains the historical pre-implementation baseline and consumer inventory.

## Governing principles

P6 and P7 must preserve the original GPE.UI goals:

```text
lightweight
modular
composable
extensible
consumer-owned state
low boilerplate
clear rollback boundaries
no global UI manager
no speculative framework
```

A visually impressive result does **not** count as success if the consumer must reconstruct focus, hit-testing, layout, typography plumbing, input routing, scaling or style resolution by hand.

The productionization criterion is therefore two-dimensional:

> **runtime/visual capability + developer experience**

If a real consumer needs kilometres of bespoke UI code to obtain a polished result, that is evidence of an abstraction gap and must be treated as such.

---

# P6 — Real consumer proof and migration pressure

## Mission

P6 proves that converged GPE.UI capabilities survive contact with real consumers. It may discover missing primitives, but every new shared abstraction must be justified by observed consumer friction.

P6 remains **evidence-driven**. It is not a repository-wide migration and does not freeze public v1 naming.

## P6.1 — Arcade catalog / launcher

Arcade remains the first consumer because it exercises a broad but bounded real-world UI surface:

```text
responsive card grid
keyboard navigation
mouse hover/click
gamepad navigation/activation
search
filters
outline typography
Native + Web/Touch compatibility
launch / return lifecycle
pixel gameplay + non-pixel launcher UI
```

### P6.1a — Functional spatial migration

Goal: replace the legacy six-row menu with converged spatial/card interaction while preserving launch behavior.

Evidence obtained:

- stable `UiId` per game;
- shared spatial/card layout;
- keyboard/pointer semantics exercised;
- launch/return boundaries preserved;
- migration can remain consumer-local.

Status: **FUNCTIONAL PROOF OBTAINED**.

### P6.1b — Real launcher capability proof

Goal: prove more than behavioral parity. Arcade must exercise meaningful GPE.UI features on a real consumer:

- responsive cards;
- themes/styles;
- hover/focus/active state;
- search/filter composition;
- gamepad navigation;
- outline typography;
- richer launcher composition.

Evidence obtained:

- functional search + fuzzy matching;
- category filtering;
- 3×2 responsive card launcher;
- keyboard + gamepad path validated after runtime diagnostics;
- styling/theme semantics proven on a real consumer.

Finding:

> low-resolution framebuffer presentation makes outline typography visually soft/pixelated even when the outline rasterizer itself is correct.

### P6.1c — Input and typography runtime proof

Goal: distinguish consumer bugs from engine/backend bugs.

Required / observed:

- arrows/WASD map to spatial navigation;
- multiple gamepads are accepted, not only pad 0;
- gamepad backend exposes connected devices and live state;
- temporary diagnostics move to stdout and are not part of production UI;
- release performance, not debug performance, is the runtime gate for outline-heavy UI.

Status: **INPUT PATH PROVEN; GAMEPAD MULTI-DEVICE SUPPORT PROVEN**.

### P6.1d — High-resolution presentation

Consumer evidence exposed a missing presentation primitive:

> pixel gameplay and modern UI should not be forced to share one low logical resolution.

The validated MFE direction is deliberately smaller than a general GPU compositor:

```text
high-resolution host framebuffer
+ low-resolution gameplay framebuffer
+ integer nearest composition via `present_pixel_surface(...)`
+ outline/UI rendered directly at host resolution
```

This keeps existing games unchanged unless they opt in and avoids introducing a render graph or generic layer engine prematurely.

Current evidence:

- 320×180 pixel surface remains crisp at integer scale;
- 1280×720 outline typography renders cleanly;
- release runtime remains near the existing frame budget;
- host-space pointer interaction works in the MFE;
- the primitive is opt-in and has no cost for ordinary single-framebuffer games.

### P6.1e — Arcade high-resolution integration

Goal: migrate Native Arcade to the validated presentation model.

Target structure:

```text
HOST 1280×720
├─ GPE.UI launcher at host resolution
└─ active game in low-resolution framebuffer
   └─ present_pixel_surface(...)
```

Acceptance gates:

```text
[ ] launcher outline typography is visually crisp
[ ] responsive card layout remains correct
[ ] keyboard navigation PASS
[ ] mouse hover/click PASS
[ ] all connected gamepads may navigate/activate
[ ] search/filter PASS
[ ] game launch PASS
[ ] game return PASS
[ ] active game remains pixel-crisp
[ ] child-game pointer/touch mapping is defined, not guessed
[ ] release performance acceptable
[ ] cargo fmt --check PASS
[ ] cargo test PASS
[ ] wasm32 compile gate PASS where relevant
```

Native-first is allowed for this slice. Web/Touch may retain the previous path until pointer/touch remapping for nested pixel surfaces is proven.

### P6.1f — Developer Experience / composition-cost audit

This gate is mandatory **before P6.2**.

Audit the Arcade UI implementation and classify code into:

```text
A. consumer data / product decisions
B. screen composition
C. unavoidable integration glue
D. repeated mechanical boilerplate
E. engine/UI responsibilities reconstructed by the consumer
```

The desired outcome is that most consumer code is A+B, with C small and D/E minimal.

Questions to answer:

- Does the consumer manually calculate too many rectangles?
- Does it duplicate focus/navigation logic already owned by GPE.UI?
- Does it manually fit text repeatedly?
- Is style/theme declaration concise and reusable?
- Is search/filter composition consumer-specific or evidence for a shared primitive?
- Is high-res pixel-surface composition one obvious call or a fragile mini-runtime?
- Could a tutorial explain the architecture without hiding pages of plumbing?

Possible verdicts:

```text
PASS
PASS WITH FOLLOW-UP HELPERS
FAIL — abstraction level is wrong
```

No P6.2 migration is authorized until this audit is recorded.

---

## P6.2 — Unlike second consumer

P6.2 must be structurally unlike Arcade so that P7 has evidence from at least two real shapes.

Preferred candidates, in order:

1. **Breakout end menu** — tiny linear menu, gameplay transition boundary, touch relevance.
2. **Pong match-end menu** — linear menu plus multiplayer/gamepad context.
3. **Space Invaders menu/controls** — multi-page controls/status UI, substantially unlike a catalog.
4. **Tool window controls** — useful for sliders/tabs/settings but weaker as a real-game consumer.

The exact choice should be made after the P6.1 DX audit.

P6.2 must not copy Arcade-specific abstractions merely to make the APIs look uniform.

---

## P6.3 — Migration synthesis

After two unlike consumers, record:

- abstractions that survived unchanged;
- helpers justified by repetition across both consumers;
- APIs that were difficult or misleading;
- compatibility surfaces still needed;
- what should remain internal/experimental;
- what is ready for P7 stabilization;
- what should be deleted rather than promoted.

P6 stops here. No public API freeze occurs in P6.

---

# P7 — Consolidation, public shape and showcase

## Mission

P7 converts **proven** P6 patterns into a coherent teachable API. It is not a feature grab-bag.

P7 asks:

> What is the smallest stable GPE.UI surface that lets real consumers build polished, responsive, input-complete UI without boilerplate or hidden global machinery?

## P7.1 — API consolidation / naming review

Review and classify:

```text
UiStateStore / stable identity
transactional builder surface
spatial/card compatibility adapters
layout primitives
style/theme resolution
outline typography
pixel-surface presentation
input adaptation / navigation semantics
legacy MenuState/toolkit compatibility
```

For each API decide one of:

```text
PROMOTE
KEEP INTERNAL
KEEP COMPATIBILITY
RENAME
MERGE
DEPRECATE
REMOVE
```

No public naming is frozen without evidence from P6.1 + P6.2.

## P7.2 — Ergonomic composition helpers

Only helpers justified by repeated consumer friction may be introduced.

Target tutorial-level composition should conceptually approach:

```rust
ui.launcher()
    .theme(neon_arcade)
    .header(...)
    .search(...)
    .filters(...)
    .game_grid(...);
```

This is a direction, not a required fluent API. The actual Rust API may remain more explicit if that is clearer and cheaper.

P7.2 must specifically reduce:

- repeated layout boilerplate;
- text-fit boilerplate;
- manual style-state branching;
- device-specific navigation glue;
- high-res/pixel-surface presentation plumbing.

## P7.3 — Typography and high-resolution UI boundary

Stabilize the model discovered in P6:

```text
pixel content can remain low-res
UI can render at host/presentation resolution
input coordinates have an explicit mapping boundary
ordinary single-framebuffer games remain simple
```

Decide whether `present_pixel_surface(...)` is sufficient as the stable primitive or whether repeated consumers justify a higher-level wrapper.

A generic GPU compositor/render graph remains a non-goal unless P6/P7 evidence proves the CPU/host model inadequate.

## P7.4 — Arcade Visual Showcase

Arcade becomes the primary visual/reference consumer for GPE.UI.

The visual target is a polished neon/pixel-arcade launcher, inspired by the accepted reference direction, while keeping Arcade-specific art out of the core UI API.

Desired capabilities:

```text
crisp outline typography
responsive game cards
game thumbnails / runtime captures
strong per-game accent colors
search + categories
focus / hover / active polish
header artwork / branded composition
subtle scanlines / vignette / glow where appropriate
CRT / arcade framing as optional presentation decoration
keyboard / mouse / multi-gamepad / touch parity
pixel gameplay remains crisp when launched
```

### Ownership boundary

Core GPE/UI may own generic primitives such as:

```text
responsive layout
cards
styles/themes
images/thumbnails
outline text
focus/input semantics
pixel-surface presentation
generic lightweight post/presentation effects if independently justified
```

Arcade consumer owns:

```text
Neon Arcade theme values
specific artwork
specific thumbnails
header illustration
copy / labels
category taxonomy
CRT cabinet composition if it is Arcade-specific
```

Do **not** add `NeonArcadeWidget`, `ArcadeHeader`, or game-specific visual concepts to core GPE.UI.

## P7.5 — Tutorial / teaching gate

Before declaring GPE.UI productionized, produce small tutorials/examples that teach the reusable bricks independently.

Minimum tutorial set:

```text
1. button/focus/input basics
2. responsive row/grid/cards
3. themes and state styles
4. outline typography
5. search/filter composition
6. keyboard + multi-gamepad navigation
7. pixel game + high-res UI presentation
8. real consumer walkthrough: Arcade launcher
```

Tutorial code is itself a DX gate. If a tutorial requires large unexplained plumbing, revisit the API before freeze.

## P7.6 — Compatibility / deprecation decision

Only after P6 evidence + P7 tutorial proof:

- identify legacy helpers that remain useful;
- deprecate only responsibilities fully covered by the new surface;
- preserve simple direct framebuffer drawing where it remains the right tool;
- avoid forcing HUDs and bespoke game-world overlays into widgets;
- document migration recipes rather than deleting APIs abruptly.

---

# P6 → P7 transition gate

P7 implementation may begin only when:

```text
[ ] P6.1 Arcade high-res Native runtime is visually validated
[ ] P6.1 input families pass on available hardware
[ ] P6.1 release performance passes
[ ] P6.1 DX audit is recorded
[ ] one unlike second consumer is migrated and reviewed
[ ] repeated friction is distinguished from Arcade-specific needs
[ ] compatibility regressions are absent or explicitly documented
```

# Definition of success

GPE.UI productionization succeeds when a developer can build a UI that is:

```text
functional
responsive
stylable
input-complete
visually polished
pixel-game compatible
```

without turning the consumer into a UI framework implementation.

The final standard is not merely:

> "GPE.UI can draw widgets."

It is:

> **"GPE.UI gives small games practical, composable building blocks for building polished interfaces while preserving the simplicity and pixel-first nature of GPE."**
