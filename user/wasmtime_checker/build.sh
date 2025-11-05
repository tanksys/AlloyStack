# $CPP checker.cpp -o checker.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast
$CPP checker_ori.cpp -o checker.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast
# $CC checker.c -o checker.wasm
wasmtime compile --target x86_64-unknown-none -W threads=n,tail-call=n checker.wasm

cargo build --target x86_64-unknown-none --release && cc \
  -Wl,--gc-sections -nostdlib \
  -Wl,--whole-archive \
  target/x86_64-unknown-none/release/libwasmtime_checker.a \
  -Wl,--no-whole-archive \
  -shared \
  -o target/x86_64-unknown-none/release/libwasmtime_checker.so

ln -s /home/wyj/dyx_workplace/mslibos/user/wasmtime_checker/target/x86_64-unknown-none/release/libwasmtime_checker.so /home/wyj/dyx_workplace/mslibos/target/release/libwasmtime_checker.so

