#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

echo "==> snake_web"
cargo build --target wasm32-unknown-unknown --example snake_web

echo "==> web_demo"
cargo build --target wasm32-unknown-unknown --example web_demo

echo "==> OK"
