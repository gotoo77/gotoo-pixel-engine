# GPE — Game Presentation Shell Charter v0.1

Status: **P7 DIRECTION / FIRST-CLASS PRESENTATION COMPONENT**

## Intent

GPE must support a coherent modern presentation layer around pixel-oriented games without forcing those games to abandon their native logical resolution or rewrite their gameplay rendering.

The target is a first-class **Game Presentation Shell / High-Resolution Host** that can host an existing low-resolution `Game` while providing high-resolution GPE.UI around it.

This charter is motivated by direct P6 evidence:

```text
Arcade launcher -> high-resolution, crisp outline typography, modern composition
Breakout gameplay -> functional but visually tied to legacy low-resolution presentation
```

The gap is therefore not "pixel art is bad".

The gap is:

> pixel gameplay and modern game UI currently have no common polished presentation boundary.

## Core principle

Separate two responsibilities:

```text
GAME WORLD RENDERING
= gameplay-owned
= may remain low resolution
= may remain pixel-perfect

PRESENTATION SHELL
= host/presentation-owned
= high-resolution capable
= GPE.UI capable
= input-aware
= reusable across games
```

A game must not need to render its own high-resolution chrome, pause shell, settings layer, framing, transitions or debug overlays merely to look coherent beside the Arcade launcher.

## Target structure

Conceptually:

```text
GamePresentationShell
├─ high-resolution host framebuffer
├─ low-resolution game surface
├─ integer-nearest presentation
├─ explicit host -> game input mapping
├─ optional high-res HUD layer
├─ pause/menu layer
├─ settings/submenu layer
├─ optional debug/tool overlay
├─ framing/backdrop/borders
├─ transitions
└─ optional lightweight presentation effects
```

The low-resolution game surface remains authoritative for gameplay pixels.

The high-resolution host becomes the natural place for modern UI and presentation chrome.

## Existing evidence and starting primitives

P6 already produced useful low-level primitives:

```text
PixelPresentation
present_pixel_surface(...)
PixelGameHost
```

These are evidence, not automatically final public P7 APIs.

P7 must review whether they should be promoted, renamed, merged or kept internal.

The presentation shell must not become a generic render graph merely because a host boundary exists.

## Primary use cases

The shell must eventually support at least:

```text
Arcade-launched legacy games
standalone games
pause overlays
settings overlays
game-over overlays
high-res HUDs
controller hints
notifications/toasts
loading/transition screens
debug overlays
tooling/debug menus
optional decorative framing
```

The same architecture should work for Snake, Breakout, Pong, Tetris, Space Invaders, Smart Boy Hero and future GPE games without introducing one host implementation per game.

## Adoption goal

A simple existing game should be hostable with very little consumer plumbing.

Conceptually:

```rust
let shell = GamePresentationShell::new(game, game_size)
    .host_size(Size::new(1280, 720));
```

or an equivalent explicit Rust API.

The exact fluent shape is not prescribed.

The architectural requirement is:

> the consumer should not construct child `Frame`s, manage low-resolution framebuffer ownership, calculate integer presentation geometry, or manually remap host input just to host a game.

## Presentation policies

The shell must support explicit presentation policy rather than hidden assumptions.

At minimum future policy space should be able to express:

```text
integer-nearest fit
centered letterbox/pillarbox
custom content bounds
background/fill policy
safe areas
optional decorative frame regions
```

Stretching pixel content non-uniformly must never be the accidental default.

## Input mapping

This is a mandatory correctness boundary.

Keyboard and gamepad semantic states are generally resolution-independent.

Pointer/touch positions are not.

The shell must define explicit host-space -> game-space mapping using the resolved presentation geometry.

Required behavior includes:

```text
host pointer inside game surface -> mapped game coordinate
host pointer in letterbox/frame area -> no game coordinate
touch id/phase preserved while position is mapped
keyboard state preserved
gamepad state preserved
text/wheel policy explicit
```

No consumer should manually reinvent this mapping.

## High-resolution UI layer

GPE.UI rendered above or around the game surface must be able to use host resolution directly.

This enables:

```text
crisp outline fonts
modern menu presentation
high-res icons/images
controller glyphs
rich pause/settings menus
debug UI
high-res HUDs where desired
```

A game may still choose a fully pixel-art HUD inside its low-resolution surface.

The shell must support both approaches rather than mandate one visual style.

## Menu System relationship

The Presentation Shell and Menu System are separate first-class components that compose naturally.

```text
Presentation Shell = where/how game + UI are presented
Menu System        = interaction/content structure for menus
```

