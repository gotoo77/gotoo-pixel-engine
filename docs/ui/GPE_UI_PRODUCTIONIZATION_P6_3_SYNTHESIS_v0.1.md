# GPE.UI — P6.3 Two-Consumer Synthesis v0.1

Status: **PASS / STOP — P6 CLOSED, P7 AUTHORIZED**

## Scope

P6.3 synthesizes the two unlike real-consumer proofs completed during P6:

```text
P6.1  Arcade high-resolution launcher
P6.2  Breakout game-over linear menu
```

The purpose is not to invent more features. It is to separate what is genuinely reusable from what was consumer-specific, decide which abstractions are ready for P7 review, and explicitly reject premature generalization.

P6 ends with this document.

---

# 1. What the two consumers proved together

Arcade and Breakout are structurally unlike:

```text
Arcade
- 3x2 spatial cards
- search / fuzzy matching
- category filters
- high-resolution outline typography
- mouse + keyboard + multi-gamepad
- launcher -> child-game lifecycle
- low-res gameplay hosted inside high-res presentation

Breakout
- 2-item linear menu
- gameplay -> game-over transition
- REPLAY / QUIT semantic dispatch
- low-resolution pixel presentation
- keyboard + gamepad + existing touch action path
- no search, cards, high-res typography or launcher composition
```

Despite this contrast, both consumers expose the same reusable foundations:

```text
stable semantic identity
persistent interaction state
one focus authority
semantic navigation input
activation -> consumer-owned domain action
input-family convergence
state-dependent visual styling
layout separated from semantic action
consumer-owned game/business state
```

This is the strongest P6 result.

GPE.UI should not become a catalog of per-game menu helpers. It should provide one coherent interaction/composition foundation plus a small number of first-class higher-level components justified by real use.

---

# 2. Classification of current abstractions

## 2.1 PROMOTE CANDIDATE — proven concepts, naming still reviewed in P7

### Stable identity + persistent UI state

Evidence:

- Arcade requires stable game-card identity across filtering/rebuilds.
- Breakout requires stable REPLAY / QUIT identity across redraws.

Direction:

```text
UiId concept       PROMOTE CANDIDATE
UiStateStore concept PROMOTE CANDIDATE
```

The exact public names remain a P7.1 decision.

### Semantic navigation / activation

Both consumers need navigation independent of physical device details.

Direction:

```text
UiNavInput-like semantic navigation  PROMOTE CANDIDATE
single interaction/focus authority   PROMOTE CANDIDATE
semantic activated action/result     PROMOTE CANDIDATE
```

Device-specific translation should become easier, but the semantic boundary itself is validated.

### Style state resolution

Arcade proves meaningful normal/focused/hovered/active styling.
Breakout confirms focus state is a generic menu concern even though its legacy pixel renderer remains simple.

Direction:

```text
component state styling  PROMOTE CANDIDATE
consumer-defined themes  PROMOTE CANDIDATE
```

Do not promote an Arcade-specific visual theme.

### Layout primitives

Responsive grid layout is proven by Arcade.
Linear/column ordering is proven by Breakout.

Direction:

```text
row / column / grid composition  PROMOTE CANDIDATE
responsive layout primitives     PROMOTE CANDIDATE
```

Do not promote screen-specific `ArcadeLayout`-style geometry.

### Pixel/high-resolution presentation boundary

Arcade proves the need for low-resolution game pixels inside a high-resolution host.
Breakout proves why this must become reusable across games rather than stay an Arcade trick.

Direction:

```text
PixelPresentation concept       PROMOTE CANDIDATE
integer-nearest presentation    PROMOTE CANDIDATE
high-res host / low-res game separation PROMOTE CANDIDATE
```

The final public abstraction is not yet frozen.

---

# 3. KEEP INTERNAL / PROVISIONAL

## Experimental transactional builder names

The current `experimental` module proved useful semantics, but its namespace and exact builder API are not P6 deliverables.

```text
experimental::* naming  KEEP INTERNAL / RENAME REVIEW
```

P7 should promote concepts, not accidentally freeze experiment-era names.

## PixelGameHost

`PixelGameHost` successfully removed child `Frame` / framebuffer / viewport plumbing from Arcade and passed runtime validation.

However:

- pointer/touch mapping remains outside its validated contract;
- overlay/menu composition is not yet integrated;
- the future Presentation Shell may subsume or wrap it.

Verdict:

```text
PixelGameHost  KEEP PROVISIONAL
```

Do not delete it; do not freeze it as the final shell API yet.

## Breakout EndMenuState

`EndMenuState` is valuable evidence but is explicitly consumer-local.

Verdict:

```text
EndMenuState  DO NOT PROMOTE
```

Its successful behavior feeds the first-class Menu System design instead.

## Spatial/card compatibility adapter

Arcade proves card/grid utility, but Breakout confirms not every menu is spatial/card-shaped.

Verdict:

```text
card/grid adapter  KEEP AS SPECIALIZED COMPOSITION PRIMITIVE
```

Do not make it the universal UI abstraction.

---

# 4. KEEP COMPATIBILITY

The following remain valid and should not be removed merely because P7 begins:

```text
legacy MenuState
legacy draw_menu_item / direct framebuffer UI helpers
single-framebuffer games
direct pixel HUD drawing
PauseGame compatibility
ControlMap / VirtualPad
```

Reason:

P6 proves a better future architecture, not that every legacy helper has an immediate safe replacement.

Deprecation belongs to P7.8 only after tutorial/reference consumers prove replacement coverage.

---

# 5. FOLLOW-UP REQUIRED IN P7

