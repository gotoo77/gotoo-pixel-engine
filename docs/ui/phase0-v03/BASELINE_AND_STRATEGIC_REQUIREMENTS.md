# Baseline and Strategic Requirements

## 1. Evidence model

This document distinguishes:

```text
OBSERVED REQUIREMENT
STRATEGIC REQUIREMENT
SPECULATIVE FEATURE
```

`OBSERVED` means demonstrated by current GPE/consumer code.

`STRATEGIC` means explicitly desired as part of GPE.UI's product capability.

`SPECULATIVE` means neither currently demonstrated nor intentionally selected as a strategic capability.

A strategic requirement is legitimate even if a current consumer can work around its absence.

---

# 2. GPE baseline

Repository:

`gotoo77/gotoo-pixel-engine`

Exact audited code baseline:

`6ff4f8baddae269baa6a7d182f0ba0c9d985f886`

Phase 0 research branch later contains documentation only; runtime evidence below remains tied to the exact code baseline.

## Cargo / compilation boundary

Provenance:

```text
repo: gotoo77/gotoo-pixel-engine
ref: 6ff4f8baddae269baa6a7d182f0ba0c9d985f886
file: Cargo.toml
```

Facts:

- package `gotoo-pixel-engine`, version `0.1.0`, Rust `1.97.1`, edition 2024;
- default Cargo feature set is empty;
- current engine dependencies already include `wgpu`, `winit`, `png`, `serde`, `serde_json`;
- native and WASM have target-specific dependencies;
- there is currently no dedicated UI Cargo feature and no separate UI crate.

Implication:

> "pay for what you use" cannot simply mean "one UI concept per crate".

For v1 it should primarily mean:

- no heavy new dependency required by the minimal UI path;
- optional external-dependency capabilities are feature-gated;
- the mental/API surface can be consumed incrementally;
- later physical crate separation remains possible if dependency boundaries become clean.

---

# 3. Existing public surface

Provenance:

```text
repo: gotoo77/gotoo-pixel-engine
ref: 6ff4f8baddae269baa6a7d182f0ba0c9d985f886
file: src/lib.rs
```

GPE already exports the primitives a UI system should reuse rather than duplicate:

```text
Framebuffer
Font
TextRenderer
Image
ImageFit
ImageFilter
Pixel
Rect
Size
Viewport

Input
Key
MouseButton
Touch
TouchPhase
GamepadButton

ActionId
ControlBinding
ControlMap

Frame
ToolFrame
Game
```

`pub mod ui` already establishes a public UI namespace.

**Classification: KEEP.**

---

# 4. Existing `src/ui` inventory

## 4.1 `UiTheme`

Provenance:

```text
file: src/ui/toolkit.rs
symbol: UiTheme
```

Current responsibilities:

- font;
- text scale;
- padding;
- row height/spacing;
- text/muted/control/border/accent colors.

Strengths:

- tiny;
- copied by value;
- deterministic;
- no external style engine.

Limitations relative to strategic ambition:

- one flat theme structure;
- no component styles;
- no composition/inheritance model;
- no local style tokens;
- no image/nine-slice/sprite skinning;
- not suited to highly custom visual systems without bypassing the toolkit.

Classification:

```text
GENERALIZE
```

Do not delete it. It is an obvious compatibility source for a default token/style set.

## 4.2 `UiResponse`

Provenance:

```text
file: src/ui/toolkit.rs
symbol: UiResponse
```

Fields:

```text
focused
hovered
active
clicked
changed
```

Strength:

- ergonomic immediate response;
- easy consumer code.

Strategic limitation:

- response is tied to the current single-pass immediate call;
- no semantic action identity;
- no output queue/trace;
- no propagation/consumption model;
- difficult to reconcile with a future multi-pass frame graph without either changing timing or adding an adapter.

Classification:

```text
WRAP / RETHINK
```

The *concept* of a small interaction result is good. The timing contract requires MFE validation.

## 4.3 `UiState`

Provenance:

```text
file: src/ui/toolkit.rs
symbol: UiState
```

Current retained state:

```text
focused ordinal
previous interactive count
pointer-active ordinal
repeat owner
left/right repeat state
scroll_y
previous content height
```

The type explicitly documents ordinal identity and requires `reset_interaction()` before intentional structural changes.

Strengths:

- very small;
- clear ownership;
- no hidden global;
- tested;
- interaction state is already separated from product/gameplay state.

Observed limitation:

> identity is ordinal and therefore structural changes can rebind interaction state unless the consumer explicitly resets it.

Gate 0 proved this is a real contract limitation, not proof that current UI is broken.

Classification:

```text
RETHINK INTERNALS
KEEP EXPLICIT STATE OWNERSHIP
WRAP FOR MIGRATION
```

## 4.4 `Ui`

Provenance:

```text
file: src/ui/toolkit.rs
symbol: Ui
```

Current shape:

```text
&mut Framebuffer
&Input
delta_time
&mut UiState
UiTheme
TextRenderer
vertical logical cursor
scroll state
interactive ordinal count
```

Current flow:

- constructor processes Up/Down focus from physical keyboard;
- each widget obtains the next vertical row;
- each interactive obtains the next ordinal;
- interaction is resolved while widget call executes;
- drawing is immediate into `Framebuffer`;
- `Drop` finalizes scroll and interactive count.

Strengths:

- exceptionally direct Rust ergonomics;
- consumer owns gameplay values;
- framebuffer integration is trivial;
- no widget object hierarchy;
- headless tests are practical;
- little conceptual overhead.

Observed/strategic limitations:

1. root layout is essentially one vertical column;
2. physical keyboard/mouse is consumed directly, while other GPE paths already have `ActionId`/`ControlMap`/`VirtualPad`;
3. ordinal identity cannot naturally survive dynamic insertion/removal;
4. measure/layout/interaction/paint happen together, limiting sophisticated same-frame responsive/intrinsic layout;
5. extension points are mostly "draw outside Ui", not a first-class custom component protocol.

Classification:

```text
KEEP AS MIGRATION BASELINE
DO NOT REWRITE IN PLACE BLINDLY
INTRODUCE NEW INTERNAL MODEL ALONGSIDE IT
```

## 4.5 Current controls

Observed in `toolkit.rs`:

```text
label
section
tabs
button
toggle
select
slider_f32
scroll behavior
keyboard focus
mouse hover/capture
repeat
```

Classification:

```text
SEMANTICS: KEEP/GENERALIZE
CURRENT IMPLEMENTATION: COMPATIBILITY BASELINE
```

## 4.6 `RepeatConfig` / `RepeatState`

Provenance:

```text
file: src/ui/toolkit.rs
symbols: RepeatConfig, RepeatState
```

The repeat implementation explicitly returns logical pulse counts and handles long frames.

This is a useful deterministic mechanism independent of the current visual structure.

Classification:

```text
KEEP
```

## 4.7 `MenuState` and menu helpers

Provenance:

```text
file: src/ui/mod.rs
symbols:
- MenuState
- standard_menu_controls
- menu_up_pressed
- menu_down_pressed
- menu_confirm_pressed
- draw_panel
- draw_text_centered
- draw_menu_item
```

`standard_menu_controls` already bridges keyboard and gamepad through `ControlMap`.

This is important evidence:

> semantic input is not alien to GPE. It already exists in a parallel UI path.

Classification:

```text
MenuState: KEEP as small utility / compatibility
ControlMap-based menu semantics: GENERALIZE INTO NEW INPUT ADAPTER
drawing helpers: KEEP low-level or legacy
```

## 4.8 `VirtualPad`

Provenance:

```text
file: src/ui/virtual_pad.rs
symbols:
- VirtualPad
- VirtualButton
- VirtualPadUpdate
```

Important properties:

- maps raw touch contacts into `ActionId`;
- keeps contact IDs;
- preserves ordered action-entry events within a frame;
- feeds virtual held states into `ControlMap`;
- tests touch press/release, retargeting, multi-contact, ordered moves.

Classification:

```text
KEEP
```

Strategic conclusion:

> GPE.UI should integrate with this semantic-input direction rather than inventing an unrelated touch abstraction for navigation.

It may still need direct pointer/touch coordinates for spatial widget interaction.

## 4.9 `PauseGame`

Provenance:

```text
file: src/ui/pause.rs
symbol: PauseGame
```

Important evidence:

- semantic `ActionId` values for pause/up/down/confirm;
- standard menu controls combine keyboard/gamepad;
- `VirtualPad` adds touch;
- explicit resume gate prevents input leaking into child game;
- product state remains in the wrapper.

Classification:

```text
KEEP
MIGRATE LATER AS A CONSUMER PROBE
```

The input-leakage handling is a strong design probe for future modal/focus/event semantics.

---

# 5. Real consumer evidence

## Tool window probe

Provenance:

```text
repo: gotoo77/gotoo-pixel-engine
ref: 6ff4f8baddae269baa6a7d182f0ba0c9d985f886
file: examples/tool_window_probe.rs
symbols:
- ToolWindowProbe
- Game::update_tool_window
```

The probe uses current `Ui` for a realistic settings/debug surface:

```text
tabs
sections
toggles
selects
sliders
scroll overflow
button
```

Consumer-owned state remains explicit.

It also demonstrates the structural-reset contract:

```text
requested tab
→ drop Ui
→ reset_interaction()
→ change selected_tab
```

Observed requirement:

> preserve or improve this level of directness and testability.

## Arcade

Existing Gate 1/2 and the A3 closure established:

- Arcade's current A3 can be implemented locally;
- no generic GPE UI gap is mandatory for A3;
- stable `ArcadeGameId` already exists;
- current logical surface policy is deterministic.

This does **not** remove responsive grid from GPE.UI's strategic requirements.

Instead Arcade becomes a high-value design probe for the reusable capability.

---

# 6. Strategic requirements

The following are intentional product requirements.

| Requirement | Class | Why it belongs |
|---|---|---|
| Small consumer path | STRATEGIC | GPE targets simple pixel games as well as richer games. |
| Composition | STRATEGIC | Reusable complex UI should arise from small primitives. |
| Responsive layout | STRATEGIC | GPE should support multiple logical surfaces and UI scales without local geometry boilerplate everywhere. |
| Pixel-aware geometry | STRATEGIC + OBSERVED | Engine is framebuffer/pixel-centric. |
| Multi-input | STRATEGIC + OBSERVED | Keyboard/gamepad/mouse/touch all already exist in GPE. |
| Semantic navigation | STRATEGIC + OBSERVED | `ActionId`/`ControlMap`/`VirtualPad` already demonstrate the direction. |
| Stable interaction identity where needed | STRATEGIC + OBSERVED pressure | Current ordinal contract is intentionally limited. |
| High customization | STRATEGIC | UI must support distinct game art directions, not only generic debug panels. |
| Custom widget escape hatch | STRATEGIC | Diablo-like inventory/orbs/radial selectors cannot depend on a closed widget catalog. |
| Headless testing | STRATEGIC + OBSERVED | Existing UI already has strong headless unit-test characteristics worth preserving. |
| Determinism | STRATEGIC + OBSERVED | Current engine and repeat/input paths are deterministic by design. |
| Game feedback hooks | STRATEGIC | UI feel should compose audio/haptic/animation without hardwiring them into widgets. |
| Migration from current `src/ui` | STRATEGIC + OBSERVED | Existing consumers must not be rewritten gratuitously. |
| Future declarative frontend compatibility | STRATEGIC | Authoring ergonomics are intentionally part of the long-term vision. |

---

# 7. Speculative features

These may be explored as prior art pressure but do not belong to v1 without new evidence:

```text
JavaScript
full DOM
full HTML compatibility
full CSS cascade/selectors
full SVG runtime including animation/script
remote UI content
general reactive runtime
browser-grade accessibility implementation
browser-grade shaping/text stack
world-space UI as a core requirement
```

---

# 8. Principles verdict

| Principle | Verdict | Note |
|---|---|---|
| Composition over specialization | KEEP | Central. |
| Policy != mechanism | KEEP | Especially for gameplay/audio/navigation policy. |
| Semantic input | KEEP | Build on `ControlMap`; preserve pointer coordinates separately. |
| Semantic output | KEEP | Needed for tracing, feedback and multi-pass architecture. |
| One gameplay source of truth | KEEP | UI state != gameplay state. |
| Deterministic by default | KEEP | Integer geometry and trace tests reinforce it. |
| Headless first | KEEP | Architectural requirement. |
| Pay only for what you use | REVISE | Primarily dependency/feature/module cost in same crate at first, not forced crate splitting. |
| Rust API first-class | KEEP | Markup must compile/feed same model. |
| Pixel-aware, not imprisoned | KEEP | Integer final geometry; future richer text/vector can exist. |
| No hidden globals | KEEP | Explicit `UiState` ownership is already a strength. |
| Escape hatches | KEEP | First-class custom paint/component path required. |
| Feedback composable | KEEP | Events/actions feed optional feedback layer. |
| Migration over rewrite | KEEP | Mandatory. |
| Testability is architecture | KEEP | Mandatory. |
| Strategic capability legitimacy | KEEP | Prevents "current consumer prison". |
| Capability != kernel machinery | KEEP | Keeps ambitious features out of the minimal core. |

---

# 9. Baseline conclusion

```text
EXISTING UI BROKEN:
NO

EXISTING UI SUFFICIENT FOR FULL STRATEGIC VISION:
NO

REWRITE REQUIRED:
NO

EVOLUTION / NEW INTERNAL MODEL WORTH RESEARCHING:
YES
```

The existing toolkit is a good **T1 baseline**, not the final strategic architecture.
