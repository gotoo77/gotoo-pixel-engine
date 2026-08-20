# Void Canticle choice art overrides

VC2.7 can override the procedural art used by chassis, level-up, mutation and support choice cards with external PNG sprites on native builds.

## Default location

The resolver reads `manifest.json` from this directory and loads any PNG file that exists. Missing or invalid files are ignored and the procedural renderer remains the fallback.

Example:

```json
{
  "death_nova": "death_nova.png",
  "vital_spark": "vital_spark.png"
}
```

Dropping `death_nova.png` here and restarting the game replaces only the Death Nova card art. No Rust rebuild is required.

## PNG contract

- RGB or RGBA PNG; grayscale variants are accepted as well.
- Alpha is preserved; transparent pixels leave the framebuffer untouched.
- Sprites are rendered at native 1:1 presentation-space size and centered in the existing icon slot.
- Keep icon sprites roughly within the current procedural art footprint (about 48-64 px) unless the card layout is intentionally changed.
- Chassis illustrations can be larger, but should still fit their existing showcase slot.

## Alternate development directory

Set `GPE_VC_CHOICE_ASSET_DIR` to point at another directory containing a compatible `manifest.json` and PNG files.

This is useful for iterating on Aseprite exports without copying them into the repository on every test.

The catalog is loaded once when the process first renders choice art, so restart the game after changing an override.

## Web

WASM currently keeps the procedural fallback. External Web asset fetching/packaging is deliberately left for a separate integration step rather than coupling the VC-local catalog to a generic GPE asset manager prematurely.
