#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"
source scripts/web-examples.sh

for example in web_demo "${GPE_WEB_GAME_EXAMPLES[@]}"; do
    echo "==> ${example}"
    cargo build --target wasm32-unknown-unknown --example "${example}"
done

echo "==> OK"
