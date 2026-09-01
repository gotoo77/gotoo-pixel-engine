# GPE.UI Phase 0 v0.3 — Independent Hardening Review

## Review target

Repository:

`gotoo77/gotoo-pixel-engine`

Hardening commit reviewed:

`e5d227fccbabbc37165c62bed110f77f306ecb45`

Parent independent adversarial review:

`c82f2d23732f2b0e77d2fa0eecd44eb07348c703`

Phase 0 v0.3 baseline:

`f008bcf2642067906826200d1f603c595c8ebf60`

Runtime/code evidence baseline:

`6ff4f8baddae269baa6a7d182f0ba0c9d985f886`

Primary document reviewed:

`docs/ui/phase0-v03/HARDENING_ADDENDUM.md`

External Rust prior-art spot-check date:

`2026-09-01`

This review is intentionally narrow.

It asks only:

> Did the hardening addendum close the conditions raised by the independent adversarial review well enough to authorize MFE-001A?

It does **not** reopen the full GPE.UI Phase 0.

---

# 1. Final verdict

```text
PHASE 0 v0.3 HARDENING:
PASS WITH MINOR MFE CONDITIONS

ARCHITECTURE B — BALANCED HYBRID:
REMAINS LEADING HYPOTHESIS
NOT FROZEN

MFE-001A:
AUTHORIZED AS THE NEXT EXPERIMENT

MFE-001B:
NOT AUTHORIZED
BLOCKED ON 001A

MFE-001C:
NOT AUTHORIZED
BLOCKED ON 001B

PRODUCTION GPE.UI:
NOT AUTHORIZED

LEGACY MIGRATION:
NOT AUTHORIZED
```

The hardening succeeds at its intended job.

The two blocking architectural holes from the previous review are no longer vague:

1. one explicit candidate frame transaction now exists;
2. one explicit typed value-flow candidate now exists.

Neither candidate is declared correct.

Both are now specific enough to falsify in compiled Rust.

That is the correct threshold for leaving paper design and entering MFE-001A.

The review found two additional bounded conditions that must be included in MFE-001A:

```text
C1  define WidgetRef<T> cross-transaction semantics explicitly

C2  detect/reset state when the same UiId is reused with an incompatible
    widget/value kind across transactions
```

These conditions do not justify another architecture addendum.

They belong in the experiment.

---

# 2. Review of previous Blocking Finding A — frame transaction

Previous status:

```text
BLOCKING BEFORE MFE
```

Previous problem:

The Phase 0 simultaneously implied:

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

Those are different contracts.

## Hardening resolution

The addendum now defines one candidate T1:

```text
1. input snapshot
2. consumer describes UI from current authoritative state
3. build transient graph
4. resolve identity/style
5. measure
6. arrange final Rects
7. resolve interaction/focus
8. derive frame-local effective widget values
9. emit activation / typed value proposals
10. paint using interaction state + effective widget values
11. finalize UiStateStore
12. return output/handles
13. consumer commits authoritative state changes
```

This is coherent.

A slider may therefore behave as:

```text
consumer value at transaction start = 0.65
interaction proposes              = 0.70
slider effective paint value      = 0.70
consumer value during ui::run     = 0.65
consumer commits after return     = 0.70
next transaction input value      = 0.70
```

The addendum also makes the unavoidable consequence explicit:

```text
other UI nodes already described from consumer state
are not rebuilt inside the same transaction
```

Therefore a separate label derived from `volume` may display the old value until the next transaction.

That is a real tradeoff, but it is no longer an architectural contradiction.

## Independent verdict

```text
PASS AS A FALSIFIABLE CANDIDATE
```

The experiment must judge whether the dependent-UI-next-transaction behavior is acceptable.

If it is not, T1 may fail.

That would be a legitimate Architecture B failure/revision signal.

## Note on T2/T3

The previous review requested explicit comparison with alternatives such as:

```text
T2 prepared session:
build/layout/interact
→ expose result
→ consumer mutation
→ paint

T3 deferred setter callbacks:
build
→ interact
→ execute stored mutations
→ paint
```

The hardening does not provide a symmetrical formal comparison table for T2/T3.

However it does explicitly reject the dangerous hidden behaviors that made those alternatives relevant:

```text
no hidden closure replay
no mutation during measure/layout
no gameplay-state mutation from paint
no retained arbitrary &mut gameplay references
```

It also preserves Architecture A/direct mutation as a fallback if T1 ergonomics fail.

