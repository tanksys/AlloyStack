# $CPP sorter.cpp -o sorter.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast
$CPP sorter_ori.cpp -o sorter.wasm -fno-exceptions -fno-rtti -ffast-math -fomit-frame-pointer -Ofast  #-funroll-loops
# $CC sorter.c -o sorter.wasm
wasmtime compile --target x86_64-unknown-none -W threads=n,tail-call=n sorter.wasm

cargo build --target x86_64-unknown-none --release && cc \
  -Wl,--gc-sections -nostdlib \
  -Wl,--whole-archive \
  target/x86_64-unknown-none/release/libwasmtime_sorter.a \
  -Wl,--no-whole-archive \
  -shared \
  -o target/x86_64-unknown-none/release/libwasmtime_sorter.so


ln -s /home/wyj/dyx_workplace/mslibos/user/wasmtime_sorter/target/x86_64-unknown-none/release/libwasmtime_sorter.so /home/wyj/dyx_workplace/mslibos/target/release/libwasmtime_sorter.so



# cargo build --target x86_64-unknown-none && cc \
#   -Wl,--gc-sections -nostdlib \
#   -Wl,--whole-archive \
#   target/x86_64-unknown-none/debug/libwasmtime_sorter.a \
#   -Wl,--no-whole-archive \
#   -shared \
#   -o target/x86_64-unknown-none/debug/libwasmtime_sorter.so

# ln -s /home/wyj/dyx_workplace/mslibos/user/wasmtime_sorter/target/x86_64-unknown-none/debug/libwasmtime_sorter.so /home/wyj/dyx_workplace/mslibos/target/debug/libwasmtime_sorter.so
