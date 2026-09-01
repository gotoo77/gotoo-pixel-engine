# Modularity and Architecture Candidates

---

# 1. Physical boundary problem

The strategic desire for modularity does **not** automatically imply a separate crate.

Current GPE UI-adjacent code consumes types owned by the engine crate:

```text
Framebuffer
Rect
Size
Input
ActionId
ControlMap
Image
TextRenderer
Pixel
```

A standalone `gpe-ui` crate that depends on `gotoo-pixel-engine` creates this shape:

```text
gpe-ui
  ↓
gotoo-pixel-engine
```

If the engine then wants to re-export `gpe-ui`, it would require:

```text
gotoo-pixel-engine
  ↓
gpe-ui
  ↓
gotoo-pixel-engine
```

which is a Cargo dependency cycle.

Avoid solving a UI problem by prematurely extracting half the engine into `gpe-core`.

---

# 2. Recommended v1 modularity

Keep the public namespace:

```text
gpe::ui
```

but introduce strong internal module boundaries.

Candidate:

```text
src/ui/
├── mod.rs
├── legacy.rs
├── core/
│   ├── id.rs
│   ├── state.rs
│   ├── graph.rs
│   ├── constraints.rs
│   └── event.rs
├── layout/
│   ├── row.rs
│   ├── column.rs
│   ├── stack.rs
│   ├── grid.rs
│   └── anchor.rs
├── input/
│   ├── navigation.rs
│   ├── pointer.rs
│   └── focus.rs
├── widgets/
│   ├── panel.rs
│   ├── text.rs
│   ├── image.rs
│   ├── button.rs
│   ├── toggle.rs
│   ├── select.rs
│   ├── slider.rs
│   └── scroll.rs
├── style/
│   ├── theme.rs
│   └── component.rs
└── debug/
    └── dump.rs
```

This is conceptual, not a file-count mandate.

The important separation is dependencies, not folders.

---

# 3. Candidate dependency DAG

```text
GPE primitives
Framebuffer / Rect / Size / Input / ActionId / Image / Text
          ↑
          │
       ui::core
  identity / state / constraints / events
          ↑
     ┌────┼─────────────┐
     │    │             │
 ui::input ui::layout ui::style
     │    │             │
     └────┴──────┬──────┘
                 ↑
             ui::widgets
                 ↑
             ui::debug
```

Optional future layers:

```text
ui::animation
        ↑
core/layout/style only

ui::markup
        ↑
validated description / core concepts
must not depend on widgets' concrete runtime internals unnecessarily

ui::svg
        ↑
Image / asset conversion boundary
must not be required by ui::core
```

Game feedback:

```text
UiEvent / ActionId
        ↓
consumer or optional feedback helper
        ↓
Audio / haptic backend
```

Direction must **not** become:

```text
ui::core → Audio
ui::core → game scene
ui::core → platform window
```

---

# 4. Pay-for-what-you-use

## Minimal path

A consumer using:

```text
Panel
Text
Button
```

should conceptually need only:

```text
ui core
basic layout
basic widgets
basic style
existing GPE text/framebuffer/input primitives
```

No new mandatory external dependency should be required by the architecture.

## Optional heavy capabilities

Future Cargo feature candidates, only if actual dependencies justify them:

```text
ui-svg
ui-markup
ui-inspector
ui-animation
ui-accessibility
ui-advanced-layout
```

Do not add feature flags merely for small source modules with no meaningful dependency/build impact.

---

# 5. Architecture 0 — Evolve current toolkit only

Definition:

> Keep the current `Ui` one-pass immediate model and incrementally add more methods/layout helpers.

Possible changes:

```text
row()
column()
grid()
semantic controls
style structs
explicit ids in selected widgets
```

## Strengths

- smallest migration;
- preserves current direct `UiResponse` ergonomics;
- zero new architectural machinery;
- easy small-game story.

## Weaknesses

- sophisticated same-frame measure/layout remains awkward;
- identity/state semantics grow ad hoc;
- responsive nested layout can turn one-pass cursor logic into special-case accumulation;
- custom widget protocol remains difficult to regularize;
- future markup has no obvious neutral semantic target;
- headless layout dumps become less natural.

## Verdict

```text
REJECT AS FINAL STRATEGIC ARCHITECTURE
KEEP AS MIGRATION / CONTROL BASELINE
```

Reason:

It can extend useful capability, but it risks accumulating exactly the special cases that a deliberate Phase 0 is intended to avoid.

---

# 6. Architecture A — Minimal compositional immediate core

Definition:

- preserve immediate call style;
- add explicit layout scopes (`row`, `column`, `grid`);
- introduce `UiId` for stateful cases;
- normalize semantic nav input;
- still resolve most layout/interaction as calls execute.

## Strengths

- familiar ergonomics;
- modest state model;
- small runtime;
- likely easy migration.

## Weaknesses

- intrinsic/respond-to-child-size layout remains constrained;
- complex centering/wrapping/grid may require extra passes or previous-frame geometry;
- future markup still awkward;
- custom components either inherit immediate limitations or bypass layout.

## Verdict

```text
VIABLE FALLBACK
```

Could win if MFE shows the transient-graph architecture is significantly less ergonomic or too costly.

---

# 7. Architecture B — Balanced Hybrid

Definition:

- Rust describes UI each frame;
- description produces a transient frame graph;
- persistent interaction state is keyed separately;
- measure/layout happens before final interaction/paint;
- semantic events/actions are returned;
- existing renderer/text/image/input primitives are reused;
- no mandatory external layout engine;
- heavier capabilities remain optional.

