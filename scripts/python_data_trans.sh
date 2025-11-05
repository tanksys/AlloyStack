#!/bin/bash

WORKDIR="/home/wyj/dyx_workplace/mslibos"

# 定义执行次数
EXECUTIONS=10

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

# 初始化变量来累加 total_dur 的值
total_dur_sum=0
total_trans_time=0

total_register_time=0
total_access_time=0

# 循环执行十次
for (( i=1; i<=EXECUTIONS; i++ ))
do
    echo "Running iteration $i..."

    # 运行项目并提取 "total_dur(ms)" 的值
    # output=$(cargo run --release  -- --preload --metrics all --files ./isol_config/wasmtime_cpython_pass_args.json 2>&1)
    output=$(cargo run --release  -- --preload --metrics all --files ./isol_config/wasmtime_cpython_pass_args_inner.json 2>&1)
    total_dur=$(echo "$output" | grep -o '"total_dur(ms)": [0-9.]*' | awk -F': ' '{print $2}')
    start_time=$(echo "$output" | grep -o 'start time: [0-9.]*' | awk -F': ' '{print $2}')
    end_time=$(echo "$output" | grep -o 'end time: [0-9.]*' | awk -F': ' '{print $2}')
    total_size=$(echo "$output" | grep -o 'buffer_size: [0-9]*' | awk -F': ' '{print $2 / 1024}')

    start_time1=$(echo "$output" | grep -o 'start1: [0-9.]*' | awk -F': ' '{print $2}')
    end_time1=$(echo "$output" | grep -o 'end1: [0-9.]*' | awk -F': ' '{print $2}')

    start_time2=$(echo "$output" | grep -o 'start2: [0-9.]*' | awk -F': ' '{print $2}')
    end_time2=$(echo "$output" | grep -o 'end2: [0-9.]*' | awk -F': ' '{print $2}')


    # 提取 trans data time
    trans_data_time=$(echo "$end_time * 1000 - $start_time * 1000" | bc)
    # register time
    trans_data_time1=$(echo "$end_time1 * 1000 - $start_time1 * 1000" | bc)
    # access time
    trans_data_time2=$(echo "$end_time2 * 1000 - $start_time2 * 1000" | bc)

    total_dur_sum=$(echo "$total_dur_sum + $total_dur" | bc)
    total_trans_time=$(echo "$total_trans_time + $trans_data_time" | bc)
    total_register_time=$(echo "$total_register_time + $trans_data_time1" | bc)
    total_access_time=$(echo "$total_access_time + $trans_data_time2" | bc)


    # 打印结果
    echo "$i: trans time: $trans_data_time ms register time: $trans_data_time1 ms access time: $trans_data_time2 ms buffer size: $total_size KB"
done

# 计算平均值
average_total_dur=$(echo "scale=3; $total_dur_sum / $EXECUTIONS" | bc)
echo "Average Total Dur (ms): $average_total_dur"
average_total_trans=$(echo "scale=3; $total_trans_time / $EXECUTIONS" | bc)
echo "Average Total Trans (ms): $average_total_trans"
average_total_register=$(echo "scale=3; $total_register_time / $EXECUTIONS" | bc)
echo "Average Total Register (ms): $average_total_register"
average_total_access=$(echo "scale=3; $total_access_time / $EXECUTIONS" | bc)
echo "Average Total Access (ms): $average_total_access"