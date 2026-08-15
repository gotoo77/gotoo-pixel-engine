#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

WEB_EXAMPLES=(
    web_demo
    snake_web
    breakout_web
    tetris_web
    pong_web
    space_invaders_web
    arcade_web
)

for example in "${WEB_EXAMPLES[@]}"; do
    echo "==> ${example}"
    cargo build --target wasm32-unknown-unknown --example "${example}"
done

echo "==> OK"