Therefore this omission is **not blocking**.

Do not write another decision document solely to complete a comparison table.

MFE-001A should test T1 first.

If T1 fails specifically because post-run commit timing is unacceptable, then T2/Architecture A become targeted alternatives.

---

# 3. Review of previous Blocking Finding B — typed value flow

Previous status:

```text
BLOCKING BEFORE MFE
```

Previous problem:

Current GPE can express:

```rust
ui.toggle(..., &mut bool);
ui.slider_f32(..., &mut f32);
```

while the proposed multi-pass architecture only discussed:

```text
UiEvent
ActionId
```

Neither explained how a new typed value reaches consumer-owned state after layout/interaction.

## Hardening resolution

The addendum now separates:

```text
ActionId
→ discrete semantic intent

WidgetRef<T> + UiOutput
→ typed value proposal
```

Candidate shape:

```rust
// CANDIDATE — must be compiled/falsified
let (output, controls) = ui::run(..., |ui| {
    let enabled = ui.toggle("enabled", "ENABLED", settings.enabled);
    let volume = ui.slider_f32(
        "volume",
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

The addendum deliberately restricts the experimental internal value domain to the values needed by the MFE:

```text
Bool
F32
Index only if Select is nearly free
```

and rejects premature:

```text
Any
dynamic downcasting as the public model
serialization-based value transport
general reactive binding runtime
```

## Independent verdict

```text
PASS AS A FALSIFIABLE CANDIDATE
```

This is sufficient to start code.

The important architectural separation is now explicit:

```text
consumer authoritative value
→ copied into frame description
→ UI computes typed proposal
→ proposal returned explicitly
→ consumer commits
```

The MFE is responsible for determining whether this feels good in real Rust.

---

# 4. New condition C1 — WidgetRef<T> transaction semantics

Severity:

```text
MINOR DESIGN CONDITION
MUST BE RESOLVED IN MFE-001A
```

The addendum conceptually defines:

```rust
struct WidgetRef<T> {
    id: UiId,
    _type: PhantomData<fn() -> T>,
}
```

and also says the token is:

```text
valid only as a query key for the matching UiOutput transaction
```

Those two statements do not enforce one another.

A token containing only:

```text
UiId + type marker
```

has no transaction generation or lifetime marker.

Therefore a caller can conceptually retain it and query a later `UiOutput`.

This is not memory-unsafe by itself.

But the semantics must be explicit.

## Preferred MFE starting rule

Do **not** add generation machinery preemptively.

Start with:

```text
WidgetRef<T> is a stable identity/type query token.
UiOutput itself contains only events/value proposals from one transaction.
```

Therefore:

```text
querying an old WidgetRef<T> against a later UiOutput
is equivalent to querying that UiId/type in the later transaction.
```

If no matching value change exists:

```text
None
```

This keeps the token simple and may be useful for consumers that cache control references.

If experiment evidence shows cross-transaction handle reuse is confusing or dangerous, then consider:

```text
generation stamp
transaction token
lifetime-bound handle
```

Only with evidence.

## Required MFE test

```text
create handle in transaction N
query output N
query output N+1 with no change
query output N+1 after same stable widget changes
```

The behavior must be deterministic and documented.

---

# 5. New condition C2 — UiId kind/value compatibility

Severity:

```text
MAJOR LOCAL INVARIANT
MUST BE TESTED IN MFE-001A
```

The previous adversarial review explicitly requested testing key reuse under a different widget kind.

The hardening addendum closes duplicate-key handling but does not fully specify this cross-transaction case:

```text
transaction N:
UiId X = Slider<f32>

transaction N+1:
UiId X = Toggle<bool>
```

If `UiStateStore` blindly keys all retained state only by `UiId`, stale state may cross semantic widget boundaries.

Likewise an old:

```text
WidgetRef<f32>(X)
```

could query an output where X now means `bool`.

## Required invariant

Every stateful/interactable node participating in persistent state must have an implementation-private compatibility fingerprint sufficient to detect incompatible identity reuse.

This need not be a public `WidgetKind` API.

Conceptually:

```text
StateIdentity = UiId + compatibility kind
```

Examples of compatible/incompatible changes should be deliberately tested.

Minimum rule for MFE-001A:

```text
same UiId + same compatible widget/value kind
→ state may survive

