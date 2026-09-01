# Prior Art Review

Access date for all web sources unless otherwise noted:

`2026-09-01`

This review extracts mechanisms and failure modes. It does not recommend importing an entire external framework.

Facts likely to evolve are paired with a version/date or explicitly marked as current-at-access.

---

# 1. Dear ImGui

Version observed:

`v1.92.9` on the GitHub releases page at access time.

Sources:

- https://github.com/ocornut/imgui/releases
- https://github.com/ocornut/imgui

## FACT

Dear ImGui remains an immediate-mode C++ GUI focused on minimal dependencies and engine/backend integration. Its renderer integration is deliberately decoupled: Dear ImGui emits drawing data rather than owning the GPU backend.

## What it solves well

- very low friction inside game/tool loops;
- direct imperative UI authoring;
- strong custom rendering/widget culture;
- backend portability;
- practical keyboard/gamepad navigation and tool UI.

## GPE lesson

Keep the integration boundary narrow:

```text
UI semantics/layout
→ paint commands / framebuffer operations
```

Do not make GPE.UI own the renderer or platform event loop.

## Do not copy blindly

- historical ID conventions and immediate-layout constraints;
- tool-first visual conventions;
- assumption that desktop mouse/keyboard is the dominant interaction;
- a global context model if explicit GPE-owned state is sufficient.

---

# 2. egui

Version:

`0.36.1` — crates.io/docs.rs release visible August 2026.

Sources:

- https://docs.rs/crate/egui/latest
- https://docs.rs/crate/egui/latest/source/src/lib.rs
- https://docs.rs/egui/latest/egui/struct.Id.html
- https://docs.rs/egui/latest/egui/struct.Memory.html

## FACT

egui is a Rust immediate-mode GUI running on web and native. It reconstructs UI each frame while retaining selected state in `Memory`, and it uses stable IDs for interaction/state where needed.

Its own documentation explicitly discusses an important immediate-mode layout problem: some layouts need size before final position is known, causing previous-frame or extra-pass techniques.

## What it solves well

- excellent Rust call-site ergonomics;
- immediate responses;
- automatic IDs plus explicit ID escape hatches;
- retained interaction state without requiring retained widget objects;
- portable rendering integration;
- practical layout/composition.

## GPE lesson

This is strong evidence for:

```text
transient UI description
+ retained keyed interaction state
```

rather than assuming "immediate" means "no stable identity".

It also demonstrates that multi-pass layout is a first-class architectural concern, not an implementation detail.

## Do not copy blindly

egui's own documentation calls immediate mode easier but less powerful for certain layout problems. GPE.UI's strategic responsive-layout ambition should therefore not be forced into a strictly one-pass immediate model.

---

# 3. microui

Version model:

No GitHub releases are published. Review uses repository `master` as observed on 2026-09-01.

Source:

- https://github.com/rxi/microui

## FACT

microui describes itself as a tiny immediate-mode UI library, around 1100 SLOC of ANSI C, operating in fixed-size memory with no additional allocation, with a simple layout system and user-provided rendering.

## What it solves well

- extreme conceptual size discipline;
- renderer independence;
- custom controls from primitives;
- bounded memory model.

## GPE lesson

The Small Game adversary should remain active throughout design.

A useful UI kernel should not require:

```text
asset database
scene graph
CSS engine
async runtime
global manager
```

just to draw `Panel + Text + Button`.

## Do not copy blindly

Its intentionally small scope does not cover the strategic responsive/layout/style/authoring ambitions.

---

# 4. Iced

Version:

`0.14.0` at access time.

Sources:

- https://docs.rs/crate/iced/latest
- https://docs.rs/crate/iced/latest/source/src/lib.rs
- https://docs.rs/crate/iced/latest/source/README.md

## FACT

Iced is a cross-platform Rust GUI inspired by Elm, with a message-oriented/reactive model, responsive layout and modular internal crates.

## What it solves well

- explicit application messages;
- separation between view description and update;
- declarative composition;
- responsive widgets/layout;
- strong type-driven architecture.

## GPE lesson

Semantic UI output is valuable:

```text
widget interaction
→ message/action
→ application decides mutation
```

This supports:

- deterministic traces;
- feedback mapping;
- testing;
- multi-pass layout.

## Do not copy blindly

A full Elm application runtime would be too much policy for GPE games. GPE.UI should emit actions/events without becoming the owner's gameplay state machine.

---

# 5. Flutter

Version:

Official docs state they reflect **Flutter 3.44.7**.

Sources:

- https://docs.flutter.dev/ui/layout
- https://docs.flutter.dev/resources/architectural-overview

Docs observed updated July/August 2026.

## FACT

Flutter's layout model is based on hierarchical composition and constraints. Its documentation summarizes the core model as constraints flowing downward, sizes flowing upward, and parents assigning positions.

Flutter also separates high-level widgets from lower render objects.

## What it solves well

- compositional layout;
- clear constraint protocol;
- intrinsic/custom render objects;
- responsive UIs;
- powerful custom rendering.

## GPE lesson

A small constraint protocol is a better conceptual starting point than copying CSS.

Candidate GPE model:

```text
parent Constraints
→ child measured Size
→ parent assigns integer Rect
```

## Do not copy blindly

Flutter's retained element/render-object machinery, reactive rebuild infrastructure and platform-scale architecture are far heavier than GPE needs.

---

# 6. Slint

Version:

`1.17.1` at access time.

Sources:

- https://docs.slint.dev/latest/docs/rust/slint/
- https://docs.slint.dev/latest/docs/slint/guide/language/coding/positioning-and-layouts/
- https://docs.slint.dev/latest/docs/slint/guide/language/coding/functions-and-callbacks/

## FACT

Slint provides a declarative UI language with Rust integration. Its layout system includes horizontal/vertical/grid layouts and min/max/preferred constraints. Public properties/callbacks are exposed to Rust through generated APIs. Runtime interpretation exists, but precompiled `.slint` is the normal path.

## What it solves well

- declarative authoring without HTML compatibility;
- compile-time generated strongly typed Rust boundary;
- layouts and properties separated from backend logic;
- callback boundary between UI and application.

## GPE lesson

A future GPE markup does **not** need to be HTML/XML.

A dedicated restricted language can remain:

```text
authoring frontend
→ validated/compiled semantic UI description
→ same runtime model as Rust API
```

Build-time compilation is especially attractive for GPE's deterministic/small-runtime goals.

## Do not copy blindly

- full component/property runtime;
- generated handle model;
- global singletons;
- a separate window/event-loop abstraction.

---

# 7. Godot Control / Containers

Version reviewed:

Stable **Godot 4.5** documentation, plus current 4.x class docs.

Sources:

- https://docs.godotengine.org/en/4.5/tutorials/ui/gui_containers.html
- https://docs.godotengine.org/en/4.x/classes/class_control.html

## FACT

Godot `Control` nodes use anchors/offsets for relative positioning. Containers take ownership of child positioning for more advanced automatic layout. Godot's documentation explicitly frames anchors as effective for simpler responsive cases and Containers as more suitable for complex game/tool UIs.

## GPE lesson

Two levels of layout can coexist cleanly:

```text
anchors / absolute escape hatch
+
container-driven automatic layout
```

This is especially relevant for:

- HUD;
- menus;
- RPG/inventory;
- tool/debug surfaces.

## Do not copy blindly

GPE does not need a general scene-node object hierarchy just to obtain these layout ideas.

---

# 8. Unity UI Toolkit

Version reviewed:

Unity **6.0 / 6000.0** advanced guide; guide states validation for Unity 6.0 and publication in April 2025.

Sources:

- https://docs.unity3d.com/current/Manual/best-practice-guides/ui-toolkit-for-advanced-unity-developers/bpg-uiad-index.html
- https://learn.unity.com/tutorial/getting-started-with-ui-toolkit
- https://docs.unity.cn/Manual/UIE-UXML.html

## FACT

UI Toolkit separates:

```text
UXML       structure
USS        styling
UI Builder authoring
UI Debugger inspection
runtime visual tree / event/layout systems
```

It supports runtime and tooling UI.

## GPE lesson

Structure, style, authoring and debugging should be separate concerns.

If GPE.UI eventually gets markup:

```text
markup != style engine != inspector != runtime kernel
```

## Do not copy blindly

UXML+USS's web-inspired stack demonstrates exactly how an authoring system can grow toward browser-like complexity. GPE should take the separation, not the full surface.

---

# 9. Bevy UI

Version:

`bevy_ui 0.19.1`, published 2026-08-13.

Sources:

- https://docs.rs/crate/bevy_ui/latest
- https://docs.rs/bevy_ui/latest/bevy_ui/
- https://bevy.org/learn/migration-guides/0-18-to-0-19/

## FACT

Bevy UI is an ECS-driven game UI framework. It supports Flexbox and CSS Grid models, directional navigation, interaction states and accessibility integration.

Bevy 0.19 also makes UI opt-in relative to the high-level 2D/3D feature collections, reflecting deliberate modularity.

## GPE lesson

Useful game-specific pressures:

```text
directional navigation
UI-specific feature modularity
2D / 3D compatibility pressure
accessibility semantics
```

## Do not copy blindly

GPE is not ECS-first. Importing ECS entity/component architecture solely for UI would violate the current engine model.

---

# 10. Taffy

Version:

`0.14.0`, published 2026-08-24.

Source:

- https://docs.rs/crate/taffy/latest

## FACT

Taffy is a Rust UI layout library implementing CSS Block, Flexbox and CSS Grid and is used by multiple UI/browser projects, including Bevy and Slint.

## What it solves well

- mature reusable layout algorithms;
- separates layout engine from widget framework;
- optional feature/dependency surface;
- avoids every framework independently implementing Flex/Grid.

## GPE lesson

Taffy is a **serious implementation candidate for an optional or experimental advanced-layout backend**, not something to dismiss because GPE is a game engine.

## Risk for GPE

- CSS-derived semantics may be more than GPE needs;
- dependency/build footprint must be measured rather than guessed;
- floating/subpixel layout must be reconciled with deterministic integer pixel output;
- adding it to the minimal path would violate the small-consumer principle unless evidence supports the cost.

