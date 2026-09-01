# GPE.UI Phase 0 v0.3 — Independent Adversarial Review

## Review target

Repository:

`gotoo77/gotoo-pixel-engine`

Reviewed Phase 0 commit:

`f008bcf2642067906826200d1f603c595c8ebf60`

Reviewed master mission:

`docs/GPE_UI_PHASE_0_STRATEGIC_CAPABILITY_MASTER_MISSION_v0.3.md`

Reviewed deliverables:

```text
docs/ui/phase0-v03/README.md
docs/ui/phase0-v03/BASELINE_AND_STRATEGIC_REQUIREMENTS.md
docs/ui/phase0-v03/PRIOR_ART.md
docs/ui/phase0-v03/KERNEL_LAYOUT_INPUT_STYLING.md
docs/ui/phase0-v03/MODULARITY_ARCHITECTURE_CANDIDATES.md
docs/ui/phase0-v03/TESTING_MIGRATION_ADVERSARIAL.md
docs/ui/phase0-v03/SYNTHESIS_AND_MFE_001.md
```

Runtime/code baseline used by the research:

`6ff4f8baddae269baa6a7d182f0ba0c9d985f886`

Access date for external prior-art spot checks:

`2026-09-01`

---

# 1. Independent verdict

```text
PHASE 0 v0.3 STRATEGIC DIRECTION:
PASS WITH MAJOR CONDITIONS

ARCHITECTURE B — BALANCED HYBRID:
REMAINS LEADING HYPOTHESIS
NOT YET FROZEN

MFE-001 AS CURRENTLY WRITTEN:
NOT READY TO IMPLEMENT

RUNTIME IMPLEMENTATION:
STOP
```

The review does **not** reject GPE.UI, the strategic-capability framing, or the Balanced Hybrid direction.

It finds that the Phase 0 corrected the previous strategic-starvation failure successfully: GPE.UI is now evaluated as a deliberate reusable capability rather than only as a response to present consumer pain.

However, Architecture B currently has two architectural holes at exactly the seam where a multi-pass UI becomes difficult in Rust:

1. the frame transaction contract is internally contradictory;
2. the output/data-flow model does not define how value-producing widgets such as `Toggle`, `Select`, and `Slider` commit typed consumer-owned values after interaction is resolved.

Those are not documentation cosmetics. They determine the fundamental Rust API shape.

The current MFE also tries to test too many independent architectural hypotheses simultaneously, making a failure difficult to attribute.

The recommended response is **not another broad Phase 0 rewrite**.

It is a small pre-MFE hardening slice that resolves the transaction/data-flow seam, adds one missing Rust-specific prior-art comparison, and stages MFE-001 behind explicit gates.

---

# 2. What survives adversarial review

## 2.1 Strategic-requirement framing — PASS

The taxonomy:

```text
OBSERVED REQUIREMENT
STRATEGIC REQUIREMENT
SPECULATIVE FEATURE
```

is the correct correction to the earlier methodology.

It successfully prevents the invalid inference:

```text
current consumer can implement locally
→ reusable capability has no strategic value
```

No reversal is recommended.

## 2.2 Existing-system posture — PASS

The research correctly preserves the facts that:

```text
current src/ui is not structurally broken
current consumers do not force a rewrite
current Ui remains a valuable ergonomic baseline
migration must be incremental
```

This is a strong foundation because the new architecture can fail without damaging working consumers.

## 2.3 Hybrid direction — PASS AS HYPOTHESIS

Separating:

```text
one-frame UI description
persistent interaction state
one-frame layout/paint output
consumer-owned gameplay state
```

is a strong conceptual move.

It is supported by contemporary prior art and avoids the false binary of strictly immediate versus permanently retained UI.

The transient-graph idea remains credible.

## 2.4 Stable identity — PASS AS KERNEL CANDIDATE

The justification is stronger than the old consumer-only argument.

Stable identity serves several strategic pressures simultaneously:

```text
dynamic structure
focus restoration
state retention
future declarative authoring
diagnostics
possible future accessibility projection
```

The research also correctly avoids requiring explicit IDs everywhere.

## 2.5 Constraint/layout direction — PASS WITH ONE CONDITION

A parent-constraint / child-measure / parent-placement model is a sound basis.

The review agrees that GPE should not copy CSS wholesale and agrees that Taffy should not become a mandatory dependency before measurement.

Condition:

> do not freeze the **public/internal scalar domain** to integer-only constraints before the MFE proves that this does not conflict with future richer text/vector measurement.

Final authoritative GPE framebuffer `Rect`s should remain deterministic integer geometry.

