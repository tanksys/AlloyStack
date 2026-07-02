#!/usr/bin/env bash
set -euo pipefail

WASI_CXX="${WASI_CXX:-${WASI_SDK_PATH:-/opt/wasi-sdk}/bin/clang++}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"
cd "$SCRIPT_DIR"

# $CPP spliter.cpp -o spliter.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast
"$WASI_CXX" spliter_ori.cpp -o spliter.wasm # -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast
# $CC spliter.c -o spliter.wasm
wasmtime compile --target x86_64-unknown-none -W threads=n,tail-call=n spliter.wasm

cargo build --target x86_64-unknown-none --release && cc \
  -Wl,--gc-sections -nostdlib \
  -Wl,--whole-archive \
  target/x86_64-unknown-none/release/libwasmtime_spliter.a \
  -Wl,--no-whole-archive \
  -shared \
  -o target/x86_64-unknown-none/release/libwasmtime_spliter.so


ln -sfn "$SCRIPT_DIR/target/x86_64-unknown-none/release/libwasmtime_spliter.so" \
  "$REPO_ROOT/target/release/libwasmtime_spliter.so"
