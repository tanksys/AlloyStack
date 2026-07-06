#!/usr/bin/env bash
set -euo pipefail

WASI_CXX="${WASI_CXX:-${WASI_SDK_PATH:-/opt/wasi-sdk}/bin/clang++}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"
cd "$SCRIPT_DIR"

"$WASI_CXX" trans_data.cpp -o trans_data.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast

wasmtime compile --target x86_64-unknown-none -W threads=n,tail-call=n trans_data.wasm

cargo build --target x86_64-unknown-none --release && cc \
  -Wl,--gc-sections -nostdlib \
  -Wl,--whole-archive \
  target/x86_64-unknown-none/release/libwasmtime_trans_data.a \
  -Wl,--no-whole-archive \
  -shared \
  -o target/x86_64-unknown-none/release/libwasmtime_trans_data.so

ln -sfn "$SCRIPT_DIR/target/x86_64-unknown-none/release/libwasmtime_trans_data.so" \
  "$REPO_ROOT/target/release/libwasmtime_trans_data.so"
