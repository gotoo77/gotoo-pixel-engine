#!/usr/bin/env python3
from __future__ import annotations

import argparse
import base64
from functools import partial
import http.server
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import tarfile
from typing import Iterable
import zipfile

ROOT = Path(__file__).resolve().parents[1]
WEB_GAME_EXAMPLES = [
    "snake_web",
    "breakout_web",
    "tetris_web",
    "pong_web",
    "space_invaders_web",
    "arcade_web",
    "void_canticle_web",
]
PAGES_STATIC_FILES = [
    "index.html",
    "snake.html",
    "breakout.html",
    "tetris.html",
    "space_invaders.html",
    "pong.html",
    "void_canticle.html",
    "favicon.svg",
    "audio-unlock.js",
    "fullscreen.js",
]
GAME_CHOICES = [
    ("Arcade", "arcade"),
    ("Snake", "snake"),
    ("Space Invaders", "space_invaders"),
    ("Tetris", "tetris"),
    ("Pong", "pong"),
    ("Breakout", "breakout"),
    ("Void Canticle", "void_canticle"),
    ("Smart Boy Hero", "smart_boy_hero"),
    ("Smart Boy Hero ISO", "smart_boy_hero_iso"),
]
NATIVE_PACKAGES = {
    "void_canticle": {
        "slug": "void-canticle",
        "windows_binary": "VoidCanticle.exe",
        "linux_binary": "void_canticle",
        "runtime_paths": ["assets/void_canticle/ui/choice"],
    },
}


class CommandFailed(RuntimeError):
    def __init__(self, returncode: int):
        super().__init__(f"command failed with exit code {returncode}")
        self.returncode = returncode


def display_command(command: Iterable[object]) -> str:
    return " ".join(str(part) for part in command)


def run(command: list[str]) -> None:
    print(f"==> {display_command(command)}", flush=True)
    result = subprocess.run(command, cwd=ROOT, check=False)
    if result.returncode != 0:
        raise CommandFailed(result.returncode)


def cargo_build_web(example: str, *, release: bool) -> Path:
    command = ["cargo", "build"]
    if release:
        command.append("--release")
    command.extend(["--target", "wasm32-unknown-unknown", "--example", example])
    run(command)
    profile = "release" if release else "debug"
    return ROOT / "target" / "wasm32-unknown-unknown" / profile / "examples" / f"{example}.wasm"


def wasm_bindgen(wasm: Path, out_dir: Path) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    run(
        [
            "wasm-bindgen",
            "--target",
            "web",
            "--out-dir",
            str(out_dir),
            str(wasm),
        ]
    )


def sync_void_canticle_web_assets(destination_root: Path) -> None:
    source = ROOT / "assets" / "void_canticle" / "ui" / "choice"
    destination = destination_root / "assets" / "void_canticle" / "ui" / "choice"
    if destination.exists():
        shutil.rmtree(destination)
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copytree(source, destination)


def command_check(args: argparse.Namespace) -> None:
    run(["cargo", "fmt", "--check"])
    run(["cargo", "test", "--lib", "--bins", "--examples", "--tests"])
    run(["cargo", "clippy", "--all-targets", "--", "-D", "warnings"])
    if args.print_lock_base64:
        lock = (ROOT / "Cargo.lock").read_bytes()
        print("==> CARGO_LOCK_BASE64_BEGIN")
        print(base64.b64encode(lock).decode("ascii"))
        print("==> CARGO_LOCK_BASE64_END")
    run(["git", "diff", "--check"])
    print("==> OK")


def command_check_web(_: argparse.Namespace) -> None:
    for example in ["web_demo", *WEB_GAME_EXAMPLES]:
        cargo_build_web(example, release=False)
    print("==> OK")


def prepare_pages() -> Path:
    dist = ROOT / "dist"
    if dist.exists():
        shutil.rmtree(dist)
    pkg = dist / "pkg"
    pkg.mkdir(parents=True)
    for name in PAGES_STATIC_FILES:
        shutil.copy2(ROOT / "web" / name, dist / name)
    sync_void_canticle_web_assets(dist)
    return pkg


