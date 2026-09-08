# GPE Branding Toolchain

Generate reusable application branding assets from one square PNG source.

## Usage

```bash
python tools/branding/branding.py source.png --out path/to/output --stem app_icon
```

Outputs:

- a multi-resolution Windows `.ico`;
- Web/PWA PNG variants at 32, 180, 192 and 512 px.

The source image must be square.

Requires Pillow:

```bash
python -m pip install Pillow
```
