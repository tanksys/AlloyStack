#!/usr/bin/env bash
set -euo pipefail

# Activate environment: export CPP="/opt/wasi-sdk/bin/clang++" &&  export CC="/opt/wasi-sdk/bin/clang"
# Before compiling other modules: unset CPP && export CPP=/usr/bin/clang++ && echo "CPP is now: $CPP" 

cd "$(dirname "$0")"
$CPP merger.cpp -o merger.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast -DMAX_ARRAY_LENGTH=1600000   -DMAX_BUFFER_SIZE=15000000  

cargo build --target x86_64-unknown-none --release && cc \
  -Wl,--gc-sections -nostdlib \
  -Wl,--whole-archive \
  target/x86_64-unknown-none/release/libwasmtime_merger.a \
  -Wl,--no-whole-archive \
  -shared \
  -o target/x86_64-unknown-none/release/libwasmtime_merger.so

  SYMLINK_PATH="./target/release/libwasmtime_merger.so"
if [ -L "$SYMLINK_PATH" ]; then
  echo "Symlink already exists, updating..."
  rm "$SYMLINK_PATH"
fi

ln -s ./user/wasmtime_merger/target/x86_64-unknown-none/release/libwasmtime_merger.so ./target/release/libwasmtime_merger.so