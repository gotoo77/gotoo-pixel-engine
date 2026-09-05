# GPE.UI — P6.1f Developer Experience / Composition-Cost Audit v0.1

Status: **PASS WITH FOLLOW-UP HELPERS / P6.2 BLOCKED UNTIL SMALL EXTRACTION**

## Scope

This audit reviews the Native high-resolution Arcade consumer introduced during P6.1e. It does not judge whether the launcher is visually finished; P7.4 owns the Visual Showcase. The question here is whether the current implementation is a healthy example of how a GPE consumer should compose GPE.UI.

The governing criterion is:

> a polished UI is not a productionization success if the consumer must rebuild engine/UI responsibilities by hand.

## Evidence reviewed

Primary implementation:

```text
examples/arcade/high_res.rs
examples/arcade.rs
src/presentation.rs
```

Historical/compatibility comparison:

```text
examples/arcade/game.rs
```

Validated runtime facts entering this audit:

```text
high-resolution outline typography is visually crisp
3x2 spatial cards are readable and stable
keyboard navigation works
multi-gamepad navigation works
search and category filtering work
release runtime remains near 60 FPS on the validated machine
low-resolution gameplay can be integer-nearest composited into a high-resolution host
```

## Classification

### A. Consumer data / product decisions — KEEP IN ARCADE

Healthy consumer-owned code includes:

```text
GAME_LABELS / GAME_KEYS
GAME_TAGS / GAME_MASKS
GAME_ACCENTS
category taxonomy
search query semantics and fuzzy scoring policy
Arcade-specific colors and copy
which six games are launched
```

This is product/domain data. It should not migrate into GPE.UI core.

### B. Screen composition — MOSTLY HEALTHY

The following are legitimate composition choices:

```text
header/search/filter/cards/footer regions
3x2 launcher composition
game-specific accent bars
font sizes chosen for this screen
Neon/Arcade palette values
status copy
```

Manual macro-layout rectangles are acceptable at this stage because they express one authored screen composition. They become a shared-helper candidate only if a second unlike consumer repeats the same mechanical structure.

The card interior does correctly reuse the shared spatial grid/style path rather than reconstructing focus, hover, active and hit-testing manually.

### C. Unavoidable integration glue — ACCEPTABLE BUT WATCH

Small translation functions such as:

```text
nav_input(...)
select_held(...)
catalog_ids(...)
cards(...)
```

are currently understandable and local. Some may disappear during P7 API consolidation, but P6.1 alone does not justify a global launcher DSL.

### D. Repeated mechanical boilerplate — FOLLOW-UP CANDIDATE

`draw_outline_fit_centered(...)` now exists as consumer-side typography plumbing even though the same need occurred in the prior Arcade implementation.

This responsibility is generic:

```text
measure
shrink-to-fit
center
paint
```

Verdict: **shared helper justified by repetition**.

The helper should remain small and typography-focused; do not create a text widget framework merely to remove this function.

### E. Engine/UI responsibilities reconstructed by the consumer — MUST EXTRACT

The high-resolution Arcade host currently performs a nested-game mini-runtime manually:

```text
own a low-resolution gameplay Framebuffer
construct a child Frame
forward Input / delta_time / storage / audio
invent child surface_size / Viewport
call child Game::update(...)
present_pixel_surface(...) into the high-resolution host
manage the child presentation boundary
```

This is the strongest DX failure found by P6.1.

A tutorial should not teach application code to construct an internal `Frame` merely to host a pixel game beneath high-resolution UI. This is engine responsibility reconstructed by the consumer.

Verdict: **extract one small reusable nested pixel-game host abstraction before P6.2**.

## Input-coordinate boundary

Keyboard and gamepad state can be forwarded unchanged to a nested pixel game.

Pointer/touch coordinates cannot be silently forwarded from the host framebuffer because they are expressed in host coordinates. `PixelPresentation::map_point(...)` already proves the geometric mapping primitive, but the nested-game abstraction must make the policy explicit.

For the current Native Arcade slice, launched games use their Native/keyboard-oriented constructors, so host-pointer forwarding is not required for the validated gameplay path. This is an explicit scope boundary, not evidence that coordinate mapping can be ignored.

Before Web/Touch adopts the high-resolution host, nested pointer/touch input must have a first-class mapped-input contract.

## Search/filter ownership decision

Search and category filtering remain Arcade-owned in P6.

Reason:

- fuzzy matching policy is product-specific;
- category taxonomy is product-specific;
- only one real migrated consumer currently needs this composition.

Do not add a generic `SearchableFilterableGrid` based on one launcher.

P7 may extract generic text-edit or filter-chip primitives if P6.2 supplies unlike evidence.

## Layout ownership decision

Do not extract an `ArcadeLayout` or generic `LauncherLayout`.

The shared responsive card layout is already generic and reused. Header/search/filter/footer placement remains authored screen composition. A future composition helper must be justified by repeated mechanics, not visual similarity.

## Style/theme decision

The existing shared style resolution is doing useful work for card focus/hover/active state.

Arcade-specific palette constants remain consumer-owned.

P7 may improve theme declaration ergonomics, but P6.1 does not justify a `NeonArcadeTheme` in core.

## High-resolution presentation decision

`present_pixel_surface(...)` is a valid low-level primitive and should remain small.

However, the current consumer experience around it is incomplete. The primitive handles pixels; the consumer still manually hosts the nested `Game` runtime.

Required follow-up is not a compositor or render graph. It is a narrow convenience abstraction around:

```text
low-res framebuffer ownership
child Frame construction
child update
integer presentation
explicit input-coordinate policy
```

Possible naming is deliberately not frozen in P6. Candidate concepts include:

```text
PixelGameHost
PixelSurfaceHost
NestedGameSurface
```

The final name belongs to P7.1 unless implementation requires a temporary internal name.

## Tutorial test

Current Arcade high-res architecture is explainable conceptually:

```text
high-res host UI
+ low-res game surface
+ integer nearest presentation
```

But the implementation currently exposes too much plumbing for a beginner-facing tutorial because application code manually constructs the child `Frame`.

Therefore the current implementation **fails the tutorial ergonomics sub-gate** until the narrow host helper is extracted.

## Follow-up before P6.2

Only the following extractions are authorized:

1. **Nested pixel-game host helper** — mandatory.
2. **Outline shrink-to-fit/center helper** — allowed and recommended because repetition is already observed.
3. No generic launcher DSL.
4. No generic search/filter framework.
5. No generic GPU compositor/render graph.
6. No Arcade-specific core widgets/theme.

After these small extractions, re-evaluate Arcade consumer code. P6.2 may begin if the nested-game mini-runtime disappears from consumer code and no new architectural regression is introduced.

## Verdict

```text
visual/runtime capability          PASS
consumer product ownership         PASS
shared spatial/style reuse         PASS
screen composition                 PASS
high-res pixel primitive           PASS
nested child-game DX               FAIL -> mandatory helper
outline text-fit DX                FOLLOW-UP HELPER JUSTIFIED
search/filter abstraction          KEEP CONSUMER-LOCAL
launcher-specific abstraction      REJECT
```

Overall:

> **PASS WITH FOLLOW-UP HELPERS**

P6.1 has successfully discovered an abstraction gap rather than hiding it. P6.2 remains blocked until the narrow nested pixel-game host extraction is implemented and reviewed.
