# GPE Touch Controls — Specification v0.3

**Status:** Draft for approval  
**Supersedes:** `GPE_TOUCH_CONTROLS_SPEC_v0.2.md`  
**Scope:** Gotoo Pixel Engine / GPE.ui  
**Target:** Web/mobile first, compatible with native GPE input  
**First consumer / validation target:** Void Canticle  

---

## 1. Objective

Provide GPE games with a reusable, declarative and runtime-safe touch-control system.

The engine must provide sensible presentation defaults while allowing each game to configure:

- the movement-control type,
- the visible action buttons,
- the action mapping,
- the placement and size of controls,
- the visual style,
- the effective touch hitboxes,
- the active touch-control configuration at runtime.

The design must avoid a hard-coded mobile controller layout that suits one game but not another.

The system must integrate with the existing GPE `Input` / `ControlMap` model without creating a parallel game-facing input API.

---

## 2. Architectural principle

A game declares:

```text
which controls exist?
which ActionId does each control drive?
which presentation overrides are needed?
```

GPE owns:

```text
layout resolution
rendering
touch ownership
hit-testing
gesture interpretation
touch-action aggregation
ControlMap forwarding
```

The touch layer must not know game semantics such as:

```text
Fire
Bomb
Jump
Rotate
Drop
Pause
```

Game-facing flow:

```text
touch input
    ↓
TouchControlsRuntime
    ↓
held ActionId set
    ↓
ControlMap
    ↓
game logic
```

Keyboard, physical gamepad and touch controls must converge on the same `ControlMap`.

---

## 3. Runtime configuration

A game may expose different touch configurations depending on current state.

Examples:

```text
main menu
gameplay
pause
game over
character selection
settings
```

GPE must support replacing the active touch-control configuration at runtime.

Example:

```rust
self.touch_controls = gameplay_touch_controls();
```

then:

```rust
self.touch_controls = pause_touch_controls();
```

GPE does not own the game's state machine.

It consumes the currently active `TouchControlsConfig`.

---

## 4. Configuration replacement lifecycle

Replacing the active touch configuration is a hard runtime boundary.

When the configuration changes:

```text
1. all current touch-control ownership is cleared
2. all touch-derived held actions from the previous config are released
3. active touch IDs from the previous config are quarantined
4. quarantined touch IDs remain ignored until Ended / Cancelled
5. only a new TouchPhase::Started may acquire ownership in the new config
```

This prevents an existing finger from accidentally activating a newly displayed control.

Example prevented:

```text
finger holds FIRE
        ↓
pause opens
        ↓
QUIT appears under same finger
        ↓
same finger must NOT activate QUIT
```

---

## 5. No semantic `TouchControlsConfig::default()`

GPE cannot invent game actions.

Therefore:

```rust
TouchControlsConfig::default()
```

must not imply a complete usable controller.

Defaults apply only to presentation-oriented components:

```rust
TouchControlsLayout::default()
TouchControlsStyle::default()
TouchVisibility::default()
```

A complete `TouchControlsConfig` must receive all required game `ActionId`s explicitly.

---

## 6. Movement-control modes

A game chooses one movement-control mode:

```rust
pub enum MovementControl {
    None,
    DPad(DPadConfig),
    VirtualStick(VirtualStickConfig),
}
```

The v0.3 design deliberately excludes generic trackpad / trackball semantics.

---

## 7. `MovementControl::None`

No movement control is rendered and no movement touch region exists.

Typical consumers:

```text
puzzle games
direct-touch games
menus
games using only action buttons
```

---

## 8. `MovementControl::DPad`

A D-pad emits directional `ActionId`s supplied by the game:

```text
UP
DOWN
LEFT
RIGHT
```

Required behavior:

- press,
- hold,
- release,
- slide between directions,
- simultaneous use with other controls,
- deterministic touch ownership.

### 8.1 Ownership

A D-pad captures one touch when that touch starts inside its capture region.

