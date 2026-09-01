# GPE.UI Phase 0 v0.3 — Hardening Addendum

## Status

This addendum responds to the independent adversarial review at:

`c82f2d23732f2b0e77d2fa0eecd44eb07348c703`

Reviewed Phase 0 baseline:

`f008bcf2642067906826200d1f603c595c8ebf60`

Master mission:

`docs/GPE_UI_PHASE_0_STRATEGIC_CAPABILITY_MASTER_MISSION_v0.3.md`

Independent review:

`docs/ui/phase0-v03/INDEPENDENT_ADVERSARIAL_REVIEW.md`

This is a **targeted hardening slice**.

It does not reopen the full Phase 0 and it does not implement runtime code.

The goals are only:

```text
H1  close frame transaction + typed value flow
H2  add Rust-specific Xilem / Masonry prior art
H3  stage MFE-001 into A / B / C with hard gates
H4  define UiStateStore / stale-ID lifecycle
H5  close thin coverage debt:
    data flow
    world-space pressure
    authoring/hot reload
    diagnostics/versioning/security
```

# ABSOLUTE STOP BEFORE IMPLEMENTATION

---

# 1. Hardening verdict

```text
ARCHITECTURE B — BALANCED HYBRID:
REMAINS LEADING HYPOTHESIS

FRAME TRANSACTION:
HARDENED TO ONE COHERENT CANDIDATE

TYPED VALUE FLOW:
HARDENED TO A TESTABLE RUST API CANDIDATE

UiStateStore LIFECYCLE:
HARDENED FOR MFE-001A

MFE-001:
REPLACED BY STAGED MFE-001A / 001B / 001C

MFE-001A IMPLEMENTATION:
NOT AUTHORIZED BY THIS DOCUMENT

NEXT:
INDEPENDENT HARDENING REVIEW / EXPLICIT MFE-001A AUTHORIZATION
```

The addendum does not claim that the proposed transaction/API is correct.

It claims only that it is now **specific enough to falsify in compiled Rust**.

---

# 2. H1 — Authoritative candidate frame transaction

The Phase 0 documents previously described two incompatible orders:

```text
interaction
→ consumer mutation
→ paint
```

and:

```text
interaction
→ paint
→ return output
→ consumer mutation
```

For MFE-001A, use exactly one candidate contract.

## 2.1 Candidate transaction T1

```text
1. input snapshot
2. consumer describes UI from current consumer-owned state
3. build transient UiGraph
4. resolve identity/style
5. measure
6. arrange integer final Rects
7. resolve focus / pointer / semantic interaction
8. apply interaction results to FRAME-LOCAL EFFECTIVE WIDGET VALUES
9. emit activation + typed proposed value changes
10. paint from resolved interaction state + frame-local effective values
11. finalize UiStateStore
12. return UiOutput + typed handles/results
13. consumer commits gameplay/product state changes
```

This is the **only** transaction model MFE-001A is allowed to prototype unless it first reports a blocker.

## 2.2 Why frame-local effective values exist

Example:

```text
consumer volume before UI = 0.65
pointer changes slider proposal to 0.70
```

The slider's frame-local resolved value becomes:

```text
0.70
```

before paint.

Therefore the thumb/value display may paint `0.70` immediately even though the consumer's authoritative state is still `0.65` until `ui::run(...)` returns.

After return:

```text
consumer commits 0.70
```

Next frame starts from authoritative `0.70`.

This avoids a visible one-frame lag for the control itself without replaying consumer UI code.

## 2.3 What does NOT update in the same UI transaction

If another unrelated UI node was built from the same consumer field before interaction resolution, it is **not automatically rebuilt** after the value change.

Example:

```text
Slider(volume = 0.65)
Text(format!("GLOBAL VOLUME = {}", volume))
```

If the slider proposes `0.70`, the slider itself can paint `0.70` from its effective value, but the independently built `Text` still reflects the pre-transaction snapshot.

The consumer state becomes `0.70` after `run`, so the dependent text updates on the next UI transaction.

This is an explicit candidate tradeoff, not an accidental latency.

MFE-001A must determine whether this is acceptable in real settings-like UI.

## 2.4 Forbidden hidden behavior

The implementation MUST NOT:

```text
re-run the consumer UI closure silently
execute arbitrary consumer code twice
mutate consumer gameplay state during measure/layout
make paint call consumer mutation callbacks
```

A second internal layout/paint traversal over the already-built graph is allowed.

A second execution of user description code is not.

---

# 3. H1 — Separate activation from value transport

The existing GPE `ActionId` is:

```rust
pub struct ActionId(&'static str);
```

Provenance:

```text
repo: gotoo77/gotoo-pixel-engine
ref: 6ff4f8baddae269baa6a7d182f0ba0c9d985f886
file: src/control.rs
symbol: ActionId
```

`ActionId` is useful for semantic discrete intent:

```text
menu.resume
menu.quit
arcade.launch.snake
settings.reset
```

It is NOT a typed transport for:

```text
bool
usize
f32
arbitrary future widget values
```

Therefore the candidate API must keep these concepts separate.

## 3.1 Discrete semantic activation

Candidate:

```text
Ui activation
→ optional ActionId
```

Example:

```text
Button RESUME
→ Activated(target=resume_button, action=ui.pause.resume)
```

## 3.2 Continuous/choice value proposal

Candidate:

```text
value-producing widget
→ typed WidgetRef<T>
→ UiOutput::changed(ref) -> Option<T>
```

`ActionId` may coexist for semantic meaning but does not carry the value.

---

# 4. H1 — Typed WidgetRef<T> candidate

This section is **CANDIDATE API — NOT VALIDATED BY COMPILATION**.

Concept:

```rust
struct WidgetRef<T> {
    id: UiId,
    _type: PhantomData<fn() -> T>,
}
```

Properties:

```text
Copy / cheap
contains stable UiId
carries compile-time expected value type
contains no borrow into UiGraph
valid only as a query key for the matching UiOutput transaction
```

Example:

```rust
// CANDIDATE — NOT VALIDATED BY COMPILATION
let (output, controls) = ui::run(frame, &mut ui_state, |ui| {
    let enabled = ui.toggle("audio.enabled", "ENABLED", settings.enabled);
    let volume = ui.slider_f32(
        "audio.volume",
        "VOLUME",
        settings.volume,
        0.0..=1.0,
        0.05,
    );

    Controls { enabled, volume }
});

if let Some(value) = output.changed(controls.enabled) {
    settings.enabled = value;
}
if let Some(value) = output.changed(controls.volume) {
    settings.volume = value;
}
```

Alternative ergonomic forms may be tested, but they MUST preserve the same semantic separation:

```text
input value copied into UI description
interaction resolved after layout
new typed value returned explicitly
consumer owns final mutation
```

---

# 5. H1 — Internal value representation

MFE-001A should NOT attempt arbitrary type erasure or a universal data-binding runtime.

For the experiment, a bounded internal representation is acceptable:

```text
Bool(bool)
F32(f32)
Index(usize)   [only if Select is included]
```

A typed `WidgetRef<T>` query can validate/decode the internal value kind.

This is an implementation experiment, not a promise that the final public API exposes a `UiValue` enum.

## 5.1 Custom arbitrary values

Arbitrary custom typed payloads are **not required in MFE-001A**.

A custom component in later MFE-001B may initially emit:

```text
activation + ActionId
```

or compose primitive typed controls.

If a real custom-component probe demonstrates a need for arbitrary typed payloads, revisit deliberately.

Do NOT introduce `Any`, dynamic downcasting, serialization, or a generic reactive store preemptively.

---

# 6. H1 — Why direct `&mut T` is not the default new-core contract

Current egui-like/current-GPE ergonomics are excellent:

```rust
ui.slider_f32("VOLUME", &mut volume, ...);
```

But a multi-pass graph has not resolved the final widget geometry/interactions when description is first created.

Holding arbitrary mutable consumer references inside a graph until interaction resolution creates difficult lifetime/aliasing pressure and makes future declarative frontends harder to share.

Therefore MFE-001A should test:

```text
value-in
proposal-out
```

before attempting graph-retained mutable bindings.

This is a hypothesis, not dogma.

If the typed-output API is materially worse in real Rust, Architecture A/direct mutation remains a valid fallback.

---

# 7. H1 — Borrow/ownership contract for the transient graph

The previous Phase 0 said "transient graph" without defining what it may borrow.

