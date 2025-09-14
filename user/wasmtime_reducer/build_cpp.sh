#!/usr/bin/env bash
set -euo pipefail

# Activate environment: export CPP="/opt/wasi-sdk/bin/clang++" &&  export CC="/opt/wasi-sdk/bin/clang"
# Before compiling other modules: unset CPP && export CPP=/usr/bin/clang && echo "CPP is now: $CPP" 

cd "$(dirname "$0")"
$CPP reducer_new.cpp -o reducer.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast -DMAX_WORD_LENGTH=20 -DMAX_WORDS=18000000 -DMAX_SLOT_NUM=10 -DMAX_BUFFER_SIZE=500000
wasmtime compile --target x86_64-unknown-none -W threads=n,tail-call=n reducer.wasm

cargo build --target x86_64-unknown-none --release && cc \
  -Wl,--gc-sections -nostdlib \
  -Wl,--whole-archive \
  target/x86_64-unknown-none/release/libwasmtime_reducer.a \
  -Wl,--no-whole-archive \
  -shared \
  -o target/x86_64-unknown-none/release/libwasmtime_reducer.so

  SYMLINK_PATH="./target/release/libwasmtime_reducer.so"
if [ -L "$SYMLINK_PATH" ]; then
  echo "Symlink already exists, updating..."
  rm "$SYMLINK_PATH"
fi

ln -s ./user/wasmtime_reducer/target/x86_64-unknown-none/release/libwasmtime_reducer.so ./target/release/libwasmtime_reducer.so