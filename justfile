
set positional-arguments

enable_mpk := "0"
enable_pkey_per_func := "0"
enable_file_buffer := "0"

enable_release := "1"

mpk_flag := if enable_mpk == "1" {
    if enable_pkey_per_func == "1" { 
        "pkey_per_func" 
    } else { 
        "mpk"
    } 
} else { "" }

mpk_feature_flag := if mpk_flag == "" { "" } else { "--features " + mpk_flag }

buffer_feature_flag := if enable_file_buffer == "1" { "--features file-based" } else { "" }

release_flag := if enable_release == "1" { 
    "--release" 
} else { "" }

rust_func func_name:
    cargo build {{ release_flag }} {{ mpk_feature_flag }} {{ buffer_feature_flag }} \
        --manifest-path user/{{ func_name }}/Cargo.toml

libos lib_name:
    cargo build {{ release_flag }} {{ if enable_mpk == "1" { "--features mpk" } else { "" } }} \
        --manifest-path common_service/{{ lib_name }}/Cargo.toml

pass_args:
    just rust_func func_a
    just rust_func func_b

map_reduce:
    for name in time fdtab fatfs stdio mm; do \
    just libos $name; \
    done

    just rust_func mapper
    just rust_func reducer

parallel_sort:
    for name in time fdtab fatfs stdio mm; do \
    just libos $name; \
    done

    for func in file_reader sorter splitter merger; do \
    just rust_func $func; \
    done

long_chain:
    for name in time stdio mm; do \
    just libos $name; \
    done

    just rust_func array_sum

simple_file:
    just rust_func simple_file

all_libos:
    ./scripts/build_all_common{{ if enable_mpk == "1" { "_mpk" } else { "" } }}.sh {{ release_flag }}

all_rust:
    just all_libos
    ./scripts/build_user.sh {{ release_flag }} {{ mpk_flag }}

run_rust_test:
    just all_libos
    just all_rust
    ./scripts/run_tests.sh {{ mpk_flag }}

cc_flags_p1 := "-Wl,--gc-sections -nostdlib -Wl,--whole-archive"
cc_flags_p2 := "-Wl,--no-whole-archive -shared"
target := "x86_64-unknown-none"

profile := if enable_release == "1" { 
    "release"
} else { 
    "debug"
}

symbol_link func_name:
    @ln -s $(pwd)/user/{{ func_name }}/target/{{target}}/{{profile}}/lib{{ func_name }}.so target/{{profile}}/

wasm_func func_name:
    cd user/{{ func_name }} \
        && cargo build {{ release_flag }} \
            --target {{target}} {{mpk_feature_flag}}  \
        && cc {{cc_flags_p1}} \
            target/{{target}}/{{profile}}/lib{{ func_name }}.a \
            {{cc_flags_p2}} \
            -o target/{{target}}/{{profile}}/lib{{ func_name }}.so
    
    @-rm target/{{profile}}/lib{{ func_name }}.so
    just symbol_link {{ func_name }}

c_wordcount: 
    just wasm_func wasmtime_mapper
    just wasm_func wasmtime_reducer

c_parallel_sort:
    just wasm_func wasmtime_sorter
    just wasm_func wasmtime_spliter
    just wasm_func wasmtime_merger
    just wasm_func wasmtime_checker

c_long_chain:
    just wasm_func wasmtime_longchain

all_c_wasm: c_wordcount c_parallel_sort c_long_chain

python_wordcount: 
    just wasm_func wasmtime_cpython_wordcount

python_parallel_sort:
    just wasm_func wasmtime_cpython_parallel_sort

python_long_chain:
    just wasm_func wasmtime_cpython_func

all_py_wasm: python_wordcount python_parallel_sort python_long_chain

all_wasm: all_c_wasm all_py_wasm

measure_avg isol_file:
    #!/bin/bash
    for (( i=1; i<=10; i++ )); do \
        output=$(cargo run {{ release_flag }} {{mpk_feature_flag}} -- --files {{ isol_file }} --metrics total-dur 2>&1); \
        total_dur=$(echo "$output" | grep -o '"total_dur(ms)": [0-9.]*' | awk -F': ' '{print $2}'); \
        total_dur_rounded=$(printf "%.3f\n" "$total_dur") ;\
        echo "$total_dur_rounded ," ;\
    done ;

gen_data:
    sudo -E ./scripts/gen_data.py

init:
    rustup override set 'nightly-2023-12-01'
    rustup target add x86_64-unknown-linux-musl
    [ -f fs_images/fatfs.img ] || unzip fs_images/fatfs.zip -d fs_images
    mkdir -p image_content