MFE-001A candidate ownership rules:

## 7.1 Consumer gameplay state

```text
UiGraph MUST NOT retain &mut references to consumer gameplay/product state.
```

## 7.2 Immutable presentation data

The graph MAY borrow immutable presentation data for the duration of `ui::run`:

```text
&str
&Image
read-only custom presentation data
```

because measure/layout/interaction/paint all complete before `run` returns.

The graph MUST NOT retain these references beyond the transaction.

## 7.3 Small scalar widget values

Core control values should be copied into frame-local node data when practical:

```text
bool
f32
usize
```

## 7.4 Custom paint

A custom paint hook may borrow immutable data for the transaction.

It must not be the mechanism for mutating gameplay state.

The preferred callable contract is side-effect-light with respect to consumer state.

Exact Rust trait/closure shape remains an MFE-001B question.

## 7.5 Output handles

`WidgetRef<T>` must not borrow UiGraph storage.

It is an ID/type token only.

---

# 8. H1 — Public vs internal event surface

The previous candidate event list risked prematurely exposing every interaction transition.

For MFE-001A, minimize the public semantic output.

## Public output candidate

```text
Activated
TypedValueChanged
optional semantic ActionId
```

## Internal/debug interaction state

May include:

```text
focused
hovered
pressed
captured
repeat state
pointer entered/left
```

These states may appear in debug traces without becoming stable public event variants.

## Deferred public events

```text
DragStarted
Dragged
DragEnded
PointerEntered
PointerLeft
generic bubbling/capture events
```

Only promote with a concrete component requirement.

---

# 9. H1 — Data-flow examples to compile in MFE-001A

The MFE must compile realistic forms for at least these cases.

## 9.1 Button

```rust
// CANDIDATE
let (output, resume) = ui::run(..., |ui| {
    ui.button("resume", "RESUME").action(RESUME)
});

if output.activated(resume) || output.action_pressed(RESUME) {
    resume_game();
}
```

## 9.2 Toggle

```rust
// CANDIDATE
let (output, enabled) = ui::run(..., |ui| {
    ui.toggle("enabled", "ENABLED", settings.enabled)
});

if let Some(value) = output.changed(enabled) {
    settings.enabled = value;
}
```

## 9.3 Slider

```rust
// CANDIDATE
let (output, volume) = ui::run(..., |ui| {
    ui.slider_f32("volume", "VOLUME", settings.volume, 0.0..=1.0, 0.05)
});

if let Some(value) = output.changed(volume) {
    settings.volume = value;
    audio.set_volume(value);
}
```

## 9.4 Multiple fields on one owner

This must compile without artificial destructuring gymnastics:

```rust
settings.enabled
settings.volume
settings.mode
```

If the API forces awkward split borrows or broad `self` workarounds for ordinary settings code, that is a material MFE finding.

---

# 10. H2 — Targeted Rust prior art: egui value mutation

Version checked:

`egui 0.36.1`

Access date:

`2026-09-01`

Sources:

- https://docs.rs/crate/egui/latest/source/src/lib.rs
- https://docs.rs/egui/latest/egui/widgets/struct.Slider.html
- https://docs.rs/egui/latest/egui/response/struct.Response.html

## FACT

egui's slider accepts `&mut` user-owned value and mutates it during the immediate widget call. Widgets return a `Response` describing interaction.

## Lesson

This is the ergonomic benchmark GPE.UI must respect.

The hardening proposal deliberately gives up direct same-call mutation in exchange for a multi-pass graph.

That trade must earn its cost through responsive layout, stable identity, custom composition and testability.

If it does not, Architecture A/current-immediate evolution is stronger.

---

# 11. H2 — Targeted Rust prior art: Iced typed messages

Version checked:

`iced 0.14.0`

Access date:

`2026-09-01`

Sources:

- https://docs.rs/iced/latest/iced/widget/fn.vertical_slider.html
- https://docs.rs/iced/latest/iced/widget/toggler/
- https://docs.rs/crate/iced_widget/latest/source/src/helpers.rs

## FACT

Iced value widgets accept the current value and a typed mapping from new value to application `Message`.

Example shape:

```text
Slider(value, on_change: T -> Message)
Toggler(value, on_toggle: bool -> Message)
```

The application then mutates its authoritative state in `update`.

