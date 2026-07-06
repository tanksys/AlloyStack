#!/usr/bin/env bash
set -euo pipefail

WASI_CC="${WASI_CC:-${WASI_SDK_PATH:-/opt/wasi-sdk}/bin/clang}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

cd "$SCRIPT_DIR"
"$WASI_CC" func.c -o func.wasm -O2
wasmtime compile --target x86_64-unknown-none -W threads=n,tail-call=n func.wasm