same UiId + incompatible widget/value kind
→ old per-widget state is reset/pruned
→ focus/capture is repaired if required
→ deterministic diagnostic emitted in debug/test output
```

## Required tests

```text
Slider<f32>(id=X)
→ next transaction Slider<f32>(id=X)
→ compatible

Slider<f32>(id=X)
→ next transaction Toggle<bool>(id=X)
→ reset + diagnostic

Button(id=X)
→ next transaction noninteractive Text(id=X)
→ stale interaction state does not survive silently
```

This closes a real stale-state/type-confusion edge without requiring a larger architecture change.

---

# 6. Review of previous Finding C — MFE too broad

Previous status:

```text
MAJOR
```

## Hardening resolution

The experiment is now staged:

```text
MFE-001A
Transaction / typed values / identity / Rust ergonomics

        ↓ pass only

MFE-001B
Responsive / spatial / custom component

        ↓ pass only

MFE-001C
Cost / Native / Web runtime
```

001A explicitly excludes:

```text
Grid
responsive auto-fit
Image widget
custom Card
custom widget protocol
mouse/touch expansion
spatial navigation
markup
SVG
animation
Taffy
separate crate
legacy migration
```

## Independent verdict

```text
PASS
```

This is materially better experimental design.

A failure in 001A can now be attributed primarily to:

```text
transaction semantics
value flow
identity/state
basic layout/Rust ergonomics
```

rather than being confounded by Grid, touch or custom painting.

---

# 7. Review of previous Finding D — transient graph ownership

Previous status:

```text
MAJOR
```

## Hardening resolution

The addendum now says:

```text
no retained &mut references to gameplay/product state

immutable presentation data may be borrowed for ui::run only

small scalar control values are copied into frame-local node data

WidgetRef<T> is an ID/type token, not a borrow into graph storage
```

This is enough for 001A.

## Deliberate staging difference

The previous review suggested testing in 001A:

```text
borrowed Image
custom paint borrowing consumer presentation data
```

The hardening instead moves:

```text
Image
custom Card
custom component protocol
```

to 001B.

The independent review accepts this change.

Reason:

Those concerns are important, but they are not needed to falsify the first transaction/value-flow seam.

001A can still test a borrowed `&str`/text path and ordinary scalar consumer values.

001B remains explicitly responsible for borrowed image/custom-paint ergonomics.

## Independent verdict

```text
PASS FOR MFE-001A
DEFERRED OWNERSHIP PRESSURE REMAINS REQUIRED IN 001B
```

---

# 8. Review of previous Finding E — Rust-specific prior art

Previous status:

```text
MAJOR RESEARCH GAP, SMALL FIX
```

## Hardening coverage

The addendum adds targeted comparisons for:

```text
egui 0.36.1
Iced 0.14.0
Xilem 0.4.0
Masonry 0.4.0
```

The external spot check on 2026-09-01 confirms the material facts used by the addendum:

### egui 0.36.1

Official docs.rs source states that `Slider::new` receives a mutable user value and that egui changes that value directly during the widget call.

This remains the direct-immediate ergonomic benchmark.

### Iced 0.14.0

The current `slider`/`vertical_slider` helpers accept:

```text
current value
+
Fn(T) -> Message
```

and the example updates application state when the typed message is processed.

This validates typed value-out as a real Rust UI pattern.

### Xilem 0.4.0

Current architecture docs state that Xilem regenerates lightweight view trees, compares them with previous views, and updates a retained element tree; native Xilem uses Masonry as that retained backend.

It also explicitly places mutation inside a framework-controlled reactive cycle.

### Masonry 0.4.0

Current architecture docs define an explicit retained widget tree and multiple passes including event/update/layout/compose/paint/accessibility.

They emphasize working with Rust ownership rather than bypassing it.

## Independent verdict

```text
PASS
```

The hardening draws the correct lesson:

> multi-pass Rust UI needs an explicit ownership/transaction model.

It does **not** incorrectly infer that GPE must adopt reconciliation or a retained widget tree.

---

# 9. Review of previous Finding F — event model expansion

Previous status:

```text
MAJOR
```

## Hardening resolution

Public MFE-001A output is reduced to approximately:

```text
Activated
TypedValueChanged
optional ActionId
```

while interaction detail stays internal/debug:

```text
focused
hovered
pressed
captured
repeat
pointer enter/leave
```

Generic public drag/pointer transition events are deferred.

## Independent verdict

```text
PASS
```

This is the correct minimum.

Do not grow a second framework-level event taxonomy until a consumer/custom component demonstrates a need.

---

# 10. Review of previous Finding G — state lifecycle

Previous status:

```text
MAJOR BUT LOCAL
```

## Hardening resolution

The MFE now has an explicit seen-set model:

```text
node seen this transaction
→ eligible retained state remains

