# Activate environment：export CPP="/opt/wasi-sdk/bin/clang++" &&  export CC="/opt/wasi-sdk/bin/clang"
# Before compiling other modules: unset CC && export CC=/usr/bin/clang && echo "CC is now: $CC" 

# Ensure we run from the script directory so relative paths work
cd "$(dirname "$0")"

# C1
$CC checker.c -o checker.wasm -O3 -DMAX_ARRAY_LENGTH=1600000   -DMAX_BUFFER_SIZE=15000000  

# C3
# $CC checker.c -o checker.wasm -O3 -DMAX_ARRAY_LENGTH=8000000   -DMAX_BUFFER_SIZE=80000000  

# C5
# $CC checker.c -o checker.wasm -O3 -DMAX_ARRAY_LENGTH=8000000     -DMAX_BUFFER_SIZE=80000000   

# $CPP checker.cpp -o checker.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast  -DMAX_ARRAY_LENGTH=1600000 -DMAX_BUFFER_SIZE=15000000  

wasmtime compile --target x86_64-unknown-none -W threads=n,tail-call=n checker.wasm
# $CPP checker.cpp -o checker.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast
# $CPP checker_ori.cpp -o checker.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast
# $CC checker.c -o checker.wasm


cargo build --target x86_64-unknown-none --release && cc \
  -Wl,--gc-sections -nostdlib \
  -Wl,--whole-archive \
  target/x86_64-unknown-none/release/libwasmtime_checker.a \
  -Wl,--no-whole-archive \
  -shared \
  -o target/x86_64-unknown-none/release/libwasmtime_checker.so

SYMLINK_PATH="./target/release/libwasmtime_checker.so"
if [ -L "$SYMLINK_PATH" ]; then
  echo "Symlink already exists, updating..."
  rm "$SYMLINK_PATH"
fi


ln -s ./user/wasmtime_checker/target/x86_64-unknown-none/release/libwasmtime_checker.so ./target/release/libwasmtime_checker.so