## Lesson

Typed value proposals are a proven Rust UI pattern.

GPE should learn the **typed value-out** idea without importing the entire Elm runtime or forcing every game to define one giant application message enum.

The `WidgetRef<T> + UiOutput` candidate is intentionally a smaller experiment in that direction.

---

# 12. H2 — Targeted Rust prior art: Xilem

Published crate checked:

`xilem 0.4.0`

Published:

`2025-10-29`

Repository architecture also checked on:

`2026-09-01`

Sources:

- https://docs.rs/crate/xilem/latest
- https://docs.rs/crate/xilem/latest/source/ARCHITECTURE.md
- https://github.com/linebender/xilem/blob/main/xilem/ARCHITECTURE.md

## FACT

Xilem creates lightweight short-lived view trees from Rust app logic, compares them to the previous view tree, and updates a retained backend element tree.

Its native backend is Masonry.

Xilem explicitly targets idiomatic Rust and states that one goal is to avoid the borrowing/state-management problems traditional retained GUIs can create.

Its normal application model gives UI callbacks mutable access to application state and regenerates/reconciles the view after changes.

## Why this matters to GPE

Xilem demonstrates a credible Rust solution to:

```text
short-lived declarative view
+
retained backend state
+
application state mutation
```

But it obtains that solution by owning a **reactive application cycle** and maintaining/reconciling a retained element tree.

GPE currently does not want GPE.UI to own the game's state machine or event loop.

Therefore Xilem is not evidence that GPE should add reconciliation.

It is evidence that Rust ownership around multi-pass UI is a real architectural problem deserving an explicit solution.

## Hardening consequence

MFE-001A must compare its compiled ergonomics against two poles:

```text
egui/current GPE:
direct &mut mutation, immediate response

Xilem/Iced family:
interaction produces state update/message in a controlled application cycle
```

GPE.UI's proposed value-in/proposal-out transaction is intentionally between these poles.

---

# 13. H2 — Targeted Rust prior art: Masonry

Published crate checked:

`masonry 0.4.0`

Access date:

`2026-09-01`

Sources:

- https://docs.rs/crate/masonry/latest/source/ARCHITECTURE.md
- https://github.com/linebender/xilem/blob/main/masonry/ARCHITECTURE.md

## FACT

Masonry is a retained widget-tree foundation with explicit passes including event handling, update, layout, paint and accessibility-related work.

Its architecture emphasizes:

```text
well-defined abstraction boundaries
working with Rust ownership rather than bypassing it
testing/debugging as core design concerns
```

Layout is an explicit pass in which containers measure/place children.

## Lesson

The key lesson is not "GPE should retain widgets".

It is:

> multi-pass UI needs an explicit transaction/pass model and explicit ownership boundaries.

That directly validates the independent review's criticism of the previous ambiguous transaction model.

## Do not copy

GPE v1 does not currently justify:

```text
permanent Widget object tree
Masonry's platform/runtime stack
full accessibility/text/render stack
mutate callback queue as a general game-state mechanism
```

---

# 14. H2 synthesis

The targeted Rust prior art yields three useful archetypes:

```text
EGUI
current user state passed by mutable reference
interaction resolved immediately
excellent local ergonomics
limited by immediate execution order for some layout classes

ICED
current state passed by value into view
interaction emits typed Message carrying new value
application commits later
clear data flow, more message plumbing

XILEM + MASONRY
lightweight view + retained backend tree
callbacks can mutate app state
framework controls reactive reconciliation/pass lifecycle
stronger machinery and lifecycle ownership
```

GPE MFE candidate:

```text
current state passed by value into transient one-frame graph
multi-pass layout/interaction inside one GPE update call
typed proposed changes returned to caller
consumer commits after ui::run
no permanent widget tree
no framework-owned game loop
```

This is now a specific hypothesis rather than a vague hybrid label.

---

# 15. H4 — UiStateStore authoritative MFE lifecycle

The persistent state store requires deterministic cleanup.

For MFE-001A use a deliberately simple rule.

## 15.1 Seen set

Every UI transaction records all live `UiId`s encountered in the graph.

## 15.2 End-of-transaction cleanup

After interaction and before returning output:

```text
focus target not seen
→ clear/fallback deterministically

pointer/touch capture target not seen
→ cancel capture

per-widget transient interaction state not seen
→ prune immediately

scroll/repeat/widget-local state not seen
→ prune immediately for MFE-001A
```

No TTL.

No hidden multi-frame zombie retention.

## 15.3 Consequence

If a widget disappears for one complete transaction and later reappears with the same ID, its UI-local state starts fresh.

This is intentional for the MFE.

If later consumers require hidden-page state preservation, introduce an explicit concept such as:

```text
separate UiStateStore per page/surface
explicit retained scope
consumer-owned state
```

Do not add hidden stale-state retention preemptively.

---

# 16. H4 — Focus fallback when focused ID disappears

MFE-001A uses a simple column/list rule only.

Before resolving a structural change, retain:

```text
previous focus order
previous focused index
```

If focused ID disappears:

```text
new focus order empty
→ focus NONE

otherwise
→ focus item at min(previous_index, new_len - 1)
```

This is deterministic and preserves approximate position without pretending to solve general spatial navigation.

MFE-001B must revisit fallback for spatial Grid layouts.

---

# 17. H4 — Hierarchical identity hardening

Candidate identity model:

```text
parent scope
+
local declaration slot
+
optional explicit key
```

Rules for MFE-001A:

1. same unchanged declaration path produces same implicit `UiId`;
2. dynamic/reorderable collections MUST use explicit keys;
3. keyed identity follows the key across reorder;
4. duplicate explicit keys in one identity scope produce deterministic diagnostics;
5. identity hashing/representation must not depend on randomized process hash state;
6. no consumer should need explicit IDs for every static label/container unless state/diagnostics require them.

Exact binary representation remains implementation-private during MFE.

---

# 18. H3 — Replace monolithic MFE-001 with staged protocol

The previous MFE combined too many independent unknowns.

The hardening protocol is:

```text
MFE-001A — Transaction / Value Flow / Identity / Rust Ergonomics
        ↓ PASS or PASS WITH CONDITIONS only
MFE-001B — Responsive / Spatial / Custom Component
        ↓ PASS or PASS WITH CONDITIONS only
MFE-001C — Cost / Native / Web Runtime
```

A failure in an earlier stage blocks later stages.

---

# 19. MFE-001A — Transaction / Value Flow / Identity / Rust Ergonomics

## 19.1 Purpose

Answer the hardest Rust/API questions before Grid, images, touch and custom Card complexity are introduced.

## 19.2 Scope

Only enough experimental runtime code for:

```text
UiId
UiStateStore
UiGraph or equivalent transient representation
Constraints sufficient for vertical layout
Column
Panel
Text
Button
Toggle
Slider<f32>
semantic navigation:
- Up
- Down
- Left
- Right
- Confirm
- Cancel
minimal paint to existing Framebuffer/TextRenderer
UiOutput
WidgetRef<bool>
WidgetRef<f32>
ActionId activation
headless dump
```

`Select<usize>` may be included only if it is nearly free after Toggle/Slider; it is not mandatory.

## 19.3 Explicit exclusions

```text
Grid
responsive auto-fit
Image widget
custom Card
general custom widget protocol
mouse hover/click beyond what is unavoidable
touch
spatial navigation
markup
SVG
animation
inspector
accessibility backend
Taffy
separate crate
legacy migration
```

## 19.4 Required compiled call sites

### Tiny

```text
Panel
Text
Button
```

### Settings

```text
Toggle
Slider
consumer-owned bool/f32
```

### Dynamic identity

```text
conditional insertion/removal of one keyed interactive row
```

## 19.5 Required tests

### Transaction

Prove:

```text
interaction proposal occurs after layout
slider effective value paints immediately
consumer authoritative value changes only after run returns
no consumer closure replay occurs
```

### Typed value output

Prove:

```text
WidgetRef<bool> queries bool change
WidgetRef<f32> queries f32 change
wrong-type query is impossible or deterministically rejected by API design
no ActionId value smuggling
```

### Identity

Prove:

```text
static implicit identity stable
keyed item remains focused across sibling insertion/reorder
focused removed item gets deterministic fallback
unseen widget state pruned at end of transaction
duplicate key diagnostic deterministic
```

### Rust ergonomics

Compile realistic consumer structures with multiple fields.