node absent this transaction
→ state pruned
```

and explicit behavior for:

```text
vanished focus
vanished capture
scroll/repeat/widget-local state
```

No TTL or hidden stale-state grace period exists in 001A.

Focus fallback for the initial Column experiment is deterministic.

## Independent verdict

```text
PASS WITH CONDITION C2
```

The missing final invariant is incompatible kind reuse for the same UiId, defined in §5 of this review.

Once that is included in MFE-001A tests, the lifecycle model is sufficiently specified for experimentation.

---

# 11. Review of previous Finding H — integer measurement over-freeze

Previous status:

```text
MAJOR DESIGN CONDITION
```

## Hardening resolution

The addendum now states:

> authoritative final GPE widget Rects are deterministic integer logical-pixel geometry.

while allowing future internal measurement to become fractional if text/vector needs justify it.

001A stays integer-only because current bitmap/font/framebuffer primitives do not need fractional metrics.

## Independent verdict

```text
PASS
```

This preserves GPE's pixel discipline without turning today's implementation simplification into a permanent typography constraint.

---

# 12. Review of documentation-completeness debt

Previous missing/thin areas:

```text
data flow
world-space pressure
authoring/hot reload
diagnostics/versioning/security
```

## Hardening result

### Data flow

Closed sufficiently for 001A through explicit:

```text
consumer authoritative state
UiStateStore interaction state
UiGraph one-transaction description
UiOutput typed proposals/events
```

### World-space

Correctly retained only as compatibility pressure.

Kernel remains target/surface based rather than desktop-window-owned.

No 3D machinery enters 001A/B.

### Authoring/hot reload

Correctly kept later.

The important architectural constraint is preserved:

```text
Rust description ──┐
                   ├─> same semantic frame model
future markup ─────┘
```

### Diagnostics/versioning/security

Sufficiently bounded for pre-MFE:

```text
experimental API
no stability promise
textual diagnostics first
strict future parsing
no arbitrary scripting
```

## Independent verdict

```text
PASS FOR PRE-MFE
```

No further broad Phase 0 document is justified before 001A.

---

# 13. Additional pressure — ActionId and future authoring

Severity:

```text
NON-BLOCKING FUTURE COMPATIBILITY NOTE
```

Current GPE `ActionId` is backed by:

```rust
&'static str
```

This is excellent for compiled Rust constants.

A future **runtime-hot-reloaded** markup system might need action identifiers whose source strings are not `'static`.

Therefore MFE-001A may freely reuse current `ActionId`, but Phase 0 must not silently freeze the following claim:

```text
current ActionId representation is necessarily the final action-identity representation for all future authoring modes
```

Possible future outcomes include:

```text
build-time markup -> static ActionId
interned runtime UiActionId
mapping markup symbol -> compiled ActionId
other explicit adapter
```

No action is needed now.

This is only a revisit trigger if runtime markup/hot reload becomes real.

---

# 14. Additional pressure — effective values and dependent UI

The most consequential remaining UX semantic is now intentionally exposed:

```text
control paints its proposed value this transaction
independent dependent UI built from consumer state updates next transaction
```

Example:

```text
Slider paints 0.70 immediately
GLOBAL VOLUME label may still paint 0.65 this transaction
```

This is not a documentation bug anymore.

It is the key human/API experiment.

MFE-001A must include one realistic settings probe where:

```text
slider
+
separate derived text/value presentation
```

are visible together.

The human review must explicitly answer:

```text
ACCEPTABLE
or
UNACCEPTABLE
```

If unacceptable, do not hide the problem with closure replay or implicit game-state mutation.

Instead revisit:

```text
prepared-session transaction
more immediate Architecture A
a deliberately scoped same-frame binding mechanism
```

---

# 15. MFE-001A authorization contract

The next implementation slice is authorized **only** for MFE-001A.

## Allowed

```text
experimental UI core sufficient for 001A
UiId
compatibility kind/fingerprint
UiStateStore
transient graph or equivalent
simple Constraints
Column
Panel
Text
Button
Toggle
Slider<f32>
semantic navigation needed by these controls
UiOutput
WidgetRef<bool>
WidgetRef<f32>
ActionId activation
headless dump/tests
minimal rendering through existing GPE primitives
measurements requested by 001A
```

