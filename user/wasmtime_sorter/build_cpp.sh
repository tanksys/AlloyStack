#!/usr/bin/env bash
set -euo pipefail

# Activate environment: export CPP="/opt/wasi-sdk/bin/clang++" &&  export CC="/opt/wasi-sdk/bin/clang"
# Before compiling other modules: unset CPP && export CPP=/usr/bin/clang++ && echo "CPP is now: $CPP" 

cd "$(dirname "$0")"
$CPP sorter.cpp -o sorter.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast -DMAX_ARRAY_LENGTH=1600000   -DMAX_BUFFER_SIZE=30000000  
wasmtime compile --target x86_64-unknown-none -W threads=n,tail-call=n sorter.wasm

cargo build --target x86_64-unknown-none --release && cc \
  -Wl,--gc-sections -nostdlib \
  -Wl,--whole-archive \
  target/x86_64-unknown-none/release/libwasmtime_sorter.a \
  -Wl,--no-whole-archive \
  -shared \
  -o target/x86_64-unknown-none/release/libwasmtime_sorter.so

  SYMLINK_PATH="./target/release/libwasmtime_sorter.so"
if [ -L "$SYMLINK_PATH" ]; then
  echo "Symlink already exists, updating..."
  rm "$SYMLINK_PATH"
fi

ln -s ./user/wasmtime_sorter/target/x86_64-unknown-none/release/libwasmtime_sorter.so ./target/release/libwasmtime_sorter.so