Typical composition:

```text
GamePresentationShell
├─ Game Surface
└─ Overlay Layer
   └─ Menu System
      ├─ Pause
      ├─ Settings
      └─ Confirm Quit
```

Neither system should absorb the other.

A Menu must remain usable without a Presentation Shell, and the Presentation Shell must remain useful without menus.

## Customization and visual quality

The shell must not hard-code a visual identity.

It should eventually allow consumer-defined presentation such as:

```text
minimal black letterbox
retro monitor frame
arcade cabinet composition
fantasy ornamental frame
sci-fi HUD surround
transparent/no frame
custom background artwork
animated transitions
```

Generic mechanisms may live in GPE/GPE.UI.

Game-specific art direction remains consumer-owned.

The objective is not one "GPE look".

The objective is:

> a reusable shell capable of anything from invisible plumbing to a visually striking WOW presentation.

## Lightweight effects boundary

Optional effects may eventually include:

```text
scanlines
vignette
subtle glow
CRT-like treatment
fade/dissolve transitions
background dim/blur equivalent where technically appropriate
```

But effects are subordinate to architecture.

Do not introduce a general post-processing framework before independent consumer evidence requires it.

Effects must be opt-in and should impose no cost on games that do not use them.

## HUD boundary

The shell may host high-resolution HUD/UI overlays, but gameplay HUDs are not automatically migrated out of games.

Valid models include:

```text
all-pixel HUD inside game surface
all-high-res HUD in host UI
hybrid HUD
```

The correct choice belongs to the game.

GPE must provide capability, not force homogenization.

## Debug UI relationship

Debug menus and overlays are a first-class use case.

The shell should make it trivial to expose an optional high-resolution debug layer over a running low-resolution game.

Potential future debug widgets include:

```text
FPS/frame timing
input state
entity counts
render diagnostics
sliders/toggles
teleport/select actions
runtime flags
logs/status
```

The same GPE.UI primitives used by production menus should power debug tooling where appropriate.

## Compatibility requirement

Existing single-framebuffer games must continue to work unchanged.

The Presentation Shell is opt-in.

No ordinary game should pay mandatory complexity or runtime cost merely because the capability exists.

Migration should be incremental:

```text
existing Game
    ↓ optional host
Game inside Presentation Shell
    ↓ optional UI layers
Pause / Settings / HUD / Debug
```

## DX requirements

The shell fails if tutorials need to teach consumers to manually:

```text
allocate nested framebuffers
construct child Frame values
forward storage/audio/delta-time
calculate viewport scaling
map pointer/touch coordinates
compose presentation pixels
manage pause/settings overlay plumbing
```

Those are engine/presentation responsibilities.

Consumer code should primarily describe:

```text
which game
logical game size
presentation policy
which UI layers
which theme/art assets
which semantic actions
```

## P7 proof matrix

Before stabilization, prove the concept on unlike examples.

Minimum evidence:

```text
1. Arcade launches one low-res game in the shell
2. Breakout hosted with modern high-res framing/UI while gameplay remains pixel-perfect
3. Pause menu overlay using the shared Menu System
4. Settings submenu with sliders/toggles
5. Pointer/touch mapping on a hosted game
6. Debug overlay on a running game
7. Native + wasm32 compile/runtime evidence where applicable
```

## P7 architecture questions

P7 must answer:

```text
Is PixelGameHost the right public abstraction or only an internal mechanism?
Should input remapping live on Input, PixelPresentation, or the shell boundary?
How are overlays ordered without inventing a render graph?
What is the minimal layout API for game surface + side/top/bottom UI regions?
How are pause/settings stacks composed with the Menu System?
How are high-res HUD and game-space HUD allowed to coexist?
Which presentation effects are generic enough to promote?
```

## Non-goals

This charter does not authorize by itself:

```text
full scene graph
general GPU render graph
arbitrary compositing engine
mandatory high-resolution conversion of every game
mandatory outline fonts
mandatory CRT effects
one universal visual theme
rewriting all existing games
```

## Definition of success

The Presentation Shell succeeds when an existing GPE game can remain logically pixel-native while gaining a modern host presentation with very little integration code.

The developer experience should feel like:

> "My game owns gameplay. GPE owns the boring presentation plumbing. GPE.UI gives me the expressive layer to make the surrounding experience polished or spectacular."

The visual range should span:

```text
invisible host
↓
clean modern frame
↓
rich game-specific HUD/menu shell
↓
Arcade-grade WOW presentation
```

without changing the underlying architectural model.
