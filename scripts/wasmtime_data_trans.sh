#!/bin/bash

# 定义执行次数
EXECUTIONS=10

# 初始化变量来累加 total_dur 的值
total_dur_sum1=0
total_dur_sum2=0
total_dur_sum34=0
total_dur_sum=0

# 循环执行十次
for (( i=1; i<=EXECUTIONS; i++ ))
do
    echo "Running iteration $i..."

    output=$(cargo run --release -- --preload --files ./isol_config/wasmtime_trans_data.json 2>&1)
    
    numbers=($(echo "$output" | grep -o '[0-9]\+\.[0-9]\+'))

    # 获取最后两个数字
    num0=${numbers[-4]}
    num1=${numbers[-3]}
    num2=${numbers[-2]}
    num3=${numbers[-1]}

    # 计算差值
    phase1=$(echo "$num1 * 1000 - $num0 * 1000" | bc)
    phase2=$(echo "$num2 * 1000 - $num1 * 1000" | bc)
    phase34=$(echo "$num3 * 1000 - $num2 * 1000" | bc)
    total=$(echo "$num3 * 1000 - $num0 * 1000" | bc)

    # 累加 total_dur 的值
    total_dur_sum1=$(echo "$total_dur_sum1 + $phase1" | bc)
    total_dur_sum2=$(echo "$total_dur_sum2 + $phase2" | bc)
    total_dur_sum34=$(echo "$total_dur_sum34 + $phase34" | bc)
    total_dur_sum=$(echo "$total_dur_sum + $total" | bc)

    # 打印结果
    echo "$i: phase1: $phase1 ms phase2: $phase2 ms phase3+4: $phase34 ms"
done


# 计算平均值
average_total_dur1=$(echo "scale=3; $total_dur_sum1 / $EXECUTIONS" | bc)
echo "Average Phase1 (ms): $average_total_dur1"
average_total_dur2=$(echo "scale=3; $total_dur_sum2 / $EXECUTIONS" | bc)
echo "Average Phase2 (ms): $average_total_dur2"
average_total_dur34=$(echo "scale=3; $total_dur_sum34 / $EXECUTIONS" | bc)
echo "Average Phase3+4 (ms): $average_total_dur34"
average_total_dur=$(echo "scale=3; $total_dur_sum / $EXECUTIONS" | bc)
echo "Average Total (ms): $average_total_dur"