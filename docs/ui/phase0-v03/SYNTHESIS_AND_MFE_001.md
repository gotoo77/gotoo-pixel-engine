# Synthesis and MFE-001 Proposal

# 1. Phase 0 v0.3 result

```text
RECOMMENDED DIRECTION:
Architecture B — Balanced Hybrid

PHYSICAL LOCATION FOR V1:
evolve inside existing gpe::ui / src/ui

NEW SEPARATE CRATE:
NO, not now

CURRENT UI REWRITE:
NO

CURRENT UI DEPRECATION:
NO

RUNTIME IMPLEMENTATION AUTHORIZED BY THIS DOCUMENT:
NO
```

---

# 2. Why this is the recommended direction

The current UI proves that GPE values:

```text
direct Rust
explicit state
small surface
headless tests
framebuffer-native rendering
```

The strategic vision adds:

```text
responsive composition
stable interaction identity
multi-input semantics
custom widgets
rich styling
headless exact layout/events
future declarative authoring
```

Architecture B is the smallest candidate that naturally spans both sets without requiring:

```text
DOM
CSS
JavaScript
retained scene hierarchy
global manager
new renderer
new event loop
new crate ecosystem
```

---

# 3. Recommended kernel

Concepts to carry into MFE:

```text
UiId
UiStateStore
Constraints
transient UiGraph
resolved integer Rect tree
semantic UiNavInput
pointer/touch input snapshot
focus/capture state
UiEvent
ActionId output
PaintCtx over existing Framebuffer/Text/Image
headless dump
```

These are candidates, not frozen public API names.

---

# 4. Recommended core modules

```text
core
layout
input/focus
widgets
style
debug/headless
```

Existing primitives reused:

```text
Framebuffer
Rect
Size
Pixel
TextRenderer / Font
Image / ImageFit / ImageFilter
Input
ActionId / ControlMap
Touch
```

---

# 5. Optional / later capabilities

## Animation

```text
OPTIONAL
```

Do not enter kernel beyond allowing keyed state and state-based paint.

## Markup

```text
LATER
```

Strategically valuable, but only after Rust/internal model stability.

Target:

```text
markup frontend
→ same semantic graph
```

No JavaScript. No HTML compatibility promise.

## SVG

```text
OPTIONAL / LATER
```

Preferred first implementation direction:

```text
static/build-time SVG
→ raster GPE Image
```

A `usvg/resvg`-style dependency is plausible but must be measured and feature-gated if adopted.

## Inspector

```text
LATER
```

Textual dumps first.

## Advanced accessibility

```text
LATER
```

Kernel should retain stable semantics/identity hooks.

## Advanced text shaping

```text
LATER / OPTIONAL
```

Do not block future integration by assuming ASCII metrics.

---

# 6. Small-consumer minimum

Strategic success criterion:

A tiny game should be able to conceptually write something near:

```rust
// CANDIDATE — NOT VALIDATED BY COMPILATION
let output = ui::run(frame, &mut ui_state, |ui| {
    ui.panel(|ui| {
        ui.text("PAUSED");
        ui.button("resume", "RESUME").action(RESUME);
    });
});

if output.action_pressed(RESUME) {
    resume();
}
```

This exact API is not approved.

The point is the concept budget:

```text
UiState
run
container
text
button
semantic action
output
```

A tiny consumer should not need to understand:

```text
DOM nodes
stylesheets
signals
component factories
reactive stores
layout engines
markup compiler
```

---

# 7. Candidate richer usage

```rust
// CANDIDATE — NOT VALIDATED BY COMPILATION
let output = ui::run(frame, &mut ui_state, |ui| {
    ui.column(|ui| {
        ui.heading("ARCADE");

        ui.grid(
            Grid::auto_fit()
                .min_cell_width(144)
                .gap(8),
            |ui| {
                for game in catalog.entries() {
                    ui.keyed(game.id(), |ui| {
                        arcade_card(ui, game)
                            .action(game.launch_action());
                    });
                }
            },
        );
    });
});
```

Desired properties:

- consumer loop is ordinary Rust;
- catalogue identity is reused;
- layout details are not hand-coded Rect arithmetic;
- custom `arcade_card` is normal composition/custom paint;
- no gameplay callback lives inside widget state.

Verdict:

```text
CONCEPTUALLY ERGONOMIC
REQUIRES MFE
```

---

# 8. MFE candidate comparison

Qualitative only.

