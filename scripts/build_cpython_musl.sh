#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CACHE="$ROOT/target/cpython-musl"
TOOLCHAIN="$CACHE/toolchain"
SOURCE="$ROOT/third_party/cpython"
BUILD="$CACHE/build-submodule"
CONFIG_SITE_FILE="$CACHE/config.site"

resolve_build_python() {
    local candidate=""
    local version=""

    if [[ -n "${BUILD_PYTHON:-}" ]]; then
        candidate="$BUILD_PYTHON"
    elif command -v python3.11 >/dev/null 2>&1; then
        candidate="$(command -v python3.11)"
    elif command -v python3 >/dev/null 2>&1; then
        candidate="$(command -v python3)"
    else
        echo "missing build python; set BUILD_PYTHON to a Python 3.11 executable" >&2
        return 1
    fi

    version="$("$candidate" -c 'import sys; print(f"{sys.version_info[0]}.{sys.version_info[1]}")')"
    if [[ "$version" != "3.11" ]]; then
        echo "build python must be Python 3.11, got $version from $candidate" >&2
        echo "set BUILD_PYTHON=/path/to/python3.11 and rerun" >&2
        return 1
    fi

    printf '%s\n' "$candidate"
}

compiler_supports() {
    local option="$1"

    "$TOOLCHAIN/bin/musl-gcc" "$option" \
        -x c -c /dev/null -o /dev/null >/dev/null 2>&1
}

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

MUSL_CFLAGS="-O2 -fPIC"
if compiler_supports -fno-link-libatomic; then
    # GCC 16 may inject the internal placeholder -latomic_asneeded. The
    # musl-gcc specs do not expand it, so suppress automatic libatomic linking.
    MUSL_CFLAGS+=" -fno-link-libatomic"
fi

mkdir -p "$CACHE"
cat >"$CONFIG_SITE_FILE" <<EOF
ac_cv_file__dev_ptmx=yes
ac_cv_file__dev_ptc=no
EOF

rm -rf "$BUILD"
mkdir -p "$BUILD"
(
    cd "$BUILD"
    BUILD_PYTHON_PATH="$(resolve_build_python)"
    PATH="$TOOLCHAIN/bin:$PATH" \
    CONFIG_SITE="$CONFIG_SITE_FILE" \
    CC="$TOOLCHAIN/bin/musl-gcc" \
    CFLAGS="$MUSL_CFLAGS" \
    "$SOURCE/configure" \
        --host=x86_64-linux-musl \
        --build=x86_64-pc-linux-gnu \
        --with-build-python="$BUILD_PYTHON_PATH" \
        --disable-shared \
        --disable-ipv6 \
        --without-ensurepip
)

PATH="$TOOLCHAIN/bin:$PATH" \
    CONFIG_SITE="$CONFIG_SITE_FILE" \
    make -C "$BUILD" -j"${JOBS:-$(nproc)}" libpython3.11.a
echo "built $BUILD/libpython3.11.a"
