#!/usr/bin/env bash
set -euo pipefail

WASI_CXX="${WASI_CXX:-${WASI_SDK_PATH:-/opt/wasi-sdk}/bin/clang++}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"
cd "$SCRIPT_DIR"

"$WASI_CXX" mapper_new.cpp -o mapper.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast
# $CC mapper.c -o mapper.wasm -O3

wasmtime compile --target x86_64-unknown-none -W threads=n,tail-call=n mapper.wasm

cargo build --target x86_64-unknown-none --release && cc \
  -Wl,--gc-sections -nostdlib \
  -Wl,--whole-archive \
  target/x86_64-unknown-none/release/libwasmtime_mapper.a \
  -Wl,--no-whole-archive \
  -shared \
  -o target/x86_64-unknown-none/release/libwasmtime_mapper.so

ln -sfn "$SCRIPT_DIR/target/x86_64-unknown-none/release/libwasmtime_mapper.so" \
  "$REPO_ROOT/target/release/libwasmtime_mapper.so"

# cargo build --target x86_64-unknown-none && cc \
#   -Wl,--gc-sections -nostdlib \
#   -Wl,--whole-archive \
#   target/x86_64-unknown-none/debug/libwasmtime_mapper.a \
#   -Wl,--no-whole-archive \
#   -shared \
#   -o target/x86_64-unknown-none/debug/libwasmtime_mapper.so

# ln -s /home/wyj/dyx_workplace/mslibos/user/wasmtime_mapper/target/x86_64-unknown-none/debug/libwasmtime_mapper.so /home/wyj/dyx_workplace/mslibos/target/debug/libwasmtime_mapper.so
