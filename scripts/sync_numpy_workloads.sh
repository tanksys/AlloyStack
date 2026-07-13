#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
workload_source="$repo_root/user/python_workloads/wasm_bench/numpy_smoke.py"
numpy_mode="${NUMPY_MODE:-minimal}"
if [[ "$numpy_mode" == "unikraft" ]]; then
  numpy_mode="full"
fi
base_image="${BASE_FS_IMAGE:-$repo_root/fs_images/fatfs.img}"
image="${FS_IMAGE:-$repo_root/fs_images/fatfs_numpy.img}"
mount_dir="${FS_MOUNT_DIR:-$repo_root/image_content}"
rootfs="$repo_root/target/numpy-musl/rootfs-$numpy_mode"
numpy_source="$rootfs/Lib/site-packages/numpy"

dir_has_entries() {
  [[ -d "$1" ]] && find "$1" -mindepth 1 -maxdepth 1 -print -quit | grep -q .
}

restore_rootfs_owner() {
  if [[ "${EUID:-$(id -u)}" == "0" && -n "${SUDO_UID:-}" && -d "$rootfs" ]]; then
    chown -R "$SUDO_UID:${SUDO_GID:-$SUDO_UID}" "$rootfs"
  fi
}

if [[ "$numpy_mode" == "minimal" ]]; then
  NUMPY_MUSL_ROOTFS="$rootfs" python3 "$repo_root/scripts/build_numpy_musl.py" --minimal --rootfs-only
elif [[ "$numpy_mode" == "full" ]]; then
  NUMPY_MUSL_ROOTFS="$rootfs" python3 "$repo_root/scripts/build_numpy_musl.py" --rootfs-only
else
  echo "unknown NUMPY_MODE=$numpy_mode; expected minimal or full" >&2
  exit 1
fi
restore_rootfs_owner

if [[ ! -d "$numpy_source" ]]; then
  echo "missing prepared numpy rootfs: $numpy_source" >&2
  exit 1
fi

if dir_has_entries "$mount_dir"; then
  sudo mkdir -p "$mount_dir/Lib/site-packages" "$mount_dir/wasm_bench"
  sudo rm -rf "$mount_dir/Lib/site-packages/numpy"
  sudo cp -r "$numpy_source" "$mount_dir/Lib/site-packages/"
  sudo cp "$workload_source" "$mount_dir/wasm_bench/numpy_smoke.py"
  echo "synchronized NumPy workload and package files to mounted FAT directory $mount_dir"
  sync
  exit 0
fi

command -v mcopy >/dev/null || {
  echo "mcopy is required to update the FAT image when $mount_dir is empty or unmounted" >&2
  exit 1
}
command -v mmd >/dev/null || {
  echo "mmd is required to update the FAT image when $mount_dir is empty or unmounted" >&2
  exit 1
}

if [[ ! -f "$image" || "${REBUILD_NUMPY_IMAGE:-0}" == "1" ]]; then
  if [[ ! -f "$base_image" ]]; then
    echo "missing base image: $base_image" >&2
    exit 1
  fi
  mkdir -p "$(dirname -- "$image")"
  cp "$base_image" "$image"
fi

mmd -i "$image" "::/Lib" 2>/dev/null || true
mmd -i "$image" "::/Lib/site-packages" 2>/dev/null || true
mmd -i "$image" "::/wasm_bench" 2>/dev/null || true
if command -v mdeltree >/dev/null; then
  mdeltree -i "$image" "::/Lib/site-packages/numpy" 2>/dev/null || true
fi
mcopy -s -o -i "$image" "$numpy_source" "::/Lib/site-packages/"
mcopy -o -i "$image" "$workload_source" "::/wasm_bench/numpy_smoke.py"

echo "synchronized NumPy workload and package files to FAT image $image"