Ownership remains with the D-pad until:

```text
Ended
Cancelled
```

The touch does not transfer to another control while owned.

### 8.2 Direction resolution

Capture and direction state are separate.

A captured finger may produce:

```text
LEFT
UP
RIGHT
neutral
```

as it moves.

If the finger leaves all directional zones:

```text
ownership remains
direction becomes neutral
```

If it re-enters a direction before release:

```text
that direction becomes active again
```

---

## 9. `MovementControl::VirtualStick`

The virtual stick is a fixed-center thumb control.

It is not a floating joystick, generic trackpad or physical-model trackball.

Conceptual model:

```text
fixed stick center
        ↓
captured touch
        ↓
displacement (dx, dy)
        ↓
normalized displacement
        ↓
dead zone
        ↓
direction resolver
        ↓
ActionId states
```

### 9.1 Fixed center

The stick center is derived from resolved layout geometry.

The initial finger position does not redefine the center.

Floating-stick behavior is explicitly out of scope for v0.3.

### 9.2 Ownership

The virtual stick captures one touch when a `Started` event occurs inside its effective hit region.

The same touch remains owned until:

```text
Ended
Cancelled
```

even if the finger moves outside the visual or hit region.

### 9.3 Dead zone

Dead zone is expressed as a ratio of the stick radius:

```rust
pub dead_zone_ratio: f32
```

Valid range:

```text
0.0 <= dead_zone_ratio < 1.0
```

Example:

```text
0.20 = 20% of stick radius
```

This keeps the control feeling consistent when resized.

---

## 10. Direction modes

Movement controls must define how diagonals are handled.

```rust
pub enum DirectionMode {
    FourWay,
    EightWay,
}
```

### 10.1 `FourWay`

At most one directional action is active.

The dominant axis determines the direction.

Example:

```text
abs(dx) > abs(dy) -> LEFT or RIGHT
otherwise         -> UP or DOWN
```

Tie behavior must be deterministic.

### 10.2 `EightWay`

One horizontal and one vertical direction may be active simultaneously.

Examples:

```text
UP + RIGHT
DOWN + LEFT
```

Void Canticle should use:

```text
VirtualStick + EightWay
```

unless consumer validation demonstrates otherwise.

---

## 11. Touch ownership model

Touch ownership is a hard contract.

Each active touch contact has a unique `touch_id`.

Each control may own at most one touch.

Each touch may be owned by at most one control.

Therefore:

```text
one control ↔ zero or one touch
one touch   ↔ zero or one control
```

### 11.1 Duplicate touches on one control

If `A` already owns touch #1:

```text
touch #2 starts on A -> ignored
```

until touch #1 releases or is cancelled.

### 11.2 Multi-touch across controls

Valid:

```text
touch #1 -> VirtualStick
touch #2 -> A
touch #3 -> B
```

All active actions must coexist in `ControlMap`.

---

## 12. Action buttons

Standard face buttons:

```text
A
B
X
Y
```

Each button is independently optional.

```rust
pub struct FaceButtons {
    pub a: Option<TouchButtonConfig>,
    pub b: Option<TouchButtonConfig>,
    pub x: Option<TouchButtonConfig>,
    pub y: Option<TouchButtonConfig>,
}
```

If a game uses only A/B:

```text
A rendered
B rendered
X absent
Y absent
```

Absent controls create:

```text
no visual element
no hitbox
no touch ownership
```

Each visible button maps to an explicit game `ActionId`.

---

## 13. Action-button capture semantics

Action buttons capture a touch on:

```text
TouchPhase::Started inside effective hitbox
```

Default behavior:

```text
touch down inside button -> held
move outside button      -> remains held
release / cancel         -> released
```

A touch owned by a button must not retarget to another button while moving.

`cancel-on-exit` is out of scope for v0.3.

---

## 14. Navigation controls

Navigation controls are distinct from gameplay face buttons.

Initial required navigation control:

