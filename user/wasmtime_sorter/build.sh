# Activate environment：export CPP="/opt/wasi-sdk/bin/clang++" &&  export CC="/opt/wasi-sdk/bin/clang"
# Before compiling other modules: unset CC && export CC=/usr/bin/clang && echo "CC is now: $CC" 

# Ensure we run from the script directory so relative paths work
cd "$(dirname "$0")"

# C1
$CC sorter.c -o sorter.wasm -O3 -DMAX_ARRAY_LENGTH=1600000   -DMAX_BUFFER_SIZE=15000000  

# C3
# $CC sorter.c -o sorter.wasm -O3 -DMAX_ARRAY_LENGTH=8000000   -DMAX_BUFFER_SIZE=80000000   

# C5
# $CC sorter.c -o sorter.wasm -O3 -DMAX_ARRAY_LENGTH=8000000   -DMAX_BUFFER_SIZE=80000000  

# $CPP sorter.cpp -o sorter.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast  -DMAX_ARRAY_LENGTH=1600000 -DMAX_BUFFER_SIZE=15000000  


wasmtime compile --target x86_64-unknown-none -W threads=n,tail-call=n sorter.wasm
# $CPP sorter.cpp -o sorter.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast
# $CPP sorter_ori.cpp -o sorter.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast
# $CC sorter.c -o sorter.wasm


cargo build --target x86_64-unknown-none --release && cc \
  -Wl,--gc-sections -nostdlib \
  -Wl,--whole-archive \
  target/x86_64-unknown-none/release/libwasmtime_sorter.a \
  -Wl,--no-whole-archive \
  -shared \
  -o target/x86_64-unknown-none/release/libwasmtime_sorter.so

SYMLINK_PATH="/home/as-group/wyd/final/AlloyStack/target/release/libwasmtime_sorter.so"
if [ -L "$SYMLINK_PATH" ]; then
  echo "Symlink already exists, updating..."
  rm "$SYMLINK_PATH"
fi


ln -s /home/as-group/wyd/final/AlloyStack/user/wasmtime_sorter/target/x86_64-unknown-none/release/libwasmtime_sorter.so /home/as-group/wyd/final/AlloyStack/target/release/libwasmtime_sorter.so



# cargo build --target x86_64-unknown-none && cc \
#   -Wl,--gc-sections -nostdlib \
#   -Wl,--whole-archive \
#   target/x86_64-unknown-none/debug/libwasmtime_sorter.a \
#   -Wl,--no-whole-archive \
#   -shared \
#   -o target/x86_64-unknown-none/debug/libwasmtime_sorter.so

# ln -s /home/wyj/dyx_workplace/mslibos/user/wasmtime_sorter/target/x86_64-unknown-none/debug/libwasmtime_sorter.so /home/wyj/dyx_workplace/mslibos/target/debug/libwasmtime_sorter.so