The internal measurement scalar can remain an implementation decision until tested.

## 2.6 Logical modularity inside `src/ui` — PASS PROVISIONALLY

Keeping the first implementation inside the existing `gpe::ui` dependency boundary is reasonable.

The research correctly identifies the cycle problem created by a naïve:

```text
gpe-ui -> gotoo-pixel-engine
```

combined with a desired engine re-export.

The review only changes the confidence label:

```text
"no separate crate now"
= DEPENDENCY-TOPOLOGY-BACKED + STRATEGY-BACKED
```

not a permanent architecture decision.

## 2.7 Markup/SVG restraint — PASS

Markup remains strategically legitimate without being allowed to dictate the kernel before the Rust model is proven.

Static/build-time SVG rasterization remains the preferred initial vector direction.

No browser stack is justified.

---

# 3. BLOCKING FINDING A — Frame transaction is contradictory

Severity:

```text
BLOCKING BEFORE MFE IMPLEMENTATION
```

Source:

`docs/ui/phase0-v03/KERNEL_LAYOUT_INPUT_STYLING.md`

The recommended semantic order states:

```text
build
→ measure
→ layout
→ interaction
→ emit events
→ consumer applies gameplay mutations
→ paint
```

But the same document's Policy A states:

```text
build
→ layout
→ interact
→ paint
→ return output
→ consumer mutates after run
```

And the candidate API in `SYNTHESIS_AND_MFE_001.md` is shaped as:

```rust
let output = ui::run(..., |ui| {
    ...
});

if output.action_pressed(RESUME) {
    resume();
}
```

That API cannot naturally allow:

```text
consumer mutation before paint
```

unless one of these is introduced:

```text
A. explicit prepare/interact/paint phases exposed to caller
B. callback execution inside the UI transaction
C. hidden replay of consumer description
D. paint old value now, product mutation visible next frame
E. another transaction shape
```

The research explicitly rejects hidden closure replay, correctly.

Therefore the transaction contract must be made explicit before code is written.

## Required resolution

Write a short decision note comparing at least:

### T1 — Output after paint

```text
build -> layout -> interact -> paint -> return output -> mutate gameplay
```

Properties:

- simplest ownership model;
- no callback storage;
- no hidden replay;
- consumer value changes become visible next frame unless UI-local interaction state provides immediate visual feedback.

### T2 — Explicit prepared session

Conceptually:

```text
prepare/build/layout/interact
→ expose output
→ consumer applies mutations
→ paint prepared frame
```

Properties:

- permits same-frame product-state paint only if prepared paint can observe updated state;
- may introduce awkward borrow lifetimes;
- may force paint to capture consumer references/closures.

### T3 — Deferred typed mutation callbacks

Conceptually:

```text
build graph with deferred setters
→ layout/interact
→ execute selected setters
→ paint
```

Properties:

- supports same-frame mutation;
- may be difficult with multiple `&mut` captures, trait objects, lifetimes and allocation;
- can become implicit framework magic.

No option is selected by this review.

The MFE must test the smallest credible alternatives rather than silently choosing one during implementation.

---

# 4. BLOCKING FINDING B — Typed value flow is missing

Severity:

```text
BLOCKING BEFORE MFE IMPLEMENTATION
```

This is the most important finding.

## Existing evidence

At code baseline:

```text
repo: gotoo77/gotoo-pixel-engine
ref: 6ff4f8baddae269baa6a7d182f0ba0c9d985f886
file: src/ui/toolkit.rs
symbols:
- Ui::toggle
- Ui::select
- Ui::slider_f32
```

The current toolkit mutates consumer-owned values directly during the widget call:

```text
Toggle     -> &mut bool
Select     -> &mut usize
Slider     -> &mut f32
```

That gives the current API excellent directness.

The new architecture proposes:

```text
UiEvent
+
optional ActionId
+
consumer applies gameplay mutations later
```

but never defines how value-producing controls transport a new **typed value**.

`ActionId` cannot solve this alone.

Current source:

```text
repo: gotoo77/gotoo-pixel-engine
ref: 6ff4f8baddae269baa6a7d182f0ba0c9d985f886
file: src/control.rs
symbol: ActionId
```

Current definition is conceptually:

```rust
ActionId(&'static str)
```

It is an action identity, not a typed value payload.

## Why this matters

The proposed MFE explicitly requires a compiled Settings-like call site:

```text
Column + Toggle + Slider
```

but the architecture does not currently say whether those controls use:

```text
&mut T direct mutation
callbacks
UiEvent::ValueChanged payloads
output queries keyed by UiId
bindings
UI-owned draft state
commands
```

