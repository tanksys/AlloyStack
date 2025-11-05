# $CPP spliter.cpp -o spliter.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast
$CPP spliter_ori.cpp -o spliter.wasm # -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast
# $CC spliter.c -o spliter.wasm
wasmtime compile --target x86_64-unknown-none -W threads=n,tail-call=n spliter.wasm

cargo build --target x86_64-unknown-none --release && cc \
  -Wl,--gc-sections -nostdlib \
  -Wl,--whole-archive \
  target/x86_64-unknown-none/release/libwasmtime_spliter.a \
  -Wl,--no-whole-archive \
  -shared \
  -o target/x86_64-unknown-none/release/libwasmtime_spliter.so


ln -s /home/wyj/dyx_workplace/mslibos/user/wasmtime_spliter/target/x86_64-unknown-none/release/libwasmtime_spliter.so /home/wyj/dyx_workplace/mslibos/target/release/libwasmtime_spliter.so

