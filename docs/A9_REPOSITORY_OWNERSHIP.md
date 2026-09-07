# A9 — Repository ownership boundary

## Canonical ownership

### `gotoo-pixel-engine`

Owns reusable engine/platform capabilities and reusable GPE.UI primitives.

Examples in this repository are engine/UI probes and executable documentation only. They must not become canonical game implementations.

### `gpe_arcade`

Owns the Arcade shell, catalog, launch/integration adapters, and the small Arcade games whose canonical implementations live under `gpe_arcade/src/games/**`.

Current Arcade-owned small games:

- Snake
- Breakout
- Tetris
- Pong
- Space Invaders

### Standalone game repositories

Substantial games own their own runtime, assets, native/Web entrypoints, and reusable game factory surfaces.

Examples include:

- `gpe_void-canticle`
- `gpe_smartboyhero`

Arcade consumes such games as dependencies through small adapters. Their runtime/assets must not be copied into Arcade or GPE.

## Dependency direction

```text
standalone game ──> GPE
Arcade          ──> GPE
Arcade          ──> standalone game public factory
GPE             -X-> Arcade
GPE             -X-> game implementation
standalone game -X-> Arcade
```

## Pages/Web ownership

The GPE repository may host the GitHub Pages deployment workflow and static shell, but game artifacts are built from their canonical owner:

- Arcade Web from `gpe_arcade`;
- standalone Web games from their standalone repositories.

GPE must not rebuild Arcade-owned game implementations from copied `examples/**` sources.

## Regression guard

`scripts/check_separation.py` is part of the normal `scripts/dev.py check` path. It fails if the Arcade-owned mini-games are reintroduced as GPE examples/tests/build targets.

## Rule

> GPE provides capabilities. Arcade composes games. Games own their runtime and assets.