def command_build_web(args: argparse.Namespace) -> None:
    if args.pages:
        out_dir = prepare_pages()
        for example in WEB_GAME_EXAMPLES:
            wasm_bindgen(cargo_build_web(example, release=True), out_dir)
        print(f"==> Pages artifact ready: {ROOT / 'dist'}")
        return

    sync_void_canticle_web_assets(ROOT / "web")
    out_dir = ROOT / "web" / "pkg"
    for example in WEB_GAME_EXAMPLES:
        wasm_bindgen(cargo_build_web(example, release=args.release), out_dir)
    cargo_build_web("web_demo", release=args.release)
    print("==> OK")


def command_serve_web(args: argparse.Namespace) -> None:
    sync_void_canticle_web_assets(ROOT / "web")
    directory = str(ROOT / "web")
    handler = partial(http.server.SimpleHTTPRequestHandler, directory=directory)
    server = http.server.ThreadingHTTPServer((args.bind, args.port), handler)
    print(f"==> Serving {directory} on http://{args.bind}:{args.port}")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()


def available_games() -> list[tuple[str, str]]:
    return [
        (label, example)
        for label, example in GAME_CHOICES
        if (ROOT / "examples" / f"{example}.rs").is_file()
    ]


def select_game_with_fzf(games: list[tuple[str, str]]) -> str | None:
    if shutil.which("fzf") is None:
        return None
    labels = "\n".join(label for label, _ in games) + "\n"
    result = subprocess.run(
        ["fzf", "--height=40%", "--reverse", "--prompt=Game > "],
        cwd=ROOT,
        input=labels,
        text=True,
        stdout=subprocess.PIPE,
        check=False,
    )
    if result.returncode != 0:
        return ""
    return result.stdout.strip()


def select_game_interactively(games: list[tuple[str, str]]) -> str:
    print("Available games:")
    for index, (label, _) in enumerate(games, start=1):
        print(f"  {index}. {label}")
    try:
        raw = input("Game > ").strip()
    except EOFError:
        return ""
    if not raw:
        return ""
    try:
        index = int(raw) - 1
    except ValueError:
        return raw
    if 0 <= index < len(games):
        return games[index][0]
    return ""


def resolve_game(requested: str | None) -> str | None:
    games = available_games()
    if requested:
        normalized = requested.strip().casefold().replace("_", "-").replace(" ", "-")
        for label, example in games:
            aliases = {
                label.casefold().replace(" ", "-"),
                example.casefold().replace("_", "-"),
            }
            if normalized in aliases:
                return example
        raise ValueError(f"unknown or unavailable game: {requested}")

    selected = select_game_with_fzf(games)
    if selected is None:
        selected = select_game_interactively(games)
    if not selected:
        return None
    for label, example in games:
        if selected == label:
            return example
    return resolve_game(selected)


def command_run_game(args: argparse.Namespace) -> None:
    example = resolve_game(args.game)
    if example is None:
        return
    command = ["cargo", "run"]
    if args.release:
        command.append("--release")
    command.extend(["--example", example])
    run(command)


def native_package_target() -> tuple[str, str, str]:
    machine = platform.machine().casefold()
    if machine not in {"x86_64", "amd64"}:
        raise ValueError(
            f"native packaging currently supports x86_64 only, got {platform.machine()!r}"
        )

    if sys.platform == "win32":
        return "windows", "x86_64", ".exe"
    if sys.platform.startswith("linux"):
        return "linux", "x86_64", ""
    raise ValueError(
        f"native packaging currently supports Windows and Linux only, got {sys.platform!r}"
    )


def copy_native_runtime_path(relative_path: str, package_dir: Path) -> None:
    source = ROOT / relative_path
    if not source.exists():
        raise ValueError(f"runtime package path does not exist: {relative_path}")

    destination = package_dir / relative_path
    destination.parent.mkdir(parents=True, exist_ok=True)
    if source.is_dir():
        shutil.copytree(source, destination)
    else:
        shutil.copy2(source, destination)


