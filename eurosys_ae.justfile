cold_start_latency: 
    # just cold_start_latency
    @echo '\nDocker cold start cost (seconds): '
    @echo "$(date '+%s.%N') $(sudo docker run --rm -t ubuntu date '+%s.%N')" | awk '{print $2 - $1}'
    
    @echo '\ngVisor cold start cost (seconds): '
    @echo "$(date '+%s.%N') $(sudo docker run --runtime runsc --rm -t ubuntu date '+%s.%N')" | awk '{print $2 - $1}'
    
    @echo '\nKata-container cold start cost (seconds): '
    @echo "$(date '+%s.%N') $(sudo ctr run --snapshotter run --snapshotter devmapper --runtime io.containerd.kata-fc.v2 --rm docker.io/library/word_count_5_10_asstlane_image:latest acv508781 date '+%s.%N')" | awk '{print $2 - $1}'

data_transfer_latency:
    # just data_transfer_latency
    
    @echo 'Faastlane (reference passing) data transfer cost (ns): '
    cd /home/yjn/asstlane-main/trans_data_one_thread; \
    for data_size in '4*1024' '64*1024' '1024*1024' '16*1024*1024' '256*1024*1024'; do \
        echo $data_size > data_size.config; \
        echo "\ndata size=$data_size bytes" ; \
        cargo build --release 1>/dev/null 2>/dev/null; \
        ./target/release/trans_data | grep 'total_dur'; \
    done

end_to_end_latency:
    # just end_to_end_latency

    @echo '\nFaastlane (reference passing)'
    @echo '(Figure 12) Word Count Application (data size 100MB, 3 instances) cost (ms):'
    echo "`cd /home/yjn/asstlane-cyc/word_count_ref && ./target/release/word_count_3_100`" | tail -n 1

    @echo '\n(Figure 12) Parallel Sorting Application (data size 25MB, 3 instances) cost (ms):'
    echo "`cd /home/yjn/asstlane-cyc/parallel_sort_ref && ./target/release/parallel_sort_3_25`" | tail -n 1

    @echo '\n(Figure 12) Function Chain Application (data size 64MB, 15 functions) cost (us):'
    echo "`cd /home/yjn/asstlane-cyc/long_chain && ./target/release/long_chain_15_64`" | tail -n 1

    @echo '\nFaastlane-ref-kata'
    @echo '(Figure 12) Word Count Application (data size 100MB, 3 instances) cost (s):'
    echo `date '+%s.%N' && sudo ctr run --snapshotter run --snapshotter devmapper --runtime io.containerd.kata-fc.v2 --rm docker.io/library/word_count_3_100_asstlane_image:latest acv508781` | awk '{ print $16 + $18/1e9 - $1 }'

    @echo '\n(Figure 12) Parallel Sorting Application (data size 25MB, 3 instances) cost (s):'
    echo `date '+%s.%N' && sudo ctr run --snapshotter run --snapshotter devmapper --runtime io.containerd.kata-fc.v2 --rm docker.io/library/parallel_sort_3_25_asstlane_image:latest acv508781` | awk '{ print $16 + $18/1e9 - $1 }'

    @echo '\n(Figure 12) Function Chain Application (data size 64MB, 15 functions) cost (s):'
    echo `date '+%s.%N' && sudo ctr run --snapshotter run --snapshotter devmapper --runtime io.containerd.kata-fc.v2 --rm docker.io/library/long_chain_15_64_asstlane_image:latest acv508781` | awk '{ print $(NF-5) + $(NF-3)/1e9 - $1 }'
    
    @-sudo pkill -9 firecracker

p99_latency:
    # just p99_latency

    @-sudo rm monitor.log

    @-sudo pkill -9 firecracker
    @echo '\n(Figure 13(a)) Faastlane-ref-kata p99 QPS=10 (ms) (this part takes a bit longer to run...)'
    python3 p99test.py 10 | grep 'p99'
    
    @-sudo pkill -9 firecracker
    @echo '\n(Figure 13(a)) Faastlane-ref-kata p99 QPS=20 (ms)'
    python3 p99test.py 20 | grep 'p99'
    
    @-sudo pkill -9 firecracker
    @echo '\n(Figure 13(a)) Faastlane-ref-kata p99 QPS=40 (ms)'
    python3 p99test.py 40 | grep 'p99'
    
    @-sudo pkill -9 firecracker
    @echo '\n(Figure 13(a)) Faastlane-ref-kata p99 QPS=80 (ms)'
    python3 p99test.py 80 | grep 'p99'

    @-sudo pkill -9 firecracker

resource_consume:
    # just resource_consume
    
    @-sudo pkill -9 firecracker
    @sleep 3
    @echo '\n(Figure 13(b)) number of faasltane-ref-kata instances=20  (this part takes a bit longer to run...)'
    sudo ./fasstlane-resourcetester 20 | grep 'total consume mem:'
    mv monitor.log faastlane_parallel_sort_resouce_c5_25_20.txt

    @sleep 3
    @echo '\n(Figure 13(b)) number of faasltane-ref-kata instances=40'
    sudo ./fasstlane-resourcetester 40 | grep 'total consume mem:'
    mv monitor.log faastlane_parallel_sort_resouce_c5_25_40.txt

    @sleep 3
    @echo '\n(Figure 13(b)) number of faasltane-ref-kata instances=60'
    sudo ./fasstlane-resourcetester 60 | grep 'total consume mem:'
    mv monitor.log faastlane_parallel_sort_resouce_c5_25_60.txt

    @sleep 3
    @echo '\n(Figure 13(b)) number of faasltane-ref-kata instances=80'
    sudo ./fasstlane-resourcetester 80 | grep 'total consume mem:'
    mv monitor.log faastlane_parallel_sort_resouce_c5_25_80.txt

    @-sudo rm monitor.log
    @-sudo pkill -9 firecracker

    @echo '\n(Figure 13(b)) Cost CPU (AlloyStack and faasltane-ref-kata): '
    -./scripts/comp_resource.py