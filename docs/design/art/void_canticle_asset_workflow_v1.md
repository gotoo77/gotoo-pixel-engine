# Void Canticle — Asset Workflow V1

Status: **VC1.9 concrete consumer**.

This workflow exists for the playable Grave Orbit slice. It is intentionally **not** a generic GPE asset manager, atlas format or animation framework.

## Goal

Move the authored enemy pixels out of Rust source code and into a real image asset that can be edited in Aseprite.

Current runtime path:

```text
Aseprite / PNG
      ↓
assets/void_canticle/grave_orbit_enemies.png
      ↓
VC1.9 PNG decoder
      ↓
7 fixed 32×32 cells
      ↓
GPE Sprite
      ↓
playable Grave Orbit
```

The same checked-in PNG is embedded for native and Web builds, so both targets render the same authored art.

## Sheet contract

File:

```text
assets/void_canticle/grave_orbit_enemies.png
```

Format:

- PNG;
- RGBA, 8 bits per channel;
- 224×32 px;
- seven fixed cells;
- each cell is exactly 32×32 px;
- transparent background;
- no padding between cells;
- no trimming.

Slot order from left to right:

| Cell | Enemy |
| ---: | --- |
| 0 | Carrion Drone |
| 1 | Grave Knight |
| 2 | Bell Wraith |
| 3 | Relic Carrier |
| 4 | Choir Node |
| 5 | Void Leech |
| 6 | Bellkeeper |

VC1.9 deliberately hard-codes this tiny contract. There is no JSON atlas because the first consumer does not need one.

## Editing directly in Aseprite

The checked-in PNG can be opened directly in Aseprite.

Recommended setup:

- grid: 32×32;
- keep the canvas at exactly 224×32;
- preserve transparent pixels;
- do not resize;
- do not reorder cells.

Saving the PNG in place is enough for the next build.

If layers or animation become necessary, keep an `.aseprite` source and export the final sheet to the same PNG path.

## Aseprite CLI export

A seven-frame 32×32 Aseprite source can be exported horizontally with:

```bash
aseprite -b grave_orbit_enemies.aseprite \
  --sheet assets/void_canticle/grave_orbit_enemies.png \
  --sheet-type horizontal
```

The export must stay untrimmed and without padding so the resulting sheet remains 224×32.

## Native no-rebuild art iteration

VC1.9 also has one **Void-Canticle-local** native override:

```bash
GPE_VC_ART_SHEET=/absolute/path/grave_orbit_enemies.png \
cargo run --release --example void_canticle
```

When `GPE_VC_ART_SHEET` is present, the native game reads that PNG at startup instead of the embedded sheet.

This means the art loop can be:

```text
edit/export PNG
→ restart the game
→ inspect
```

without recompiling Rust, as long as code did not change.

The override is intentionally startup-only; VC1.9 does not add filesystem watching or hot reload.

Web builds continue to use the embedded checked-in PNG because arbitrary local filesystem access is not available there.

## Failure behaviour

VC1.9 fails early with a clear error if the sheet:

- cannot be decoded as PNG;
- is not 8-bit RGBA;
- is not exactly 224×32.

This is preferable to silently drawing incorrect cells.

## Why the PNG decoder is a dependency

VC1.9 introduces the `png` crate for one concrete need: decoding the actual Grave Orbit sheet on native and Web.

It does **not** introduce:

- `AssetManager`;
- `TextureAtlas`;
- generic sprite metadata;
- generic animation state;
- asynchronous asset loading;
- filesystem abstraction.

Those abstractions need additional demonstrated consumers before they earn their place in GPE.

## Acceptance criterion

VC1.9 succeeds when all of the following are true:

1. removing/changing pixels in the PNG changes the rendered enemy art;
2. native and Web CI both decode the embedded sheet;
3. all seven enemy cells are extracted as GPE `Sprite`s;
4. the native override can load an edited Aseprite-exported PNG without a Rust rebuild;
5. gameplay, pacing and collision data remain unchanged.
