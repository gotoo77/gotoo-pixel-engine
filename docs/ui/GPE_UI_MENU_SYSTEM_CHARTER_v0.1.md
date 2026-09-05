# GPE.UI — First-Class Menu System Charter v0.1

Status: **AUTHORITATIVE DESIGN CONSTRAINT FOR P6/P7**

This document records a product/architecture constraint discovered during P6.2 and applies to all subsequent GPE.UI productionization work.

## Core statement

Menus are a primary game UI primitive, not incidental consumer glue.

GPE.UI must eventually provide a first-class menu system that is:

```text
simple to declare
simple to embed
composable
customizable
extensible
themeable
input-complete
animation-friendly
resolution-independent
usable in pixel and high-resolution contexts
capable of visually polished / "WOW" presentation
```

The system must serve both small games and sophisticated tooling without forcing every consumer to rebuild focus, navigation, interaction, styling, layout, transition and presentation behavior.

The long-term ambition is closer to a compact Rust-native GUI toolkit for games than to a collection of ad-hoc menu helpers. GPE.UI should aim for the expressive power associated with mature widget toolkits while remaining substantially lighter, game-oriented, immediate/transactional where appropriate, and compatible with GPE's pixel-first architecture.

## Primary consumers

The menu system must be suitable for, at minimum:

```text
main menus
pause menus
game-over / victory menus
settings menus
nested submenus
controls / bindings menus
audio/video/accessibility settings
save/load/profile selection
inventory or equipment-style panels when menu semantics fit
dialog / confirmation flows
debug menus
developer tooling overlays
editor/tool windows
launcher/catalog navigation
```

No architecture decision should optimize only for Arcade or Breakout.

## Required separation of concerns

A first-class menu system should separate these layers:

```text
1. semantic model
   items, actions, values, hierarchy, enabled/disabled state

2. interaction
   focus, navigation, activation, cancel/back, pointer, touch, gamepad

3. layout
   row, column, grid, tabs, split panes, scrolling, responsive behavior

4. styling
   colors, borders, spacing, typography, state styles, images/icons

5. presentation
   transitions, animation, hover/focus effects, high-res/pixel composition

6. application ownership
   game state, business logic, callbacks/actions, domain data
```

The game should own its domain state and semantic actions. GPE.UI should own generic UI mechanics.

## Declaration ergonomics

The common case must be concise enough to teach directly.

Conceptual target only:

```rust
ui.menu("pause", |menu| {
    menu.item("resume", "RESUME", PauseAction::Resume);
    menu.item("settings", "SETTINGS", PauseAction::Settings);
    menu.item("quit", "QUIT", PauseAction::Quit);
});
```

A richer menu should be able to add capabilities without changing architecture:

```rust
ui.menu("settings", |menu| {
    menu.slider("music", "MUSIC", settings.music, 0.0..=1.0);
    menu.toggle("fullscreen", "FULLSCREEN", settings.fullscreen);
    menu.submenu("controls", "CONTROLS", controls_menu);
});
```

These snippets are **directional**, not frozen API design.

The requirement is that simple menus remain simple while advanced consumers can progressively opt into deeper layout/style/animation/custom widget capabilities.

## Customization depth

Customization must not stop at changing colors.

The system should be able to support, when justified by P7 work:

```text
fonts / font stacks
text sizing and fitting
icons and images
background artwork
per-item accent colors
state-specific visuals
borders / rounded or custom frames where renderer supports them
spacing / padding / alignment
custom item renderers
custom widget composition
focus cursor styles
selection markers
transitions
entry/exit animation
focus/hover animation
scanlines / glow / presentation effects where generic
responsive rules
high-resolution host presentation
pixel-perfect low-resolution presentation
```

Arcade-specific art direction remains consumer-owned, but the generic primitives required to express it belong in GPE.UI when independently justified.

## Extensibility contract

The menu system must not become a closed enum of built-in widgets.