## Strengths

### Strategic coverage

Naturally supports:

```text
responsive layout
stable identity
custom widgets
headless layout
semantic events
future markup frontend
debug dumps
```

### Small-game control

The graph can remain tiny when the UI is tiny, provided implementation avoids mandatory heavyweight dependencies and excessive allocation.

### Migration

Current `Ui` can remain as a compatibility path while the new model proves itself.

### Future authoring

Rust and markup can target the same transient semantic model.

## Weaknesses / unresolved risks

1. Rust response ergonomics:
   interaction may be available only after graph completion.
2. Allocation cost:
   transient graph construction must be measured.
3. API complexity:
   layout/style/node builders can become verbose.
4. identity ergonomics:
   automatic vs explicit ID rules must be understandable.
5. multi-pass cost:
   expected to be small for game menus but must be measured.

## Verdict

```text
RECOMMENDED
REQUIRES MFE
```

---

# 8. Architecture C — Ambitious retained/declarative platform

Definition:

- retained component tree;
- property/binding runtime;
- declarative markup from the beginning;
- rich style system;
- integrated animation/inspector;
- broad text/accessibility semantics.

## Strengths

- maximum authoring/tooling potential;
- natural hot reload;
- strong retained identity;
- sophisticated dynamic UI.

## Weaknesses

- large mental/runtime/API surface;
- risks global framework behavior;
- dependency/tooling growth;
- harder migration;
- easy route to browser/Unity/Qt-like complexity;
- no evidence that GPE needs a property/reactivity runtime.

## Verdict

```text
REJECT FOR V1
PRESERVE SELECTED COMPATIBILITY PRESSURES ONLY
```

---

# 9. Comparison matrix

Qualitative only.

| Criterion | Architecture 0 | A Minimal | B Balanced Hybrid | C Ambitious |
|---|---|---|---|---|
| Strategic capability coverage | weak-medium | medium | **strong** | very strong |
| Small-game conceptual cost | **excellent** | strong | must prove | weak |
| Existing migration | **excellent** | strong | strong via coexistence | weak |
| Same-frame rich layout | weak | medium | **strong** | strong |
| Stable identity | local retrofit | supported | **native concept** | native |
| Headless layout model | limited | medium | **strong** | strong |
| Custom widgets | ad hoc | medium | **strong target** | strong |
| Semantic actions/traces | retrofit | supported | **native target** | native |
| Future markup | awkward | medium | **clean frontend target** | native |
| Rust direct response ergonomics | **proven** | likely strong | **risk / MFE** | heavier |
| New dependency requirement | none | none expected | none expected | likely |
| Browser-accident risk | low | low | controllable | high |
| Enterprise-framework risk | low | low-medium | medium | high |
| API lock-in risk | medium via ad hoc growth | medium | controllable by MFE | high |

---

# 10. Why Architecture B is not "framework for framework's sake"

It is justified by strategic requirements that are difficult to serve cleanly in the current execution model:

```text
responsive composition
stable interaction identity
headless exact layout
custom component protocol
future Rust/markup shared model
semantic event traces
```

It is deliberately constrained by the existing engine:

```text
existing Framebuffer
existing TextRenderer
existing Image
existing Input/ControlMap
existing explicit UiState ownership
no new renderer
no new event loop
no global runtime
```

---

# 11. Crate boundary verdict

```text
NOW:
gpe::ui internal/public modules inside gotoo-pixel-engine

NOT NOW:
separate gpe-ui crate
gpe-core extraction
multi-crate UI workspace
```

Revisit a separate crate only if at least one becomes true:

1. non-GPE applications want to consume the UI runtime;
2. GPE core primitives are independently extracted for other reasons;
3. optional UI dependencies materially harm engine consumers;
4. independent UI release/version cadence becomes valuable;
5. compilation measurements show physical separation provides meaningful benefit.

---

# 12. Markup boundary

Future target:

```text
Rust builder ─────────┐
                      ├─> validated transient semantic graph
markup compiler ──────┘
```

Not:

```text
Rust runtime A
markup runtime B
```

This is a strategic reason for Architecture B but **not** a mandate to implement markup during v1.

---

# 13. SVG boundary

Preferred future direction:

```text
SVG source
  ↓ optional tool/build step
usvg-like simplified tree
  ↓
rasterize
  ↓
GPE Image
```

Runtime SVG stays outside core.

---

# 14. Decision confidence

| Decision | Confidence basis |
|---|---|
| Keep v1 inside `src/ui` | EVIDENCE-BACKED + STRATEGY-BACKED |
| Do not split crate now | EVIDENCE-BACKED |
| Architecture B recommended | STRATEGY-BACKED + PRIOR-ART-BACKED + REQUIRES MFE |
| Transient graph vs permanent tree | STRATEGY-BACKED + PRIOR-ART-BACKED + REQUIRES MFE |
| Stable Ui identity | OBSERVED-PRESSURE + STRATEGY-BACKED + PRIOR-ART-BACKED |
| Custom integer constraint layer first | STRATEGY-BACKED + PRIOR-ART-BACKED + REQUIRES MFE |
| Taffy not mandatory | UNKNOWN COST + SMALL-GAME CONSTRAINT |
| Markup later, same model | STRATEGY-BACKED + PRIOR-ART-BACKED |
| SVG optional/static-first | PRIOR-ART-BACKED + STRATEGY-BACKED |
