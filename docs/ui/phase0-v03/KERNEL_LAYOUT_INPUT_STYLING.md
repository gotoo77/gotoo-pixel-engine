# Kernel, Layout, Input, Styling and Customization

This document defines the **recommended conceptual direction**, not an implementation contract.

Decision class for the central architecture:

```text
STRATEGY-BACKED
PRIOR-ART-BACKED
EVIDENCE-INFORMED
REQUIRES MFE
```

---

# 1. State/lifetime decomposition

Do not ask only:

> immediate or retained?

Separate four lifetimes.

## 1.1 Gameplay state

Examples:

```text
volume
selected weapon
inventory contents
arcade active game
pause state
```

Owner:

```text
consumer/game
```

GPE.UI must not become the source of truth.

## 1.2 UI description

Recommended lifetime:

```text
one frame
```

The consumer describes the current UI from current application state.

## 1.3 Interaction state

Examples:

```text
focused id
pointer capture
scroll offset
repeat timing
drag state
optional local animation state
```

Recommended lifetime:

```text
persistent across frames
```

Owner:

```text
explicit UiState / UiStateStore owned by consumer or embedding surface
```

No global manager.

## 1.4 Layout/paint output

Recommended lifetime:

```text
one frame
```

May be dumped for tests/diagnostics.

---

# 2. Recommended internal model: transient frame graph

Candidate conceptual pipeline:

```text
Rust composition
        ↓
Transient UiGraph
        ↓
resolve IDs + style
        ↓
measure
        ↓
layout
        ↓
focus / pointer / nav interaction
        ↓
UiOutput events/actions
        ↓
paint
```

The graph is not a DOM.

It does not need to survive the frame.

It exists because strategic layout/customization requires information that the current one-row-at-a-time API does not always have at the point a call is made.

Potential node metadata:

```text
identity
kind
children
layout policy
style references
interaction sense
semantic action
content measurement
custom paint hook
```

Do not freeze this structure before MFE.

---

# 3. Identity

## 3.1 Why identity is justified

Observed evidence:

- current `UiState` already retains focus/capture/scroll state;
- current identity is ordinal;
- structural changes require `reset_interaction()`.

Strategic pressures:

- conditional children;
- responsive restructuring;
- future markup;
- focus restoration;
- accessibility projection;
- animation state.

Therefore stable identity is justified as a **kernel concept candidate**.

## 3.2 Do not make explicit IDs mandatory everywhere

Preferred model:

```text
hierarchical identity
```

A child receives an identity derived from:

```text
parent identity
+
local declaration slot
```

with explicit stable key/salt available for:

```text
dynamic lists
conditional reordering
consumer-controlled state preservation
markup IDs
```

This follows the useful egui lesson:

> stable identity can coexist with a transient immediate description.

## 3.3 Determinism

Identity generation must not depend on randomized process hash state.

Exact representation is unresolved:

```text
u64 hash
interned path
small path structure
other deterministic key
```

MFE should test:

```text
same tree → same IDs
conditional sibling insertion does not steal state when keyed
duplicate explicit IDs produce deterministic diagnostics
```

---

# 4. Frame transaction

Recommended semantic order:

```text
1. capture input snapshot / semantic nav snapshot
2. build pure-ish transient description
3. resolve styles and identities
4. measure
5. arrange integer Rects
6. resolve hover / capture / focus / spatial navigation
7. emit semantic UiEvents / ActionIds
8. consumer applies gameplay mutations
9. paint resolved frame
```

There is an important unresolved timing question between steps 7–9:

> Should gameplay mutations produced by UI events affect paint in the same frame?

Two legitimate policies exist.

## Policy A — paint pre-mutation description

```text
build
layout
interact
paint
return output
consumer mutates after run
```

Pros:

- pure transaction;
- easy borrow model;
- no closure replay.

Con:

- visible value change may appear next frame.

At 60+ FPS this may be acceptable for many controls but must be tested.

## Policy B — allow selected UI-local visual state to reflect interaction immediately

Example:

```text
pressed/highlight/focus
```

while product value changes remain output events.

This is recommended for MFE because it avoids a hidden second execution of user UI code.

**Do not replay the consumer's closure automatically**: arbitrary user code may have side effects.

---

# 5. Output/event model

Recommended kernel-level distinction:

```text
UiEvent
≠
gameplay mutation
```

Candidate event kinds:

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

