#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

echo "==> cargo fmt --check"
cargo fmt --check

echo "==> cargo test --lib --bins --examples --tests"
cargo test --lib --bins --examples --tests

echo "==> cargo clippy --all-targets -- -D warnings"
cargo clippy --all-targets -- -D warnings

echo "==> CARGO_LOCK_BASE64_BEGIN"
base64 -w0 Cargo.lock
echo
echo "==> CARGO_LOCK_BASE64_END"

echo "==> git diff --check"
git diff --check

echo "==> OK"