Verdict for Phase 0:

```text
DO NOT SELECT YET
MEASURE IN A FUTURE LAYOUT SPIKE IF CUSTOM LAYOUT PROVES COSTLY
```

---

# 11. Yoga

Version:

Current repository reviewed on 2026-09-01; repository/documentation describes Yoga as an embeddable Flexbox engine. No "latest release" claim is made here because the retrieved release evidence was insufficiently authoritative.

Sources:

- https://github.com/facebook/yoga
- https://www.yogalayout.dev/

## FACT

Yoga focuses on an embeddable Flexbox layout engine rather than a complete UI framework.

Qt Quick 6.11 documentation states that its preliminary `FlexboxLayout` uses Yoga internally; that Qt type was introduced in Qt 6.10.

## GPE lesson

A narrow layout engine can be independently reusable.

This supports keeping the GPE kernel/widget model conceptually separate from any future Flex implementation.

---

# 12. Qt Quick / QML

Version reviewed:

Qt docs **6.11.2**.

Sources:

- https://doc.qt.io/qt-6/qtquick-layouts-qmlmodule.html
- https://doc.qt.io/qt-6/qml-qtquick-layouts-flexboxlayout.html

## FACT

Qt Quick provides declarative QML plus Row/Column/Grid/Stack layouts and, since Qt 6.10, a preliminary FlexboxLayout backed by Yoga.

## GPE lesson

Mature toolkits frequently provide:

```text
simple native layout primitives
+
specialized richer layout capability
```

rather than requiring every UI to express itself in one universal algorithm.

---

# 13. resvg / usvg

Version:

`resvg 0.48.1`
`usvg 0.48.1`

Sources:

- https://docs.rs/crate/resvg/latest
- https://docs.rs/crate/usvg/latest
- https://docs.rs/usvg/latest/usvg/

## FACT

`usvg` parses and simplifies SVG into a strongly typed resolved tree. `resvg` renders static SVG and explicitly excludes scripting/events/animation from its supported static subset.

## GPE lesson

This is almost exactly the direction GPE should prefer if SVG is eventually retained:

```text
restricted/static SVG
→ simplify
→ rasterize/cache
→ GPE Image
```

Potentially at build time.

That avoids importing browser event/script semantics into the UI kernel.

---

# 14. AccessKit

Version:

`0.24.1`

Source:

- https://docs.rs/accesskit/latest/accesskit/

## FACT

AccessKit represents accessibility as a tree with stable `NodeId` values and platform-neutral semantic nodes.

## GPE lesson

Even if advanced accessibility is deferred, the kernel should avoid an identity/semantics design that would make a future accessibility projection impossible.

This is **pressure for stable semantic identity**, not proof that AccessKit must be a v1 dependency.

---

# 15. Cross-system deductions

## 15.1 Immediate vs retained is not binary

Evidence:

- egui: transient immediate description + retained memory/IDs;
- Flutter: declarative rebuild + retained element/render infrastructure;
- Slint: declarative source + compiled/runtime component model;
- Iced: view description + semantic messages.

Recommendation:

> Treat "description lifetime", "interaction-state lifetime" and "render-object lifetime" as separate design decisions.

## 15.2 Constraint layout is the common reusable primitive

Flutter, Slint, Godot containers, Qt layouts, Taffy and Bevy all converge on some notion of:

```text
available space
child size requirements
parent placement
```

Recommendation:

> GPE should first define its own small geometry/constraint contract before deciding whether a richer engine such as Taffy belongs underneath it.

## 15.3 Stable identity is useful without a permanent DOM

egui is direct evidence.

Recommendation:

> A `UiId`-like concept may exist purely for interaction/state/semantics, without implying retained widget objects.

## 15.4 Markup should be a frontend

Slint and Unity demonstrate the value of separating authoring source from application logic.

Recommendation:

> Rust and future markup should target one semantic model. Do not implement a second UI runtime for markup.

## 15.5 "Pay for what you use" is a real architecture concern

microui demonstrates the value of a tiny minimum; Bevy demonstrates feature-profile modularity; Taffy demonstrates layout can be a separable engine.

Recommendation:

> keep the minimal GPE UI path free of new heavy mandatory dependencies.

## 15.6 Static SVG is tractable; browser SVG is not the target

`usvg/resvg` demonstrate a mature static/simplified model.

Recommendation:

> if SVG proceeds, start with build-time/static rasterization, not runtime DOM/event semantics.

---

# 16. Prior-art verdict

Keep:

```text
immediate-style authoring ergonomics
retained keyed interaction state
constraint-based layout
composition
semantic messages/actions
renderer independence
explicit custom component escape hatch
authoring/runtime separation
feature modularity
static SVG strategy
```

Reject copying wholesale:

```text
DOM
CSS cascade
browser event model
ECS solely for UI
Flutter-scale retained infrastructure
full Elm application runtime
Unity-style authoring stack in v1
```

Still requires MFE:

```text
exact Rust authoring shape
transient graph allocation cost
same-frame multi-pass interaction ergonomics
custom integer layout vs Taffy
stable ID ergonomics
```
