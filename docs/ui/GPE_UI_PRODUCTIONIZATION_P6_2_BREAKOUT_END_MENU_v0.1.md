# GPE.UI — P6.2 Breakout End Menu v0.1

Status: **DEFINED / IMPLEMENTATION NOT STARTED**

## Mission

Migrate one second real consumer that is deliberately unlike the Arcade launcher.

The selected consumer is the **Breakout game-over / end menu only**:

```text
GAME OVER
SCORE / LEVEL
REPLAY
QUIT
```

This slice is intentionally small. It must test whether the converged UI path is also healthy for a linear in-game transition menu, not only for a spatial launcher/grid.

P6.2 must not copy Arcade-specific composition, search/filter concepts, high-resolution launcher structure or visual theming.

## Why Breakout

Breakout is the preferred P6.2 consumer because it provides evidence that Arcade cannot:

```text
real gameplay -> end-state UI transition
2-item linear menu
keyboard + gamepad navigation
existing touch mode via VirtualPad / ACTION
restart-vs-exit semantic dispatch
low-resolution pixel framebuffer
menu rendered over gameplay state
```

It is structurally unlike Arcade while remaining small enough to review atomically.

## Current consumer path

Primary file:

```text
examples/breakout/game.rs
```

Current end-menu state:

```rust
end_menu: MenuState
```

Current Lost-state interaction:

```text
menu_up_pressed(...) / MOVE_LEFT
menu_down_pressed(...) / MOVE_RIGHT
menu_confirm_pressed(...) / ACTION
```

Current semantic dispatch:

```text
selected 0 -> restart_round()
selected 1 -> GameResult::Exit
```

Current rendering uses:

```text
draw_panel
draw_text_centered
draw_menu_item
```

The gameplay model, ball/paddle/bricks, audio, score/lives/level and touch gameplay controls are out of scope.

## What P6.2 must prove

P6.2 is not a visual-showcase task. It must answer whether the new GPE.UI interaction/state/style path can express a tiny linear menu with **less or equal conceptual complexity** than the legacy `MenuState` path.

Required evidence:

```text
stable semantic identity for REPLAY / QUIT
initial focus deterministic on REPLAY
up/down or equivalent linear navigation
keyboard activation
multi-gamepad activation/navigation
existing touch ACTION compatibility preserved
REPLAY still calls restart_round()
QUIT still returns GameResult::Exit
no gameplay-state ownership moves into GPE.UI
no global UI manager
no Arcade-only helper copied into Breakout
```

## Compatibility boundary

The following must remain unchanged unless direct P6.2 evidence proves otherwise:

```text
Breakout gameplay controls
ControlMap
VirtualPad
ACTION / MOVE_LEFT / MOVE_RIGHT semantics
sound behavior
touch panel layout
gameplay HUD
restart_round() implementation
GameResult::Exit boundary
```

The legacy helpers remain available to other consumers. P6.2 does not authorize deleting `MenuState` or global compatibility cleanup.

## Target interaction shape

Prefer the smallest converged linear-menu representation available from the existing Architecture B kernel.

The target should conceptually be:

```text
persistent UI state owned by Breakout
stable keyed REPLAY / QUIT items
UiNavInput from existing Breakout actions/input
semantic activated item
consumer dispatches restart / exit
```

Do **not** force Breakout through the Arcade card-grid API merely because Arcade used it. If the current converged surface lacks an ergonomic linear-menu adapter, that is evidence to record.

## Rendering scope

Keep the current low-resolution Breakout presentation.

P6.2 does not require:

```text
high-resolution host
outline fonts
PixelGameHost
thumbnails
search/filter
new post effects
```

The goal is to test the interaction/composition abstraction on a different screen shape, not to upgrade Breakout art direction.

Shared style semantics may be used for focused/normal item state if they reduce consumer-specific branching without adding machinery.

## Touch boundary

Breakout Touch currently drives existing actions through `VirtualPad`.

P6.2 should preserve that model unless a very thin translation to the converged navigation vocabulary is sufficient.

