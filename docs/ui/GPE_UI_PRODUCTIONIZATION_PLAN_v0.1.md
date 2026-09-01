# GPE.UI — PRODUCTIONIZATION PLAN v0.1

Status: **ACTIVE PLAN**

Baseline:

`7a89ddc09ed941c409b2b459e1361fbfb80e639f`

Prerequisites:

```text
Phase 0 v0.3   PASS
MFE-001A       PASS
MFE-001B       PASS
MFE-001C       PASS WITH CONDITIONS
Architecture B GO
```

## Mission

Convert the proven GPE.UI Architecture B experimental model into a deliberately shaped production implementation without prematurely freezing the public API, removing the legacy toolkit, or broadening into a browser-like UI framework.

Productionization must preserve the strategic direction validated by the MFE sequence:

- frame-local Rust composition;
- transient UI graph / resolved frame model;
- explicit consumer-owned persistent interaction state;
- stable deterministic identity;
- integer/pixel-aware geometry;
- semantic navigation plus pointer/touch;
- typed value proposals / `ActionId` outputs;
- custom paint escape hatches;
- headless deterministic testing;
- Native + Web compatibility;
- no global UI manager;
- no mandatory heavy external dependency.

## 0. Non-negotiable correction before stabilization

The current experimental code contains **two separate runtime/state/output paths**:

```text
experimental.rs
    UiStateStore
    UiOutput
    UiBuilder
    run / run_headless
    vertical focus/navigation

experimental_spatial.rs
    SpatialState
    SpatialOutput
    run_card_grid / run_card_grid_headless
    pointer/touch capture
    spatial focus/navigation
```

This split was valid for falsifiable MFEs, but it is **not an acceptable production endpoint**.

Productionization must converge these paths around one kernel-level identity/input/state/output transaction model before stable public naming is chosen.

Therefore:

```text
DO NOT simply rename experimental.rs -> core.rs and call it production.
DO NOT stabilize SpatialState/SpatialOutput as an independent second UI runtime.
DO NOT create a permanent Card-only spatial subsystem beside the main UI transaction.
```

The spatial MFE remains evidence and migration input, not a second architecture.

---

# 1. Compatibility policy

The existing legacy toolkit remains available during productionization:

```text
Ui
UiState
UiResponse
UiTheme
RepeatConfig
RepeatState
PauseGame
VirtualPad
existing menu helpers
```

Rules:

- no behavior-breaking rewrite of the legacy toolkit in place;
- no legacy removal until real consumers have migrated successfully;
- `UiTheme` is a compatibility source for default production style tokens;
- compatibility adapters are allowed when small and explicit;
- deprecation, if any, happens only after consumer evidence.

---

# 2. Production dependency direction

Keep the public namespace:

```text
gpe::ui
```

No separate `gpe-ui` crate during this phase.

Logical dependency direction:

```text
GPE primitives
Framebuffer / Rect / Size / Image / Text / Input / ActionId
        ↑
     ui kernel
identity / state / constraints / events / input snapshot
        ↑
  layout + style
        ↑
     widgets
        ↑
debug / diagnostics / adapters
```

Forbidden dependencies:

```text
ui kernel -> Audio
ui kernel -> scene/game state
ui kernel -> platform window/event loop
ui kernel -> Web/DOM
```

---

# 3. Productionization slices

## P0 — Convergence contract and module boundaries

Goal:

Define and implement the smallest shared kernel boundary that both MFE paths can use without changing accepted behavior.

Required outcomes:

- one canonical `UiId`;
- one canonical semantic navigation type;
- one canonical interaction-state owner;
- one canonical output/event surface;
- pointer/touch state integrated into the same transaction model rather than a separate runtime;
- diagnostics/debug dump separated conceptually from mandatory production output;
- no public API freeze.

Gate:

```text
MFE-001A transaction semantics preserved
MFE-001B spatial behavior preserved
headless traces deterministic
legacy toolkit unchanged
```

A transitional adapter may keep the existing MFE probes compiling while internals converge.

## P1 — Unified input / focus / interaction

Goal:

Introduce one UI input snapshot capable of carrying:

```text
semantic navigation
pointer position + transitions
touch contacts/events
```

Required behavior:

- linear and spatial navigation share one focused `UiId` authority;
- pointer/touch capture is keyed by the same identity store;
- dynamic insertion/removal/reorder preserves or deterministically repairs focus;
- modal/focus-scope hooks remain possible without implementing a full scene manager;
- physical key/gamepad policy stays outside the kernel.

## P2 — Layout convergence

Goal:

Move the responsive Grid evidence into the transient graph/layout model.

Core candidates to prove, not quota-fill:

```text
Padding
Column
Row
Grid (auto-fit minimum width + deterministic remainder)
Align / Sized / Constrained as required
```

Rules:

