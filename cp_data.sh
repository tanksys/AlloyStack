#!/bin/bash

# 检查是否有参数传递
if [ -z "$1" ]; then
  echo "Usage: $0 <command>"
  exit 1
fi

case $1 in
  "c1")
    case $2 in
        "10m")
            src_path="/home/wyj/alloy_stack/asstlane/word_count/fake_data_0_"
            dest_path="./image_content/"
            sudo cp ./../asstlane/word_count/fake_data_
            ;;
        "n10")
            
            ;;
        "n15")
            
            ;;
        *)
            echo "Unknown command: $2"
            exit 1
            ;;
    esac
        
    ;;
  "map_reduce")
    echo "Executing map_reduce"
  *)
    echo "Unknown command: $1"
    exit 1
    ;;
esac



sudo cp ./