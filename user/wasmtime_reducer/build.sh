#!/usr/bin/env bash
set -euo pipefail

# Activate environment: export CPP="/opt/wasi-sdk/bin/clang++" &&  export CC="/opt/wasi-sdk/bin/clang"
# Before compiling other modules: unset CC && export CC=/usr/bin/clang && echo "CC is now: $CC" 



# Ensure we run from the script directory so relative paths work
cd "$(dirname "$0")"

# Compile C to WASM with optimizations and specified macros
$CC reducer.c -o reducer.wasm -O3 \
  -DMAX_WORD_LENGTH=20 \
  -DMAX_WORDS=1000 \
  -DMAX_SLOT_NUM=100 \
  -DMAX_BUFFER_SIZE=50000
# $CPP reducer_new.cpp -o reducer.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast

wasmtime compile --target x86_64-unknown-none -W threads=n,tail-call=n reducer.wasm

cargo build --target x86_64-unknown-none --release && cc \
  -Wl,--gc-sections -nostdlib \
  -Wl,--whole-archive \
  target/x86_64-unknown-none/release/libwasmtime_reducer.a \
  -Wl,--no-whole-archive \
  -shared \
  -o target/x86_64-unknown-none/release/libwasmtime_reducer.so

# Create symlink, handling the case where it already exists
SYMLINK_PATH="/home/as-group/wyd/final/AlloyStack/target/release/libwasmtime_reducer.so"
if [ -L "$SYMLINK_PATH" ]; then
  echo "Symlink already exists, updating..."
  rm "$SYMLINK_PATH"
fi

ln -s /home/as-group/wyd/final/AlloyStack/user/wasmtime_reducer/target/x86_64-unknown-none/release/libwasmtime_reducer.so /home/as-group/wyd/final/AlloyStack/target/release/libwasmtime_reducer.so


# cargo build --target x86_64-unknown-none && cc \
#   -Wl,--gc-sections -nostdlib \
#   -Wl,--whole-archive \
#   target/x86_64-unknown-none/debug/libwasmtime_reducer.a \
#   -Wl,--no-whole-archive \
#   -shared \
#   -o target/x86_64-unknown-none/debug/libwasmtime_reducer.so

# ln -s /home/wyj/dyx_workplace/mslibos/user/wasmtime_reducer/target/x86_64-unknown-none/debug/libwasmtime_reducer.so /home/wyj/dyx_workplace/mslibos/target/debug/libwasmtime_reducer.so
# # 设置编译参数
# CFLAGS="-DMAX_WORD_LENGTH=14 -DMAX_WORDS=743 -DMAX_SLOT_NUM=100 -DMAX_BUFFER_SIZE=50000"

# # 确保当前目录是脚本所在目录
# cd "$(dirname "$0")"

# # 编译C到WASM
# echo "编译C到WASM..."

# # 使用Emscripten
# emcc reducer.c -o reducer.wasm $CFLAGS \
#   -s STANDALONE_WASM=1 \
#   -s IMPORTED_MEMORY=1 \
#   -s ALLOW_MEMORY_GROWTH=1 \
#   -s ERROR_ON_UNDEFINED_SYMBOLS=0 \
#   -s "EXPORTED_FUNCTIONS=['_main']"

# # 使用WASI SDK的替代命令 (如果需要的话取消注释)
# # /opt/wasi-sdk/bin/clang reducer.c -o reducer.wasm $CFLAGS -O2

# # 编译WASM到CWASM
# echo "编译WASM到CWASM..."
# ~/.wasmtime/bin/wasmtime compile \
#   --target x86_64-unknown-none \
#   -W threads=n,tail-call=n \
#   reducer.wasm \
#   -o reducer.cwasm

# echo "编译完成!"
# echo "输出文件: $(pwd)/reducer.wasm, $(pwd)/reducer.cwasm"