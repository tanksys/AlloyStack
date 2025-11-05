#!/bin/bash

WORKDIR="/home/wyj/dyx_workplace/mslibos"

# 定义执行次数
EXECUTIONS=100

cd $WORKDIR/user/wasmtime_cpython
if [ -f "./Cargo.toml" ]; then
    cargo clean
fi

cargo build --release --target x86_64-unknown-none && cc \
  -Wl,--gc-sections -nostdlib \
  -Wl,--whole-archive \
  target/x86_64-unknown-none/release/libwasmtime_cpython.a \
  -Wl,--no-whole-archive \
  -shared \
  -o target/x86_64-unknown-none/release/libwasmtime_cpython.so

cd $WORKDIR
ln -s $WORKDIR/user/wasmtime_cpython/target/x86_64-unknown-none/release/libwasmtime_cpython.so target/release/libwasmtime_cpython.so

# 初始化变量来累加 total_cold 的值
total_cold_sum=0

# 循环执行十次
for (( i=1; i<=EXECUTIONS; i++ ))
do
    echo "Running iteration $i..."

    # 运行项目并提取 "total_dur(ms)" 的值
    start_time=$(date '+%s.%N')
    end_time=$(cargo run --release -- --files ./isol_config/wasmtime_cpython_time.json)

    # 提取 cold start time
    cold_start_time=$(echo "$end_time * 1000 - $start_time * 1000" | bc)
    total_cold_sum=$(echo "$total_cold_sum + $cold_start_time" | bc)

    # 打印结果
    echo "$i: $cold_start_time ms"
done

# 计算平均值
average_total_cold=$(echo "scale=3; $total_cold_sum / $EXECUTIONS" | bc)
echo "Average Total Cold (ms): $average_total_cold"
