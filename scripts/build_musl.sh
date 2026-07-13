#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
upstream="$repo_root/third_party/musl"
source_dir="$repo_root/target/musl-alloy-src"
build_dir="$repo_root/target/musl-alloy-build"
install_dir="$repo_root/target/musl-alloy"
patch_dir="$repo_root/musl/patches"
log_dir="$repo_root/target/logs"
log_file="$log_dir/build_musl.log"

mkdir -p "$log_dir"
: >"$log_file"

log() {
    printf '[%(%Y-%m-%dT%H:%M:%S%z)T] %s\n' -1 "$*" >>"$log_file"
}

fail_with_log() {
    echo "$*" >&2
    echo "see $log_file for details" >&2
    exit 1
}

run_logged() {
    log "+ $*"
    "$@" >>"$log_file" 2>&1 || fail_with_log "command failed: $*"
}

if [[ ! -f "$upstream/VERSION" ]]; then
    echo "missing musl submodule; run: git submodule update --init third_party/musl" >&2
    exit 1
fi

echo "building musl alloy; log: $log_file"

rm -rf "$source_dir" "$build_dir" "$install_dir"
mkdir -p "$source_dir" "$build_dir" "$install_dir/lib"
cp -a "$upstream/." "$source_dir/"

for patch_file in "$patch_dir"/*.patch; do
    run_logged patch -d "$source_dir" -p1 -i "$patch_file"
done

cd "$build_dir"
run_logged "$source_dir/configure" \
    --prefix="$install_dir" \
    --syslibdir="$install_dir/lib" \
    CC="${CC:-cc}" \
    CFLAGS="-O2 -fPIC -ftls-model=global-dynamic"

run_logged make -s -j"${JOBS:-$(nproc)}" lib/libc-alloy.a install-headers
cp lib/libc-alloy.a "$install_dir/lib/libmusl_alloy.a"

echo "built $install_dir/lib/libmusl_alloy.a"
