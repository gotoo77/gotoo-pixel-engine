# GPE.UI — PRODUCTIONIZATION P3 / STYLING, THEMING & CUSTOMIZATION v0.1

Status: **ACTIVE IMPLEMENTATION CONTRACT — P3 ONLY**

Parent roadmap:

`docs/ui/GPE_UI_PRODUCTIONIZATION_PLAN_v0.1.md`

P2 result:

`docs/ui/GPE_UI_PRODUCTIONIZATION_P2_RESULT_v0.1.md`

Baseline entering P3:

`ff292ec5cdf91f4a74e3de3c9e9024b43ab0be5a`

---

# 1. Mission

Productionize a typed, deterministic styling layer for the converged GPE.UI transaction without reopening layout/input architecture and without drifting toward CSS or a browser-style cascade.

P3 must make the accepted UI kernel deliberately customizable for game UI while preserving the compatibility behavior already proven by P0–P2.

The target precedence is:

```text
UiTheme compatibility defaults
    ↓
component style
    ↓
explicit local override
    ↓
frame-local visual state (focus / hover / active)
```

The first three layers resolve static style. Visual state is applied afterwards as an explicit deterministic overlay; it is not a hidden retained-style system.

---

# 2. Observed baseline

At P3 entry:

- `UiTheme` is a flat public compatibility type also consumed by the legacy toolkit;
- transactional Panel/Button/Toggle/Slider painting reads colors directly from `UiTheme`;
- transactional painting currently receives focused state only;
- the shared interaction kernel already owns focused, hovered and pointer/touch active/capture facts;
- `experimental_spatial::DefaultCardPainter` independently maps the same `UiTheme` colors onto Card visuals;
- the Spatial path already preserves a consumer-defined `CardPainter` escape hatch;
- no rounded-rectangle framebuffer primitive exists in the current baseline.

P3 therefore has enough evidence for typed color/border/state customization without requiring new platform, renderer or input infrastructure.

---

# 3. Compatibility rule

`UiTheme` MUST remain source-compatible during P3.

In particular:

```text
DO NOT add required fields to UiTheme
DO NOT rename/remove UiTheme fields
DO NOT silently redirect or remove legacy Ui / UiState behavior
```

Existing calls must remain valid:

```text
experimental::run
experimental::run_with_input
experimental::run_headless
experimental::run_headless_with_input
experimental_spatial::run_card_grid
experimental_spatial::run_card_grid_headless
```

With no explicit P3 styles supplied, accepted P2 rendering/layout/interaction behavior must remain unchanged.

---

# 4. Typed style model

P3 must introduce a small typed style vocabulary owned by GPE.UI rather than by any one consumer.

Target concepts:

```text
UiStyleSheet
UiStyleOverride
UiResolvedStyle (internal)
UiVisualState
```

Exact public naming is still provisional until P7; the semantic contract is authoritative.

## 4.1 Static precedence

For one widget, resolution is deterministic:

```text
compatibility style derived from UiTheme
    apply component override
    apply local override
    => resolved static style
```

An override changes only fields explicitly present in that override.

No selector engine, inheritance graph, string class names, runtime stylesheet parser, specificity score or CSS-like cascade is allowed.

## 4.2 Initial proven style fields

P3 may productionize fields directly supported by current framebuffer/text/layout primitives:

```text
background
border
text
muted text where semantically relevant
accent/state border
border width using integer-pixel drawing
panel/container padding where already part of measured layout
vertical gap where already part of measured layout
```

Layout-affecting style values must resolve before measure/arrange and must use the existing P2 integer layout primitives.

## 4.3 Component styles

The stylesheet must be finite and typed. Initial components justified by the accepted transaction are:

```text
Panel
Text
Button
ToggleBool
SliderF32
```

Spatial Card may consume the same style vocabulary through its default compatibility painter, but must not create a second independent style system.

## 4.4 Local override

Local customization must be explicit at the builder call/site and affect only the target node/container.

P3 should prefer explicit `*_styled` / equivalent typed APIs over an implicit inherited style scope. Ergonomic sugar can be reconsidered later; P3 prioritizes falsifiable precedence and isolation.

---

# 5. Visual-state contract

P1 already owns interaction facts. P3 may read them but must not redefine them.

Frame-local style state:

```text
focused
hovered
active
```

