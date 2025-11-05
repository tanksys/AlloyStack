#!/bin/bash

WORKDIR="/home/wyj/dyx_workplace/mslibos"

# 定义执行次数
EXECUTIONS=10

cd $WORKDIR/user/wasmtime_cpython_mapper
if [ -f "./Cargo.toml" ]; then
    cargo clean
fi
cargo build --release --target x86_64-unknown-none && cc \
    -Wl,--gc-sections -nostdlib \
    -Wl,--whole-archive \
    target/x86_64-unknown-none/release/libwasmtime_cpython_mapper.a \
    -Wl,--no-whole-archive \
    -shared \
    -o target/x86_64-unknown-none/release/libwasmtime_cpython_mapper.so

cd $WORKDIR/user/wasmtime_cpython_reducer
if [ -f "./Cargo.toml" ]; then
    cargo clean
fi
cargo build --release --target x86_64-unknown-none && cc \
    -Wl,--gc-sections -nostdlib \
    -Wl,--whole-archive \
    target/x86_64-unknown-none/release/libwasmtime_cpython_reducer.a \
    -Wl,--no-whole-archive \
    -shared \
    -o target/x86_64-unknown-none/release/libwasmtime_cpython_reducer.so

cd $WORKDIR
ln -s $WORKDIR/user/wasmtime_cpython_mapper/target/x86_64-unknown-none/release/libwasmtime_cpython_mapper.so target/release/libwasmtime_cpython_mapper.so
ln -s $WORKDIR/user/wasmtime_cpython_reducer/target/x86_64-unknown-none/release/libwasmtime_cpython_reducer.so target/release/libwasmtime_cpython_reducer.so

# 初始化变量来累加 total_phase 的值
total_phase1=0
total_phase2=0
total_phase3=0
total_phase4=0

# 循环执行十次
for (( i=1; i<=EXECUTIONS; i++ ))
do
    echo "Running iteration $i..."

    # 运行项目并提取 'phase' 的值
    output=$(cargo run --release  -- --metrics all --files ./isol_config/wasmtime_cpython_wordcount_c3.json 2>&1)
    phase0=$(echo "$output" | grep -o 'phase0: [0-9.]*' | awk -F': ' '{print $2}' | sort | head -n 1)
    phase1=$(echo "$output" | grep -o 'phase1: [0-9.]*' | awk -F': ' '{print $2}' | sort | head -n 1)
    phase2=$(echo "$output" | grep -o 'phase2: [0-9.]*' | awk -F': ' '{print $2}' | sort | head -n 1)
    phase3=$(echo "$output" | grep -o 'phase2: [0-9.]*' | awk -F': ' '{print $2}' | sort -r | head -n 1)
    phase4=$(echo "$output" | grep -o 'phase3: [0-9.]*' | awk -F': ' '{print $2}' | sort -r | head -n 1)

    # 提取 time
    p1=$(echo "$phase1 - $phase0" | bc)
    p2=$(echo "$phase2 - $phase1" | bc)
    p3=$(echo "$phase3 - $phase2" | bc)
    p4=$(echo "$phase4 - $phase3" | bc)

    total_phase1=$(echo "$total_phase1 + $p1" | bc)
    total_phase2=$(echo "$total_phase2 + $p2" | bc)
    total_phase3=$(echo "$total_phase3 + $p3" | bc)
    total_phase4=$(echo "$total_phase4 + $p4" | bc)

    # 打印结果
    echo "$i: $p1 s $p2 s $p3 s $p4 s"
done

# 计算平均值
average_total_phase1=$(echo "scale=6; $total_phase1 / $EXECUTIONS" | bc)
echo "Average Total Phase1 (s): $average_total_phase1"
average_total_phase2=$(echo "scale=6; $total_phase2 / $EXECUTIONS" | bc)
echo "Average Total Phase2 (s): $average_total_phase2"
average_total_phase3=$(echo "scale=6; $total_phase3 / $EXECUTIONS" | bc)
echo "Average Total Phase3 (s): $average_total_phase3"
average_total_phase4=$(echo "scale=6; $total_phase4 / $EXECUTIONS" | bc)
echo "Average Total Phase4 (s): $average_total_phase4"
