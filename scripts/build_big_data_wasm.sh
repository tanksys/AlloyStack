#!/bin/bash
# filepath: /home/as-group/wyd/final/AlloyStack/scripts/build_big_data_wasm.sh
# CUSTOM_FLAGS="-DMAX_WORD_LENGTH=14 -DMAX_WORDS=743 -DMAX_SLOT_NUM=100 -DMAX_BUFFER_SIZE=50000" # 根据 fake_data_0.txt 分析结果设置的最优参数

# # 清理之前的构建结果，确保重新编译
# echo "Cleaning previous build files..."
# rm -f user/wasmtime_mapper/mapper.wasm user/wasmtime_mapper/mapper.cwasm user/wasmtime_mapper/target/*/*/*.a user/wasmtime_mapper/target/*/*/*.so
# rm -f user/wasmtime_reducer/reducer.wasm user/wasmtime_reducer/reducer.cwasm user/wasmtime_reducer/target/*/*/*.a user/wasmtime_reducer/target/*/*/*.so

# 使用 justfile 中的 wasm_func 规则编译wasm模块
echo "Compiling WASM modules with custom parameters..."
# just c_custom_flags="$CUSTOM_FLAGS" wasm_func wasmtime_mapper
# just c_custom_flags="$CUSTOM_FLAGS" wasm_func wasmtime_reducer
just  wasm_func wasmtime_mapper
just  wasm_func wasmtime_reducer

# 测试一组最优参数
echo "Testing optimal parameters"

# 创建结果文件
result_file="test_results/optimal_params_average.md"
echo "# Optimal Parameters Test" > $result_file
echo "| Run | Duration (ms) | Execution Time (s) |" >> $result_file
echo "|-----|--------------|-------------------|" >> $result_file

total_duration=0
total_exec_time=0

# 运行测试 3 次并记录时间
for i in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20; do
  echo "Run $i"
  # 运行 wordcount 并捕获执行时间
  start_time=$(date +%s.%N)
  duration=$(target/release/asvisor --files isol_config/wasmtime_wordcount_c1.json --metrics total-dur 2>&1 | grep 'total_dur' | grep -oP '[\d.]+')
  end_time=$(date +%s.%N)
  execution_time=$(echo "$end_time - $start_time" | bc)
  
  # 添加到结果文件
  echo "| $i | $duration | $execution_time |" >> $result_file
  
  # 累计总时间
  total_duration=$(echo "$total_duration + $duration" | bc)
  total_exec_time=$(echo "$total_exec_time + $execution_time" | bc)
done

# 计算平均值
avg_duration=$(echo "scale=4; $total_duration / 20" | bc)
avg_exec_time=$(echo "scale=4; $total_exec_time / 20" | bc)

# 添加平均值到文件
echo "|-----|--------------|-------------------|" >> $result_file
echo "| **Average** | **$avg_duration** | **$avg_exec_time** |" >> $result_file

echo "Testing complete. Results saved in test_results/ directory"