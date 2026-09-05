# GPE.UI — P6.2 Breakout End Menu Result v0.1

Status: **PASS / STOP — TWO-CONSUMER SYNTHESIS REQUIRED NEXT**

## Scope

P6.2 migrated the Breakout game-over menu as the second real GPE.UI consumer, deliberately unlike the high-resolution Arcade launcher.

The migrated responsibility is limited to:

```text
GAME OVER
SCORE / LEVEL
REPLAY
QUIT
```

Gameplay physics, audio, HUD, touch transport, paddle/ball/bricks and the main Breakout application menu remain outside this slice.

## Implementation result

The legacy end-menu state:

```rust
MenuState
```

was replaced locally by a small Breakout-owned adapter around the converged GPE.UI interaction kernel:

```text
EndMenuState
├─ UiStateStore
├─ stable keyed REPLAY identity
├─ stable keyed QUIT identity
├─ linear UiNavInput
└─ semantic EndMenuAction::{Replay, Quit}
```

The Breakout consumer still owns the semantic dispatch:

```text
Replay -> restart_round()
Quit   -> GameResult::Exit
```

No gameplay state moved into GPE.UI.

## Runtime evidence

Human Native validation confirmed:

```text
GAME OVER renders correctly
REPLAY is initially focused
linear navigation works
REPLAY activates and restarts
QUIT activates and exits/returns through the existing boundary
release runtime remains near 60 FPS on the validated machine
```

The game remains visually low-resolution/pixel-oriented, which was explicitly allowed by P6.2.

The global `wasm32-unknown-unknown` compile gate passed during this slice.

## Interaction verdict

```text
stable semantic identity        PASS
initial deterministic focus     PASS
linear navigation               PASS
semantic activation             PASS
REPLAY behavior                 PASS
QUIT behavior                   PASS
consumer-owned gameplay state   PASS
Arcade-specific coupling        NONE
```

P6.2 therefore succeeds as an interaction/architecture probe.

## Important visual finding

Runtime validation exposed a broader presentation gap that is intentionally **not** fixed inside P6.2:

> Breakout and the other legacy games remain visually tied to their low-resolution framebuffer and old bitmap presentation, while the new Arcade launcher now renders as a modern high-resolution GPE.UI surface.

This creates a clear ecosystem-level discontinuity:

```text
modern high-res launcher
        ↓ launch
legacy low-res game presentation
```

The problem is not that pixel gameplay is low resolution. Pixel gameplay should remain allowed — and often preferred.

The missing capability is a reusable presentation boundary that lets low-resolution gameplay live inside a polished high-resolution shell.

This evidence directly authorizes a P7 **Game Presentation Shell / High-Resolution Host** workstream.

## Menu-system finding

P6.2 also confirms that a two-item end menu should not require game-specific state/navigation machinery.

`EndMenuState` is a P6 probe adapter, not the target public API.

Together with Arcade, PauseGame, settings/debug-menu needs and future consumers, this supports the first-class Menu System charter:

```text
simple declaration
stable semantic actions
focus/navigation owned by GPE.UI
keyboard/gamepad/mouse/touch convergence
styling/theming
submenus
settings widgets
debug menus
extensible/custom content
```

## DX comparison with Arcade

Arcade and Breakout are structurally different, but they expose recurring shared responsibilities:

```text
stable identity
semantic focus/navigation
input-family convergence
style-state representation
high-level consumer actions
presentation boundary
```

They do **not** justify forcing every UI through a card-grid or launcher-specific abstraction.

The second-consumer evidence strengthens the direction:

> GPE.UI should provide a small composable kernel plus first-class higher-level primitives such as Menu and Presentation Shell, rather than a collection of per-game helper patterns.

## P6.2 verdict

```text
consumer migration             PASS
runtime behavior               PASS
interaction architecture       PASS
rollback boundary              PASS
visual modernization           OUT OF SCOPE / GAP CONFIRMED
```

Overall:

> **P6.2 = PASS / STOP**

## Next authorized step

No third consumer migration is authorized yet.

Next:

> **P6.3 — two-consumer synthesis: Arcade + Breakout**

The synthesis must explicitly feed P7 with at least these two first-class directions:

```text
1. GPE.UI Menu System
2. GPE Game Presentation Shell / High-Resolution Host
```

Both must remain customizable, composable, extensible and simple to adopt. Neither may require consumers to rebuild engine/UI plumbing manually.
