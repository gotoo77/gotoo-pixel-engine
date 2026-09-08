#!/usr/bin/env python3
from __future__ import annotations

import argparse
import base64
from functools import partial
import http.server
from pathlib import Path
import shutil
import subprocess
import sys
from typing import Iterable

ROOT = Path(__file__).resolve().parents[1]
PAGES_STATIC_FILES = [
    "index.html",
    "smart_boy_hero.html",
    "smart_boy_hero_iso.html",
    "favicon.svg",
    "audio-unlock.js",
    "fullscreen.js",
    "diagnostics.js",
]


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


def check_rust_format() -> None:
    run(["cargo", "fmt", "--check"])


def command_fmt_check(_: argparse.Namespace) -> None:
    check_rust_format()
    print("==> OK")


def command_check(args: argparse.Namespace) -> None:
    run([sys.executable, str(ROOT / "scripts" / "check_separation.py")])
    check_rust_format()
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
    cargo_build_web("web_demo", release=False)
    print("==> OK")


def prepare_pages() -> Path:
    dist = ROOT / "dist"
    if dist.exists():
        shutil.rmtree(dist)
    pkg = dist / "pkg"
    pkg.mkdir(parents=True)
    for name in PAGES_STATIC_FILES:
        shutil.copy2(ROOT / "web" / name, dist / name)
    return pkg


def command_build_web(args: argparse.Namespace) -> None:
    if args.pages:
        prepare_pages()
        print(f"==> Pages shell ready: {ROOT / 'dist'}")
        return

    cargo_build_web("web_demo", release=args.release)
    print("==> OK")


def command_serve_web(args: argparse.Namespace) -> None:
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


def command_install_hooks(_: argparse.Namespace) -> None:
    run(["git", "config", "core.hooksPath", ".githooks"])
    print("==> Git hooks installed from .githooks")


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description="Portable GPE development commands")
    sub = root.add_subparsers(dest="command", required=True)

    check = sub.add_parser("check", help="run native format/tests/clippy validation")
    check.add_argument("--print-lock-base64", action="store_true")
    check.set_defaults(handler=command_check)

    fmt_check = sub.add_parser("fmt-check", help="check Rust formatting")
    fmt_check.set_defaults(handler=command_fmt_check)

    check_web = sub.add_parser("check-web", help="compile GPE Web/WASM engine demos")
    check_web.set_defaults(handler=command_check_web)

    build_web = sub.add_parser("build-web", help="build/package GPE Web/WASM surfaces")
    build_web.add_argument("--release", action="store_true")
    build_web.add_argument(
        "--pages",
        action="store_true",
        help="assemble the Pages shell; game artifacts come from Arcade/standalone repositories",
    )
    build_web.set_defaults(handler=command_build_web)

    serve = sub.add_parser("serve-web", help="serve the local web directory")
    serve.add_argument("--bind", default="0.0.0.0")
    serve.add_argument("--port", type=int, default=8000)
    serve.set_defaults(handler=command_serve_web)

    install_hooks = sub.add_parser("install-hooks", help="configure versioned Git hooks")
    install_hooks.set_defaults(handler=command_install_hooks)
    return root


def main() -> int:
    args = parser().parse_args()
    try:
        args.handler(args)
    except CommandFailed as error:
        return error.returncode
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
