#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CACHE="$ROOT/target/cpython-musl"
TOOLCHAIN="$CACHE/toolchain"
SOURCE="$ROOT/third_party/cpython"
BUILD="$CACHE/build-submodule"
CONFIG_SITE_FILE="$CACHE/config.site"
CONFIG_SIGNATURE_FILE="$CACHE/config.signature"
LOG_DIR="$ROOT/target/logs"
LOG_FILE="$LOG_DIR/build_cpython_musl.log"

mkdir -p "$LOG_DIR"
: >"$LOG_FILE"

log() {
    printf '[%(%Y-%m-%dT%H:%M:%S%z)T] %s\n' -1 "$*" >>"$LOG_FILE"
}

fail_with_log() {
    echo "$*" >&2
    echo "see $LOG_FILE for details" >&2
    exit 1
}

run_logged() {
    log "+ $*"
    "$@" >>"$LOG_FILE" 2>&1 || fail_with_log "command failed: $*"
}

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

rebuild_toolchain=0
if [[ ! -x "$TOOLCHAIN/bin/musl-gcc" ]]; then
    rebuild_toolchain=1
elif ! grep -Fq "$TOOLCHAIN/lib/musl-gcc.specs" "$TOOLCHAIN/bin/musl-gcc"; then
    rebuild_toolchain=1
fi

if [[ "$rebuild_toolchain" == "1" ]]; then
    echo "building musl CPython toolchain; log: $LOG_FILE"
    rm -rf "$CACHE/musl-build" "$TOOLCHAIN"
    mkdir -p "$CACHE/musl-build" "$TOOLCHAIN"
    (
        cd "$CACHE/musl-build"
        run_logged "$ROOT/third_party/musl/configure" --prefix="$TOOLCHAIN"
        run_logged make -j"${JOBS:-$(nproc)}"
        run_logged make install
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

BUILD_PYTHON_PATH="$(resolve_build_python)"
CONFIGURE_ARGS=(
    --host=x86_64-linux-musl
    --build=x86_64-pc-linux-gnu
    --with-build-python="$BUILD_PYTHON_PATH"
    --disable-shared
    --disable-ipv6
    --without-ensurepip
)

NEW_CONFIG_SIGNATURE="$(
    {
        printf 'source=%s\n' "$SOURCE"
        printf 'build_python=%s\n' "$BUILD_PYTHON_PATH"
        printf 'cc=%s\n' "$TOOLCHAIN/bin/musl-gcc"
        printf 'cflags=%s\n' "$MUSL_CFLAGS"
        printf 'configure_args='
        printf '%q ' "${CONFIGURE_ARGS[@]}"
        printf '\n'
        printf 'config_site:\n'
        cat "$CONFIG_SITE_FILE"
    }
)"

need_configure=0
if [[ "${CLEAN_CPYTHON_MUSL:-0}" == "1" ]]; then
    rm -rf "$BUILD"
    need_configure=1
elif [[ ! -x "$BUILD/config.status" ]]; then
    need_configure=1
elif [[ ! -f "$CONFIG_SIGNATURE_FILE" ]]; then
    need_configure=1
elif [[ "$(cat "$CONFIG_SIGNATURE_FILE")" != "$NEW_CONFIG_SIGNATURE" ]]; then
    need_configure=1
fi

mkdir -p "$BUILD"

echo "building musl CPython incrementally; log: $LOG_FILE"

if [[ "$need_configure" == "1" ]]; then
    log "configure required"
    (
        cd "$BUILD"
        log "+ $SOURCE/configure ${CONFIGURE_ARGS[*]}"
        PATH="$TOOLCHAIN/bin:$PATH" \
        CONFIG_SITE="$CONFIG_SITE_FILE" \
        CC="$TOOLCHAIN/bin/musl-gcc" \
        CFLAGS="$MUSL_CFLAGS" \
        "$SOURCE/configure" "${CONFIGURE_ARGS[@]}" >>"$LOG_FILE" 2>&1
    ) || fail_with_log "CPython configure failed"
    printf '%s\n' "$NEW_CONFIG_SIGNATURE" >"$CONFIG_SIGNATURE_FILE"
else
    log "reuse existing CPython configure result"
fi

run_logged env \
    PATH="$TOOLCHAIN/bin:$PATH" \
    CONFIG_SITE="$CONFIG_SITE_FILE" \
    make -C "$BUILD" -j"${JOBS:-$(nproc)}" libpython3.11.a
echo "built $BUILD/libpython3.11.a"
