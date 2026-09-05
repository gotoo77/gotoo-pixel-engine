# GPE.UI — PRODUCTIONIZATION PLAN v0.1

Status: **ACTIVE PLAN — P0/P1/P2 COMPLETE, P3 PASS WITH CONDITIONS, P4 ACTIVE**

Baseline:

`7a89ddc09ed941c409b2b459e1361fbfb80e639f`

Prerequisites:

```text
Phase 0 v0.3   PASS
MFE-001A       PASS
MFE-001B       PASS
MFE-001C       PASS WITH CONDITIONS
Architecture B GO
P0             PASS
P1             PASS
P2             PASS
P3             PASS WITH CONDITIONS
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

## 0. Productionization convergence checkpoint

P0 and P1 removed the experimental split at the interaction-kernel level. P2 has now converged the proven layout algorithms and passed its Native human runtime gate.

The productionization checkpoint is:

```text
one UiId model
one UiInput vocabulary
one UiInteractionState authority
one UiInteractionOutput semantic result
one generic kernel interaction pass
Linear / Spatial as deterministic navigation policies
one generic responsive Grid algorithm
one generic vertical child-placement algorithm
```

The compatibility modules still exist as evidence/migration surfaces, but they no longer justify separate interaction or layout runtimes.

The original prohibition remains in force:

```text
DO NOT simply rename experimental.rs -> core.rs and call it production.
DO NOT stabilize SpatialState/SpatialOutput as an independent second UI runtime.
DO NOT create a permanent Card-only spatial subsystem beside the main UI transaction.
```

P0 result:

`docs/ui/GPE_UI_PRODUCTIONIZATION_P0_RESULT_v0.1.md`

P1 result:

`docs/ui/GPE_UI_PRODUCTIONIZATION_P1_RESULT_v0.1.md`

P2 result:

`docs/ui/GPE_UI_PRODUCTIONIZATION_P2_RESULT_v0.1.md`

P3 result:

`docs/ui/GPE_UI_PRODUCTIONIZATION_P3_RESULT_v0.1.md`

---

# 1. Compatibility policy

The existing legacy toolkit remains available during productionization:

```text
Ui
UiState
UiTheme
UiResponse
RepeatState
```

Do not delete or silently redirect it during kernel work.

The experimental Architecture B surfaces remain compatibility/evidence adapters until an explicit later migration slice decides their production names and removal path.

---

# 2. Productionization sequence

## P0 — Kernel convergence

Status: **PASS / STOP**.

Converged:

```text
shared input vocabulary
shared interaction state primitives
generic resolved targets
semantic output convergence
adapter closure over shared state/output primitives
```

## P1 — Unified input / interaction

Status: **PASS / STOP**.

Converged:

```text
one kernel interaction pass
explicit Linear / Spatial navigation policy
shared hover
shared pointer/touch capture semantics
shared confirm activation
shared cancel propagation
Spatial adapter delegation
transactional full-UiInput entry points
nav-only compatibility adapters preserved
```

P1 did not change platform/input transport semantics.

## P2 — Consumer-grade layout composition

Status: **PASS / STOP**.

Validated implementation head:

`2170c3c2656fb247ae682f439986c34e2ddfb104`

CI #668: PASS.

Human Native runtime: PASS.

Converged:

```text
shared integer/pixel-aware layout module
one responsive Grid track algorithm
one vertical child-placement algorithm
Spatial compatibility Grid delegates to shared cells
transactional UiBuilder::grid composes arbitrary widgets
Root/Column/Panel final vertical placement uses shared primitive
```

P2 changed resolved geometry but did not change platform/render/input transport semantics. Web build/package passed; a browser runtime rerun was therefore not required.

## P3 — Styling / theming / customization

Status: **PASS WITH CONDITIONS / HUMAN RUNTIME PENDING**.

Implemented:

```text
typed style vocabulary
UiTheme compatibility defaults
component style
explicit local override
focus / hover / active visual overlays
transactional styled entry points
Spatial DefaultCardPainter style alignment
dedicated Native visual probe
```

Remaining conditions:

```text
Final Native runtime confirmation for the revised P3 probe (layout, toggle, slider)
all-target Clippy allocator conflict outside P3 UI changes
```

Strategic requirement, not speculative polish.

Target precedence:

```text
Theme defaults
    ↓
component style
    ↓
local override
```

Candidate production capabilities:

```text
background
border
border width
text / muted text
accent
padding / spacing
hover
focus
pressed
disabled
corner radius / rounded rect where renderer support is appropriate
custom painter / custom widget escape hatch
sprite / nine-slice where consumer evidence supports it
```

Do not turn this into CSS.

P3 is **not started by the P2 closure**. It must begin explicitly from the merged P2 `main` baseline on its own implementation branch.

## P4 — Typography follow-up

Status: **ACTIVE / BITMAP AND EXPERIMENTAL 20-FONT GALLERY IMPLEMENTED**.

The maintainer explicitly authorized local P4 work on 2026-09-05 ("go P4").
This does not assert that P3 was merged or that its outstanding gates passed.

Audit, implementation evidence and remaining outline-font decision:
`docs/ui/GPE_UI_PRODUCTIONIZATION_P4_RESULT_v0.1.md`.

Track issue #69 separately.

- explicit missing-glyph behavior;
- preserve bitmap pixel font option;
- improve typography quality;
- assess TTF/OTF path without forcing a giant text stack.

## P5 — Cost attribution / debug boundary

MFE-001C found no CPU red flag but allocation attribution remains a productionization condition.

- distinguish mandatory runtime output from opt-in debug dump;
- remeasure allocation calls/bytes after convergence;
- no optimization before attribution.

## P6 — First real consumers

Only after kernel/layout/styling are coherent enough:

1. Arcade-like card/grid consumer;
2. unlike consumer such as Settings/Pause/HUD.

A single consumer must not dictate the final API.

## P7 — v1 candidate review

Only after at least two unlike consumers:

- API ergonomics;
- customization coverage;
- deterministic/headless testing;
- Native/Web behavior;
- cost/module surface;
- migration from legacy `src/ui`;
- decide what can be called GPE.UI v1 candidate.

---

# 3. Explicit non-goals before their slice

Do not opportunistically implement:

```text
styling features before P3
rounded corners before P3
typography changes outside P4
markup
SVG
Taffy integration without evidence
animation without an explicit slice
Arcade migration before consumer gates
legacy UI removal before migration review
public API freeze before P7
```

---

# 4. Branch / review discipline

Use one active implementation branch per explicit productionization slice.

Reviews and checkpoints are commits/PR state on that branch, not branch proliferation.

CI green proves build/test/package gates only; human runtime gates remain explicit where required.

Rust changes must be formatter-clean before commit/push; CI is a validator, not the primary rustfmt detector.

---

# 5. Current STOP

```text
P0 = PASS
P1 = PASS
P2 = PASS
P3 = PASS WITH CONDITIONS (local implementation and probe follow-ups)
STOP
```

Active implementation slice: P4 typography, tracked by issue #69 and
`docs/ui/phase0-v03/TEXT_RENDERING_FOLLOWUP.md`.

The maintainer's explicit P4 start supersedes the local sequencing hold.
P3 closure/merge evidence remains outstanding and is not manufactured here.
P4 bitmap audit and the explicitly requested twenty-font outline gallery are
implemented. Component-wide font styling, automatic fallback and browser runtime
validation remain open; see the P4 result for the experimental integration.