A consumer should eventually be able to compose or introduce a custom control without forking the interaction kernel.

The extensibility direction should preserve:

```text
stable semantic identity
one focus authority
one interaction transaction per surface
custom visual composition
custom value/state adapters
consumer-defined semantic actions
```

Do not create parallel focus/navigation systems for custom widgets.

## Hierarchy and navigation

Nested menus and submenus are first-class requirements.

The system must eventually define explicit semantics for:

```text
push submenu
pop/back
modal confirmation
focus restoration
initial focus
focus memory per menu where useful
wrap/clamp policies
keyboard/gamepad parity
pointer/touch parity
disabled items
hidden/conditional items
```

This should not require each game to hand-roll a state machine merely to open Settings from Pause and return to the previously focused item.

## Debug / tooling requirement

GPE.UI is also expected to support debug and development interfaces.

That implies the shared widget vocabulary may eventually include proven primitives such as:

```text
buttons
toggles
sliders
labels
text fields
lists
combos/selectors
tabs
collapsible sections
scrolling panels
property rows
status badges
progress indicators
```

However, breadth must be added by evidence and coherent architecture, not by cloning GTK/egui widget catalogs mechanically.

The goal is a **game-oriented toolkit**, not a desktop application framework accidentally embedded in GPE.

## Visual quality requirement

Functional gray boxes are not the final success criterion.

GPE.UI must permit interfaces that can be visually distinctive and polished without forcing the consumer to bypass the toolkit.

A successful architecture should make both of these valid:

```text
minimal pixel pause menu
high-end animated neon launcher
```

using the same interaction/state foundations.

If a consumer must abandon GPE.UI to achieve a strong visual identity, the abstraction is incomplete.

## P6 interpretation

P6 remains an evidence-gathering phase.

The Breakout `EndMenuState` introduced in P6.2 is **not** the target public menu API. It is a consumer-local probe used to test the current converged kernel against a tiny linear menu.

P6.2 must therefore answer:

```text
what mechanics are already reusable?
what glue remains consumer-local?
what becomes repetitive versus Arcade?
what requirements appear for a real first-class Menu abstraction?
```

Do not promote `EndMenuState` itself to core merely because it works.

## P7 sequencing change

P7 must include an explicit first-class Menu/Widget architecture track before final API freeze.

Recommended sequence:

```text
P7.1  API consolidation / naming
P7.2  Menu system architecture + ergonomic declaration surface
P7.3  Widget composition / styling / theming / extensibility
P7.4  Typography + high-resolution presentation boundary
P7.5  Arcade Visual Showcase as stress test
P7.6  Pause/settings/debug-menu reference consumers
P7.7  Tutorial / teaching gate
P7.8  Compatibility / deprecation decisions
```

Exact numbering may be folded into the main roadmap during P6.3 synthesis, but the architectural requirement itself is now fixed.

## Mandatory reference scenarios before freeze

Before GPE.UI can be considered productionized, the same underlying menu/widget architecture should prove at least these scenarios:

```text
A. tiny 2-item game-over menu
B. pause menu with nested Settings
C. settings page with toggle + slider + submenu
D. debug/tool menu with mixed widgets
E. Arcade launcher with rich visual composition
```

These scenarios are deliberately unlike one another.

## Anti-goals

Do not solve this mandate by introducing:

```text
a giant global UI manager
a retained scene tree required for every game
an Arcade-specific DSL
a Breakout-specific menu class
a mandatory desktop-GUI architecture
a monolithic god-widget system
parallel interaction kernels per widget family
a renderer rewrite without evidence
```

## Definition of success

A developer should be able to start with something conceptually as small as:

```rust
menu.item("resume", "RESUME", Action::Resume);
```

and grow, without changing UI architecture, toward a deeply customized animated menu or tool panel.

The final target is:

> **simple by default, deep when needed, visually unconstrained, and consistent across games and tools.**