No artificial `RefCell`, `Rc<RefCell<_>>`, global store or unsafe workaround may be introduced merely to satisfy the API.

## 19.6 Measurements

Record:

```text
nodes/frame
allocations/frame
allocated bytes/frame
persistent UiStateStore entries
build time
layout time
interaction time
paint time
```

For:

```text
Tiny
Settings
```

No pass/fail numeric threshold is predefined.

## 19.7 Human/API review questions

1. Is Tiny materially less readable than current `Ui`?
2. Is `WidgetRef<T>` intuitive or ceremony-heavy?
3. Does committing values after `run` feel natural in GPE's `Game::update`?
4. Is the frame-local effective value rule understandable?
5. Is dependent-UI-next-transaction behavior acceptable?
6. Are explicit keys rare and predictable?
7. Does a user need to understand UiGraph internals for ordinary controls?

## 19.8 MFE-001A hard fail conditions

```text
typed value output requires pervasive dynamic typing/downcasting
ordinary settings ownership causes severe borrow gymnastics
consumer closure must be replayed
stable identity requires explicit IDs everywhere
UiStateStore cleanup is ambiguous/non-deterministic
Tiny UI is clearly burdened by architecture ceremony
frame-local effective values cannot paint coherently
```

On FAIL:

```text
STOP
revisit Architecture A / direct immediate mutation
or revise transaction model
```

Do not continue to 001B.

---

# 20. MFE-001A result vocabulary

Only:

```text
PASS
PASS WITH CONDITIONS
FAIL
```

`PASS` means the architecture seam is good enough to justify adding spatial complexity.

It does not mean GPE.UI is production-ready.

---

# 21. MFE-001B — Responsive / Spatial / Custom Component

Authorized only after 001A passes.

## Scope

Add:

```text
Row / Stack as needed
bounded responsive Grid / auto-fit equivalent
Image
custom Card component
keyed dynamic list
spatial focus navigation
mouse hover/press/release
touch Started/Moved/Ended/Cancelled
pointer/touch capture
small/medium/wide headless fixtures
```

Use 6–9 fake Arcade-like entries as a **design probe**.

No production Arcade integration.

## Main hypotheses

```text
responsive integer layout is sufficient without CSS
stable IDs survive responsive structural changes
spatial focus is deterministic
custom Card integrates without private hacks
pointer/touch share resolved geometry cleanly
```

## Gate

If custom Grid implementation becomes algorithmically complex or fragile:

```text
STOP
run targeted custom-layout vs Taffy spike
```

Do not silently grow a CSS clone.

---

# 22. MFE-001C — Cost / Native / Web Runtime

Authorized only after 001B passes.

## Scope

Measure the accepted experimental implementation.

Collect:

```text
allocations/frame
bytes/frame
node count
build/layout/interaction/paint timings
persistent state size
native binary delta
WASM package delta where practical
```

Runtime gates:

```text
Native actual window/runtime
Web build/package
Web actual browser runtime
```

Never report Web runtime PASS from build/package only.

## Purpose

001C can still reject/reduce Architecture B if its actual cost violates GPE's small-engine goals.

---

# 23. H5 — Data flow closure

Authoritative candidate separation:

```text
consumer/game state
    owns durable product/gameplay values

UiStateStore
    owns only UI interaction state

UiGraph
    owns/borrows one-transaction presentation description

UiOutput
    owns one-transaction interaction proposals/events
```

Forbidden ownership inversion:

```text
UiStateStore owns inventory
UiStateStore owns audio volume as product authority
Button owns game scene transition
UiGraph owns persistent game model
```

---

# 24. H5 — World-space pressure

World-space UI remains:

```text
LATER / COMPATIBILITY PRESSURE
```

Hardening requirement:

The kernel should not depend on the physical desktop window.

It should operate against an explicit logical target:

```text
Size / Rect
input coordinates in that logical target
paint target / Framebuffer
```

Therefore a future world-space or sub-surface adapter could conceptually provide:

```text
logical surface
coordinate transform
paint target
```

No world-space transforms, camera integration or 3D projection enter MFE-001A/B.

---

# 25. H5 — Authoring / markup / hot reload closure

Markup remains:

```text
LATER
```

Hardening constraint:

The new Rust runtime must not require Rust-only closures for fundamental layout/semantics that a future declarative frontend could never represent.