| MFE | Risk addressed | Uncertainty | Cost-to-learn | Information gain |
|---|---|---|---|---|
| Tiny menu only | small-game/API | medium | low | medium |
| Stable identity dynamic list only | identity | high | low-medium | medium |
| Responsive layout only | layout | high | medium | high |
| Semantic multimodal navigation only | input/focus | medium | medium | medium |
| Custom HealthOrb | escape hatch/paint | high | medium | medium-high |
| **Responsive Card Grid + dynamic structure + multimodal input** | **layout + identity + focus + custom widget + headless + Rust API** | **high** | **medium** | **very high qualitatively** |

Recommended:

> **Responsive Card Grid + dynamic structure + multimodal input**

Not because Arcade currently requires a framework, but because it is a compact design probe covering multiple strategic risks.

---

# 9. MFE-001 — Responsive Card Grid + Dynamic Structure + Multimodal Navigation

## 9.1 Hypotheses

### H1 — Hybrid transient graph

A transient UI graph can support same-frame responsive layout without requiring a permanent retained widget tree.

### H2 — Stable identity

Hierarchical/default identity plus explicit keys can preserve focus/capture correctly across dynamic structural changes.

### H3 — Integer-first layout

A small constraint/grid model can produce useful responsive card layouts while preserving pixel-deterministic Rects.

### H4 — Semantic navigation

Keyboard and gamepad can feed one semantic navigation model while pointer/touch remain spatial inputs.

### H5 — Custom component escape hatch

A Card composed from image/text/panel plus custom paint can participate in measurement, focus, pointer interaction and actions without engine-internal hacks.

### H6 — Headless-first

Most correctness can be validated from text/layout/event traces without opening a GPE window.

### H7 — Small-game discipline

The same kernel can still express `Panel + Text + Button` without excessive boilerplate or mandatory dependencies.

---

# 10. MFE scope

Implement in a future authorized slice only:

```text
experimental UI core sufficient for MFE
integer Constraints / Size / Rect
Column
Grid auto-fit or equivalent minimal responsive grid
Text
Image
Panel
Button/Card interaction
stable UiId/key
UiStateStore
semantic nav: Up/Down/Left/Right/Confirm/Cancel
mouse pointer click
touch Started/Moved/Ended/Cancelled
focus
UiEvent / ActionId output
headless dump
```

Probe data:

```text
6–9 fake Arcade-like entries
stable IDs
title
optional card image
```

Dynamic structure test:

```text
toggle/filter one entry while focus is elsewhere
reorder keyed entries
```

No actual external game launch required.

---

# 11. MFE explicit non-goals

```text
no legacy migration
no PauseGame migration
no production Arcade integration
no markup
no SVG
no animation framework
no inspector
no accessibility backend
no outline-font redesign
no Taffy dependency
no separate crate
no public API stabilization
no runtime hot reload
```

---

# 12. MFE candidate API rules

The exact API is deliberately unfrozen.

It must provide enough code to evaluate:

```text
borrow friction
ID friction
event handling friction
custom widget friction
layout readability
```

At least three compiled call sites are required:

## Tiny

```text
Panel + Text + Button
```

## Settings-like

```text
Column + Toggle + Slider
```

## Card grid

```text
responsive Grid + custom Card + events
```

If the design only feels good for the Card Grid but makes Tiny UI cumbersome:

```text
MFE FAILS small-consumer hypothesis
```

---

# 13. MFE headless tests

## Layout fixtures

At least:

```text
small
medium
wide
```

No hard requirement that they correspond to specific physical devices.

Assert:

```text
deterministic columns
all Rects inside content bounds
no overlap
integer positions
stable gap rules
same input -> exact same dump
```

## Identity

```text
focus card C
insert unrelated keyed card before A
focus remains C

remove focused card C
deterministic fallback selected

reorder keyed list
state follows key, not ordinal
```

## Navigation

```text
Right
Left
Up
Down
Confirm
```

Across incomplete final row and page/layout-width change.

## Pointer

```text
hover
press
release
leave
```

## Touch

```text
Started
Moved
Ended
Cancelled
same-touch capture
```

## Modal/focus scope

Optional only if MFE stays small; do not expand scope merely to test Pause.

---

# 14. Measurements

Record, do not pre-judge:

```text
allocations/frame
allocated bytes/frame
node count
build time
layout time
interaction time
paint prep time
persistent state size
native binary delta
WASM package delta if practical
```

Comparison:

```text
Tiny probe
Card Grid probe
```

No fabricated threshold.

Human judgment may still say an observed cost is unacceptable for GPE even without an industry-standard numeric limit.

---

# 15. Rust ergonomics gate

