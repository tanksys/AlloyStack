$CPP trans_data.cpp -o trans_data.wasm -fno-exceptions -fno-rtti -ffast-math -funroll-loops -fomit-frame-pointer -Ofast

wasmtime compile --target x86_64-unknown-none -W threads=n,tail-call=n trans_data.wasm

cargo build --target x86_64-unknown-none --release && cc \
  -Wl,--gc-sections -nostdlib \
  -Wl,--whole-archive \
  target/x86_64-unknown-none/release/libwasmtime_trans_data.a \
  -Wl,--no-whole-archive \
  -shared \
  -o target/x86_64-unknown-none/release/libwasmtime_trans_data.so

ln -s /home/wyj/dyx_workplace/mslibos/user/wasmtime_trans_data/target/x86_64-unknown-none/release/libwasmtime_trans_data.so /home/wyj/dyx_workplace/mslibos/target/release/libwasmtime_trans_data.so