asvisor:
    cargo build {{ release_flag }}

cold_start_latency: asvisor all_libos
    just rust_func hello_world
    just rust_func load_all
    @-./scripts/del_tap.sh 2>/dev/null

    @echo 'Section 8.2: Cold start Latency (Figure 10)'
    @echo '\ncold start with on-demand loading cost (ms):'
    target/{{profile}}/asvisor --files isol_config/base_config.json --metrics total-dur 2>&1 | grep 'ms'
    @echo '\ncold start without on-demand loading cost (ms):'
    target/{{profile}}/asvisor --files isol_config/load_all.json --metrics total-dur 2>&1 | grep 'ms'

data_transfer_latency: asvisor all_libos
    @echo 'Section 8.3: Intermediate Data Transfer Latency (ns) (Figure 11)'
    for data_size in '4*1024' '64*1024' '1024*1024' '16*1024*1024' '256*1024*1024'; do \
        echo $data_size > user/data_size.config; \
        just pass_args 1>/dev/null 2>/dev/null; \
        target/{{profile}}/asvisor --files isol_config/pass_complex_args.json 2>&1 | grep 'bytes'; \
        echo ''; \
    done

end_to_end_latency: asvisor all_libos map_reduce parallel_sort long_chain
    @echo '\nSection 8.4: Rust Benchmark Evaluation (Figure 12)'
    -sudo mount fs_images/fatfs.img image_content 2>/dev/null
    sudo -E ./scripts/gen_data.py 3 '100 * 1024 * 1024' 3 '25 * 1024 * 1024'
    
    @echo '\nword count (data size 100MB, 3 instances) cost: '
    target/{{profile}}/asvisor --files isol_config/map_reduce_large_c3.json --metrics total-dur 2>&1 | grep 'total_dur'

    @echo '\nparallel sorting (data size 25MB, 3 instances) cost: '
    target/{{profile}}/asvisor --files isol_config/parallel_sort_c3.json --metrics total-dur 2>&1 | grep 'total_dur'

    @echo '\nfunction chain (data size 64MB, 15 functions) cost: '
    echo '64 * 1024 * 1024' > user/function_chain_data_size.config
    target/{{profile}}/asvisor --files isol_config/long_chain_n15.json --metrics total-dur 2>&1 | grep 'total_dur'

    @echo '\nSection 8.5: C and Python Benchmark Evaluation (Figure 14)'
    just c_end_to_end_latency
    just py_end_to_end_latency

c_end_to_end_latency: asvisor all_libos all_c_wasm
    @echo 'C word count (data size 100MB, 3 instances) cost: '
    target/{{profile}}/asvisor --files isol_config/wasmtime_wordcount_c3.json --metrics total-dur 2>&1 | grep 'total_dur'

    @echo 'C parallel sorting (data size 25MB, 3 instances) cost: '
    target/{{profile}}/asvisor --files isol_config/wasmtime_parallel_sort_c3.json --metrics total-dur 2>&1  | grep 'total_dur'

    @echo 'C function chain (data size 1MB, 10 functions) cost: '
    target/{{profile}}/asvisor --files isol_config/wasmtime_longchain.json --metrics total-dur 2>&1 | grep 'total_dur'

py_end_to_end_latency: asvisor all_libos all_py_wasm
    # Python applications.
    -sudo mount fs_images/fatfs.img image_content 2>/dev/null
    sudo -E ./scripts/gen_data.py 1 '1 * 1024 * 1024' 1 '1 * 1024 * 1024'

    sleep 3
    @echo 'Python word count (data size 1MB, 1 instance) cost: '
    target/{{profile}}/asvisor --files isol_config/wasmtime_cpython_wordcount_c1.json --metrics total-dur 2>&1 | grep 'total_dur'
    
    @echo 'Python parallel sorting (data size 1MB, 1 instance) cost: '
    target/{{profile}}/asvisor --files isol_config/wasmtime_cpython_parallel_sort_c1.json --metrics total-dur 2>&1 | grep 'total_dur'
    
    @echo 'Python long chain (data size 1MB, 5 functions) cost: '
    target/{{profile}}/asvisor --files isol_config/wasmtime_cpython_functionchain_n5.json --metrics total-dur 2>&1 | grep 'total_dur'

