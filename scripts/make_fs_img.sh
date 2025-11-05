#!/bin/bash

dir="./fs_images"
image="$dir/fatfs.img"

if [ ! -d "$dir" ]; then
  mkdir "$dir"
fi

if [ -f "$image" ]; then
  rm "$image"
fi

dd if=/dev/zero of="$image" bs=1M seek=512 count=0 && \
    mkfs.fat "$image"