A valid solution may keep:

```text
VirtualPad -> ControlMap ACTION/MOVE_* -> UiNavInput
```

Do not rewrite touch transport as collateral work.

Touch acceptance requires that the existing ACTION path can still activate the selected end-menu item and that the user can still change selection through the controls available in the current touch layout.

If current touch behavior itself exposes an ambiguity, document it rather than silently redesigning touch controls inside this slice.

## Tests required

At minimum add/adapt deterministic tests for:

```text
Lost state starts with REPLAY focused
navigation reaches QUIT and wraps/clamps according to the selected contract
REPLAY activation resets round state / score / lives / level as before
QUIT activation returns GameResult::Exit
stable identity survives Lost-state redraw/rebuild
keyboard navigation semantics
multi-gamepad semantic input path
Touch ACTION activation compatibility
```

Where direct platform input synthesis is not public from example tests, test the semantic/headless layer rather than adding test-only public engine API.

Existing gameplay physics/audio tests should remain semantically unchanged.

## Human runtime gate

Native:

```text
force/reach GAME OVER
REPLAY initially focused
keyboard navigation works
D-pad / gamepad navigation works
A/South activation works
REPLAY restarts cleanly
QUIT exits/returns through the existing wrapper boundary
visual focus is obvious and unclipped
```

Touch/Web where applicable:

```text
existing touch controls still function
end-menu selection/activation remains usable
browser build remains valid
```

If a platform/input family is unavailable in the validation environment, record it explicitly rather than marking it PASS by assumption.

## DX audit embedded in P6.2

Because this is the second unlike consumer, implementation review must compare it directly with Arcade.

Classify any repeated friction:

```text
A. truly shared UI responsibility
B. generic input/navigation glue repeated in both consumers
C. typography/style boilerplate repeated in both consumers
D. presentation concerns unique to Arcade
E. gameplay-specific logic unique to Breakout
```

A helper becomes a P7 candidate only if the repetition is real across unlike consumers.

Particular questions:

```text
Does linear navigation need a small shared adapter?
Does stable keyed item creation still require too much boilerplate?
Does action -> UiNavInput translation repeat Arcade mechanically?
Does style resolution reduce code compared with draw_menu_item(selected)?
Is the resulting Breakout code easier to teach than MenuState?
```

## Explicit non-goals

P6.2 does not authorize:

```text
migrating Breakout gameplay HUD
migrating touch gameplay controls wholesale
high-resolution Breakout conversion
outline-font adoption for Breakout
new generic menu scene framework
new launcher DSL
search/filter abstractions
removing MenuState globally
rewriting PauseGame
rewriting ControlMap
bulk migration of Pong/Space Invaders
```

## Rollback boundary

A revert of P6.2 must restore only the Breakout end menu:

```text
end_menu: MenuState
legacy Lost-state navigation
legacy draw_menu_item rendering
```

without reverting P6.1, `PixelGameHost`, presentation primitives or unrelated Breakout gameplay.

If implementation cannot preserve this rollback boundary, stop and reduce scope.

## PASS criteria

P6.2 is PASS only when:

```text
[ ] only Breakout end-menu responsibility is migrated
[ ] REPLAY / QUIT semantics unchanged
[ ] deterministic initial focus
[ ] keyboard navigation/activation PASS
[ ] multi-gamepad navigation/activation PASS
[ ] touch compatibility PASS where applicable
[ ] Native runtime gate PASS
[ ] wasm32 compile gate PASS
[ ] relevant browser/touch gate recorded
[ ] gameplay tests unchanged except necessary end-menu assertions
[ ] no Arcade-specific abstraction copied into core
[ ] consumer code is not more complex than the legacy responsibility it replaces
[ ] repeated friction vs Arcade is recorded for P6.3/P7
```

## Stop condition

After P6.2 implementation and validation:

> **STOP and perform the two-consumer synthesis before any third migration.**

No additional consumer migration is authorized until the evidence from Arcade + Breakout is compared and classified.
