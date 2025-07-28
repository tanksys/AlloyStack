#!/bin/bash
# filepath: \home\k423\Desktop\alloystacknew\AlloyStack\scripts\build_big_data_wasm.sh

C_CUSTOM_FLAGS="-DMAX_BUFFER_SIZE=1000000 -DMAX_SLOT_NUM=10 -DMAX_WORDS=18000000" #自己定义
TARGET="x86_64-unknown-none"
PROFILE="release"  # 或 "debug"
CC_FLAGS_P1="-Wl,--gc-sections -nostdlib -Wl,--whole-archive"
CC_FLAGS_P2="-Wl,--no-whole-archive -shared"

# 构建 mapper
cd user/wasmtime_mapper \
    && cargo build --release --target $TARGET \
    && cc $CC_FLAGS_P1 \
        target/$TARGET/$PROFILE/libwasmtime_mapper.a \
        $CC_FLAGS_P2 \
        $C_CUSTOM_FLAGS \
        -o target/$TARGET/$PROFILE/libwasmtime_mapper.so

# 构建 reducer
cd ../../user/wasmtime_reducer \
    && cargo build --release --target $TARGET \
    && cc $CC_FLAGS_P1 \
        target/$TARGET/$PROFILE/libwasmtime_reducer.a \
        $CC_FLAGS_P2 \
        $C_CUSTOM_FLAGS \
        -o target/$TARGET/$PROFILE/libwasmtime_reducer.so

# 创建符号链接
cd ../..
ln -s $(pwd)/user/wasmtime_mapper/target/$TARGET/$PROFILE/libwasmtime_mapper.so target/$PROFILE/
ln -s $(pwd)/user/wasmtime_reducer/target/$TARGET/$PROFILE/libwasmtime_reducer.so target/$PROFILE/

echo "Big data WASM modules built successfully!"