This is mandatory.

Questions:

1. Can a user understand a tiny UI without learning the internal graph?
2. Are explicit IDs rare and explainable?
3. Does semantic `ActionId` wiring become noisy?
4. Can consumer-owned state be updated without borrow gymnastics?
5. Is custom Card code local and readable?
6. Does consuming events after `run()` feel natural?
7. Is one-frame visible product-state latency noticeable or architecturally awkward?
8. Does any workaround require replaying the user's UI closure?

Hard rule:

> **Do not replay arbitrary consumer UI code as a hidden second pass.**

If correct layout requires a second pass, repeat internal pure graph/layout work, not user side effects.

---

# 16. MFE failure criteria

`FAIL` if any is demonstrated and not cheaply correctable:

```text
tiny UI becomes materially cumbersome
stable identity semantics are confusing/unreliable
responsive layout requires pervasive ad hoc exceptions
custom Card must access private internals
semantic navigation cannot handle grid focus cleanly
event-after-build model causes severe Rust/UX friction
per-frame graph cost is clearly incompatible with GPE goals
native/Web-neutral core cannot be maintained
```

`PASS WITH CONDITIONS` if architecture is sound but one bounded issue needs revision.

`PASS` only if:

```text
functional hypotheses survive
tests are strong
Rust API is acceptable
cost measurements are understood
no hidden browser/framework machinery appeared
```

---

# 17. Rollback boundary

MFE implementation must be isolated.

At all times:

```text
current Ui remains available
existing consumers remain unchanged
```

A failed MFE means:

```text
delete/revert experimental path
retain Phase 0 documents
revise Architecture B/A
```

No sunk-cost migration.

---

# 18. Post-MFE decision tree

```text
MFE PASS
→ independent review
→ choose first real consumer migration

MFE PASS WITH CONDITIONS
→ one bounded correction slice
→ rerun gates

MFE FAIL due ergonomics
→ revisit Architecture A / immediate facade

MFE FAIL due layout implementation burden
→ targeted Taffy comparison spike

MFE FAIL due cost
→ profile/flatten/reuse OR reduce architecture

MFE reveals separate-crate pressure
→ only then study gpe-core / crate extraction
```

---

# 19. Final Phase 0 v0.3 decisions

## Keep in kernel

```text
explicit persistent UI state ownership
stable UI identity concept
constraints / integer layout result
transient semantic description
focus/capture basics
semantic UI events/actions
headless dump capability
custom component escape hatch
```

## Keep as core modules

```text
Row / Column / Stack
bounded Grid
basic style/theme
semantic navigation adapter
basic widgets
scroll
```

## Optional capability

```text
animation
advanced layout backend
SVG tooling/runtime adapter
accessibility projection
```

## Later

```text
markup
hot reload
graphical inspector
world-space adapter
advanced text shaping
```

## Reject for v1

```text
JavaScript
full HTML
full CSS cascade
DOM compatibility
full browser event model
permanent framework-owned gameplay state
new renderer
global UI manager
crate-per-concept
```

---

# 20. Markup verdict

```text
LATER
```

Reason:

Strategically valuable and Architecture B deliberately preserves a shared target model, but syntax/runtime should not influence kernel before the Rust MFE proves identity/layout/event semantics.

---

# 21. SVG verdict

```text
OPTIONAL / LATER
```

Preferred first route:

```text
build-time or static SVG rasterization
```

using a mature parser/rasterizer if later justified.

---

# 22. Final confidence

```text
Evolve within src/ui:
EVIDENCE-BACKED + STRATEGY-BACKED

Architecture B:
STRATEGY-BACKED + PRIOR-ART-BACKED + REQUIRES MFE

Stable identity:
EVIDENCE-INFORMED + PRIOR-ART-BACKED + STRATEGY-BACKED

Integer constraint layout:
STRATEGY-BACKED + PRIOR-ART-BACKED + REQUIRES MFE

Semantic actions:
OBSERVED-PRESSURE + PRIOR-ART-BACKED + STRATEGY-BACKED

Markup later:
STRATEGY-BACKED + PRIOR-ART-BACKED

Static-first SVG:
PRIOR-ART-BACKED + STRATEGY-BACKED

No separate crate now:
EVIDENCE-BACKED
```

---

# 23. Phase status

```text
PHASE 0 v0.3:
COMPLETE — READY FOR INDEPENDENT ADVERSARIAL REVIEW
```

# STOP

Do not implement MFE-001 until Phase 0 v0.3 receives an independent adversarial review.