Future frontend target remains:

```text
Rust description ─────┐
                      ├─> same semantic frame model
Markup compiler ──────┘
```

Hot reload remains later.

If introduced, source-location metadata may attach to graph nodes for diagnostics without changing core interaction semantics.

No parser/schema/source-location machinery enters MFE-001A.

---

# 26. H5 — Diagnostics closure

MFE-001A debug output must at least be able to expose:

```text
UiId
widget kind
parent/scope identity
resolved Rect
focus state
activation/value change
stale-state pruning
identity diagnostics
```

Required deterministic diagnostics:

```text
duplicate explicit key in one scope
invalid/empty focus after structure change [trace, not necessarily error]
type mismatch if internal typed-output invariant is violated
```

Do not build a graphical inspector.

---

# 27. H5 — Versioning/API stability closure

During MFE stages:

```text
experimental API
no compatibility promise
no legacy deprecation
```

The implementation should be isolated under an explicitly experimental internal/public namespace or similarly clear boundary.

Exact naming is an implementation decision.

No SemVer stability claim is allowed before:

```text
001A PASS
001B PASS
001C acceptable
at least one real consumer migration
```

---

# 28. H5 — Security closure

MFE-001A/B contain no parser and no scripting surface.

Therefore no new content-execution security boundary is introduced.

Future markup rules remain:

```text
no arbitrary code execution
no JavaScript
strict validation
bounded parser/resource behavior
unknown-field/version policy explicit
```

Future SVG preference remains static/raster-first.

---

# 29. Integer geometry hardening

Revise the previous wording from:

```text
all layout must fundamentally be integer
```

to:

> **authoritative final GPE widget Rects are deterministic integer logical-pixel geometry.**

Internal measurement is not permanently forbidden from using fractional precision if future text/vector metrics justify it.

Required boundary rule:

```text
internal measurement
→ deterministic explicit rounding policy
→ final integer Rect tree
```

MFE-001A should remain integer-only because current bitmap text/framebuffer primitives do not require fractional measurement.

Do not freeze this as a universal future typography constraint.

---

# 30. Custom component hardening

The Phase 0 strategic requirement remains:

```text
custom-widget friendly
```

But custom component protocol is removed from 001A.

Reason:

It is not necessary to answer transaction/value-flow questions and would confound the first experiment.

MFE-001B must test at least one genuinely custom visual component:

```text
Arcade-like Card
```

A successful Card must be able to participate in:

```text
measure/layout
focus
pointer/touch
semantic activation
custom paint
style
```

without accessing private graph internals.

---

# 31. Feedback hardening

No UI core widget owns audio/haptics.

Candidate downstream path remains:

```text
UiOutput activation/value change
→ consumer or optional feedback policy
→ AudioBus::Ui / haptic / animation request
```

MFE-001A does not implement feedback.

It only ensures output is structured enough that feedback could be mapped later.

---

# 32. Architecture B after hardening

Architecture B now means specifically:

```text
consumer-owned gameplay state
+
one-frame Rust UI description
+
one-frame transient graph
+
persistent keyed UI interaction state
+
explicit multi-pass measure/layout/interaction/paint
+
frame-local effective control values
+
typed value proposals returned to caller
+
discrete semantic ActionId activation
+
no permanent widget object tree
+
no hidden reconciliation
+
no UI-owned game loop
```

This definition is substantially narrower and more falsifiable than the pre-review "Balanced Hybrid" label.

---

# 33. Architecture A remains a real competitor

Architecture A is not a defeated historical option.

If 001A demonstrates that:

```text
WidgetRef<T> plumbing is awkward
post-run commits are unnatural
multi-pass graph cost/complexity is disproportionate
```

then the project MUST revisit a more immediate design:

```text
direct responses
explicit IDs only where needed
simpler layout scopes
possibly previous-frame/limited two-pass techniques
```

No sunk-cost protection for Architecture B.

---

# 34. Taffy decision remains deferred but sharpened

No Taffy in 001A.

No Taffy by default in 001B.

Trigger a targeted Taffy spike if 001B finds one of:

```text
layout algorithm complexity grows rapidly
wrap/flex/grid correctness becomes hard to test
custom algorithm duplicates substantial mature behavior
responsive probes require accumulating special cases
```