```text
BACK
```

`BACK` is optional and maps to a game-supplied `ActionId`.

Recommended default placement:

```text
top-right
```

Navigation controls are not part of the face-button group.

---

## 15. Layout model

The layout system uses:

```text
anchor
+ offset
+ visual size
+ hit padding
```

Suggested anchors:

```rust
pub enum TouchAnchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}
```

Controls are resolved in GPE logical framebuffer coordinates.

---

## 16. Placement geometry

Generic placement:

```rust
pub struct ControlPlacement {
    pub anchor: TouchAnchor,
    pub offset: (i32, i32),
    pub visual_size: Size,
    pub hit_padding: Insets,
}
```

Conceptual `Insets`:

```rust
pub struct Insets {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}
```

Effective hitbox:

```text
visual rect expanded by hit padding
```

This keeps visible geometry and interaction geometry related by construction.

---

## 17. One source of truth for geometry

Rendering and input must consume the same resolved geometry.

Required internal representation:

```rust
pub struct ResolvedTouchControls {
    // movement geometry
    // face-button geometry
    // navigation geometry
}
```

Flow:

```text
TouchControlsConfig
        ↓
LayoutResolver
        ↓
ResolvedTouchControls
        ├── renderer
        └── TouchControlsRuntime
```

Independent visual/input position calculations are forbidden.

---

## 18. Group layout and per-control overrides

Placement resolution order:

```text
group-generated placement
        ↓
optional individual override
        ↓
resolved placement
```

Rule:

```text
resolved_placement =
    individual_override
    ?? group_generated_placement
```

Only the resolved placement reaches rendering and input.

---

## 19. Face-button arrangement

v0.3 exposes only the arrangement actually required:

```rust
pub enum FaceButtonArrangement {
    Diamond,
}
```

Future variants such as:

```text
Row
Column
Arc
```

must only be added when implemented and required by a real consumer.

### 19.1 Diamond positions

Canonical semantic positions:

```text
    Y
X       B
    A
```

Missing buttons do not create interactive zones.

With only A/B:

```text
        B
    A
```

A game may override individual placements for a more compact arrangement.

---

## 20. Hitbox overlap policy

Sibling top-level control hitboxes must not overlap.

Examples of invalid overlap:

```text
A ↔ B
A ↔ VirtualStick
BACK ↔ B
DPad ↔ A
```

The layout resolver should report these in tests/debug validation.

Sub-zones inside one composite control are excluded from this rule.

Examples:

```text
internal DPad direction zones
internal VirtualStick geometry
```

There is no implicit runtime priority such as:

```text
first control wins
last control wins
```

Intentional overlap requires a future explicit arbitration policy.

---

## 21. Style

Layout and style are separate.

Suggested model:

```rust
pub struct TouchControlsStyle {
    pub face_button_shape: TouchButtonShape,
    pub opacity: f32,
    pub pressed_opacity: f32,
    pub border_width: u32,
    pub a_color: Pixel,
    pub b_color: Pixel,
    pub x_color: Pixel,
    pub y_color: Pixel,
}
```

Suggested shapes:

```rust
pub enum TouchButtonShape {
    Circle,
    RoundedRect,
}
```

Recommended default colors:

```text
A = green
B = red
X = blue
Y = yellow
```

These are presentation defaults, not semantic rules.

---

## 22. Visibility policy

```rust
pub enum TouchVisibility {
    Always,
    AfterFirstTouch,
    Hidden,
}
```

Recommended Web default:

```text
AfterFirstTouch
```

### 22.1 First-touch semantics

For `AfterFirstTouch`, the first `Started` event:

```text
1. makes controls visible
2. is processed by hit-testing in the same frame
3. may immediately acquire a control
```

No extra "wake-up tap" is required.

Visibility remains sticky for the current `TouchControlsRuntime` session unless explicitly reset.

### 22.2 Hidden

`Hidden` means:

