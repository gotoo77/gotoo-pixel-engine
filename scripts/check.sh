#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

echo "==> cargo fmt --check"
cargo fmt --check

echo "==> cargo test --all-targets"
cargo test --all-targets

echo "==> cargo clippy --all-targets -- -D warnings"
cargo clippy --all-targets -- -D warnings

echo "==> git diff --check"
git diff --check

echo "==> OK"
