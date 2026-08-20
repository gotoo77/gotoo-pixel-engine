# Void Canticle choice asset overrides

VC2.9 can override the presentation attached to each stable choice identity without changing Rust gameplay code.

Each choice may independently override:

- `icon`: PNG art;
- `hover_sfx`: WAV played when focus moves to the choice;
- `confirm_sfx`: WAV played when the choice is confirmed.

Every field is optional. Missing, unreadable or invalid asset files keep the existing procedural art or synthesized default SFX.

## Default location

The resolver reads `manifest.json` from this directory and loads whichever referenced assets are valid.

Preferred descriptor format:

```json
{
  "death_nova": {
    "icon": "death_nova.png",
    "hover_sfx": "death_nova_hover.wav",
    "confirm_sfx": "death_nova_confirm.wav"
  },
  "vital_spark": {
    "icon": "vital_spark.png"
  }
}
```

The VC2.7 shorthand remains supported for backwards compatibility:

```json
{
  "death_nova": "death_nova.png"
}
```

That shorthand is equivalent to:

```json
{
  "death_nova": {
    "icon": "death_nova.png"
  }
}
```

## VC2.9 manifest validation

The checked-in manifest is validated by the Void Canticle example tests, so the existing native CI rejects malformed repository descriptors before they can ship.

The canonical manifest contract is strict:

- top-level JSON value must be an object;
- choice IDs must be one of the stable IDs known by VC;
- descriptor fields are limited to `icon`, `hover_sfx` and `confirm_sfx`;
- a descriptor object must contain at least one asset field;
- every present asset field must be a non-empty string;
- asset paths are relative to this catalog, use `/` separators and may contain normal subdirectories;
- absolute paths, backslashes, `.` / `..`, empty path components, URL/query syntax and drive-like `:` paths are rejected;
- `icon` paths end in lowercase `.png`;
- `hover_sfx` and `confirm_sfx` paths end in lowercase `.wav`.

Referenced PNG/WAV files are still optional. Validation checks the descriptor contract, not asset existence, so artists can prepare the manifest before the final files land.

Runtime loading intentionally remains tolerant for external development overrides: an unavailable or undecodable optional file falls back independently instead of preventing the game from starting.

## Fallback contract

Overrides are resolved independently.

```text
icon absent / unreadable / invalid
    -> procedural choice art

hover_sfx absent / unreadable / invalid
    -> synthesized default hover SFX

confirm_sfx absent / unreadable / invalid
    -> synthesized default confirm SFX
```

A broken optional asset must not prevent Void Canticle from starting and must not disable the other valid assets in the same descriptor.

## PNG contract

- RGB or RGBA PNG; grayscale variants are accepted as well.
- Alpha is preserved; transparent pixels leave the framebuffer untouched.
- Sprites are rendered at native 1:1 presentation-space size and centered in the existing icon slot.
- Keep icon sprites roughly within the current procedural art footprint (about 48-64 px) unless the card layout is intentionally changed.
- Chassis illustrations can be larger, but should still fit their existing showcase slot.

## WAV contract

Choice SFX use the same WAV contract as GPE audio:

- signed 16-bit PCM;
- mono or stereo;
- 44.1 kHz or 48 kHz;
- at least one sample.

Unsupported or malformed WAV files are ignored and fall back to the synthesized default SFX.

## Alternate native development directory

Set `GPE_VC_CHOICE_ASSET_DIR` to point at another directory containing a compatible `manifest.json` and optional PNG/WAV files.

This is useful for iterating directly on Aseprite and audio exports without copying them into the repository or rebuilding Rust after every asset edit.

The catalog is loaded once during startup, so restart the game after changing an override.

## Web

`void_canticle_web` fetches the same `manifest.json` before starting VC2.9, then independently fetches the PNG/WAV files referenced by each descriptor.

`scripts/dev.py build-web` and `serve-web` mirror this source directory into `web/assets/void_canticle/ui/choice`, while the Pages build mirrors it into `dist/assets/void_canticle/ui/choice`.

Missing or invalid optional asset files remain valid on Web: each field falls back independently.

Typical local workflow:

```text
python3 scripts/dev.py build-web
python3 scripts/dev.py serve-web
```

Then open `http://127.0.0.1:8000/void_canticle.html`.
