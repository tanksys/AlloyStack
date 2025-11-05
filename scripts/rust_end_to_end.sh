#!/bin/bash

# 定义执行次数
EXECUTIONS=10

# 初始化变量来累加 total_dur 的值
total_dur_sum=0

# 检查是否有参数传递
if [ -z "$1" ]; then
  echo "Usage: $0 <command>"
  exit 1
fi

# 根据第一个参数决定执行哪个命令
case $1 in
  "long_chain")
    echo "Executing long_chain"
    # if [ -f "./user/array_sum/Cargo.toml" ]; then
    #     cargo clean --manifest-path ./user/array_sum/Cargo.toml
    # fi
    # cargo build --manifest-path ./user/array_sum/Cargo.toml --release
    
    # 循环执行十次
    for (( i=1; i<=EXECUTIONS; i++ ))
    do
        echo "Running iteration $i..."

        # 运行项目并提取 "total_dur(ms)" 的值
        case $2 in
            "n5")
                echo "Executing c1"
                # output=$(cargo run --release -- --metrics all --files ./isol_config/long_chain_c1.json 2>&1)
                # output=$(cargo run --release -- --preload --metrics all --files ./isol_config/long_chain_n5.json 2>&1)
                output=$(./target/release/msvisor --preload --metrics all --files ./isol_config/long_chain_n5.json 2>&1)
                ;;
            "n10")
                echo "Executing c3"
                # output=$(cargo run --release -- --metrics all --files ./isol_config/long_chain_c3.json 2>&1)
                output=$(cargo run --release -- --preload --metrics all --files ./isol_config/long_chain_n10.json 2>&1)
                ;;
            "n15")
                echo "Executing c5"
                # output=$(cargo run --release -- --metrics all --files ./isol_config/long_chain_c5.json 2>&1)
                output=$(cargo run --release -- --preload --metrics all --files ./isol_config/long_chain_n15.json 2>&1)
                ;;
            *)
                echo "Unknown command: $2"
                exit 1
                ;;
        esac
        total_dur=$(echo "$output" | grep -o '"total_dur(ms)": [0-9.]*' | awk -F': ' '{print $2}')

        # 保留三位小数，并进行四舍五入
        total_dur_rounded=$(printf "%.3f\n" "$total_dur")

        # 累加 total_dur 的值
        total_dur_sum=$(echo "$total_dur_sum + $total_dur_rounded" | bc)

        # 打印结果
        echo "Total Dur (ms): $total_dur_rounded"
    done
    # 计算平均值
    average_total_dur=$(echo "scale=3; $total_dur_sum / $EXECUTIONS" | bc)
    echo "Average Total Dur (ms): $average_total_dur"
    ;;
  "map_reduce")
    echo "Executing map_reduce"
    # if [ -f "./user/mapper/Cargo.toml" ]; then
    #     cargo clean --manifest-path ./user/mapper/Cargo.toml
    # fi
    # if [ -f "./user/reducer/Cargo.toml" ]; then
    #     cargo clean --manifest-path ./user/reducer/Cargo.toml
    # fi
    # cargo build --manifest-path ./user/mapper/Cargo.toml --release
    # cargo build --manifest-path ./user/reducer/Cargo.toml --release
    # 循环执行十次
    for (( i=1; i<=EXECUTIONS; i++ ))
    do
        echo "Running iteration $i..."

        # 运行项目并提取 "total_dur(ms)" 的值
        case $2 in
            "c1")
                echo "Executing c1"
                output=$(cargo run --release -- --metrics all --files ./isol_config/map_reduce_large_c1.json 2>&1)
                # output=$(cargo run --release -- --preload --metrics all --files ./isol_config/map_reduce_large_c1.json 2>&1)
                ;;
            "c3")
                echo "Executing c3"
                # output=$(cargo run --release -- --metrics all --files ./isol_config/map_reduce_large_c3.json 2>&1)
                output=$(./target/release/msvisor --metrics all --files ./isol_config/map_reduce_large_c3.json 2>&1)
                ;;
            "c5")
                echo "Executing c5"
                # output=$(cargo run --release -- --metrics all --files ./isol_config/map_reduce_large_c5.json 2>&1)
                output=$(cargo run --release -- --metrics all --files ./isol_config/map_reduce_large_c5.json 2>&1)
                ;;
            *)
                echo "Unknown command: $2"
                exit 1
                ;;
        esac
        total_dur=$(echo "$output" | grep -o '"total_dur(ms)": [0-9.]*' | awk -F': ' '{print $2}')

        # 保留三位小数，并进行四舍五入
        total_dur_rounded=$(printf "%.3f\n" "$total_dur")

        # 累加 total_dur 的值
        total_dur_sum=$(echo "$total_dur_sum + $total_dur_rounded" | bc)

        # 打印结果
        echo "Total Dur (ms): $total_dur_rounded"
    done
    # 计算平均值
    average_total_dur=$(echo "scale=3; $total_dur_sum / $EXECUTIONS" | bc)
    echo "Average Total Dur (ms): $average_total_dur"
    ;;
  "parallel_sort")
    echo "Executing parallel_sort"
    if [ -f "./user/sorter/Cargo.toml" ]; then
        cargo clean --manifest-path ./user/sorter/Cargo.toml
    fi
    if [ -f "./user/splitter/Cargo.toml" ]; then
        cargo clean --manifest-path ./user/splitter/Cargo.toml
    fi
    if [ -f "./user/merger/Cargo.toml" ]; then
        cargo clean --manifest-path ./user/merger/Cargo.toml
    fi
    if [ -f "./user/checker/Cargo.toml" ]; then
        cargo clean --manifest-path ./user/checker/Cargo.toml
    fi
    cargo build --manifest-path ./user/sorter/Cargo.toml --release
    cargo build --manifest-path ./user/splitter/Cargo.toml --release
    cargo build --manifest-path ./user/merger/Cargo.toml --release
    cargo build --manifest-path ./user/checker/Cargo.toml --release
    # 循环执行十次
    for (( i=1; i<=EXECUTIONS; i++ ))
    do
        echo "Running iteration $i..."

        # 运行项目并提取 "total_dur(ms)" 的值

        case $2 in
            "c1")
                echo "Executing c1"
                # output=$(cargo run --release -- --metrics all --files ./isol_config/parallel_sort_c1.json 2>&1)
                output=$(cargo run --release -- --preload --metrics all --files ./isol_config/parallel_sort_c1.json 2>&1)
                ;;
            "c3")
                echo "Executing c3"
                # output=$(cargo run --release -- --metrics all --files ./isol_config/parallel_sort_c3.json 2>&1)
                output=$(cargo run --release -- --preload --metrics all --files ./isol_config/parallel_sort_c3.json 2>&1)
                ;;
            "c5")
                echo "Executing c5"
                # output=$(cargo run --release -- --metrics all --files ./isol_config/parallel_sort_c5.json 2>&1)
                output=$(cargo run --release -- --preload --metrics all --files ./isol_config/parallel_sort_c5.json 2>&1)
                ;;
            *)
                echo "Unknown command: $2"
                exit 1
                ;;
        esac
        total_dur=$(echo "$output" | grep -o '"total_dur(ms)": [0-9.]*' | awk -F': ' '{print $2}')

        # 保留三位小数，并进行四舍五入
        total_dur_rounded=$(printf "%.3f\n" "$total_dur")

        # 累加 total_dur 的值
        total_dur_sum=$(echo "$total_dur_sum + $total_dur_rounded" | bc)

        # 打印结果
        echo "Total Dur (ms): $total_dur_rounded"
    done
    # 计算平均值
    average_total_dur=$(echo "scale=3; $total_dur_sum / $EXECUTIONS" | bc)
    echo "Average Total Dur (ms): $average_total_dur"
    ;;
  *)
    echo "Unknown command: $1"
    exit 1
    ;;
esac

