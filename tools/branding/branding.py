#!/usr/bin/env python3
"""Reusable branding asset generator for GPE games.

The implementation is intentionally dependency-light and uses Pillow when available.
It generates web/PWA PNG variants and a multi-resolution Windows ICO from one square PNG.
Windows .res embedding remains a build concern and is handled by the reusable Rust build helper.
"""
from __future__ import annotations

import argparse
from pathlib import Path

try:
    from PIL import Image
except ImportError as exc:  # pragma: no cover - environment guard
    raise SystemExit("Pillow is required: python -m pip install Pillow") from exc

WINDOWS_SIZES = (16, 24, 32, 48, 64, 128, 256)
WEB_SIZES = (32, 180, 192, 512)


def load_square_png(path: Path) -> Image.Image:
    image = Image.open(path).convert("RGBA")
    if image.width != image.height:
        raise SystemExit(f"branding source must be square, got {image.width}x{image.height}")
    return image


def save_png(image: Image.Image, size: int, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    image.resize((size, size), Image.Resampling.LANCZOS).save(destination, format="PNG")


def generate(source: Path, out_dir: Path, stem: str) -> None:
    image = load_square_png(source)
    out_dir.mkdir(parents=True, exist_ok=True)

    ico_path = out_dir / f"{stem}.ico"
    image.save(ico_path, format="ICO", sizes=[(size, size) for size in WINDOWS_SIZES])

    for size in WEB_SIZES:
        save_png(image, size, out_dir / f"{stem}_{size}.png")

    print(f"source : {source}")
    print(f"output : {out_dir}")
    print(f"ico    : {ico_path}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate reusable branding assets from one square PNG")
    parser.add_argument("source", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--stem", default="app_icon")
    args = parser.parse_args()
    generate(args.source, args.out, args.stem)


if __name__ == "__main__":
    main()
