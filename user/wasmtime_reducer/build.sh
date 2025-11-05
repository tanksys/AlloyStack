# $CC reducer.c -o reducer.wasm
$CPP reducer_new.cpp -o reducer.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast

wasmtime compile --target x86_64-unknown-none -W threads=n,tail-call=n reducer.wasm

cargo build --target x86_64-unknown-none --release && cc \
  -Wl,--gc-sections -nostdlib \
  -Wl,--whole-archive \
  target/x86_64-unknown-none/release/libwasmtime_reducer.a \
  -Wl,--no-whole-archive \
  -shared \
  -o target/x86_64-unknown-none/release/libwasmtime_reducer.so

ln -s /home/wyj/dyx_workplace/mslibos/user/wasmtime_reducer/target/x86_64-unknown-none/release/libwasmtime_reducer.so /home/wyj/dyx_workplace/mslibos/target/release/libwasmtime_reducer.so


# cargo build --target x86_64-unknown-none && cc \
#   -Wl,--gc-sections -nostdlib \
#   -Wl,--whole-archive \
#   target/x86_64-unknown-none/debug/libwasmtime_reducer.a \
#   -Wl,--no-whole-archive \
#   -shared \
#   -o target/x86_64-unknown-none/debug/libwasmtime_reducer.so

# ln -s /home/wyj/dyx_workplace/mslibos/user/wasmtime_reducer/target/x86_64-unknown-none/debug/libwasmtime_reducer.so /home/wyj/dyx_workplace/mslibos/target/debug/libwasmtime_reducer.so
