#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"
source scripts/web-examples.sh

build_web_example() {
    local example="$1"

    echo "==> build ${example}"
    cargo build --target wasm32-unknown-unknown --example "${example}"
    wasm-bindgen \
        --target web \
        --out-dir web/pkg \
        "target/wasm32-unknown-unknown/debug/examples/${example}.wasm"
}

for example in "${GPE_WEB_GAME_EXAMPLES[@]}"; do
    build_web_example "${example}"
done

echo "==> build web_demo"
cargo build --target wasm32-unknown-unknown --example web_demo