```text
not rendered
not interactive
no touch ownership
```

---

## 23. Disabled state

A separate runtime-disabled visual state is out of scope for v0.3.

A control is:

```text
present
or
absent
```

Dynamic disabled-state semantics may be added later if justified by a consumer.

---

## 24. Orientation and resizing

Anchors resolve from current logical surface dimensions.

Controls must remain correctly positioned after:

```text
portrait/landscape change
browser fullscreen transition
viewport resize
mobile browser chrome change
framebuffer resize
```

No control may depend on startup-only resolved coordinates.

---

## 25. Portrait and landscape configuration

Anchors are not required to solve every responsive-layout case.

Games may provide separate configs:

```rust
let controls = if portrait {
    portrait_touch_controls()
} else {
    landscape_touch_controls()
};
```

A resulting config replacement follows the lifecycle rules in section 4.

No CSS-like responsive-layout engine is required.

---

## 26. Safe margins

Layout supports configurable safe margins.

```rust
TouchControlsLayout {
    safe_margin: Insets {
        left: 24,
        right: 24,
        top: 24,
        bottom: 24,
    },
}
```

This prepares for:

```text
rounded screens
browser overlays
notches
mobile safe areas
```

Native CSS safe-area integration is not required for v0.3.

---

## 27. Runtime architecture

v0.3 introduces one touch-runtime authority:

```text
ResolvedTouchControls
        +
Input.touches()
        +
TouchRuntimeState
        ↓
TouchControlsRuntime
        ├── action-button capture
        ├── navigation capture
        ├── DPad capture / direction
        └── VirtualStick capture / direction
        ↓
final held ActionId set
        ↓
ControlMap
```

`TouchControlsRuntime` is the single aggregator for touch-derived action state.

---

## 28. `ControlMap` integration

Current GPE `ControlMap` exposes one virtual-held boolean per `ActionId`.

Therefore v0.3 must not create multiple independent touch writers competing through:

```rust
ControlMap::set_virtual(action, held)
```

Instead:

```text
all touch primitives
        ↓
TouchControlsRuntime aggregation
        ↓
final unique held ActionId set
        ↓
ControlMap::set_virtual(...)
```

The runtime must compute the complete touch-held set before applying it.

---

## 29. Duplicate touch-action bindings

For v0.3, one `ActionId` must not be assigned to multiple distinct touch controls in the same active config.

Invalid example:

```text
A button      -> FIRE
B button      -> FIRE
```

or:

```text
VirtualStick LEFT -> MOVE_LEFT
separate button   -> MOVE_LEFT
```

unless a future explicit multi-source aggregation contract is introduced.

The config validator should detect this ambiguity.

Physical keyboard/gamepad bindings may still map to the same `ActionId` through `ControlMap`.

---

## 30. Relationship with existing `VirtualPad`

Existing `VirtualPad` has different movement semantics:

- touch may retarget between zones,
- leaving all zones can remove contact ownership.

Those semantics are useful to current consumers and must not be silently changed.

Therefore v0.3 does **not** require `TouchControlsRuntime` to be implemented by modifying `VirtualPad`.

Allowed implementation choices:

```text
reuse selected VirtualPad internals
refactor shared helpers
introduce new runtime logic
```

but only if existing `VirtualPad` behavior remains backward compatible.

`VirtualPad` compatibility is an implementation concern, not a public v0.3 touch-controls contract.

---

## 31. Pure layout resolver

Layout resolution should be pure and deterministic where possible.

Preferred contract:

```rust
resolve_touch_controls(
    config,
    framebuffer_size,
) -> Result<ResolvedTouchControls, TouchLayoutError>
```

Core tests include:

```text
BottomLeft resolves inside bounds
TopRight remains top-right after resize
A/B-only config has no X/Y geometry
visual rect is contained inside hit rect
invalid top-level overlaps are rejected
duplicate touch ActionId bindings are rejected
portrait and landscape configs resolve independently
```

---
