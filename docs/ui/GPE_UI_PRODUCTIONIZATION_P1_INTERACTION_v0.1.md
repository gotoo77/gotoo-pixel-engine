# GPE.UI — PRODUCTIONIZATION P1 / UNIFIED INPUT + INTERACTION v0.1

Status: **COMPLETE — PASS / STOP**

Parent roadmap:

`docs/ui/GPE_UI_PRODUCTIONIZATION_PLAN_v0.1.md`

P0 result:

`docs/ui/GPE_UI_PRODUCTIONIZATION_P0_RESULT_v0.1.md`

P1 result:

`docs/ui/GPE_UI_PRODUCTIONIZATION_P1_RESULT_v0.1.md`

Baseline entering P1:

`2d6a2968807c804e1652c06f3e954325dd67a0cc`

Validated implementation head before result documentation:

`cd164600c23e29402a666f93b3f2b007dc0fd8a0`

---

# 1. Mission

Finish the interaction convergence that P0 made possible.

P0 established one canonical input vocabulary and one persistent interaction authority, but the two compatibility adapters still executed different interaction passes.

P1 makes both adapters consume one kernel interaction pass while preserving their accepted behavior.

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

The pass supports two deterministic navigation policies:

```text
Linear
Spatial
```

The policy selects navigation behavior only. It does not create a second state/output model.

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

P1 also adds explicit full-input transactional entry points without breaking the existing MFE calls.

---

# 4. Generic target contract

Resolved targets used by the interaction pass provide enough information for generic interaction without becoming a permanent widget tree.

Minimum semantics:

```text
UiId
integer Rect
focusable / interactive participation
activation capability
```

Action mapping remains an adapter callback/lookup so the kernel does not depend on Card or widget content types.

Important distinction:

```text
focusable != activatable
```

Example: a Slider may take focus and pointer focus without treating Confirm as a generic activation event.

---

# 5. Transactional adapter behavior

The Tiny/transactional path now has a full-input entry point using `UiInput` while retaining the nav-only adapters.

Preserved/validated P1 behavior:

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

The Grid no longer owns its own pointer/touch/confirm interaction algorithm.

It provides:

```text
resolved CardLayout targets
Spatial navigation policy
ActionId lookup for card IDs
```

and delegates generic interaction to the shared kernel pass.

Accepted MFE-001B behavior remains unchanged:

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

P1 did not implement:

```text
layout convergence / graph Grid      -> P2
styling / theming / rounded corners  -> P3
custom widget API                    -> P4
debug allocation optimization        -> P5
public v1 naming/API freeze          -> P6
consumer migration                   -> P7
```

P1 does not move platform event handling into the UI kernel.

---

# 8. Implementation sequence

## P1.1 — kernel interaction policy/pass

Status: **PASS**.

## P1.2 — migrate Spatial adapter

Status: **PASS**.

## P1.3 — migrate transactional adapter

Status: **PASS**.

## P1.4 — closure review

Status: **PASS**.

One generic interaction algorithm is used under both adapters. No duplicated production pointer/touch/capture/confirm implementation remains in the adapters.

---

# 9. Acceptance gates

Automated validation on implementation head `cd164600c23e29402a666f93b3f2b007dc0fd8a0`, GitHub Actions CI run **#655**:

```text
cargo fmt --check                                         PASS
cargo test                                                PASS
cargo clippy --all-targets --all-features -- -D warnings PASS
cargo build --release --examples                          PASS
Web build/package CI                                      PASS
Conventional Commits CI                                   PASS
```

Headless behavioral gates:

```text
Linear navigation wrap preserved                       PASS
Spatial ranking preserved                              PASS
Button confirm/action preserved                        PASS
Toggle confirm typed proposal preserved                PASS
Slider left/right typed proposal preserved             PASS
Transactional pointer focus + Button activation        PASS
Transactional pointer Toggle activation                PASS
Transactional touch activation + move-off cancellation PASS
Spatial pointer/touch behavior unchanged               PASS
hover exposed consistently                             PASS
capture pruned when target disappears                  PASS
cancel preserved                                       PASS
single consumer build closure execution preserved      PASS
```

Human Native runtime gate:

```text
P1 NATIVE RUNTIME = PASS
```

Web browser runtime was not rerun because P1 did not modify platform/input transport semantics; Web build/package is the required P1 Web gate.

---

# 10. STOP

```text
GPE.UI PRODUCTIONIZATION P1 = PASS
STOP
```

P1 is complete. Do not begin P2 on this implementation branch. P2 requires an explicit new start from the merged P1 baseline.
