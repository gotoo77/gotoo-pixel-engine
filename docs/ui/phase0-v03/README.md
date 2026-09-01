# GPE.UI Phase 0 v0.3 — Strategic Capability Research

Status: **COMPLETE — READY FOR INDEPENDENT ADVERSARIAL REVIEW**

Authoritative mission:

`docs/GPE_UI_PHASE_0_STRATEGIC_CAPABILITY_MASTER_MISSION_v0.3.md`

## Baselines

```text
GPE code baseline audited:
6ff4f8baddae269baa6a7d182f0ba0c9d985f886

Gate 0:
27af0fd096878ea84da5f41716c7dde7a031a6b7

Gate 1 / Gate 2:
c1a3646199705525d1ae8caa03b67f19a556cdbd

Phase 0 v0.3 mission commit:
b45c643de673eea981bf45e0dec21ae71ec6cfc2
```

The Gate 0/1/2 evidence remains valid, but Phase 0 v0.3 is a **capability-R&D mission** rather than a proof that current consumers are blocked.

## Executive result

The study recommends **Architecture B — Balanced Hybrid**, implemented initially **inside the existing `src/ui` boundary**, not as a new crate.

The core idea is:

```text
Rust composition API
        ↓
transient per-frame UI graph
        ↓
stable/hierarchical identity where state requires it
        ↓
integer-first constraint layout
        ↓
focus / pointer / semantic navigation resolution
        ↓
semantic UI events/actions
        ↓
paint through existing GPE framebuffer/text/image primitives
```

Persistent UI state is separate from the transient graph:

```text
UiStateStore
- focus
- capture
- scroll
- optional widget-local interaction state
```

Gameplay/product state remains consumer-owned.

This is a **hybrid** model:

- immediate/declarative *description each frame*;
- retained *interaction state keyed by UI identity*;
- multi-pass measure/layout/interaction/paint allowed internally;
- no DOM;
- no browser compatibility;
- no requirement for a permanently retained widget object tree.

## Core recommendation

Keep v1 inside GPE:

```text
gpe::ui
├── legacy          compatibility with current Ui/UiState path
├── core            identity, constraints, frame graph, events
├── layout          row / column / stack / grid / anchors
├── input           semantic navigation + pointer/touch normalization
├── widgets         panel / text / image / button / toggle / slider / etc.
├── style           theme/tokens/style resolution
└── debug           textual dumps / traces, preferably debug-only
```

Heavy or authoring-oriented capabilities remain optional/later:

```text
animation           optional capability
markup              LATER; frontend to same internal model
SVG                 OPTIONAL/LATER; prefer static/build-time raster strategy first
inspector           LATER; debug-only if introduced
advanced text       LATER/optional
advanced a11y       LATER, but kernel must not create an obvious dead-end
```

## Why not a separate `gpe-ui` crate now?

GPE.UI fundamentally consumes existing GPE types such as:

```text
Framebuffer
Rect
Size
Input
ActionId / ControlMap
TextRenderer / Font
Image / ImageFit / ImageFilter
```

A new crate depending on `gotoo-pixel-engine` could not then be re-exported by the engine without creating a dependency cycle. Solving this cleanly would require a broader `gpe-core` extraction or accepting a separate top-level consumer dependency.

That is not justified yet.

Logical modularity is required now. Physical crate separation is a future decision.

## Most important architectural risk

The recommended transient graph allows correct same-frame multi-pass layout, but it changes the ergonomics of the current style:

```rust
if ui.button("RESET").clicked {
    reset();
}
```

If interaction is resolved after graph construction, output is naturally consumed after `finish()` as events/actions.

That may be excellent architecturally and unpleasant ergonomically.

**This cannot be settled on paper.**

MFE-001 therefore makes Rust ergonomics and borrow behavior a first-class falsification target.

## MFE-001

Recommended experiment:

> **Responsive Card Grid + Dynamic Structure + Multimodal Navigation**

It deliberately combines:

- responsive integer layout;
- stable identity under structural change;
- keyboard/gamepad semantic navigation;
- mouse/touch spatial interaction;
- a custom composite Card widget;
- headless layout/focus/event traces;
- existing GPE rendering/image primitives;
- no markup, SVG, animation, inspector, or new external dependency.

A tiny `Panel + Text + Button` probe is included in the same MFE only as a small-consumer control sample.

## Files

- `BASELINE_AND_STRATEGIC_REQUIREMENTS.md`
- `PRIOR_ART.md`
- `KERNEL_LAYOUT_INPUT_STYLING.md`
- `MODULARITY_ARCHITECTURE_CANDIDATES.md`
- `TESTING_MIGRATION_ADVERSARIAL.md`
- `SYNTHESIS_AND_MFE_001.md`

## Absolute stop

No runtime implementation is authorized by this Phase 0.

Next step:

> **independent adversarial review of Phase 0 v0.3**
