# Testing, Migration and Adversarial Review

---

# 1. Testing architecture

Testability is part of the recommended kernel design.

The key artifact should be a deterministic headless frame result.

Candidate:

```text
UiFrameDump
- node ids
- node kinds
- parent/child relation
- resolved Rects
- focus state
- hover/active state
- emitted events/actions
- diagnostics
```

Paint commands may be included separately if useful.

---

# 2. Unit tests

## Identity

```text
stable path identity
explicit keyed identity
duplicate key diagnostics
dynamic list reorder
conditional sibling insertion/removal
state cleanup for vanished nodes
```

## Constraints

```text
min <= result <= max
tight constraints respected
zero-size parent
saturating/overflow edges
nested padding
```

## Layout

```text
Row
Column
Stack
Grid
alignment
weighted space
gaps
incomplete grids
scroll viewport
anchor escape hatch
```

## Input

```text
semantic nav
pointer click
touch capture/cancel
focus restoration
modal scope
disabled widgets
```

---

# 3. Property-based tests

High-value invariants:

```text
children stay inside parent unless overflow policy permits otherwise
no negative logical extents
sum of deterministic linear allocation equals parent content extent
same input graph + same constraints -> same layout
same semantic input trace -> same focus/events
focus always references a live focusable node or NONE
captured pointer/touch always resolves deterministically on cancellation/removal
```

Use property testing only where it earns value; adding a property-test dependency is an implementation decision, not authorized here.

---

# 4. Golden layout tests

Input:

```text
semantic UI description
surface size
style tokens
```

Expected output:

```text
exact textual tree + Rects
```

Example:

```text
root Column [0,0 320x180]
  title Text [8,8 304x16]
  grid Grid [8,28 304x120]
    card#snake [8,28 96x54]
    card#tetris [112,28 96x54]
    ...
```

Advantages:

- reviewable in Git;
- deterministic;
- no GPU;
- excellent regression signal for responsive changes.

Golden files should be used selectively, not for every trivial widget.

---

# 5. Render snapshots

Verdict:

```text
OPTIONAL
```

Useful for:

```text
nine-slice
sprite skins
complex clipping
custom paint
text alignment
```

Risks:

```text
font/render implementation changes
large fixture churn
platform differences
```

Prefer semantic/layout goldens before pixel snapshots.

---

# 6. Input trace replay

A key strategic capability.

Example:

```text
initial focus = card:snake

Right
Right
Down
Confirm
```

Expected:

```text
focus transitions:
snake -> tetris -> invaders -> breakout

events:
Activated(breakout, action=arcade.launch.breakout)
```

Trace format should use semantic navigation, not physical keys.

Pointer trace:

```text
Move(120, 60)
Press(Primary)
Move(130, 60)
Release(Primary)
```

Touch trace keeps contact IDs.

---

# 7. Debug textual observability

At minimum debug builds should be able to expose:

```text
layout dump
focus dump
event trace
identity path
style resolution summary
```

This is more important in v1 than a graphical inspector.

Graphical inspector verdict:

```text
LATER
```

Revisit if debugging the MFE is materially painful without it.

---

# 8. Native / Web pressure

The new core should avoid assumptions about:

```text
filesystem
threads
system fonts
clipboard
desktop-only pointer APIs
native handles
blocking I/O
```

Use existing GPE normalized primitives.

Phase 0 code-level expectation:

```text
same semantic/layout core on native and WASM
platform layer only supplies normalized input/render surface
```

Runtime result remains:

```text
RUNTIME VALIDATION REQUIRED
```

A WASM build is not a browser-runtime PASS.

---

# 9. Performance measurements for MFE

Do not invent targets before measurement.

Collect:

```text
allocations/frame
bytes allocated/frame
Ui graph node count
build time
measure/layout time
interaction time
paint preparation time
total UI time
persistent UiStateStore size
binary-size delta
WASM-size delta if practical
```

Compare at least:

```text
Tiny UI
Responsive Card Grid
```

Against the current toolkit or direct consumer implementation where comparison is meaningful.

---

# 10. Migration map

## `RepeatConfig` / `RepeatState`

```text
classification: KEEP
target: reusable repeat/input helper
```

No reason to rewrite unless new semantic-nav integration exposes an actual limitation.

## `UiTheme`

```text
classification: GENERALIZE
target: default token/style seed
```

Compatibility adapter:

```text
UiTheme -> new default Theme/StyleSet
```

## `UiResponse`