Not all need to exist in v1.

Widgets may also carry a semantic `ActionId`.

Example concept:

```text
Activated {
    target: UiId,
    action: Some(ActionId("menu.resume"))
}
```

The output can be:

```text
UiOutput {
    events,
    focus,
    diagnostics,
}
```

Benefits:

- headless traces;
- deterministic replay;
- feedback mapping;
- markup can reference action IDs without arbitrary code;
- no callback lifetime tangles.

## Propagation

Do **not** copy DOM capture/bubble by default.

v1 candidate:

```text
hit-test selects one spatial target
modal/focus scope may intercept
target emits
optional parent semantic handler only if a concrete composition case requires it
```

Generic bubbling remains `LATER/UNKNOWN`.

---

# 6. Layout model

Responsive layout is strategic.

Recommended foundation:

```text
Constraints {
    min_width
    max_width
    min_height
    max_height
}
```

using integer logical-pixel units for the final authoritative geometry.

Conceptual protocol:

```text
measure(node, constraints) -> Size
arrange(node, Rect)
```

The engine may use wider/internally signed arithmetic to avoid overflow, but public final layout is integer `Size/Rect` compatible with GPE.

## 6.1 Core containers

Recommended v1/core candidates:

```text
Padding
Row
Column
Stack
Grid
Align
Sized / Constrained
Spacer
Anchor / Absolute escape hatch
Clip
```

`Scroll` is likely a core widget/container because existing UI already has scrolling pressure.

## 6.2 Linear flex

Useful but avoid CSS naming/semantics unless intentionally compatible.

Candidate simple semantics:

```text
fixed/intrinsic children
+
weighted remaining-space children
+
gap
+
main/cross alignment
```

Call it `Flex` only if semantics are clearly documented.

## 6.3 Grid

Grid is strategically useful for:

- Arcade cards;
- inventories;
- settings dashboards;
- tactical/game selection surfaces.

v1 does not need full CSS Grid.

Candidate:

```text
fixed columns
auto-fit minimum cell width
equal tracks
explicit row/column spans only if real probes require them
```

## 6.4 Wrap

Useful for responsive chip/button/card rows.

Could be core or early optional.

MFE should determine whether responsive Grid already covers the main use.

## 6.5 Taffy decision

Phase 0 recommendation:

```text
NO mandatory Taffy dependency in v1 kernel.
```

But do not reject Taffy.

Trigger for a targeted spike:

> if implementing/validating the custom constraint/grid layer reveals significant algorithmic complexity or correctness burden.

Then compare:

```text
custom integer core
vs
Taffy backend + deterministic pixel-rounding adapter
```

Measure actual dependency/build/WASM cost.

---

# 7. Pixel geometry contract

GPE.UI should be:

> **pixel-aware, not pixel-imprisoned**

Recommended rules:

1. authoritative final widget Rects are integer logical framebuffer coordinates;
2. parent distributes rounding deterministically;
3. no accumulated floating drift across siblings;
4. image widgets expose `ImageFit` and `ImageFilter`;
5. pixel-art presets use `Nearest`;
6. physical-window scaling remains Viewport/renderer responsibility;
7. layout does not read platform DPI directly;
8. text metrics enter through GPE text measurement, not browser/system-font assumptions.

## Rounding invariant

For a row dividing width among N weighted children:

```text
sum(child widths) + gaps == allocated parent content width
```

Any remainder distribution must be deterministic, for example stable left-to-right allocation.

---

# 8. Input model

GPE has two useful worlds today:

```text
semantic:
ActionId / ControlMap / VirtualPad

spatial:
Input mouse position/buttons/touches
```

Do not choose one and throw away the other.

Recommended UI input snapshot:

```text
UiNavInput
- Up
- Down
- Left
- Right
- Confirm
- Cancel
- Next
- Previous

UiPointerInput
- pointer positions
- button transitions
- touch contacts/events
```

Adapters can derive `UiNavInput` from `ControlMap`.

The UI core should not know that "Confirm" is Space or gamepad South.

Consumers/default helpers choose bindings.

---

# 9. Focus model

Required concepts:

```text
focused UiId
focusable
disabled
focus scope
modal scope
focus restoration
```

Spatial navigation must be deterministic.

## Linear navigation

For simple lists:

```text
declaration/layout order
```

## Spatial navigation

For grids and non-linear UI:

Candidate ranking uses resolved geometry, not hardcoded row indices.