These choices have radically different Rust ownership, type-system, testability and markup consequences.

## Required resolution

Before implementing the Grid, define and compare the minimum viable value-flow candidates.

At least these should be falsified conceptually and then in compiled Rust:

### V1 — Typed output query

Example concept only:

```rust
let output = ui::run(...);

if let Some(value) = output.changed_f32(volume_id) {
    volume = value;
}
```

Advantages:

- explicit;
- no mutation closures stored in graph;
- consumer remains source of truth.

Risk:

- per-type output API can become awkward or event-enum-heavy.

### V2 — Widget-specific typed response token

A widget returns/stores a token that can be resolved from final output.

Advantages:

- potentially more typed/ergonomic.

Risk:

- lifetime/API complexity;
- may resemble a bespoke future/promise system.

### V3 — Deferred setter/callback

Advantages:

- direct mutation semantics.

Risks:

- borrow checker;
- closure storage;
- allocation;
- hidden control flow.

### V4 — UI-owned draft/binding state

Presumption:

```text
DISFAVORED
```

unless proven necessary, because it risks violating the one-source-of-truth goal.

The MFE may discover another better model.

What is not acceptable is leaving this undecided and implementing `Toggle`/`Slider` ad hoc.

---

# 5. MAJOR FINDING C — MFE-001 is too broad for clean falsification

Severity:

```text
MAJOR
```

Current MFE-001 attempts to introduce/test together:

```text
experimental UI core
Constraints
Column
responsive Grid
Text
Image
Panel
Button/Card
UiId
UiStateStore
semantic navigation
mouse
touch
focus
UiEvent/ActionId
headless dump
dynamic structure
three compiled call sites
performance measurements
native/WASM pressure
```

It also carries seven hypotheses H1–H7.

This has high information coverage but weak causal attribution.

If the experiment fails, it may be unclear whether the failure came from:

```text
transaction timing
value flow
borrow ergonomics
ID semantics
layout algorithm
spatial focus
pointer capture
custom widget protocol
allocation cost
```

## Required reduction

Keep the overall MFE objective, but stage it.

### MFE-001A — Transaction / typed values / identity / Rust ergonomics

Implement only enough for:

```text
Panel
Text
Button
Toggle
Slider
Column
stable UiId/key
UiStateStore
minimal semantic Confirm/Cancel or simple focus
headless output
```

No Grid.
No image card.
No mouse/touch requirement beyond what is strictly needed to evaluate the transaction model.

Primary questions:

```text
Can Architecture B express value controls cleanly?
Can consumer-owned state remain authoritative?
Can Rust borrows remain understandable?
Can identity/state persist without ceremony?
Does a tiny UI remain tiny?
```

Hard STOP if this gate fails.

### MFE-001B — Responsive spatial composition

Only after 001A passes:

```text
responsive Grid
custom Card
keyed dynamic list
spatial keyboard/gamepad navigation
mouse
touch
layout goldens
```

### MFE-001C — Cost/runtime characterization

Only after semantics work:

```text
allocation/time measurements
native build/runtime
WASM build/package
actual Web runtime gate when environment permits
```

This staged structure preserves the strategic ambition while improving falsifiability.

---

# 6. MAJOR FINDING D — Rust ownership model of the transient graph is underspecified

Severity:

```text
MAJOR
```

The graph may need to carry:

```text
text
image references
style data
custom paint behavior
measurement behavior
semantic metadata
possible typed value operations
```

The current research does not say whether frame nodes:

```text
own data
borrow data
store trait objects
store closures
store function pointers
store compact enums
```

This directly affects:

```text
allocation
lifetimes
borrow friction
custom-widget ergonomics
WASM size
```

It is correct not to freeze the representation in Phase 0.

But MFE-001A must explicitly test at least:

```text
borrowed &str text
borrowed Image / asset reference
consumer-owned bool/f32 state
custom paint using borrowed consumer data
multiple controls borrowing distinct consumer fields
```

A design that works only by cloning all content into the graph each frame fails the intended small-game discipline unless measurement justifies it.

---

# 7. MAJOR FINDING E — Missing Rust-specific prior art at the exact disputed seam

Severity:

```text
MAJOR RESEARCH GAP, SMALL FIX
```

The existing prior art is broad and generally useful, but the central open risk is specifically:

> idiomatic Rust ownership + lightweight view description + retained state/tree + multi-pass layout/events.

The most directly relevant contemporary projects were not included:

## Xilem 0.4.0

Sources checked 2026-09-01:

- `https://github.com/linebender/xilem/blob/main/xilem/ARCHITECTURE.md`
- `https://docs.rs/xilem/latest/xilem/`

Relevant fact:

Xilem regenerates lightweight Rust view trees and reconciles them to a retained element/widget tree.

It therefore provides useful evidence about:

```text
Rust component ergonomics
state ownership
dynamic sequences
identity/reconciliation
callbacks receiving application state
```

It is **not** a recommendation to copy reconciliation.

## Masonry 0.4.0

Sources checked 2026-09-01:

- `https://github.com/linebender/xilem/blob/main/masonry/ARCHITECTURE.md`
- `https://docs.rs/masonry/latest/masonry/`

Relevant fact:

Masonry is explicitly a lower-level retained widget foundation with multiple passes for event/update/layout/paint/accessibility and testing/inspection support.

It is highly relevant to the proposed separation:

```text
high-level Rust authoring model
vs
lower-level multi-pass UI engine
```

## Optional comparison — Floem

Floem is useful as a counterexample: a view tree built once plus fine-grained reactive state and Taffy layout.

It need not become a mandatory new study axis.

## Required action

Add a **targeted prior-art addendum**, not another broad survey.

Question to answer:

> What do Xilem/Masonry reveal about transaction boundaries, state mutation and Rust ownership that changes or challenges Architecture B?

---

# 8. MAJOR FINDING F — Event model risks becoming a second framework

Severity:

```text
MAJOR
```

Current candidate event list includes:

```text
Focused
Activated
ValueChanged
PointerEntered
PointerLeft
DragStarted
Dragged
DragEnded
Cancelled
```

while widgets can also carry `ActionId`.

This creates two risks:

1. generic UI events and product semantic actions duplicate each other;
2. the kernel starts exposing every low-level interaction transition as a public event taxonomy.

## Required rule

MFE-001A should start with the smallest output model that supports the probes.

Likely separation:

```text
internal interaction state
- hover
- focus
- capture
- pressed

external semantic output
- activation
- typed value change
- optional ActionId
```

Do not expose pointer-enter/leave/drag event types until a custom-widget or consumer probe requires them.

Debug traces may observe internal transitions without making them public API.

---

# 9. MAJOR FINDING G — State-store cleanup/lifetime policy needs an explicit invariant

Severity:

```text
MAJOR BUT LOCAL
```

The test strategy already mentions:

```text
state cleanup for vanished nodes
```

but the architecture does not define a lifecycle rule.

A keyed state store can otherwise retain stale entries indefinitely.

MFE invariant should be explicit:

```text
nodes seen in frame N mark their state entries live
entries not seen according to the selected retention policy become reclaimable
focus/capture referencing vanished nodes is repaired deterministically
```

The exact retention policy may be:

```text
immediate sweep
short grace generation
explicit persistence scope
```

but it must not be accidental.

Also test duplicate keys and key reuse under a different widget kind.

---

# 10. MAJOR FINDING H — Integer-first measurement is slightly over-frozen

Severity:

```text
MAJOR DESIGN CONDITION, NOT A BLOCKER TO EXPERIMENT
```

The research says both:

```text
pixel-aware, not pixel-imprisoned
```

and:

```text
Constraints use integer logical-pixel units
```

Final GPE framebuffer rectangles absolutely should remain deterministic integers.

However future pressures include:

```text
outline fonts
text shaping
vector assets
non-pixel-art consumers
```

Those may naturally produce fractional intrinsic metrics.

Recommended correction:

```text
MFE may use integer constraints for simplicity.
Public long-term scalar type is NOT frozen by Phase 0.
Final resolved framebuffer Rect contract remains integer/deterministic.
```

This preserves pixel control without prematurely coupling every future measurement source to integers.

---

# 11. Documentation-completeness finding

Severity:

```text
MAJOR FOR "PHASE COMPLETE" CLAIM
MINOR FOR MFE DIRECTION
```

The master mission explicitly asked to study, among other topics:

```text
data flow alternatives
world-space UI pressure
authoring / hot reload
diagnostics / versioning / security
```

The delivered synthesis classifies several of these as `LATER`, but the supporting analysis is much thinner than the master prompt requested.

The most consequential missing analysis is **data flow**, because it directly caused Blocking Finding B.

Required action:

A small hardening addendum should explicitly close:

