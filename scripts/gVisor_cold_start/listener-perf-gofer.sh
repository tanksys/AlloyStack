#!/bin/bash

GOFER_PROCESS="runsc-gofer"

echo "开始监听runsc-gofer"
while true; do
    PID=$(pgrep -f $GOFER_PROCESS)

    if [ -n "$PID" ]; then
        echo "$GOFER_PROCESS's pid is $PID"

        perf record -e cpu-clock -g -p $PID -o perf-gofer.data
        # strace -p $PID >& res2.txt
        break
    fi
done

## use "perf report -i perf-gofer.data" to check the data