#!/bin/bash

sudo umount ./image_content

rm ./fs_images/new_image.img

IMAGE_DIR="fs_images"
IMAGE_FILE="new_image.img"

IMAGE_PATH="$IMAGE_DIR/$IMAGE_FILE"

dd if=/dev/zero of="$IMAGE_PATH" bs=1M count=400

mkfs.vfat -F 32 "$IMAGE_PATH"

sudo mount ./fs_images/new_image.img ./image_content