```text
DATA FLOW
- direct mutation
- responses
- events/actions
- typed value changes
- bindings/callbacks

WORLD-SPACE
- only compatibility pressure; no v1 machinery unless a kernel dead-end is demonstrated

AUTHORING/HOT RELOAD
- frontend/tooling layer later; no kernel runtime ownership

DIAGNOSTICS/VERSIONING/SECURITY
- experimental API status
- strict future markup parsing
- no arbitrary script execution
- source diagnostics if markup is later accepted
```

Do not reopen every Phase 0 topic.

---

# 12. Prior-art spot-check result

The review spot-checked current public sources.

Confirmed at access date:

```text
egui 0.36.1
- stable Id type
- retained Memory/focus state

Iced 0.14.0
- message/update/view model
- responsive layout
- custom widgets
- headless/e2e testing additions documented in 0.14 changelog

Slint 1.17.1
- declarative .slint language
- build-time generated Rust components
- runtime interpreter also exists

Bevy UI 0.19.1
- Flexbox/Grid layout
- directional navigation modules
- game-oriented UI context

Taffy 0.14.0
- Block/Flexbox/Grid
- many algorithms/features enabled by default

AccessKit 0.24.1
- stable NodeId in semantic accessibility tree

Flutter docs
- current docs identify Flutter 3.44.7
```

No material contradiction was found in the existing prior-art claims that would invalidate the Phase 0 direction.

One practical Taffy note:

> a future Taffy spike must test a deliberately minimal feature set, not only default features, because Taffy 0.14.0 enables a broad set of layout/tree capabilities by default.

---

# 13. Revised architecture status

## Architecture 0

```text
NOT SELECTED
VALID CONTROL / MIGRATION BASELINE
```

No change.

## Architecture A

```text
VALID FALLBACK
```

Its importance increases slightly because Architecture B may fail on transaction/value ergonomics.

## Architecture B

```text
LEADING HYPOTHESIS
PASS WITH MAJOR CONDITIONS
```

Do not freeze public API or internal node representation yet.

## Architecture C

```text
REJECT FOR V1
```

No change.

---

# 14. Required hardening before implementation

Create one bounded Phase 0 v0.3 hardening slice containing exactly:

## H1 — Transaction and data-flow decision note

Resolve/falsify:

```text
frame transaction
same-frame paint policy
button action output
toggle/select/slider typed value output
consumer-owned state
borrow/lifetime implications
```

## H2 — Targeted Rust prior-art addendum

Only:

```text
Xilem
Masonry
optional Floem contrast if useful
```

## H3 — MFE-001 staged protocol revision

```text
001A transaction/value/identity/Rust ergonomics
STOP gate
001B responsive Grid/custom Card/multimodal
STOP gate
001C cost/runtime characterization
```

## H4 — Explicit state-store lifecycle invariant

Include stale state, vanished focus/capture and duplicate key behavior.

## H5 — Coverage addendum

Close only the thin master-prompt topics listed in Finding 11.

No runtime code.
No Cargo changes.
No new crate.
No implementation.

---

# 15. Exit criteria for the hardening slice

Architecture B may proceed to MFE-001A only when:

```text
1. one explicit transaction contract is selected for the experiment;
2. typed value flow is implementable in plausible Rust without hidden replay;
3. graph node ownership/borrowing candidates are explicit enough to compile-test;
4. Xilem/Masonry comparison has been incorporated;
5. MFE is staged so failures have clear attribution;
6. state-store cleanup/focus repair invariant is documented;
7. no additional runtime/code work has occurred.
```

This does **not** require final public API stability.

The purpose is to make the MFE a falsifiable architecture experiment instead of an architecture implementation disguised as an experiment.

---

# 16. Final verdict

```text
GPE.UI STRATEGIC R&D:
PASS

STRATEGIC REQUIREMENTS MODEL:
PASS

EXISTING-UI CAPITALIZATION:
PASS

PRIOR ART:
PASS WITH TARGETED RUST-SPECIFIC ADDENDUM

ARCHITECTURE B:
PASS WITH MAJOR CONDITIONS

MODULARITY DIRECTION:
PASS PROVISIONALLY

MIGRATION STRATEGY:
PASS

TESTABILITY DIRECTION:
PASS

MFE-001 CURRENT FORM:
REVISE BEFORE IMPLEMENTATION

IMPLEMENTATION AUTHORIZED:
NO
```

## Review status

```text
INDEPENDENT ADVERSARIAL REVIEW:
PASS WITH MAJOR CONDITIONS

NEXT:
PHASE 0 v0.3 HARDENING ADDENDUM

THEN:
MFE-001A ONLY IF HARDENING PASSES
```

# STOP

Do not implement GPE.UI or MFE-001 from the current Phase 0 documents before the bounded hardening addendum is complete and reviewed.
