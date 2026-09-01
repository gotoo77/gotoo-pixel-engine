# GPE.UI — PRODUCTIONIZATION P1 / UNIFIED INPUT + INTERACTION v0.1

Status: **ACTIVE IMPLEMENTATION CONTRACT — P1 ONLY**

Parent roadmap:

`docs/ui/GPE_UI_PRODUCTIONIZATION_PLAN_v0.1.md`

P0 result:

`docs/ui/GPE_UI_PRODUCTIONIZATION_P0_RESULT_v0.1.md`

Baseline entering P1:

`2d6a2968807c804e1652c06f3e954325dd67a0cc`

---

# 1. Mission

Finish the interaction convergence that P0 made possible.

P0 established one canonical input vocabulary and one persistent interaction authority, but the two compatibility adapters still execute different interaction passes:

```text
transactional/Tiny
    run(..., UiNavInput, ...)
    linear focus + confirm/value handling
    no pointer/touch transaction input

spatial/Grid
    run_card_grid(..., UiInput, ...)
    spatial focus
    pointer/touch capture
    confirm activation
```

P1 must make both adapters consume one kernel interaction pass while preserving their accepted behavior.

---

# 2. Required end state

One kernel-level interaction pass owns the generic sequence:

```text
current resolved targets
    ↓
repair focus/captures
    ↓
apply navigation policy
    ↓
resolve hover
    ↓
pointer press/release capture semantics
    ↓
touch capture/move/end/cancel semantics
    ↓
confirm activation
    ↓
commit current order
    ↓
UiInteractionOutput
```

The pass must support at least two deterministic navigation policies:

```text
Linear
Spatial
```

The policy selects navigation behavior only. It must not create a second state/output model.

---

# 3. Input contract

Canonical transaction input remains:

```rust
UiInput<'a> {
    nav: UiNavInput,
    pointer: UiPointerInput,
    touches: &'a [Touch],
}
```

Physical key/gamepad mapping remains outside the UI kernel.

Existing compatibility calls remain valid:

```text
experimental::run(..., UiNavInput, ...)
experimental::run_headless(..., UiNavInput, ...)
experimental_spatial::run_card_grid(..., SpatialInput, ...)
experimental_spatial::run_card_grid_headless(..., SpatialInput, ...)
```

P1 may add explicit full-input transactional entry points, but must not break the existing MFE calls.

---

# 4. Generic target contract

Resolved targets used by the interaction pass must provide enough information for generic interaction without becoming a permanent widget tree.

Minimum semantics:

```text
UiId
integer Rect
focusable / interactive participation
activation capability
```

Action mapping may remain an adapter callback/lookup so the kernel does not depend on Card or widget content types.

Important distinction:

```text
focusable != activatable
```

Example: a Slider may take focus and pointer focus without treating Confirm as a generic activation event.

---

# 5. Transactional adapter behavior

The Tiny/transactional path must gain a full-input entry point using `UiInput` while retaining the nav-only adapters.

Required P1 behavior:

```text
pointer press on Button/Toggle/Slider -> focus target
pointer release on same activatable target -> activate target
pointer release elsewhere -> no activation
hover is exposed through UiOutput

touch Started -> capture/focus target
touch Moved outside -> cancel capture
touch Ended inside activatable captured target -> activate
touch Cancelled -> clear capture
```

Adapter-specific semantic handling remains adapter-specific:

```text
Button activation -> optional ActionId
Toggle activation -> typed bool proposal
Slider left/right -> typed f32 proposal
```

P1 does not implement pointer dragging for Slider.

---

# 6. Spatial adapter behavior

The Grid must stop owning its own pointer/touch/confirm interaction algorithm.

It should provide:

```text
resolved CardLayout targets
Spatial navigation policy
ActionId lookup for card IDs
```

and delegate generic interaction to the shared kernel pass.

Accepted MFE-001B behavior must remain unchanged:

```text
spatial direction ranking
pointer capture/release activation
touch move-off cancellation
touch activation
focus survives keyed reorder
removed focus repairs deterministically
cancel output
```

---

# 7. Scope boundaries

P1 MUST NOT implement:

```text
layout convergence / graph Grid      -> P2
styling / theming / rounded corners  -> P3
custom widget API                    -> P4
debug allocation optimization        -> P5
public v1 naming/API freeze          -> P6
consumer migration                   -> P7
```

P1 must not move platform event handling into the UI kernel.

---

# 8. Implementation sequence

## P1.1 — kernel interaction policy/pass

- define explicit Linear/Spatial navigation policy;
- extend resolved-target semantics only as required;
- centralize generic nav + hover + pointer + touch + confirm interaction;
- preserve exact MFE spatial ranking and capture rules.

## P1.2 — migrate Spatial adapter

- replace its bespoke pointer/touch/confirm logic with the shared pass;
- keep Grid geometry/painter/output compatibility surface;
- regression tests prove MFE behavior unchanged.

## P1.3 — migrate transactional adapter

- add full `UiInput` transaction entry points;
- keep nav-only `run` / `run_headless` as compatibility adapters;
- route linear focus + pointer/touch + generic activation through the shared pass;
- keep typed value semantics in the transactional adapter.

## P1.4 — closure review

Prove there is one generic interaction algorithm under both adapters and no duplicated pointer/touch/capture/confirm implementation remains.

---

# 9. Acceptance gates

Automated:

```text
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release --examples
Web build/package CI
Conventional Commits CI
```

Headless behavioral gates:

```text
Linear navigation wrap preserved
Spatial ranking preserved
Button confirm/action preserved
Toggle confirm typed proposal preserved
Slider left/right typed proposal preserved
Transactional pointer focus + Button activation
Transactional pointer Toggle activation
Transactional touch activation + move-off cancellation
Spatial pointer/touch behavior unchanged
hover exposed consistently
capture pruned when target disappears
cancel preserved
single consumer build closure execution preserved
```

Human runtime:

A final Native smoke test is required because P1 changes input/interaction semantics.

Web browser runtime is required only if P1 changes platform/input transport. This slice must not do so; otherwise Web build/package is the required P1 Web gate.

---

# 10. STOP

When P1 passes:

```text
GPE.UI PRODUCTIONIZATION P1 = PASS / PASS WITH CONDITIONS / FAIL
STOP
```

Record a formal P1 result before starting P2.