Potential factors:

```text
requested direction
axis overlap
primary-axis distance
secondary-axis distance
stable tiebreaker by layout/declaration order
```

Exact algorithm requires MFE/property tests.

## Modal scope

Pause/dialog should establish a focus scope.

Outside targets become non-interactable until scope closes.

On close:

```text
restore previous focus if still valid
else deterministic fallback
```

This generalizes the current PauseGame input-leakage/focus pressure without forcing a scene manager.

---

# 10. Pointer/touch

Pointer interaction should be centralized around resolved Rects and identity.

Kernel candidates:

```text
hover target
active/captured target
press/release transitions
touch-id capture
cancel
```

Full gesture recognition is not v1 core.

A custom widget may request a `Sense`:

```text
hover
click
drag
scroll
focus
```

The kernel resolves generic spatial interaction; widget paint reacts to state.

This is preferable to every custom widget reimplementing pointer capture.

---

# 11. Styling

High customization is strategic, but full CSS is not.

Recommended three layers:

```text
Theme / design tokens
        ↓
component style
        ↓
local override
```

Example tokens:

```text
spacing
corner/border metrics
text roles
foreground/background roles
accent
disabled alpha
focus treatment
```

Component style examples:

```text
ButtonStyle
PanelStyle
SliderStyle
CardStyle
```

Local override should be composable, not a chain of hundreds of setters.

## State-based styling

Core widget style may resolve from:

```text
normal
hovered
focused
active/pressed
disabled
selected
```

Do not introduce CSS selectors/cascade.

---

# 12. Custom widget model

A first-class escape hatch is mandatory.

The most useful decomposition is likely:

```text
measurement
interaction declaration
paint
semantics
```

Interaction should remain mostly kernel-owned.

Candidate conceptual API:

```rust
// CANDIDATE — NOT VALIDATED BY COMPILATION
ui.custom(id, |node| {
    node.measure(...);
    node.sense(Sense::click_and_focus());
    node.paint(|paint, rect, interaction| { ... });
    node.action(OPEN_INVENTORY);
});
```

Alternative trait-based shape:

```rust
// CANDIDATE — NOT VALIDATED BY COMPILATION
trait UiComponent {
    fn measure(&self, ctx: &MeasureCtx, constraints: Constraints) -> Size;
    fn paint(&self, ctx: &mut PaintCtx, rect: Rect, state: InteractionState);
    fn semantics(&self) -> Semantics;
}
```

Exact generic/dynamic-dispatch tradeoff is unresolved.

MFE must test whether the escape hatch is pleasant in real Rust.

---

# 13. Game feedback

Widgets should not own `Audio`, haptics or scenes.

Recommended flow:

```text
UiEvent / ActionId
        ↓
optional UiFeedbackPolicy
        ↓
PlaySound / Haptic / Animate / consumer action
```

Existing `AudioBus::Ui` can be a useful downstream capability, but no Button should require audio.

Controller glyph rendering may consume "last active input kind" later, but should be presentation metadata, not input semantics.

---

# 14. Animation

Verdict:

```text
OPTIONAL CAPABILITY
NOT KERNEL REQUIREMENT
```

Useful minimal properties:

```text
opacity
offset
scale
color
clip/reveal
duration
easing
```

Animation state can live in the keyed UI state store or a dedicated optional layer.

Do not make layout depend on a general timeline engine in v1.

---

# 15. Text / i18n / accessibility pressure

v1 need not implement browser-grade typography/accessibility.

But avoid these dead ends:

```text
identity that cannot project to semantics
text API that assumes ASCII byte length
layout that assumes text never expands
focus hidden from tooling
hardcoded physical key labels
```

Recommended foundation:

```text
stable semantic identity
text measurement abstraction
semantic role/label slots
focus state observable
layout handles dynamic measured size
```

AccessKit is a plausible future projection target, not a required dependency now.

---

# 16. Kernel verdict

Recommended v1 kernel concepts:

```text
UiId
UiStateStore
Constraints
UiGraph / transient frame description
resolved integer Rect layout
interaction state
semantic navigation snapshot
pointer/touch snapshot
UiEvent / semantic ActionId
paint context using existing GPE primitives
headless frame output/dump
```

Still deliberately absent:

```text
DOM
CSS cascade
JavaScript
permanent retained widget object tree
global UI manager
full reactive property engine
full gesture framework
```
