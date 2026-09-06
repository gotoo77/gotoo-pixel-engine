# GPE.UI — P6 Consumer Migration v0.1

Status: **DEFINED / IMPLEMENTATION NOT STARTED**

## Mission

P0 through P5 converged and measured the Architecture B interaction, layout,
styling, typography and cost boundaries. P6 now has a different job: prove that
those abstractions survive contact with real repository consumers without forcing
a bulk rewrite or prematurely freezing the public API.

The governing principle is:

> migrate one real consumer, learn from it, then decide whether the abstraction is ready for wider adoption.

P6 is therefore a sequence of bounded consumer migrations, not a repository-wide
UI conversion.

## Preconditions

P6 starts from the following closed productionization state:

```text
P0 kernel convergence       PASS / STOP
P1 interaction              PASS / STOP
P2 layout                   PASS / STOP
P3 styling                  completed before this branch baseline
P4 typography               PASS / STOP
P5 cost attribution         PASS / STOP
```

P5 explicitly concluded that the remaining capture-OFF allocations do not justify
further optimization without consumer/profiler evidence. P6 must not reopen that
question as part of migration work.

## 1. Repository-local UI consumer inventory

This inventory classifies observable repository consumers by the UI path they use
at the P6 definition baseline. A consumer may legitimately appear in more than
one category when it composes compatibility helpers with gameplay-specific
rendering.

### Legacy / minimal UI

The historical compatibility surface remains centered on:

```text
MenuState
draw_panel
draw_text_centered
draw_menu_item
standard_menu_controls
menu_up_pressed / menu_down_pressed / menu_confirm_pressed
PauseGame / PauseConfig
VirtualPad / VirtualButton
```

Observed real consumers include:

| Consumer | Current use |
|---|---|
| `examples/arcade/game.rs` | `MenuState`, static draw helpers, `standard_menu_controls`, `VirtualPad`, `PauseGame` |
| `examples/breakout/game.rs` | end-state `MenuState`, static draw helpers, menu input helpers, `VirtualPad` |
| `examples/pong/game.rs` | match-end `MenuState`, static draw helpers, menu input helpers, `VirtualPad` |
| `examples/space_invaders/menu.rs` | multi-page `MenuState`, static draw helpers, keyboard/gamepad menu helpers |
| `examples/tetris/game.rs` | legacy/minimal menu/HUD path where applicable |
| `examples/snake/game.rs` | legacy/minimal helpers plus game-specific rendering where applicable |
| `src/ui/pause.rs` | compatibility pause wrapper with `MenuState`, shared controls and optional touch pad |

The important compatibility fact is that these consumers own gameplay state and
usually own their `ControlMap`; the historical UI helpers do not own the game.

### Experimental transactional UI

The transactional Architecture B surface is exercised by dedicated UI evidence
and productionization probes, including:

```text
examples/gpe_ui_mfe_001a_probe.rs
examples/gpe_ui_mfe_001c_cost_probe.rs
examples/gpe_ui_p3_style_probe.rs
examples/gpe_ui_p4_typography_probe.rs
examples/gpe_ui_p4_font_gallery.rs
```

These consumers use the transient graph / `UiBuilder` family, stable `UiId`,
consumer-owned `UiStateStore`, typed semantic output, shared layout and styling,
and opt-in debug capture where diagnostics require it.

They are evidence consumers, not first-choice P6 migration targets: migrating a
probe to the system it already proves would not falsify consumer adoption.

### Experimental spatial UI

The spatial compatibility adapter is exercised directly by
`examples/gpe_ui_mfe_001b_probe.rs` and by P5 spatial cost coverage. Its observable
surface includes:

```text
SpatialState
SpatialInput
SpatialCard
GridSpec
run_card_grid
CardPainter / DefaultCardPainter
```

The adapter now delegates interaction/layout responsibilities to the converged
kernel/shared layout rather than representing an independent second UI runtime.
It remains a compatibility/evidence surface until P7 decides public naming and
cleanup.

### Toolkit / helpers

`examples/tool_window_probe.rs` is the clear repository example using the older
immediate toolkit:

```text
Ui
UiState
UiTheme
UiResponse
```

