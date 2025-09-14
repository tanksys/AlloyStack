# Activate environment：export CPP="/opt/wasi-sdk/bin/clang++" &&  export CC="/opt/wasi-sdk/bin/clang"
# Before compiling other modules: unset CC && export CC=/usr/bin/clang && echo "CC is now: $CC" 

# Ensure we run from the script directory so relative paths work
cd "$(dirname "$0")"

# C1
$CC spliter.c -o spliter.wasm -O3 -DMAX_ARRAY_LENGTH=1600000   -DMAX_BUFFER_SIZE=15000000  

# C3
# $CC spliter.c -o spliter.wasm -O3 -DMAX_ARRAY_LENGTH=8000000   -DMAX_BUFFER_SIZE=80000000   

# C5
# $CC spliter.c -o spliter.wasm -O3 -DMAX_ARRAY_LENGTH=8000000   -DMAX_BUFFER_SIZE=80000000  

# $CPP spliter.cpp -o spliter.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast  -DMAX_ARRAY_LENGTH=1600000 -DMAX_BUFFER_SIZE=15000000  

wasmtime compile --target x86_64-unknown-none -W threads=n,tail-call=n spliter.wasm
# $CPP spliter.cpp -o spliter.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast
# $CPP spliter_ori.cpp -o spliter.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast
# $CC spliter.c -o spliter.wasm


cargo build --target x86_64-unknown-none --release && cc \
  -Wl,--gc-sections -nostdlib \
  -Wl,--whole-archive \
  target/x86_64-unknown-none/release/libwasmtime_spliter.a \
  -Wl,--no-whole-archive \
  -shared \
  -o target/x86_64-unknown-none/release/libwasmtime_spliter.so

SYMLINK_PATH="./target/release/libwasmtime_spliter.so"
if [ -L "$SYMLINK_PATH" ]; then
  echo "Symlink already exists, updating..."
  rm "$SYMLINK_PATH"
fi


ln -s ./user/wasmtime_spliter/target/x86_64-unknown-none/release/libwasmtime_spliter.so ./target/release/libwasmtime_spliter.so

