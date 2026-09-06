# GPE.UI — P6.1f Developer Experience / Composition-Cost Audit v0.1

Status: **PASS / STOP — FOLLOW-UP HELPER VALIDATED, P6.2 AUTHORIZED**

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
src/pixel_game_host.rs
```

Historical/compatibility comparison:

```text
examples/arcade/game.rs
```

Validated runtime facts entering and closing this audit:

```text
high-resolution outline typography is visually crisp
3x2 spatial cards are readable and stable
keyboard navigation works
multi-gamepad navigation works
search and category filtering work
release runtime remains near 60 FPS on the validated machine
low-resolution gameplay can be integer-nearest composited into a high-resolution host
launch -> child game -> return-to-launcher runtime path passes after PixelGameHost refactor
cargo fmt / test / compile gates pass on the validated branch
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

Verdict: **shared helper justified by repetition, but not required to unblock P6.2**.

The helper should remain small and typography-focused; do not create a text widget framework merely to remove this function. Its final public shape remains a P7 concern unless P6.2 independently repeats the same need.

### E. Engine/UI responsibilities reconstructed by the consumer — EXTRACTED

The original high-resolution Arcade host performed a nested-game mini-runtime manually:

```text
own a low-resolution gameplay Framebuffer
construct a child Frame
forward Input / delta_time / storage / audio
invent child surface_size / Viewport
call child Game::update(...)
present_pixel_surface(...) into the high-resolution host
manage the child presentation boundary
```

This was the strongest DX failure found by P6.1.

The mandatory extraction is now implemented as the provisional `PixelGameHost` helper. The Arcade consumer no longer owns the child framebuffer, constructs the child `Frame`, builds the child `Viewport`, or manually sequences child update + presentation.

The resulting consumer boundary is intentionally narrow:

```text
active_game: Option<PixelGameHost>
...
host.update_and_present(frame, HOST_BOUNDS)
```

The helper name and exact public status remain provisional until P7.1.

## Input-coordinate boundary

Keyboard and gamepad state can be forwarded unchanged to a nested pixel game.

Pointer/touch coordinates cannot be silently forwarded from the host framebuffer because they are expressed in host coordinates. `PixelPresentation::map_point(...)` proves the geometric mapping primitive, but the nested-game abstraction must make the policy explicit.

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

`present_pixel_surface(...)` remains a valid low-level primitive.

`PixelGameHost` now covers the specific repeated integration mechanics exposed by Arcade without introducing a render graph, compositor framework, global game manager or hidden state.

This is the desired P6 outcome: extract only the responsibility proven to belong below the consumer.

## Tutorial test

The high-resolution architecture is now explainable at tutorial level as:

```text
high-res host UI
+ PixelGameHost for low-res child gameplay
+ integer-nearest presentation
```

The consumer no longer needs to learn internal `Frame` construction in order to use this model. The tutorial ergonomics blocker found by the first audit is therefore removed for the Native keyboard/gamepad scope.

## Follow-up status

```text
Nested pixel-game host helper     IMPLEMENTED / VALIDATED
Outline fit/center helper         DEFERRED — still justified, not a P6.2 blocker
Generic launcher DSL              REJECTED
Generic search/filter framework   REJECTED
Generic GPU compositor            REJECTED
Arcade-specific core widgets      REJECTED
Pointer/touch nested mapping      OPEN BOUNDARY FOR LATER WEB/TOUCH WORK
```

## Verdict

```text
visual/runtime capability          PASS
consumer product ownership         PASS
shared spatial/style reuse         PASS
screen composition                 PASS
high-res pixel primitive           PASS
nested child-game DX               PASS AFTER PixelGameHost EXTRACTION
outline text-fit DX                FOLLOW-UP HELPER JUSTIFIED / NON-BLOCKING
search/filter abstraction          KEEP CONSUMER-LOCAL
launcher-specific abstraction      REJECT
```

Overall:

> **PASS / STOP**

P6.1 successfully exposed a real abstraction gap, extracted a narrow helper, and revalidated the runtime path without broadening GPE into a speculative framework.

**P6.2 is now authorized.** The next consumer must be structurally unlike Arcade and must not inherit Arcade-specific abstractions by default.