breakdown: asvisor all_libos
    @echo '\nSection 8.6: Breakdown Analysis (Figure 15)'
    -sudo mount fs_images/fatfs.img image_content 2>/dev/null
    -sudo ./scripts/del_tap.sh
    sudo -E ./scripts/gen_data.py 5 '10 * 1024 * 1024' 5 '1 * 1024 * 1024'
    @echo '1 * 1024 * 1024' > user/function_chain_data_size.config

    @echo 'base (-on-demand-loding, -reference-passing)'
    for func_name in 'mapper' 'reducer' 'file_reader' 'sorter' 'splitter' 'merger' 'array_sum'; do \
        cargo build {{ release_flag }} {{ mpk_feature_flag }} --features file-based --manifest-path user/${func_name}/Cargo.toml; \
    done ;

    @echo 'Word Count'
    target/{{profile}}/asvisor --files isol_config/map_reduce_load_all.json --metrics total-dur 2>&1 | grep 'total_dur'
    sudo rm -f ./image_content/*.imd
    @echo 'Parallel Sorting'
    target/{{profile}}/asvisor --files isol_config/parallel_sort_load_all.json --metrics total-dur 2>&1 | grep 'total_dur'
    sudo rm -f ./image_content/*.imd
    @echo 'Function Chain'
    target/{{profile}}/asvisor --files isol_config/long_chain_load_all.json --metrics total-dur 2>&1 | grep 'total_dur'
    sudo rm -f ./image_content/*.imd

    @echo '\n+on-demand-loding (-reference-passing)'
    @echo 'Word Count'
    target/{{profile}}/asvisor --files isol_config/map_reduce_large_c5.json --metrics total-dur 2>&1 | grep 'total_dur'
    sudo rm -f ./image_content/*.imd
    @echo 'Parallel Sorting'
    target/{{profile}}/asvisor --files isol_config/parallel_sort_c5.json --metrics total-dur 2>&1 | grep 'total_dur'
    sudo rm -f ./image_content/*.imd
    @echo 'Function Chain'
    target/{{profile}}/asvisor --files isol_config/long_chain_n15.json --metrics total-dur 2>&1 | grep 'total_dur'
    sudo rm -f ./image_content/*.imd

    @echo '\n+both'
    for func_name in 'mapper' 'reducer' 'file_reader' 'sorter' 'splitter' 'merger' 'array_sum'; do \
        cargo build {{ release_flag }} {{ mpk_feature_flag }} --manifest-path user/${func_name}/Cargo.toml; \
    done ;
    
    @echo 'Word Count'
    target/{{profile}}/asvisor --files isol_config/map_reduce_large_c5.json --metrics total-dur 2>&1 | grep 'total_dur'
    @echo 'Parallel Sorting'
    target/{{profile}}/asvisor --files isol_config/parallel_sort_c5.json --metrics total-dur 2>&1 | grep 'total_dur'
    @echo 'Function Chain'
    target/{{profile}}/asvisor --files isol_config/long_chain_n15.json --metrics total-dur 2>&1 | grep 'total_dur'

p99_latency: asvisor all_libos parallel_sort
    @echo 'Section 8.4: Rust Benchmark Evaluation (Figure 13(a))'
    
    @-sudo rm monitor.log
    -sudo mount fs_images/fatfs.img image_content 2>/dev/null
    sudo -E ./scripts/gen_data.py 0 0 3 '25 * 1024 * 1024'

    @echo '\np99 QPS=10 (ms)'
    ./p99tester 10 | grep 'p99'
    @echo '\np99 QPS=20'
    ./p99tester 20 | grep 'p99'
    @echo '\np99 QPS=40'
    ./p99tester 40 | grep 'p99'
    @echo '\np99 QPS=80'
    ./p99tester 80 | grep 'p99'

resource_consume: asvisor all_libos parallel_sort
    @echo 'Section 8.4: Rust Benchmark Evaluation (Figure 13(b))'

    @-sudo rm monitor.log
    -sudo mount fs_images/fatfs.img image_content 2>/dev/null
    sudo -E ./scripts/gen_data.py 0 0 5 '25 * 1024 * 1024'

    @sleep 3
    @echo '\nnumber of instances=20'
    ./resourcetester 20 | grep 'total consume mem:'
    mv monitor.log as_parallel_sort_resouce_c5_25_20.txt

    @sleep 3
    @echo '\nnumber of instances=40'
    ./resourcetester 40 | grep 'total consume mem:'
    mv monitor.log as_parallel_sort_resouce_c5_25_40.txt

    @sleep 3
    @echo '\nnumber of instances=60'
    ./resourcetester 60 | grep 'total consume mem:'
    mv monitor.log as_parallel_sort_resouce_c5_25_60.txt

    @sleep 3
    @echo '\nnumber of instances=80'
    ./resourcetester 80 | grep 'total consume mem:'
    mv monitor.log as_parallel_sort_resouce_c5_25_80.txt

    @echo '\n Cost CPU: '
    -./scripts/comp_resource.py
