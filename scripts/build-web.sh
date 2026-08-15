#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

build_web_example() {
    local example="$1"

    echo "==> build ${example}"
    cargo build --target wasm32-unknown-unknown --example "${example}"
    wasm-bindgen \
        --target web \
        --out-dir web/pkg \
        "target/wasm32-unknown-unknown/debug/examples/${example}.wasm"
}

build_web_example snake_web
build_web_example breakout_web
build_web_example tetris_web
build_web_example pong_web
build_web_example space_invaders_web
build_web_example arcade_web

echo "==> build web_demo"
cargo build --target wasm32-unknown-unknown --example web_demo
