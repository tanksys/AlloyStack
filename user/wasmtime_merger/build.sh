# $CPP merger.cpp -o merger.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast
$CPP merger_ori.cpp -o merger.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast
# $CC merger.c -o merger.wasm
wasmtime compile --target x86_64-unknown-none -W threads=n,tail-call=n merger.wasm

cargo build --target x86_64-unknown-none --release && cc \
  -Wl,--gc-sections -nostdlib \
  -Wl,--whole-archive \
  target/x86_64-unknown-none/release/libwasmtime_merger.a \
  -Wl,--no-whole-archive \
  -shared \
  -o target/x86_64-unknown-none/release/libwasmtime_merger.so

ln -s /home/wyj/dyx_workplace/mslibos/user/wasmtime_merger/target/x86_64-unknown-none/release/libwasmtime_merger.so /home/wyj/dyx_workplace/mslibos/target/release/libwasmtime_merger.so

