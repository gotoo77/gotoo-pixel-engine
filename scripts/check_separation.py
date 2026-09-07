#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
ARCADE_OWNED_GAMES = (
    "snake",
    "breakout",
    "tetris",
    "pong",
    "space_invaders",
)


def fail(message: str) -> None:
    print(f"A9 separation guard: {message}", file=sys.stderr)
    raise SystemExit(1)


def main() -> int:
    examples = ROOT / "examples"
    tests = ROOT / "tests"

    for game in ARCADE_OWNED_GAMES:
        forbidden = [
            examples / f"{game}.rs",
            examples / f"{game}_web.rs",
            examples / game,
            tests / f"{game}_game.rs",
        ]
        for path in forbidden:
            if path.exists():
                fail(f"Arcade-owned game source returned to GPE: {path.relative_to(ROOT)}")

    cargo = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    dev = (ROOT / "scripts" / "dev.py").read_text(encoding="utf-8")
    for game in ARCADE_OWNED_GAMES:
        for needle in (game, f"{game}_web"):
            if needle in cargo:
                fail(f"Cargo.toml still declares Arcade-owned game target: {needle}")
            if needle in dev:
                fail(f"scripts/dev.py still owns Arcade game: {needle}")

    print("A9 separation guard: OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
