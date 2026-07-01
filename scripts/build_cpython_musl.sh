#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CACHE="$ROOT/target/cpython-musl"
TOOLCHAIN="$CACHE/toolchain"
SOURCE="$ROOT/third_party/cpython"
BUILD="$CACHE/build-submodule"

if [[ ! -f "$SOURCE/Include/Python.h" ]]; then
    echo "missing CPython submodule; run: git submodule update --init third_party/cpython" >&2
    exit 1
fi

if [[ ! -x "$TOOLCHAIN/bin/musl-gcc" ]]; then
    mkdir -p "$CACHE/musl-build" "$TOOLCHAIN"
    (
        cd "$CACHE/musl-build"
        "$ROOT/third_party/musl/configure" --prefix="$TOOLCHAIN"
        make -j"${JOBS:-$(nproc)}"
        make install
    )
fi

ln -sf "$(command -v ar)" "$TOOLCHAIN/bin/x86_64-linux-musl-ar"
ln -sf "$(command -v readelf)" "$TOOLCHAIN/bin/x86_64-linux-musl-readelf"

if [[ ! -f "$BUILD/Makefile" ]]; then
    mkdir -p "$BUILD"
    (
        cd "$BUILD"
        PATH="$TOOLCHAIN/bin:$PATH" \
        CC="$TOOLCHAIN/bin/musl-gcc" \
        CFLAGS="-O2 -fPIC -fno-link-libatomic" \
        ac_cv_file__dev_ptmx=yes \
        ac_cv_file__dev_ptc=no \
        "$SOURCE/configure" \
            --host=x86_64-linux-musl \
            --build=x86_64-pc-linux-gnu \
            --with-build-python="${BUILD_PYTHON:-/usr/bin/python3.11}" \
            --disable-shared \
            --disable-ipv6 \
            --without-ensurepip
    )
fi

PATH="$TOOLCHAIN/bin:$PATH" \
    make -C "$BUILD" -j"${JOBS:-$(nproc)}" libpython3.11.a
echo "built $BUILD/libpython3.11.a"
