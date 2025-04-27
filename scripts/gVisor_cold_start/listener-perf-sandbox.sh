#!/bin/bash

SENTRY_PROCESS="runsc-sandbox"

echo "开始监听runsc-sandbox"
while true; do
    PID=$(pgrep -f $SENTRY_PROCESS)

    if [ -n "$PID" ]; then
        echo "$SENTRY_PROCESS's pid is $PID"
        
        perf record -e cpu-clock -g -p $PID -o perf-sandbox.data
        # strace -p $PID >& res1.txt
        break
    fi
done

## use "perf report -i perf-sandbox.data" to check the data