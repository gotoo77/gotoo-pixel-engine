# Void Canticle choice art overrides

VC2.7 can override the procedural art used by chassis, level-up, mutation and support choice cards with external PNG sprites.

## Default location

The resolver reads `manifest.json` from this directory and loads any PNG file that exists. Missing or invalid files are ignored and the procedural renderer remains the fallback.

Example:

```json
{
  "death_nova": "death_nova.png",
  "vital_spark": "vital_spark.png"
}
```

Dropping `death_nova.png` here and restarting the native game replaces only the Death Nova card art. No Rust rebuild is required.

## PNG contract

- RGB or RGBA PNG; grayscale variants are accepted as well.
- Alpha is preserved; transparent pixels leave the framebuffer untouched.
- Sprites are rendered at native 1:1 presentation-space size and centered in the existing icon slot.
- Keep icon sprites roughly within the current procedural art footprint (about 48-64 px) unless the card layout is intentionally changed.
- Chassis illustrations can be larger, but should still fit their existing showcase slot.

## Alternate native development directory

Set `GPE_VC_CHOICE_ASSET_DIR` to point at another directory containing a compatible `manifest.json` and PNG files.

This is useful for iterating on Aseprite exports without copying them into the repository on every test.

The catalog is loaded once when the process first renders choice art, so restart the game after changing an override.

## Web

`void_canticle_web` fetches the same `manifest.json` and optional PNG files before starting VC2.7. `scripts/dev.py build-web` and `serve-web` mirror this source directory into `web/assets/void_canticle/ui/choice`, while the Pages build mirrors it into `dist/assets/void_canticle/ui/choice`.

Missing PNGs remain valid on Web as well: each card falls back to its procedural art.

Typical local workflow:

```text
python3 scripts/dev.py build-web
python3 scripts/dev.py serve-web
```

Then open `http://127.0.0.1:8000/void_canticle.html`.