`active` means existing pointer/touch capture while pressed/held on the target. P3 does not invent a new input state machine.

State-overlay precedence must be explicit and deterministic. Initial precedence:

```text
base resolved static style
    ↓ focused
    ↓ hovered
    ↓ active
```

Later states win only for fields they override. Thus `active > hovered > focused > base` for conflicting style fields.

Navigation/activation semantics MUST be identical regardless of style.

---

# 6. Deliberate deferrals

The following roadmap candidates are NOT automatically authorized just because P3 is the styling slice.

## Disabled

A visually disabled widget without disabled interaction semantics would be misleading. Disabled requires an explicit semantic contract with the interaction kernel and is deferred unless P3 produces concrete consumer evidence and a narrowly reviewed follow-up.

## Rounded corners

The baseline has no rounded-rectangle framebuffer primitive. P3 MUST NOT add per-widget geometry hacks merely to claim rounded corners. A renderer primitive may be evaluated separately if justified; otherwise corner radius remains deferred.

## Sprite / nine-slice

These remain strategic candidates, not baseline requirements. Do not add them without a concrete consumer/probe proving the generic API shape.

## Animation

Out of P3 scope.

---

# 7. Customization escape hatches

P3 must preserve the existing Spatial `CardPainter` escape hatch.

It must not force every future custom widget through one enormous enum or global manager.

A new generic transactional custom-painter API is NOT mandatory in P3 unless a concrete probe demonstrates a small lifetime-safe API. P6 consumer work remains the stronger gate for freezing such an escape hatch.

---

# 8. Implementation sequence

## P3.1 — style resolution foundation

- introduce typed style/static override primitives;
- derive compatibility defaults from unchanged `UiTheme`;
- implement exact field-wise precedence tests;
- no visual behavior change for existing APIs.

## P3.2 — transactional component + local styles

- attach resolved style to transient nodes, not persistent retained widgets;
- support component-level styles for Panel/Text/Button/Toggle/Slider;
- support explicit local override at the relevant builder calls;
- if padding/gap overrides are included, resolve them before measure/arrange and preserve P2 integer geometry.

## P3.3 — focus / hover / active visual states

- paint using the existing shared interaction facts;
- deterministic state-overlay precedence;
- add framebuffer/headless evidence that style changes do not alter identity, focus, activation, typed proposals or build-once semantics.

## P3.4 — Spatial default-painter alignment

- make `DefaultCardPainter` consume the same style semantics where applicable;
- keep custom `CardPainter` behavior/source compatibility intact;
- do not create a Card-only stylesheet architecture.

## P3.5 — visual customization probe + closure review

Prove at least:

```text
default compatibility rendering remains stable
component style changes all widgets of that component only
local override affects one node only
focus / hover / active states are visibly distinct when configured
style precedence is deterministic
layout-affecting style remains integer/pixel-aware
custom CardPainter still works
no input/platform semantic change
no CSS/DOM/global style manager introduced
```

---

# 9. Acceptance gates

Automated:

```text
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release --examples
Web build/package CI
Conventional Commits CI
```

Rust process gate:

```text
cargo fmt MUST run before each Rust commit/push
CI validates formatting; it is not the primary formatter
```

Behavioral:

```text
existing P0/P1/P2 regression suite remains green
default UiTheme path stays compatible
style resolution precedence exact
local override isolation exact
visual state resolution exact
consumer build closure still runs once
interaction output independent from style choice
```

Human Native runtime:

A dedicated P3 visual probe is required before final PASS because P3 changes visible rendering. It should exercise at minimum default/component/local plus focus/hover/active styling.

Browser runtime rerun is required only if P3 changes platform/render transport semantics. Normal framebuffer color/border painting changes require Web build/package, not a fresh browser lifecycle gate.

---

# 10. Scope boundaries

P3 MUST NOT implement:

```text
typography stack changes                    -> P4
debug allocation optimization               -> P5
real consumer migration                     -> P6
public v1 API freeze                         -> P7
markup / SVG / DOM / CSS
Taffy integration
animation
legacy toolkit removal
platform/input transport changes
```

---

# 11. STOP

When P3 passes:

```text
GPE.UI PRODUCTIONIZATION P3 = PASS / PASS WITH CONDITIONS / FAIL
STOP
```

Do not begin P4 until P3 is formally closed and merged.