It exercises labels, sections, tabs, toggles, selects, sliders, buttons and root
scrolling in an auxiliary tool window. It is useful as an unlike future migration
candidate because its shape is settings/tooling rather than a game catalog.

`PauseGame`, `VirtualPad`, `ControlMap` and the stateless draw/menu helpers also
remain important compatibility helpers for real games. P6 does not treat them as
dead code merely because the new kernel exists.

### Ad-hoc UI code

Several games mix UI helpers with direct framebuffer rendering. Examples include:

- gameplay HUD text drawn directly with `Framebuffer::draw_text`;
- manually positioned touch-control rectangles and labels;
- bespoke status/control pages such as Space Invaders gamepad setup;
- game-specific overlays, score/lives/level presentation and prompts;
- Smart Boy Hero screens and HUD/rendering code whose scope is substantially
  larger than the small menu cases.

Ad-hoc rendering is not automatically migration debt. P6 migrates interaction or
composition only where the new UI abstraction provides a demonstrated benefit.
A score counter or bespoke game-world overlay may correctly remain direct
framebuffer code.

## 2. Actual migration candidates

A migration candidate must be a real consumer, have a bounded UI responsibility,
and provide evidence that cannot be obtained from another synthetic probe.

Prioritized candidates are:

| Candidate | Why it is useful | P6 suitability |
|---|---|---|
| Arcade catalog | real launcher surface; six selectable entries; keyboard/gamepad/touch; Native + Web; close structural match to proven responsive card/grid work | **FIRST** |
| Breakout end menu | tiny two-action menu; real gameplay transition boundary; touch exists | strong later linear-menu slice |
| Pong match-end menu | tiny menu plus two-player/gamepad context | strong later input-parity slice |
| Space Invaders main/controls menu | multi-page menu plus live gamepad status and adjustment actions | strong unlike/complex slice after first proof |
| `PauseGame` | shared compatibility wrapper used by several games, including touch | high leverage but too central for the first slice |
| Tool window controls | settings/tooling shape with tabs/toggles/select/sliders/scroll | excellent unlike consumer, but it is a probe rather than the first real-game migration |
| Smart Boy Hero UI | substantial real-game/ad-hoc surface | too broad for first migration; defer until smaller slices establish patterns |

Not every ad-hoc HUD is a migration candidate. P6 must resist converting drawing
code merely to increase adoption counts.

## 3. First consumer selection

The first migration target is the **Arcade catalog only**, not the games launched
from it and not `PauseGame`.

Reasons:

1. it is a real repository consumer rather than an architecture probe;
2. its selectable six-entry catalog is small enough to review as one vertical
   slice;
3. it already has explicit keyboard/gamepad/touch actions through `ControlMap`
   and `VirtualPad`;
4. it ships through both Native and Web wrappers;
5. it closely matches the responsive card/grid workload already proven by
   MFE-001B, so differences expose migration friction rather than inventing a new
   UI problem;
6. the gameplay of Snake, Tetris, Space Invaders, Smart Boy Hero, Pong and
   Breakout can remain untouched;
7. rollback is localized to the catalog state/update/render path.

This is intentionally narrower than "migrate Arcade".

## 4. Compatibility boundary during P6

The following boundary remains mandatory until consumer evidence supports a later
change:

```text
Legacy toolkit/helpers remain callable.
PauseGame remains callable and behaviorally unchanged.
VirtualPad and ControlMap remain valid input integration mechanisms.
Experimental transactional/spatial modules remain compatibility/evidence adapters.
Consumer-owned gameplay state remains outside the UI runtime.
Consumer-owned persistent UI state remains explicit.
No global UI manager is introduced.
No input transport or platform contract is rewritten merely for migration.
No public v1 naming/freeze is performed in P6.
```

A migrated consumer may coexist with non-migrated consumers in the same binary.
That coexistence is a required migration property, not temporary failure.

The compatibility APIs may only be removed after demonstrated migrations establish
that their responsibilities are covered and after the dedicated P7 API review.

## 5. PASS contract for one migrated consumer

A consumer migration is not complete merely because it compiles. Its acceptance
matrix is:

### Behavior parity

- existing user-visible actions still occur at the same gameplay boundaries;
- selection/focus remains deterministic;
- entering/leaving the UI does not leak confirm/cancel input into gameplay;
- existing consumer-owned game state remains authoritative;
- no unrelated gameplay or rendering behavior changes.

### Keyboard

- all previously supported keyboard navigation/activation/cancel behavior passes;
- held/pressed semantics remain intentional and tested where observable.

### Mouse

- if the migrated surface exposes pointer interaction, hover/click/capture must
  use the converged UI interaction path and be validated;
- existing behavior must not regress on systems where the mouse is present.

### Gamepad

- where the consumer already supports gamepad, navigation and activation parity
  are required;
- device-selection/assignment semantics owned by the consumer must not be
  silently changed.

### Touch

- where the consumer already has a touch mode, touch activation and transition
  behavior are required;
- existing `VirtualPad` responsibilities may remain at the compatibility boundary
  unless the slice explicitly proves a better replacement.

### Native

- build/tests pass;
- the migrated consumer is executed through its Native entry point;
- if rendering, focus visuals, layout or pointer behavior changed, a human runtime
  gate is mandatory.

### Web / WASM

- `cargo check --target wasm32-unknown-unknown` passes;
- the consumer's Web entry point remains buildable;
- if the migration changes rendered UI or browser pointer/touch behavior, perform
  a browser human runtime gate rather than treating compilation as visual proof.

### Tests

At minimum, add or adapt deterministic coverage for:

```text
initial focus/selection
navigation
activation -> expected consumer action
stable identity across the consumer's relevant rebuilds
cancel/return boundary if present
input release/leak guard where present
layout invariants important to the consumer
```

Tests should remain headless where the contract is semantic or geometric.
Human gates are reserved for visual/runtime facts that headless tests cannot
prove.

## 6. Explicit non-goals

P6 does **not** authorize:

```text
bulk rewrite of UI consumers
deleting compatibility APIs before demonstrated migration
speculative scene/UI framework
migration of every game at once
P7 public API cleanup or naming freeze
rewriting platform input transport
rewriting ControlMap merely to make the UI look uniform
rewriting PauseGame as collateral work
turning every HUD draw call into a widget
reopening P5 allocation optimization without new consumer evidence
```

P6 must stop after each bounded consumer slice, review what was learned, and only
then authorize another migration.

# P6.1 — Arcade catalog

## Why this consumer

`ArcadeApp` provides the smallest real end-to-end adoption test that still covers
multiple input families and both platform targets. The migration surface is the
catalog screen only: six launch choices, selection/focus, activation, catalog
layout and optional touch controls. Launched games and the shared pause wrapper
remain behind the existing compatibility boundary.

This slice directly tests whether the card/grid capability proven by MFE-001B is
usable by application code rather than only by a probe.

## Files involved

Primary expected files:

```text
examples/arcade/game.rs
```

Validation/wrapper files inspected or potentially touched only if required by the
migration contract:

```text
examples/arcade.rs
examples/arcade_web.rs
```

UI implementation files under `src/ui/` are **not expected** to change for the
first attempt. If P6.1 discovers a missing primitive that requires kernel changes,
stop and review that finding before broadening the slice.

## Current UI path

The catalog currently owns:

```text
MenuState catalog_menu
ControlMap catalog_controls
optional VirtualPad catalog_pad
manual ArcadeLayout rectangles
legacy draw_panel / draw_menu_item / draw_text_centered rendering
manual update_catalog navigation + launch dispatch
```

`standard_menu_controls` maps the catalog's keyboard/gamepad intents, while the
touch `VirtualPad` writes the same catalog actions into `ControlMap`.

## Target UI path

Use the already converged Architecture B path through the current
transactional/spatial compatibility surface, backed by the shared kernel, shared
responsive grid layout and P3 styling.

For the six launch entries, prefer the existing card/grid path proven by
MFE-001B rather than creating a new Arcade-specific widget or scene layer.

The target must preserve:

```text
stable UiId per game entry
consumer-owned persistent UI state
semantic activation mapped back to the existing launch(index) boundary
shared keyboard/gamepad/pointer/touch interaction semantics where applicable
existing Arcade game ownership and launch/return release gates
```