- final authoritative geometry remains integer `Rect` / `Size`;
- no CSS Grid implementation;
- no Taffy dependency unless custom layout complexity creates concrete evidence for a comparison spike;
- current Card-only grid function becomes adapter/test fixture or disappears after equivalent graph behavior is proven.

## P3 — Styling / theming / visual customization

Goal:

Generalize the current flat `UiTheme` into the validated three-layer model:

```text
Theme/design tokens
        ↓
component style
        ↓
local override
```

Initial production style capabilities:

```text
background / foreground
border color
border width
corner radius
padding / gap
text roles
focus treatment
hovered / focused / active / disabled / selected visual states
```

Initial component style candidates:

```text
PanelStyle
ButtonStyle
CardStyle
```

`SliderStyle` / other styles are added only when their widgets enter the production path.

### Rounded corners

Rounded/pixel-stepped corners are explicitly in scope for this slice.

Preferred direction:

- a small reusable GPE drawing primitive if the implementation remains deterministic, clipped and pixel-aware;
- integer radius;
- radius `0` exactly preserves square rendering;
- no mandatory antialiasing or vector renderer;
- custom painter/nine-slice remains an escape hatch for elaborate skins.

Do not emulate CSS selectors or a cascade.

## P4 — Custom widget / custom paint contract

Goal:

Promote the successful MFE custom Card painter lesson into a generic escape hatch.

Kernel owns as much generic interaction as possible:

```text
identity
resolved rect
focus
hover
capture
activation
semantic action
```

Consumer/custom component owns domain-specific paint/content.

Avoid forcing every custom widget to reimplement pointer/touch capture.

## P5 — Debug/cost hardening

Goal:

Close the MFE-001C allocation condition.

Required:

- debug/headless textual dumps are not mandatory production-frame work unless requested;
- measure allocation/timing again after separation;
- retain deterministic dumps for tests/diagnostics;
- no invented zero-allocation requirement;
- optimize only where measured cost justifies it.

## P6 — Production public surface candidate

Only after P0–P5 behavior is green:

- choose final module/type names;
- stop exposing `experimental*` as the intended consumer path;
- keep compatibility aliases/adapters only where they materially ease migration;
- document state ownership and transaction timing;
- no API stability promise beyond the explicit candidate surface yet.

## P7 — Real consumer validation

At least two materially different consumer probes before calling the API a v1 candidate.

Recommended order:

```text
Consumer A: Arcade / responsive Card selection
Consumer B: Settings/Pause/HUD style UI
```

Purpose:

- validate responsive grid/cards and multimodal navigation in a real app;
- validate non-Card controls and styling;
- avoid an Arcade-shaped API;
- measure migration friction;
- discover missing small-game ergonomics before deprecating legacy UI.

## P8 — v1 candidate checkpoint

Possible result:

```text
GO v1 candidate
REVISE
STOP / retain experimental
```

A v1 candidate still does not automatically include:

```text
markup
SVG
animation framework
inspector
accessibility projection
advanced layout engine
```

Those remain later/optional capabilities unless new evidence changes priority.

---

# 4. Cross-cutting gates

Every implementation slice must keep:

```text
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release --examples
Web build/package CI
```

When behavior is visible or input-dependent, add a bounded human runtime gate.

Build/package is never evidence of browser runtime behavior by itself.

---

# 5. Current carried conditions / backlog

## Allocation attribution

Source: MFE-001C.

Current experimental dumps are constructed on every transaction and contaminate allocation counts.

Disposition:

```text
P5 — mandatory before v1 candidate
not an Architecture-B blocker
```

## Web `GameResult::Exit`

Tracked independently:

`#73 — web: define GameResult::Exit browser lifecycle semantics`

Disposition:

```text
generic platform lifecycle backlog
not a GPE.UI productionization blocker
must not be hidden by UI-specific behavior
```

## Typography / missing glyph

Existing UI follow-up remains active.

Disposition:

```text
parallel visual-quality backlog
must be addressed before showcase-quality/v1 UX claim
not a P0 convergence blocker
```

---

# 6. Branch / review discipline

Use one active implementation branch for the current productionization slice.

Do not create review-only branches.

A draft PR may be used as CI harness, but it is not merge approval.

After a coherent slice passes automated and required human gates:

```text
formal result/checkpoint
squash merge
branch cleanup
next slice from updated main
```

This keeps history reviewable without maintaining multiple divergent UI implementations.

---

# 7. Immediate execution direction

Start with **P0 only**.

P0 must not implement rounded corners, a style system, consumer migration, markup, SVG, or API deprecation.

The first code change should reduce architectural duplication rather than add visible features.

Expected P0 output:

```text
shared kernel interaction types/state/output boundary
spatial MFE behavior routed toward that boundary
existing MFE probes remain valid
no consumer-visible regression
STOP for review before P1/P2
```
