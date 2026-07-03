#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
source_dir="$repo_root/user/python_workloads/wasm_bench"
image="${FS_IMAGE:-$repo_root/fs_images/fatfs.img}"
mount_dir="${IMAGE_MOUNT:-$repo_root/image_content}"
files=(wordcount.py parallel_sort.py functionchain.py transfer.py)

if mountpoint -q "$mount_dir"; then
    target_dir="$mount_dir/wasm_bench"
    if [[ ! -w "$target_dir" ]]; then
        echo "$target_dir is not writable; rerun this script with sudo" >&2
        exit 1
    fi
    for file in "${files[@]}"; do
        install -m 0755 "$source_dir/$file" "$target_dir/$file"
    done
else
    command -v mcopy >/dev/null || {
        echo "mcopy is required to update an unmounted FAT image" >&2
        exit 1
    }
    for file in "${files[@]}"; do
        mcopy -o -i "$image" "$source_dir/$file" "::/wasm_bench/$file"
    done
fi

echo "synchronized Python workloads to $image"