The spike must measure actual:

```text
dependency impact
compile impact
binary/WASM impact
rounding integration
API fit
```

No speculative rejection or adoption.

---

# 35. Hardening gates summary

## Gate H1 — Transaction/data flow

PASS if the document contains one coherent candidate and MFE-001A can directly falsify it.

Status:

```text
PASS DOCUMENTARILY
RUNTIME UNKNOWN
```

## Gate H2 — Rust-specific prior art

Status:

```text
PASS
```

Xilem/Masonry plus egui/Iced create an adequate comparison set for the ownership/data-flow seam.

## Gate H3 — Experimental reduction

Status:

```text
PASS
```

001A/B/C are now separable and failures are attributable.

## Gate H4 — State lifecycle

Status:

```text
PASS AS MFE CANDIDATE
```

Immediate pruning is intentionally simple and falsifiable.

## Gate H5 — Thin coverage debt

Status:

```text
PASS FOR PRE-MFE
```

No missing topic justifies expanding 001A.

---

# 36. Remaining uncertainties after hardening

These are intentionally left for code, not more paper design.

## U1 — Does typed WidgetRef<T> feel good in Rust?

```text
REQUIRES MFE-001A
```

## U2 — Is the product-state-after-run transaction acceptable?

```text
REQUIRES MFE-001A + HUMAN ERGONOMICS REVIEW
```

## U3 — Can transient graph borrowing remain simple?

```text
REQUIRES MFE-001A
```

## U4 — What is the real graph/allocation cost?

```text
INITIAL MEASURE IN 001A
FULL PRESSURE IN 001C
```

## U5 — Does bounded custom layout survive Grid/spatial UI?

```text
REQUIRES MFE-001B
```

## U6 — Is custom component extension genuinely ergonomic?

```text
REQUIRES MFE-001B
```

No additional broad architecture research is recommended before 001A.

---

# 37. MFE-001A pre-implementation checklist

Before writing runtime code, the implementation mission must repeat these non-negotiables:

```text
current legacy Ui untouched
no consumer migration
no external dependency
no new crate
no Grid
no custom Card
no mouse/touch expansion
no markup/SVG/animation
no public API freeze
no hidden closure replay
no game-state ownership in UiStateStore
```

And must explicitly test:

```text
Button activation
Toggle typed bool proposal
Slider typed f32 proposal
same-frame effective control paint
post-run consumer commit
stable keyed identity
stale-ID pruning
Rust borrow ergonomics
Tiny UI concept budget
```

---

# 38. Decision confidence after hardening

```text
Strategic GPE.UI R&D:
STRATEGY-BACKED — HIGHLY STABLE DIRECTION

Architecture B:
STRATEGY-BACKED + PRIOR-ART-BACKED + REQUIRES MFE

Transaction T1:
HYPOTHESIS ONLY + PRIOR-ART-INFORMED + REQUIRES MFE

Typed WidgetRef<T> output:
HYPOTHESIS ONLY + ICED-LIKE TYPED-FLOW INSPIRATION + REQUIRES MFE

No retained widget tree:
STRATEGY-BACKED + SMALL-GAME PRESSURE + REQUIRES MFE

UiStateStore immediate stale pruning:
HYPOTHESIS ONLY + DETERMINISM/BOUNDEDNESS MOTIVATION + REQUIRES MFE

MFE staging A/B/C:
ADVERSARIAL-REVIEW-BACKED

Markup later:
UNCHANGED

SVG optional/later:
UNCHANGED

No separate crate now:
UNCHANGED
```

---

# 39. Final hardening status

```text
PHASE 0 v0.3 HARDENING ADDENDUM:
COMPLETE

INDEPENDENT REVIEW CONDITIONS:
ADDRESSED DOCUMENTARILY

ARCHITECTURE B:
LEADING HYPOTHESIS — NOT FROZEN

MFE-001A:
DEFINED — NOT IMPLEMENTED

MFE-001B:
DEFINED — BLOCKED ON 001A

MFE-001C:
DEFINED — BLOCKED ON 001B

RUNTIME CHANGES:
NONE
```

# STOP

Do not implement MFE-001A from this addendum alone.

Next allowed step:

> **independent hardening review and explicit MFE-001A authorization**
