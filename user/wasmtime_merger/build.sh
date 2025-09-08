# 激活环境：source ~/.zshrc 
# 测试时记得重新运行  just c_wordcount


# Select toolchain via TOOLCHAIN env: wasi|emscripten (default: wasi)
TOOLCHAIN="${TOOLCHAIN:-wasi}"

case "$TOOLCHAIN" in
  wasi)
    export CC="${CC:-/opt/wasi-sdk/bin/clang}"
    export CPP="${CPP:-/opt/wasi-sdk/bin/clang++}"
    ;;
  emscripten)
    export CC="${CC:-emcc}"
    export CPP="${CPP:-em++}"
    ;;
  *)
    echo "Unknown TOOLCHAIN: $TOOLCHAIN"; exit 1
    ;;
 esac

echo "Using CC=$CC, CPP=$CPP"

# Ensure we run from the script directory so relative paths work
cd "$(dirname "$0")"

# C1
$CC merger.c -o merger.wasm -O3 -DMAX_ARRAY_LENGTH=1600000   -DMAX_BUFFER_SIZE=15000000  

# C3
# $CC merger.c -o merger.wasm -O3 -DMAX_ARRAY_LENGTH=8000000   -DMAX_BUFFER_SIZE=80000000   

# C5
# $CC merger.c -o merger.wasm -O3 -DMAX_ARRAY_LENGTH=8000000   -DMAX_BUFFER_SIZE=80000000 

# $CPP merger.cpp -o merger.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast  -DMAX_ARRAY_LENGTH=1600000 -DMAX_BUFFER_SIZE=15000000  

wasmtime compile --target x86_64-unknown-none -W threads=n,tail-call=n merger.wasm
# $CPP merger.cpp -o merger.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast
# $CPP merger_ori.cpp -o merger.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast
# $CC merger.c -o merger.wasm


cargo build --target x86_64-unknown-none --release && cc \
  -Wl,--gc-sections -nostdlib \
  -Wl,--whole-archive \
  target/x86_64-unknown-none/release/libwasmtime_merger.a \
  -Wl,--no-whole-archive \
  -shared \
  -o target/x86_64-unknown-none/release/libwasmtime_merger.so

SYMLINK_PATH="/home/as-group/wyd/final/AlloyStack/target/release/libwasmtime_merger.so"
if [ -L "$SYMLINK_PATH" ]; then
  echo "Symlink already exists, updating..."
  rm "$SYMLINK_PATH"
fi


ln -s /home/as-group/wyd/final/AlloyStack/user/wasmtime_merger/target/x86_64-unknown-none/release/libwasmtime_merger.so /home/as-group/wyd/final/AlloyStack/target/release/libwasmtime_merger.so

