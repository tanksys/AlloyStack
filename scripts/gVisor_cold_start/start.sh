#!/bin/bash

WORKPLACE="/home/dyx/workplace/mslibos/scripts"
# echo -n "0" > $WORKPLACE/test.sock

# timestamp_ns2=$(date +%s%N)
# echo "带纳秒的时间戳：$timestamp_ns2"

sudo docker run --runtime=runsc -v $WORKPLACE:/home --rm --entrypoint /home/writer.sh ubuntu

# timestamp_ns=$(date +%s%N)
# echo "带纳秒的时间戳：$timestamp_ns"
