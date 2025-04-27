#!/bin/bash

WORKPLACE="/home/dyx/workplace/mslibos/scripts"
rm $WORKPLACE/test.sock
touch $WORKPLACE/test.sock
echo -n "0" > $WORKPLACE/test.sock

SENTRY_PROCESS="runsc-sandbox"
GOFER_PROCESS="runsc-gofer"

echo "开始监听test.sock"
while true; do
    CONTENT=$(cat "$WORKPLACE/test.sock")

    if [ "$CONTENT" == "1" ]; then
        echo "test.sock 文件中的内容为 1, 退出轮询."
        break
    fi
done

for PROCESS in $SENTRY_PROCESS $GOFER_PROCESS; do
    echo "Checking CPU time for $PROCESS"
    PID=$(pgrep -f $PROCESS)
    
    echo "$PROCESS's pid is $PID"

    CPU_TIME=$(cat /proc/$PID/stat | awk '{print $14, $15}')
    
    UTIME=$(echo $CPU_TIME | awk '{print $1 * 10}')
    STIME=$(echo $CPU_TIME | awk '{print $2 * 10}')
    
    echo "UTIME: $UTIME ms"
    echo "STIME: $STIME ms"
done