#!/usr/bin/env bash
set -euo pipefail

WASI_CXX="${WASI_CXX:-${WASI_SDK_PATH:-/opt/wasi-sdk}/bin/clang++}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"
cd "$SCRIPT_DIR"

# $CC reducer.c -o reducer.wasm
"$WASI_CXX" reducer_new.cpp -o reducer.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast

wasmtime compile --target x86_64-unknown-none -W threads=n,tail-call=n reducer.wasm

cargo build --target x86_64-unknown-none --release && cc \
  -Wl,--gc-sections -nostdlib \
  -Wl,--whole-archive \
  target/x86_64-unknown-none/release/libwasmtime_reducer.a \
  -Wl,--no-whole-archive \
  -shared \
  -o target/x86_64-unknown-none/release/libwasmtime_reducer.so

ln -sfn "$SCRIPT_DIR/target/x86_64-unknown-none/release/libwasmtime_reducer.so" \
  "$REPO_ROOT/target/release/libwasmtime_reducer.so"


# cargo build --target x86_64-unknown-none && cc \
#   -Wl,--gc-sections -nostdlib \
#   -Wl,--whole-archive \
#   target/x86_64-unknown-none/debug/libwasmtime_reducer.a \
#   -Wl,--no-whole-archive \
#   -shared \
#   -o target/x86_64-unknown-none/debug/libwasmtime_reducer.so

# ln -s /home/wyj/dyx_workplace/mslibos/user/wasmtime_reducer/target/x86_64-unknown-none/debug/libwasmtime_reducer.so /home/wyj/dyx_workplace/mslibos/target/debug/libwasmtime_reducer.so
