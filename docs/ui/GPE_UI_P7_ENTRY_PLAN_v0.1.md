# GPE.UI — P7 Entry Plan v0.1

Status: **AUTHORIZED / IMPLEMENTATION NOT STARTED**

P6 is closed by `GPE_UI_PRODUCTIONIZATION_P6_3_SYNTHESIS_v0.1.md`.

P7 begins with consolidation and proof-oriented architecture work. It must not start by polishing Arcade visuals or migrating many games.

## P7 mission

Turn the concepts proven by P6 into a coherent, teachable, customizable game UI toolkit without freezing weak experiment-era APIs.

The governing product direction is:

> simple by default, deep when needed, visually unconstrained, and consistent across games and tools.

The two first-class P7 axes are:

```text
1. GPE.UI Menu System
2. GPE Game Presentation Shell
```

They share the same GPE.UI foundations but remain separate responsibilities.

---

# P7.1 — API consolidation / naming / ownership

## Goal

Audit the current surface and classify each relevant API before adding higher-level features.

Mandatory review set:

```text
UiId
UiStateStore
UiNavInput
UiInput / pointer/touch snapshots
experimental transactional builder
interaction kernel
row / column / grid layout
spatial/card adapters
UiTheme / UiStyleSheet / UiComponentStyle
outline typography
PixelPresentation
present_pixel_surface(...)
PixelGameHost
MenuState / draw_menu_item compatibility
PauseGame interaction with future menus/shell
```

For each item decide:

```text
PROMOTE
RENAME
MERGE
KEEP INTERNAL
KEEP PROVISIONAL
KEEP COMPATIBILITY
DEPRECATE LATER
REMOVE
```

## Hard constraints

P7.1 must preserve:

```text
one focus/interaction authority
consumer-owned domain state
no global UI manager
no mandatory retained scene tree
no renderer rewrite
single-framebuffer compatibility
```

## Deliverable

```text
GPE_UI_P7_1_API_CONSOLIDATION_RESULT_v0.1.md
```

No broad code rename should occur before the classification is reviewed.

---

# P7.2 — First-class Menu System architecture

Governed by `GPE_UI_MENU_SYSTEM_CHARTER_v0.1.md`.

## First proof target

Create the smallest reusable Menu API that can replace the consumer-local Breakout `EndMenuState` without losing extensibility.

The first slice should prove:

```text
menu identity
item identity
semantic actions
initial focus
linear navigation
confirm / back
consumer-owned dispatch
style hooks
keyboard + gamepad
```

It must be possible to express a tiny game-over menu concisely.

## Second proof target

Immediately stress the same abstraction with:

```text
Pause
├─ Resume
├─ Settings -> submenu
└─ Quit
```

Required proof:

```text
submenu push/pop
focus restoration
back/cancel
no hand-written per-game menu state machine
```

Do not add every widget yet.

---

# P7.3 — Widgets / styling / theming / extensibility

Extend the Menu System only after P7.2 proves its core.

Minimum next widgets:

```text
button/item
toggle
slider
label/text
custom composition escape hatch
```

Then prove a Settings page:

```text
Music volume slider
Fullscreen-like toggle or equivalent harmless setting
Controls submenu placeholder/real consumer
Back
```

Requirements:

```text
same UiId/state/focus authority
state-specific style resolution
consumer-defined theme
custom visual composition possible
no closed god-enum architecture
```

Debug/tool UI should use the same widget foundation.

---

# P7.4 — Presentation Shell / high-resolution boundary

Governed by `GPE_GAME_PRESENTATION_SHELL_CHARTER_v0.1.md`.

## First mandatory technical gap

Implement first-class host -> game pointer/touch coordinate mapping.

Acceptance:

```text
inside presented game rect -> mapped coordinate
letterbox/frame area -> None
Touch id/phase preserved
keyboard/gamepad unchanged
```

## Shell proof

Host Breakout while preserving its pixel gameplay and adding high-resolution GPE.UI around/above it.

The consumer must not manually:

```text
construct child Frame
own nested framebuffer plumbing
calculate integer presentation geometry
map pointer/touch
forward storage/audio/delta time
```

Determine whether `PixelGameHost` becomes:

```text
public primitive
internal mechanism
or component wrapped/subsumed by GamePresentationShell
```

## Typography

Fold outline text fitting/alignment into a small coherent text API if still justified.

---

# P7.5 — Arcade Visual Showcase

Only after Menu + Shell foundations are credible.

Use Arcade as a stress test for visual range:

```text
thumbnails/runtime captures
strong header composition
per-game accents
crisp typography
responsive cards
search/categories
polished focus/hover/active states
optional scanlines/glow/vignette
optional arcade/CRT framing
```

Arcade art remains consumer-owned.

The core should expose only generic primitives independently justified by the other P7 proofs.

---

# P7.6 — Pause / Settings / Debug reference consumers

The same architecture must prove three unlike uses:

```text
Pause menu
Settings page/submenus
Debug/tool overlay
```

The Debug case should include mixed controls such as:

```text
label/status
button
toggle
slider
```

This is the main extensibility proof before freeze.

---

# P7.7 — Tutorial / teaching gate

Required tutorials/examples:

```text
1. basic button + focus + action
2. simple Menu
3. Pause -> Settings submenu
4. toggle + slider
5. custom theme/style
6. responsive row/grid/card composition
7. keyboard + multi-gamepad + pointer/touch
8. low-res game inside high-res Presentation Shell
9. debug overlay
10. Arcade reference walkthrough
```

If these tutorials require pages of unexplained plumbing, P7 is not ready to freeze.

---

# P7.8 — Compatibility / deprecation

Only after the reference scenarios and tutorials pass:

```text
review MenuState
draw_menu_item and legacy helpers
experimental namespaces
compatibility spatial adapters
PixelGameHost naming/status
old Arcade path
```

Deprecate only when replacement coverage is demonstrated.

---

# Execution order

P7 should proceed with explicit STOP gates:

```text
P7.1 classification        -> STOP / review
P7.2 Menu core             -> STOP / Breakout + Pause proof
P7.3 widgets/settings      -> STOP / settings + debug proof
P7.4 Presentation Shell    -> STOP / hosted Breakout + input mapping proof
P7.5 Arcade showcase       -> STOP / visual + DX audit
P7.6 reference consumers   -> STOP / cross-consumer synthesis
P7.7 tutorials             -> STOP / teaching audit
P7.8 compatibility         -> final productionization decision
```

Do not skip STOP gates merely because a later visual goal is attractive.

---

# Immediate next action

Start **P7.1 only**.

Do not implement the final Menu DSL or Presentation Shell before the existing API surface is classified.