def write_native_archive(package_dir: Path, archive_base: Path, os_name: str) -> Path:
    if os_name == "windows":
        archive_path = archive_base.with_suffix(".zip")
        with zipfile.ZipFile(archive_path, "w", compression=zipfile.ZIP_DEFLATED) as archive:
            for path in sorted(package_dir.rglob("*")):
                if path.is_file():
                    archive.write(path, path.relative_to(package_dir))
        return archive_path

    archive_path = archive_base.with_suffix(".tar.gz")
    with tarfile.open(archive_path, "w:gz") as archive:
        for path in sorted(package_dir.iterdir()):
            archive.add(path, arcname=path.name)
    return archive_path


def command_package_native(args: argparse.Namespace) -> None:
    example = resolve_game(args.game)
    if example is None:
        return
    package = NATIVE_PACKAGES.get(example)
    if package is None:
        raise ValueError(f"native packaging is not configured for game: {example}")

    os_name, arch, source_suffix = native_package_target()
    slug = str(package["slug"])
    binary_name = str(package[f"{os_name}_binary"])

    run(["cargo", "build", "--release", "--example", example])
    source_binary = ROOT / "target" / "release" / "examples" / f"{example}{source_suffix}"
    if not source_binary.is_file():
        raise ValueError(f"release binary was not produced: {source_binary}")

    output_dir = ROOT / "dist" / "native"
    output_dir.mkdir(parents=True, exist_ok=True)
    package_dir = output_dir / f"{slug}-{os_name}-{arch}"
    if package_dir.exists():
        shutil.rmtree(package_dir)
    package_dir.mkdir()

    shutil.copy2(source_binary, package_dir / binary_name)
    shutil.copy2(ROOT / "LICENSE", package_dir / "LICENSE")
    for relative_path in package["runtime_paths"]:
        copy_native_runtime_path(str(relative_path), package_dir)

    archive_path = write_native_archive(package_dir, package_dir, os_name)
    print(f"==> Native package ready: {archive_path}")


def command_list_web_examples(_: argparse.Namespace) -> None:
    for example in WEB_GAME_EXAMPLES:
        print(example)


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description="Portable GPE development commands")
    sub = root.add_subparsers(dest="command", required=True)

    check = sub.add_parser("check", help="run native format/tests/clippy validation")
    check.add_argument("--print-lock-base64", action="store_true")
    check.set_defaults(handler=command_check)

    check_web = sub.add_parser("check-web", help="compile all Web/WASM entrypoints")
    check_web.set_defaults(handler=command_check_web)

    build_web = sub.add_parser("build-web", help="build/package Web/WASM entrypoints")
    build_web.add_argument("--release", action="store_true")
    build_web.add_argument(
        "--pages",
        action="store_true",
        help="build release games and assemble the dist/ Pages artifact",
    )
    build_web.set_defaults(handler=command_build_web)

    serve = sub.add_parser("serve-web", help="serve the local web directory")
    serve.add_argument("--bind", default="0.0.0.0")
    serve.add_argument("--port", type=int, default=8000)
    serve.set_defaults(handler=command_serve_web)

    run_game = sub.add_parser("run-game", help="select or launch a native game")
    run_game.add_argument("game", nargs="?")
    run_game.add_argument("--release", action="store_true")
    run_game.set_defaults(handler=command_run_game)

    package_native = sub.add_parser(
        "package-native", help="build and package one native game for distribution"
    )
    package_native.add_argument("game")
    package_native.set_defaults(handler=command_package_native)

    list_web = sub.add_parser("list-web-examples", help=argparse.SUPPRESS)
    list_web.set_defaults(handler=command_list_web_examples)
    return root


def main() -> int:
    args = parser().parse_args()
    try:
        args.handler(args)
    except CommandFailed as error:
        return error.returncode
    except ValueError as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())