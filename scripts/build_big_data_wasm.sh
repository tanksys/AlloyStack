# #!/bin/bash
# # filepath: \home\k423\Desktop\alloystacknew\AlloyStack\scripts\build_big_data_wasm.sh

# C_CUSTOM_FLAGS="-DMAX_BUFFER_SIZE=1000000 -DMAX_SLOT_NUM=10 -DMAX_WORDS=18000000" #自己定义
# TARGET="x86_64-unknown-none"
# PROFILE="release"  # 或 "debug"
# CC_FLAGS_P1="-Wl,--gc-sections -nostdlib -Wl,--whole-archive"
# CC_FLAGS_P2="-Wl,--no-whole-archive -shared"

# # 构建 mapper
# cd user/wasmtime_mapper \
#     && cargo build --release --target $TARGET \
#     && cc $CC_FLAGS_P1 \
#         target/$TARGET/$PROFILE/libwasmtime_mapper.a \
#         $CC_FLAGS_P2 \
#         $C_CUSTOM_FLAGS \
#         -o target/$TARGET/$PROFILE/libwasmtime_mapper.so

# # 构建 reducer
# cd ../../user/wasmtime_reducer \
#     && cargo build --release --target $TARGET \
#     && cc $CC_FLAGS_P1 \
#         target/$TARGET/$PROFILE/libwasmtime_reducer.a \
#         $CC_FLAGS_P2 \
#         $C_CUSTOM_FLAGS \
#         -o target/$TARGET/$PROFILE/libwasmtime_reducer.so

# # 创建符号链接
# cd ../..
# ln -s $(pwd)/user/wasmtime_mapper/target/$TARGET/$PROFILE/libwasmtime_mapper.so target/$PROFILE/
# ln -s $(pwd)/user/wasmtime_reducer/target/$TARGET/$PROFILE/libwasmtime_reducer.so target/$PROFILE/

# echo "Big data WASM modules built successfully!"
#!/bin/bash
# filepath: \home\as-group\wyd\final\AlloyStack\scripts\test_params.sh

mkdir -p test_results

# 测试不同的 MAX_BUFFER_SIZE 参数
for buffer_size in 50000 100000 200000 500000 1000000; do
  echo "Testing MAX_BUFFER_SIZE = $buffer_size"
  
  # 更新构建脚本中的参数
  sed -i "s/-DMAX_BUFFER_SIZE=[0-9]\+/-DMAX_BUFFER_SIZE=$buffer_size/" scripts/build_big_data_wasm.sh
  
  
  # 创建结果文件
  result_file="test_results/buffer_${buffer_size}_average.md"
  echo "# Buffer size: $buffer_size" > $result_file
  echo "| Run | Duration (ms) | Execution Time (s) |" >> $result_file
  echo "|-----|--------------|-------------------|" >> $result_file
  
  total_duration=0
  total_exec_time=0
  
  # 运行测试 3 次并记录时间
  for i in 1 2 3; do
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
  avg_duration=$(echo "scale=4; $total_duration / 3" | bc)
  avg_exec_time=$(echo "scale=4; $total_exec_time / 3" | bc)
  
  # 添加平均值到文件
  echo "|-----|--------------|-------------------|" >> $result_file
  echo "| **Average** | **$avg_duration** | **$avg_exec_time** |" >> $result_file
done

echo "Testing complete. Results saved in test_results/ directory"