## Explicitly not allowed

```text
Grid
responsive auto-fit
Image widget as a new generalized UI capability
custom Card
custom-widget framework
spatial focus algorithm
mouse/touch expansion beyond unavoidable existing plumbing
markup
SVG
animation
inspector
Taffy
separate UI crate
legacy Ui rewrite/deprecation
consumer migration
Arcade integration
PauseGame migration
production API freeze
```

## New mandatory tests from this review

In addition to the hardening checklist:

```text
WidgetRef<T> cross-transaction query semantics
same UiId / same value kind preserves compatible state
same UiId / incompatible value or widget kind resets state + diagnoses
old typed handle against new incompatible kind is deterministic
separate dependent label demonstrates T1 one-transaction semantics
```

---

# 16. MFE-001A result gate

Allowed results:

```text
PASS
PASS WITH CONDITIONS
FAIL
```

## PASS requires

All of the following must be credible:

```text
T1 transaction is understandable
Toggle/Slider typed value flow is ergonomic enough
ordinary consumer-owned settings fields compile without borrow gymnastics
no hidden closure replay
no pervasive dynamic typing
stable identity is understandable
kind reuse cannot silently corrupt retained UI state
Tiny UI remains genuinely small
headless transaction/identity traces are useful
measured costs are recorded rather than guessed
```

## PASS WITH CONDITIONS

Use only for bounded issues that do not call the transaction/value/identity model into question.

## FAIL

Use if any fundamental seam is poor:

```text
WidgetRef plumbing is clearly worse than benefit gained
consumer post-run commit is unnatural in ordinary GPE code
same-frame effective-value semantics are confusing
borrow model requires interior-mutability workarounds
identity/key rules require IDs everywhere
state compatibility cannot be made deterministic
transient graph machinery is disproportionate even in Tiny
```

On FAIL:

# STOP

Do not proceed to 001B.

Revisit Architecture A or the transaction model.

---

# 17. Why another paper phase is not recommended

The remaining uncertainties are now empirical Rust questions:

```text
Does it compile cleanly?
Does it read cleanly?
Does T1 feel coherent?
Does typed proposal-out feel natural?
Are stable keys rare enough?
What does the graph actually cost?
```

More prose cannot answer these reliably.

The hardening has crossed the correct threshold:

```text
specific enough to falsify
not prematurely frozen
```

Continuing architectural prose before compiling 001A would now reduce information gain.

---

# 18. Branch discipline

This review is committed on the existing active hardening branch:

`research/gpe-ui-phase0-v03-hardening`

No additional review branch is required for methodological independence.

Independence is established by:

```text
reviewing a frozen parent commit
explicit review document
separate review commit
```

not by multiplying Git refs.

A previously-created unused review ref may be removed during repository branch cleanup; it is not part of the required workflow.

Recommended branch policy from this point:

```text
one active branch per actual implementation/research stream
checkpoint commits on that branch
new branch only for real divergence, parallel work, or a separately mergeable PR
```

---

# 19. Final status

```text
INDEPENDENT HARDENING REVIEW:
PASS WITH MINOR MFE CONDITIONS

PREVIOUS BLOCKING FINDING A — TRANSACTION:
CLOSED FOR EXPERIMENT

PREVIOUS BLOCKING FINDING B — TYPED VALUE FLOW:
CLOSED FOR EXPERIMENT

MFE STAGING:
PASS

OWNERSHIP MODEL:
PASS FOR 001A
CUSTOM/IMAGE OWNERSHIP DEFERRED TO 001B

RUST PRIOR ART:
PASS

EVENT SURFACE:
PASS

STATE LIFECYCLE:
PASS WITH C2 KIND-COMPATIBILITY TEST

INTEGER GEOMETRY CONDITION:
PASS

THIN COVERAGE DEBT:
PASS FOR PRE-MFE

ARCHITECTURE B:
LEADING HYPOTHESIS — NOT FROZEN

MFE-001A:
AUTHORIZED

MFE-001B:
BLOCKED

MFE-001C:
BLOCKED

PRODUCTION / MIGRATION:
NOT AUTHORIZED
```

# STOP

The next allowed step is a **dedicated MFE-001A implementation mission**.

Do not implement 001B or any production migration until 001A receives its own PASS/PASS WITH CONDITIONS result and review.
