# GPE.UI — PRODUCTIONIZATION P0 / KERNEL CONVERGENCE v0.1

Status: **IMPLEMENTATION CONTRACT — P0 ONLY**

Parent plan:

`docs/ui/GPE_UI_PRODUCTIONIZATION_PLAN_v0.1.md`

Baseline:

`7a89ddc09ed941c409b2b459e1361fbfb80e639f`

## Mission

Converge the MFE-001A transaction path and MFE-001B spatial path into one production-kernel interaction model while preserving accepted behavior.

P0 is intentionally not a visual-feature slice.

It must not add styling, rounded corners, consumer migration, markup, SVG, Taffy, animation, or typography changes.

---

# 1. Current split to eliminate

## Transactional path

`src/ui/experimental.rs`

Owns:

```text
UiId
UiNavInput
UiStateStore
UiOutput
WidgetRef<T>
UiDiagnostic
UiMetrics
transient Node graph
measure / arrange
linear focus repair/navigation
typed value proposals
ActionId activation
paint
debug dump
```

## Spatial path

`src/ui/experimental_spatial.rs`

Owns separately:

```text
PointerInput
SpatialInput
SpatialState
SpatialOutput
spatial focus repair/navigation
pointer capture
touch capture
activation / ActionId output
resolved Card layouts
debug dump
```

The duplication is experimental debt, not a desired production abstraction.

---

# 2. Canonical P0 concepts

P0 introduces one canonical kernel vocabulary.

Names below are productionization names, not yet v1 stability promises.

## 2.1 Identity

```rust
UiId
```

Keep the deterministic identity behavior proven by MFE-001A.

No second spatial/card identity type.

## 2.2 Input snapshot

Canonical shape:

```rust
UiInput<'a> {
    nav: UiNavInput,
    pointer: UiPointerInput,
    touches: &'a [Touch],
}

UiPointerInput {
    position: Option<(i32, i32)>,
    pressed: bool,
    released: bool,
}
```

Rules:

- kernel sees semantic nav, not physical Space/South/WASD policy;
- pointer/touch are spatial facts;
- existing `experimental::run(..., UiNavInput, ...)` may remain as compatibility adapter using default pointer/touch input;
- existing `SpatialInput` / `PointerInput` names may remain temporary aliases/adapters during P0.

## 2.3 Persistent interaction state

One canonical store owns:

```text
generation
focused UiId
per-id kind/lifetime bookkeeping
previous focus order / deterministic repair data
pointer capture UiId
touch-id -> UiId capture
```

Required invariant:

> there is one focused `UiId` authority for one UI transaction surface.

`SpatialState` must not survive as an independent production state owner.

It may temporarily wrap/borrow the canonical store solely to keep the MFE probe compiling while migration is incomplete.

## 2.4 Resolved interaction target

After layout, every interactive element needed by the kernel can be represented conceptually as:

```text
UiId
Rect
interaction kind / sense
semantic action metadata
navigation grouping/policy metadata when required
stable resolved order
```

This is not a DOM node and need not survive the frame.

Pointer hit-testing and spatial navigation operate on resolved targets, not Card-specific structs.

## 2.5 Output

One canonical `UiOutput` owns:

```text
focused id
hovered id when pointer input exists
activated ids
ActionId intentions
typed proposed value changes
cancel request
diagnostics
metrics
optional/debug trace data
```

Grid-specific geometry observations may remain widget/layout-specific data returned to a probe, but activation/focus/action state must not require a second `SpatialOutput` authority.

---

# 3. Transaction order

Preserve the accepted T1 semantics:

```text
input snapshot
    ↓
build transient description
    ↓
resolve identity/style inputs
    ↓
measure
    ↓
arrange integer geometry
    ↓
collect resolved interaction targets
    ↓
repair focus/capture against current IDs
    ↓
linear/spatial nav + pointer/touch interaction
    ↓
frame-local effective values / semantic actions
    ↓
paint
    ↓
finalize persistent interaction state
    ↓
return UiOutput
    ↓
consumer commits gameplay/product state
```

Do not replay the consumer build closure.

---

# 4. Navigation convergence

Do not solve convergence by forcing one navigation algorithm globally.

Required production concept:

```text
one focus authority
multiple deterministic navigation policies/scopes
```

Initial policies proven by current MFEs:

```text
Linear
    declaration/resolved order

Spatial
    resolved Rect geometry
    direction filter
    primary distance
    secondary distance
    stable order tiebreak
```

A Grid may establish a spatial navigation scope while a simple menu/column uses linear navigation.

P0 does not require full modal/focus-scope implementation, but the representation must not prevent it.

---

# 5. Pointer/touch convergence

Pointer and touch capture move to canonical interaction state.

Required semantics preserved from MFE-001B:

```text
mouse press on target
    -> capture target
    -> focus target

mouse release on same captured target
    -> activate

mouse release elsewhere
    -> no activation
    -> clear capture

touch Started inside target
    -> capture touch-id -> target
    -> focus target

touch Moved outside captured target
    -> cancel that capture

touch Ended inside still-captured target
    -> activate
    -> clear capture

touch Cancelled
    -> clear capture
```

When a captured target disappears during a structural change, capture is deterministically pruned.

---

# 6. Dynamic identity/focus repair

Preserve both MFE guarantees:

- keyed identity survives reorder;
- if focused target disappears, deterministic fallback uses prior resolved position/order where possible.

The exact internal repair data may change, but observable behavior must remain deterministic and headless-testable.

Same `UiId` reused for an incompatible interaction/value kind must continue to reset/diagnose rather than silently inherit incompatible state.

---

# 7. Debug/diagnostic boundary

P0 introduces the architectural separation but does not need to finish allocation optimization.

Target distinction:

```text
mandatory transaction result
vs
debug/headless textual dump
```

Current `dump()` compatibility may remain during P0.

P5 will make dump construction opt-in/attributed and remeasure allocations.

Do not delete deterministic traces merely to improve a benchmark.

---

# 8. Compatibility bridge

During P0 the following existing experimental calls should continue to compile unless a smaller explicit migration is reviewed first:

```text
experimental::run
experimental::run_headless
experimental_spatial::run_card_grid
experimental_spatial::run_card_grid_headless
```

Compatibility implementation may delegate into the converged kernel.

The adapters are temporary evidence-preservation surfaces, not proposed v1 names.

---

# 9. P0 implementation sequence

## P0.1 — shared input vocabulary

- introduce canonical `UiPointerInput` + `UiInput`;
- retain compatibility aliases for `PointerInput` / `SpatialInput`;
- zero behavior change.

## P0.2 — shared interaction state primitives

- move focus/pointer/touch capture authority toward one state representation;
- keep keyed-kind/generation bookkeeping intact;
- headless tests prove reorder/removal/capture behavior.

## P0.3 — resolved generic interaction targets

- extract Card-specific hit-test/navigation mechanics into generic resolved-target helpers;
- preserve exact deterministic spatial ranking.

## P0.4 — output convergence

- activation/actions/focus/hover flow into canonical output concepts;
- Card/Grid-specific output retains geometry only where useful;
- avoid two competing semantic output authorities.

## P0.5 — adapter closure

- existing MFE probes exercise the converged internals;
- no second independent runtime remains underneath the compatibility API.

---

# 10. P0 acceptance gates

Automated:

```text
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release --examples
Web build/package CI
```

Behavioral/headless minimum:

```text
T1 typed value proposal behavior unchanged
stable WidgetRef generation behavior unchanged
incompatible UiId kind diagnostic unchanged
keyed spatial focus survives reorder
focused removed target repairs deterministically
mouse capture/release semantics unchanged
touch move-off cancellation unchanged
touch activation unchanged
spatial direction ranking unchanged
headless deterministic replay unchanged
```

Human runtime:

A single final P0 probe run is sufficient if automated tests preserve the exact previously accepted MFE behavior.

Check:

```text
Native: responsive Grid + focus/filter/reorder + mouse/gamepad
Web: build/package required
actual browser rerun only if P0 touches platform/input transport semantics
```

---

# 11. P0 STOP

When P0 passes:

```text
STOP
review convergence
record P0 result
squash merge
```

Do not opportunistically begin P1/P2/P3 in the same implementation slice.

In particular, **do not implement styling or rounded corners inside P0** even though they are already committed roadmap items for P3.
