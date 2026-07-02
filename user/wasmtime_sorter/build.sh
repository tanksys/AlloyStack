#!/usr/bin/env bash
set -euo pipefail

WASI_CXX="${WASI_CXX:-${WASI_SDK_PATH:-/opt/wasi-sdk}/bin/clang++}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd)"
cd "$SCRIPT_DIR"

# Keep the sorter optimization level aligned with the native musl C build.
"$WASI_CXX" sorter_ori.cpp -o sorter.wasm -fno-exceptions -fno-rtti -fomit-frame-pointer -O2
# $CC sorter.c -o sorter.wasm
wasmtime compile --target x86_64-unknown-none -W threads=n,tail-call=n sorter.wasm

cargo build --target x86_64-unknown-none --release && cc \
  -Wl,--gc-sections -nostdlib \
  -Wl,--whole-archive \
  target/x86_64-unknown-none/release/libwasmtime_sorter.a \
  -Wl,--no-whole-archive \
  -shared \
  -o target/x86_64-unknown-none/release/libwasmtime_sorter.so


ln -sfn "$SCRIPT_DIR/target/x86_64-unknown-none/release/libwasmtime_sorter.so" \
  "$REPO_ROOT/target/release/libwasmtime_sorter.so"



# cargo build --target x86_64-unknown-none && cc \
#   -Wl,--gc-sections -nostdlib \
#   -Wl,--whole-archive \
#   target/x86_64-unknown-none/debug/libwasmtime_sorter.a \
#   -Wl,--no-whole-archive \
#   -shared \
#   -o target/x86_64-unknown-none/debug/libwasmtime_sorter.so

# ln -s /home/wyj/dyx_workplace/mslibos/user/wasmtime_sorter/target/x86_64-unknown-none/debug/libwasmtime_sorter.so /home/wyj/dyx_workplace/mslibos/target/debug/libwasmtime_sorter.so
