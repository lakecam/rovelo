#!/usr/bin/env bash
# Build crates/rovelo-wasm to wasm and generate JS glue into src/wasm/pkg.
set -euo pipefail
cd "$(dirname "$0")/../.."
cargo build -p rovelo-wasm --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/rovelo_wasm.wasm \
  --target web --out-dir web/src/wasm/pkg --out-name rovelo_wasm
