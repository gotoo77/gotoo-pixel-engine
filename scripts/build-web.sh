#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

echo "==> build snake_web"
cargo build --target wasm32-unknown-unknown --example snake_web

echo "==> build web_demo"
cargo build --target wasm32-unknown-unknown --example web_demo
