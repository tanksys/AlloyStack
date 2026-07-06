#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
upstream="$repo_root/third_party/musl"
source_dir="$repo_root/target/musl-alloy-src"
build_dir="$repo_root/target/musl-alloy-build"
install_dir="$repo_root/target/musl-alloy"
patch_dir="$repo_root/musl/patches"

if [[ ! -f "$upstream/VERSION" ]]; then
    echo "missing musl submodule; run: git submodule update --init third_party/musl" >&2
    exit 1
fi

rm -rf "$source_dir" "$build_dir" "$install_dir"
mkdir -p "$source_dir" "$build_dir" "$install_dir/lib"
cp -a "$upstream/." "$source_dir/"

for patch_file in "$patch_dir"/*.patch; do
    patch -d "$source_dir" -p1 < "$patch_file"
done

cd "$build_dir"
"$source_dir/configure" \
    --prefix="$install_dir" \
    --syslibdir="$install_dir/lib" \
    CC="${CC:-cc}" \
    CFLAGS="-O2 -fPIC -ftls-model=global-dynamic"

make -s -j"${JOBS:-$(nproc)}" lib/libc-alloy.a install-headers
cp lib/libc-alloy.a "$install_dir/lib/libmusl_alloy.a"

echo "built $install_dir/lib/libmusl_alloy.a"