```text
classification: WRAP
target: compatibility facade over new interaction/event output where semantics match
```

Do not promise exact compatibility until MFE proves same-frame timing.

## `UiState`

```text
classification: RETHINK INTERNALS / WRAP
target: UiStateStore or compatibility-owned state surface
```

Keep explicit ownership.

## `Ui`

```text
classification: KEEP DURING MIGRATION / DEPRECATE LATER ONLY IF REPLACEMENT WINS
```

Do not delete current `Ui` during MFE.

## `MenuState`

```text
classification: KEEP
```

May remain a tiny consumer utility even after a more capable UI exists.

## `standard_menu_controls`

```text
classification: GENERALIZE / KEEP
target: default semantic navigation binding adapter
```

## `draw_panel`, `draw_text_centered`, `draw_menu_item`

```text
classification: KEEP low-level or legacy
```

They remain useful outside the structured UI.

## `VirtualPad`

```text
classification: KEEP
```

It is an input abstraction, not obsolete UI rendering.

## `PauseGame`

```text
classification: KEEP
migration: later consumer probe
```

Migrate only after the new system is proven.

## `Framebuffer`, `TextRenderer`, `Image`, `ImageFit`, `ImageFilter`

```text
classification: KEEP AS RENDER FOUNDATION
```

GPE.UI should compose them, not hide them behind another renderer abstraction unless proven necessary.

---

# 11. Migration phases

```text
Phase M0
existing UI unchanged

Phase M1
introduce experimental new core beside legacy

Phase M2
MFE only

Phase M3
migrate one bounded consumer
candidate: tool/settings probe OR Arcade card catalogue

Phase M4
human/runtime validation

Phase M5
migrate second different consumer
candidate: Pause or custom HUD

Phase M6
only then evaluate legacy deprecation

Phase M7
remove legacy pieces only when no supported consumer remains
```

Rollback boundary at every phase:

> legacy UI remains functional until new path is independently proven.

---

# 12. Adversarial review

## A — Browser Accident

Finding:

**Material risk.**

Markup + styling + layout could drift toward HTML/CSS/DOM semantics.

Mitigation:

- no HTML compatibility goal;
- no CSS cascade/selectors;
- no JavaScript;
- markup later and optional;
- small internal semantic model first.

Residual risk:

Medium until markup design is reviewed independently.

---

## B — Enterprise Framework

Finding:

**Material but controllable.**

The transient graph/state/style/event layers could acquire factories/providers/managers.

Mitigation:

- value types;
- explicit state;
- plain functions/builders;
- no service locator;
- no DI container;
- no framework-owned gameplay state.

Residual risk:

Low-medium.

---

## C — Small Game

Finding:

**Major falsification pressure.**

Architecture B may be conceptually clean but too expensive/verbose for `Panel + Text + Button`.

Mitigation:

- no new mandatory external dependency;
- tiny default style;
- concise Rust composition;
- MFE includes a tiny control sample;
- measure allocations/binary delta.

Residual risk:

**REQUIRES MFE.**

---

## D — Highly Custom Game

Finding:

**Major.**

A fixed widget catalog would fail the stated vision.

Mitigation:

- first-class custom measurement/paint/interaction semantics;
- framebuffer escape hatch remains accessible;
- no closed rendering tree.

Residual risk:

**REQUIRES MFE API trial.**

---

## E — Rust Ergonomics

Finding:

**Highest current architectural uncertainty.**

A graph that resolves interaction after build naturally wants event consumption after `finish()`, which is less direct than current `UiResponse`.

Potential failure:

```text
too many IDs
too much builder ceremony
borrow conflicts
event matching boilerplate
```

Mitigation:

- MFE must compile realistic snippets;
- retain a simple legacy/immediate facade during experimentation;
- reject Architecture B if ergonomics cannot be made concise without hidden complexity.

Residual risk:

High until MFE.

---

## F — Migration

Finding:

Material.

Risk comes from replacing working code to satisfy architecture aesthetics.

Mitigation:

- coexistence;
- adapters;
- migrate bounded consumers;
- no legacy deletion in MFE.

Residual risk:

Low-medium if discipline is preserved.

---

## G — Test Engineer

Finding:

Architecture B is favorable.

A transient semantic graph plus explicit output makes:

```text
layout goldens
focus traces
event traces
property tests
```

natural.

Risk:

If paint/interaction side effects leak into graph construction, headless determinism degrades.

Mitigation:

keep graph construction as pure-ish as practical.

---

## H — Performance

Finding:

Unknown, therefore material.

Possible costs:

```text
per-frame graph allocation
multi-pass traversal
style resolution
ID hashing
```

No evidence currently shows these are expensive or cheap.

Mitigation:

measure MFE; do not pre-optimize.

Residual risk:

UNKNOWN / REQUIRES MFE.

---

## I — Native/Web

Finding:

No conceptual blocker found.

Architecture uses GPE's platform-neutral framebuffer/input types.

Risks:

- allocation/WASM size;
- future text/SVG dependencies;
- pointer/touch browser behavior.

Mitigation:

keep optional capabilities isolated; actual browser gate later.

---

## J — Future Markup

Finding:

Architecture B creates a plausible frontend target.

Risk:

Rust API could accidentally encode behaviors not representable declaratively, or markup could force stable-ID/property machinery into core.

Mitigation:

design internal graph around semantics/layout/style, not Rust closures.

Residual risk:

Medium; markup intentionally later.

---

## K — Accessibility / i18n

Finding:

No v1 requirement for full implementation, but ignoring semantics/identity entirely would create a costly dead-end.

Mitigation:

- stable IDs;
- semantic role/label slots;
- focus observable;
- dynamic text measurement.

Residual risk:

Low for Phase 0.

---

## L — API Evolution

Finding:

Material.

GPE is version `0.1.0`; UI architecture is still experimental.

Mitigation:

- new core introduced behind clearly experimental naming/feature/module during MFE;
- do not deprecate current UI early;
- MFE may freely revise before public stabilization.

---

## M — Strategic Vision Dilution

Finding:

**CONFIRMED HISTORICALLY IN THIS PHASE.**

The previous methodology reached:

```text
no current structural problem
→ therefore stop general GPE.UI exploration
```

That was valid maintenance logic but invalid capability-R&D logic.

Mitigation:

the v0.3 requirement taxonomy explicitly recognizes strategic requirements.

Residual risk:

Low if this distinction remains explicit.

---

## N — Feature Soup

Finding:

Major.

The vision names layout, style, feedback, animation, markup, SVG, inspector, a11y.

Mitigation:

```text
kernel/core
core modules
optional capabilities
later
```

Markup/SVG/animation do not enter MFE-001.

---

## O — Crate Explosion

Finding:

Concrete architectural issue.

A separate UI crate depending on GPE creates an awkward dependency direction and prevents transparent re-export without a core extraction.

Mitigation:

logical modules first, physical crates later only with evidence.

Residual risk:

Low.

---

# 13. Failure-first review of recommended Architecture B

## Failure 1 — Rust API is unpleasant

Early signal:

- MFE has verbose ID/action plumbing;
- simple settings UI is harder to read than current `Ui`.

Impact:

High.

Mitigation:

revise API or fall back toward Architecture A.

Rollback:

legacy `Ui` unchanged.

## Failure 2 — transient graph allocates too much

Early signal:

- measurable allocation/time spike in tiny UI.

Impact:

Medium-high for GPE philosophy.

Mitigation candidates after measurement:

```text
frame arena
Vec reuse
small-vector optimization
flat graph
node specialization
```

Do not choose before measurement.

## Failure 3 — layout scope expands into CSS clone

Early signal:

- growing property set maps one-to-one to CSS;
- layout behavior becomes hard to explain without browser terminology.

Impact:

High.

Mitigation:

freeze GPE-specific constraint semantics; test only real probes.

## Failure 4 — custom widgets need internal/private hacks

Early signal:

- Card/HealthOrb cannot integrate focus/layout/paint without reaching into private graph internals.

Impact:

High.

Mitigation:

promote the minimal custom-component protocol.

## Failure 5 — markup starts dictating kernel

Early signal:

- kernel APIs are added solely for parser convenience before Rust MFE is stable.

Impact:

Medium-high.

Mitigation:

markup stays LATER until Rust core and identity model are validated.

## Failure 6 — existing consumers are forced to migrate

Early signal:

- deprecation begins before two distinct consumers pass runtime gates.

Impact:

High.

Mitigation:

hard migration gates.

---

# 14. Adversarial status

```text
Architecture B survives Phase 0 adversarial review.

It does NOT yet pass implementation validation.

Main unresolved risks:
1. Rust ergonomics
2. transient graph cost
3. custom-widget extension ergonomics
4. exact stable-ID rules
5. custom layout vs external layout engine
```