## 5.1 First-class Menu System

P6 proves that menus are not incidental glue.

Required P7 direction is governed by:

```text
GPE_UI_MENU_SYSTEM_CHARTER_v0.1.md
```

Minimum architecture must cover:

```text
simple item declaration
stable semantic identity
focus/navigation/activation
keyboard/gamepad/mouse/touch
back/cancel
submenus
focus restoration
settings widgets
style/theme customization
custom content/extensibility
pixel + high-res presentation
pause / game-over / settings / debug use cases
```

The target is not `EndMenuState` in core. The target is a coherent menu/widget abstraction that can express Breakout-simple and Arcade-rich consumers without architectural forks.

## 5.2 Game Presentation Shell

P6 proves a system-wide discontinuity between modern high-res UI and legacy low-res games.

Required P7 direction is governed by:

```text
GPE_GAME_PRESENTATION_SHELL_CHARTER_v0.1.md
```

Minimum responsibilities:

```text
low-res game surface ownership
high-res host presentation
integer-nearest fit
letterbox/pillarbox policy
explicit pointer/touch mapping
overlay ordering
Menu System composition
optional high-res HUD/debug UI
consumer-defined framing/art direction
```

The shell must remain opt-in and impose no mandatory cost on ordinary games.

## 5.3 Pointer/touch mapping

This is the most important correctness gap left by P6.1.

Required contract:

```text
host-space pointer/touch
        -> resolved presentation geometry
        -> mapped game-space coordinate or None
```

Keyboard/gamepad semantic state must remain unchanged.

This must be solved before a high-res hosted Web/Touch game is declared production-ready.

## 5.4 Input-to-navigation ergonomics

Both consumers contain mechanical translation from engine input/actions into UI semantic navigation.

P7 should determine whether a small shared adapter is justified for common cases such as:

```text
keyboard arrows / WASD
gamepad D-pad / left stick
South/A confirm
East/B back
ControlMap semantic actions
```

The adapter must be configurable and must not hide game-specific bindings.

## 5.5 Typography ergonomics

Arcade repeatedly needs measure + shrink-to-fit + center + draw for outline text.

Breakout does not independently repeat this need, so P6 does not justify a large typography framework.

P7 may introduce one small reusable fit/alignment helper as part of a coherent text API.

## 5.6 Widget/settings/debug coverage

Before freeze, P7 must prove unlike interactive controls beyond buttons:

```text
toggle
slider
submenu
mixed debug/tool controls
```

These should reuse the same identity/focus/interaction authority, not introduce parallel widget kernels.

---

# 6. Explicitly rejected generalizations

P6 evidence does NOT justify:

```text
global UI manager
mandatory retained scene tree
generic launcher DSL
SearchableFilterableGrid in core
Arcade-specific widgets
NeonArcadeTheme in core
Breakout-specific menu class
general GPU compositor/render graph
mandatory high-res conversion of all games
one universal GPE visual style
parallel focus/navigation systems by widget family
```

Search semantics, category taxonomy, game artwork and visual identity remain consumer-owned unless later consumers independently prove generic mechanics.

---

# 7. P7 architecture model

The combined direction is:

```text
GPE
├─ Game runtime / Frame / Input / Audio / Storage
├─ Pixel presentation primitives
├─ optional Game Presentation Shell
│  ├─ low-res Game surface
│  ├─ input mapping
│  └─ high-res presentation regions
└─ GPE.UI
   ├─ stable identity + state
   ├─ one interaction/focus kernel
   ├─ semantic navigation
   ├─ layout primitives
   ├─ style/theme resolution
   ├─ text/images
   ├─ first-class Menu System
   └─ extensible widgets / debug tooling
```

Important separation:

```text
Presentation Shell = where/how game pixels and host UI coexist
Menu System        = menu/widget content + interaction structure
GPE.UI kernel      = shared identity/focus/interaction/layout/style foundations
consumer           = domain state, actions, data, art direction
```

No layer should absorb the responsibilities of the others.

---

# 8. P7 proof scenarios required before API freeze

At minimum:

```text
A. Breakout-style 2-item game-over menu
B. pause menu with nested Settings
C. Settings with slider + toggle + submenu
D. mixed debug/tool overlay
E. Arcade rich launcher / visual showcase
F. low-res game inside high-res Presentation Shell
G. pointer/touch mapping through hosted game surface
H. Native + wasm32 evidence
```

The same underlying identity/interaction architecture must survive all scenarios.

---

# 9. P6 final verdict

```text
P6.1 Arcade consumer proof     PASS / STOP
P6.2 Breakout consumer proof   PASS / STOP
P6.3 synthesis                 PASS / STOP
```

P6 has done its job:

- real consumers exposed abstraction gaps;
- one gap (`PixelGameHost`) was reduced with a narrow helper;
- unlike consumers validated the common interaction foundations;
- Menu System and Presentation Shell emerged from evidence rather than speculation;
- premature generalized DSLs/frameworks were rejected.

Overall:

> **P6 = PASS / CLOSED**

---

# 10. Authorization

P7 is now authorized.

The first P7 work must be architecture consolidation, not visual polish.

Start with:

```text
P7.1  API consolidation / naming / ownership boundaries
P7.2  first-class Menu System architecture
P7.3  widget composition / styling / theming / extensibility
P7.4  Presentation Shell + typography/high-res/input mapping
P7.5  Arcade Visual Showcase
P7.6  Pause / Settings / Debug reference consumers
P7.7  tutorial / teaching gate
P7.8  compatibility / deprecation decisions
```

No repository-wide migration is authorized by this synthesis.