Do not rename or promote the experimental modules as public v1 API in P6.1; P7
owns that decision.

## Expected code changes

Expected changes are deliberately local:

1. replace `catalog_menu: MenuState` with the persistent state required by the
   converged catalog UI path;
2. express the six `GAME_LABELS` entries as stable keyed UI/card items;
3. replace manual catalog selection/update dispatch with semantic UI output that
   calls the existing `launch(index)` boundary;
4. replace the six manual `draw_menu_item` rows with the shared responsive
   card/grid composition and existing style vocabulary;
5. retain `catalog_controls`, `catalog_pad`, `waiting_for_launch_release` and
   `waiting_for_catalog_release` unless direct migration evidence proves a smaller
   safe change;
6. keep `build_game`, `pause_game`, every launched game's code and gameplay
   rendering unchanged;
7. do not opportunistically migrate the footer, touch-pad chrome or unrelated
   direct framebuffer text if doing so is not required for the catalog proof.

If adapting `ControlMap`/`VirtualPad` input to the converged `UiInput` vocabulary
requires only a thin consumer-local translation, keep it local for P6.1. A new
engine-wide adapter requires a separate review before implementation.

## Tests to add or change

Add/adjust focused Arcade tests proving at least:

```text
initial catalog focus selects the first game
keyboard navigation changes focused entry deterministically
gamepad navigation changes focused entry deterministically
activation dispatches the same game index as before
touch activation dispatches the same game index in Touch mode
mouse hover/click activates the intended card if pointer support is enabled
return-to-catalog still arms the existing release gate
launch still arms the existing launch-release gate
stable keyed identity survives the relevant catalog rebuild/frame sequence
Native and Touch layouts keep all catalog items inside their intended bounds
```

Existing Arcade surface-containment and release-gate tests remain unless the
migration explicitly replaces the contract they prove.

No gameplay tests for the six embedded games should need semantic changes.

## Human validation required

Because P6.1 changes catalog rendering/layout and interaction presentation, human
runtime validation **is required**.

Native gate:

```text
catalog renders without clipping/overlap
keyboard navigation/activation works
gamepad navigation/activation works
mouse hover/click works when exposed
launch and return transitions do not leak input
focus/hover/active styling is visually coherent
```

Web gate:

```text
catalog renders correctly in browser
keyboard works
mouse works
touch works on the touch path / suitable touch-capable browser setup
gamepad works where browser/device support is available and relevant
launch/return behavior matches Native semantics
```

A platform capability unavailable in the validation environment must be recorded
explicitly rather than silently marked PASS.

## Rollback boundary

P6.1 rollback is the Arcade catalog migration only.

A revert must be able to restore:

```text
MenuState catalog_menu
legacy catalog update logic
legacy catalog row rendering
```

without reverting P0–P5, without changing any launched game, and without changing
`PauseGame`, `VirtualPad`, `ControlMap` or the shared compatibility APIs.

If the implementation cannot preserve this rollback boundary, the slice has
become too large and must stop for review.

## PASS criteria

P6.1 is PASS only when all of the following are true:

- [ ] only the Arcade catalog is migrated;
- [ ] gameplay and launch targets are behaviorally unchanged;
- [ ] keyboard parity passes;
- [ ] mouse pointer behavior passes where exposed by the migrated surface;
- [ ] gamepad parity passes;
- [ ] touch parity passes in Arcade Touch mode;
- [ ] Native build/tests pass;
- [ ] Web/WASM compilation passes;
- [ ] Arcade Web entry remains buildable;
- [ ] deterministic migration tests pass;
- [ ] existing release-gate tests still pass;
- [ ] Native human visual/runtime gate passes;
- [ ] Web/browser human visual/runtime gate passes for rendering and applicable input paths;
- [ ] no compatibility API is deleted;
- [ ] no shared UI architecture is broadened without evidence from this migration;
- [ ] findings are recorded before authorizing another consumer migration.

Until those criteria pass, P6.1 remains an experiment in consumer adoption, not
proof that all GPE consumers should migrate.
