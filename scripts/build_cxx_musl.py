#!/usr/bin/env python3
"""Prepare a musl-compatible C++ runtime for AlloyStack.

NumPy contains C++ extension sources.  Compiling them against AlloyStack's
musl headers cannot use the host glibc libstdc++ installation: those headers
are configured for glibc and pull in glibc-only locale/configuration symbols.

This script installs or builds a small, explicit C++ runtime prefix at
target/cxx-musl.  Downstream builds should consume only this prefix for C++
headers and static libraries.
"""

from __future__ import annotations

import argparse
import hashlib
import os
import shutil
import subprocess
import sys
import tarfile
import urllib.request
from pathlib import Path


TARGET_TRIPLE = "x86_64-linux-musl"
MUSLCC_ARCHIVE = f"{TARGET_TRIPLE}-cross.tgz"
MUSLCC_URL = f"https://musl.cc/{MUSLCC_ARCHIVE}"
MUSLCC_SHA512SUMS_URL = "https://musl.cc/SHA512SUMS"


def run(command: list[str], cwd: Path | None = None) -> None:
    print("+", " ".join(command), flush=True)
    subprocess.run(command, cwd=cwd, check=True)


def remove_path(path: Path) -> None:
    if path.is_symlink() or path.is_file():
        path.unlink()
    elif path.is_dir():
        shutil.rmtree(path)


def copy_or_link(src: Path, dst: Path, symlink: bool) -> None:
    if dst.exists() or dst.is_symlink():
        remove_path(dst)
    dst.parent.mkdir(parents=True, exist_ok=True)
    if symlink:
        dst.symlink_to(src)
    elif src.is_dir():
        shutil.copytree(src, dst)
    else:
        shutil.copy2(src, dst)


def download(url: str, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    if destination.exists():
        return
    print(f"downloading {url}", flush=True)
    tmp = destination.with_suffix(destination.suffix + ".tmp")
    if tmp.exists():
        tmp.unlink()
    urllib.request.urlretrieve(url, tmp)
    tmp.rename(destination)


def sha512(path: Path) -> str:
    digest = hashlib.sha512()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def verify_muslcc_archive(archive: Path, sums_file: Path) -> None:
    expected = None
    for line in sums_file.read_text().splitlines():
        parts = line.split()
        if len(parts) >= 2 and Path(parts[-1]).name == archive.name:
            expected = parts[0]
            break
    if expected is None:
        raise SystemExit(f"missing {archive.name} entry in {sums_file}")
    actual = sha512(archive)
    if actual.lower() != expected.lower():
        raise SystemExit(f"SHA512 mismatch for {archive}: expected {expected}, got {actual}")


def safe_extract_tar(archive: Path, destination: Path) -> None:
    destination.mkdir(parents=True, exist_ok=True)
    root = destination.resolve()
    with tarfile.open(archive) as tar:
        for member in tar.getmembers():
            target = (destination / member.name).resolve()
            if root != target and root not in target.parents:
                raise SystemExit(f"unsafe tar entry outside destination: {member.name}")
        tar.extractall(destination, filter="fully_trusted")


def candidate_lib_dirs(root: Path) -> list[Path]:
    return [
        root / "lib",
        root / "lib64",
        root / TARGET_TRIPLE / "lib",
        root / "usr" / "lib",
        root / "usr" / "lib64",
        root / "usr" / TARGET_TRIPLE / "lib",
    ]


def find_existing(paths: list[Path]) -> Path | None:
    for path in paths:
        if path.exists():
            return path
    return None


def detect_runtime(sysroot: Path) -> tuple[str, list[Path], Path, list[str]]:
    libcxx_include = find_existing(
        [
            sysroot / "include" / "c++" / "v1",
            sysroot / "usr" / "include" / "c++" / "v1",
            sysroot / TARGET_TRIPLE / "include" / "c++" / "v1",
        ]
    )
    lib_dirs = [path for path in candidate_lib_dirs(sysroot) if path.is_dir()]

    if libcxx_include is not None:
        lib_dir = find_existing([path for path in lib_dirs if (path / "libc++.a").is_file()])
        if lib_dir is None:
            raise SystemExit(f"found libc++ headers at {libcxx_include}, but no libc++.a")
        libs = ["c++"]
        for optional in ["c++abi", "unwind"]:
            if (lib_dir / f"lib{optional}.a").is_file():
                libs.append(optional)
        return "libc++", [libcxx_include], lib_dir, libs

    stdcxx_roots = []
    for base in [
        sysroot / "include" / "c++",
        sysroot / "usr" / "include" / "c++",
        sysroot / TARGET_TRIPLE / "include" / "c++",
    ]:
        if base.is_dir():
            stdcxx_roots.extend(path for path in base.iterdir() if path.is_dir())
    stdcxx_roots = sorted(stdcxx_roots, reverse=True)
    for include_root in stdcxx_roots:
        target_include = include_root / TARGET_TRIPLE
        if not target_include.is_dir():
            # A host glibc libstdc++ install usually has
            # x86_64-*-linux-gnu here.  Reject it before it can be imported
            # into target/cxx-musl.
            continue
        lib_dir = find_existing([path for path in lib_dirs if (path / "libstdc++.a").is_file()])
        if lib_dir is None:
            continue
        include_dirs = [include_root]
        include_dirs.append(target_include)
        backward = include_root / "backward"
        if backward.is_dir():
            include_dirs.append(backward)
        libs = ["stdc++"]
        if (lib_dir / "libsupc++.a").is_file():
            libs.append("supc++")
        if (lib_dir / "libgcc_eh.a").is_file():
            libs.append("gcc_eh")
        return "libstdc++", include_dirs, lib_dir, libs

    raise SystemExit(
        f"{sysroot} does not look like a musl C++ sysroot. Expected "
        "include/c++/v1 + libc++.a, or include/c++/<ver> + libstdc++.a."
    )


def write_env(prefix: Path, kind: str, include_dirs: list[Path], lib_dir: Path, libs: list[str]) -> None:
    env_file = prefix / "cxx-runtime.env"
    include_flags = " ".join(f"-isystem {path}" for path in include_dirs)
    link_flags = " ".join(f"-L {prefix / 'lib'}" for _ in [0]) + " " + " ".join(
        f"-l{lib}" for lib in libs
    )
    env_file.write_text(
        "\n".join(
            [
                f"CXX_RUNTIME_KIND={kind}",
                f"CXX_INCLUDE_FLAGS=-nostdinc++ {include_flags}",
                f"CXX_LINK_FLAGS={link_flags.strip()}",
                f"CXX_LINK_LIBS={' '.join(libs)}",
                f"CXX_RUNTIME_SOURCE_LIB_DIR={lib_dir}",
                "",
            ]
        )
    )


def install_from_sysroot(repo_root: Path, sysroot: Path, prefix: Path, symlink: bool) -> None:
    kind, include_dirs, lib_dir, libs = detect_runtime(sysroot)
    if prefix.exists() and not prefix.is_dir():
        raise SystemExit(f"{prefix} exists and is not a directory")
    (prefix / "include" / "c++").mkdir(parents=True, exist_ok=True)
    (prefix / "lib").mkdir(parents=True, exist_ok=True)
    (prefix / "bin").mkdir(parents=True, exist_ok=True)

    if kind == "libc++":
        copy_or_link(include_dirs[0], prefix / "include" / "c++" / "v1", symlink)
    else:
        version = include_dirs[0].name
        # GCC's target and backward include directories live below the version
        # root.  Import the version root once; do not create children under a
        # symlinked destination.
        copy_or_link(include_dirs[0], prefix / "include" / "c++" / version, symlink)

    for lib in libs:
        src = lib_dir / f"lib{lib}.a"
        if not src.is_file():
            raise SystemExit(f"missing {src}")
        copy_or_link(src, prefix / "lib" / src.name, symlink)

    write_env(prefix, kind, [prefix / "include" / "c++" / "v1"] if kind == "libc++" else [
        prefix / "include" / "c++" / include_dirs[0].name,
        *[
            prefix / "include" / "c++" / include_dirs[0].name / include_dir.name
            for include_dir in include_dirs[1:]
        ],
    ], prefix / "lib", libs)
    write_cxx_wrapper(repo_root, prefix)
    check_prefix(repo_root, prefix)
    print(f"installed {kind} runtime into {prefix}", flush=True)


def build_from_llvm(repo_root: Path, llvm_project: Path, prefix: Path, clean: bool) -> None:
    runtimes = llvm_project / "runtimes"
    if not (runtimes / "CMakeLists.txt").is_file():
        raise SystemExit(f"{llvm_project} is not an llvm-project checkout with runtimes/CMakeLists.txt")

    toolchain = repo_root / "target" / "cpython-musl" / "toolchain"
    specs = toolchain / "lib" / "musl-gcc.specs"
    if not specs.is_file():
        raise SystemExit("missing musl toolchain; run `bash scripts/build_cpython_musl.sh` first")

    build_dir = repo_root / "target" / "cxx-musl-build"
    if clean and build_dir.exists():
        shutil.rmtree(build_dir)
    build_dir.mkdir(parents=True, exist_ok=True)

    real_cc = os.environ.get("REALCC", shutil.which("gcc") or "cc")
    real_cxx = os.environ.get("REALGXX", shutil.which("g++") or "c++")
    common_flags = f"-O2 -fPIC -specs {specs}"
    cxx_flags = (
        f"{common_flags} -nostdinc++ "
        f"-I{llvm_project / 'libcxx' / 'include'} "
        f"-I{llvm_project / 'libcxxabi' / 'include'} "
        f"-I{llvm_project / 'libunwind' / 'include'}"
    )

    cmake = os.environ.get("CMAKE", "cmake")
    build_tool = os.environ.get("CMAKE_BUILD_TOOL", "ninja")
    run(
        [
            cmake,
            "-S",
            str(runtimes),
            "-B",
            str(build_dir),
            "-G",
            "Ninja",
            f"-DCMAKE_BUILD_TOOL={build_tool}",
            f"-DCMAKE_INSTALL_PREFIX={prefix}",
            "-DCMAKE_TRY_COMPILE_TARGET_TYPE=STATIC_LIBRARY",
            f"-DCMAKE_C_COMPILER={real_cc}",
            f"-DCMAKE_CXX_COMPILER={real_cxx}",
            f"-DCMAKE_C_FLAGS={common_flags}",
            f"-DCMAKE_CXX_FLAGS={cxx_flags}",
            f"-DLLVM_ENABLE_RUNTIMES=libcxx;libcxxabi;libunwind",
            "-DLIBCXX_ENABLE_SHARED=OFF",
            "-DLIBCXX_ENABLE_STATIC=ON",
            "-DLIBCXX_ENABLE_EXCEPTIONS=OFF",
            "-DLIBCXX_ENABLE_RTTI=OFF",
            "-DLIBCXX_ENABLE_FILESYSTEM=OFF",
            "-DLIBCXX_ENABLE_LOCALIZATION=OFF",
            "-DLIBCXX_ENABLE_WIDE_CHARACTERS=ON",
            "-DLIBCXX_ENABLE_THREADS=ON",
            "-DLIBCXX_HAS_MUSL_LIBC=ON",
            "-DLIBCXXABI_ENABLE_SHARED=OFF",
            "-DLIBCXXABI_ENABLE_STATIC=ON",
            "-DLIBCXXABI_ENABLE_EXCEPTIONS=OFF",
            "-DLIBCXXABI_USE_LLVM_UNWINDER=ON",
            "-DLIBUNWIND_ENABLE_SHARED=OFF",
            "-DLIBUNWIND_ENABLE_STATIC=ON",
        ]
    )
    run([cmake, "--build", str(build_dir), "--target", "install", "-j", os.environ.get("JOBS", str(os.cpu_count() or 1))])
    write_env(prefix, "libc++", [prefix / "include" / "c++" / "v1"], prefix / "lib", ["c++", "c++abi", "unwind"])
    write_cxx_wrapper(repo_root, prefix)
    check_prefix(repo_root, prefix)
    print(f"built libc++ runtime into {prefix}", flush=True)


def install_from_muslcc(repo_root: Path, prefix: Path, clean: bool) -> None:
    cache = repo_root / "target" / "cxx-musl-downloads"
    archive = cache / MUSLCC_ARCHIVE
    sums = cache / "SHA512SUMS"
    extract_root = repo_root / "target" / "cxx-musl-toolchain"
    toolchain = extract_root / f"{TARGET_TRIPLE}-cross"

    if clean:
        for path in [archive, sums, extract_root, prefix]:
            if path.exists() or path.is_symlink():
                remove_path(path)

    download(MUSLCC_URL, archive)
    download(MUSLCC_SHA512SUMS_URL, sums)
    verify_muslcc_archive(archive, sums)

    toolchain_ready = (
        (toolchain / "bin" / f"{TARGET_TRIPLE}-g++").is_file()
        and (toolchain / TARGET_TRIPLE / "include" / "c++").is_dir()
    )
    if not toolchain_ready:
        if extract_root.exists():
            shutil.rmtree(extract_root)
        safe_extract_tar(archive, extract_root)

    install_from_sysroot(repo_root, toolchain, prefix, symlink=True)


def read_env(prefix: Path) -> dict[str, str]:
    env_file = prefix / "cxx-runtime.env"
    values: dict[str, str] = {}
    if env_file.is_file():
        for line in env_file.read_text().splitlines():
            if not line or line.startswith("#") or "=" not in line:
                continue
            key, value = line.split("=", 1)
            values[key] = value
    return values


def include_flags(prefix: Path) -> list[str]:
    env_values = read_env(prefix)
    raw = env_values.get("CXX_INCLUDE_FLAGS")
    if raw:
        return raw.split()
    if (prefix / "include" / "c++" / "v1").is_dir():
        return ["-nostdinc++", "-isystem", str(prefix / "include" / "c++" / "v1")]
    roots = sorted((prefix / "include" / "c++").glob("*"), reverse=True)
    for root in roots:
        if not root.is_dir():
            continue
        flags = ["-nostdinc++", "-isystem", str(root)]
        target = root / TARGET_TRIPLE
        if target.is_dir():
            flags.extend(["-isystem", str(target)])
        backward = root / "backward"
        if backward.is_dir():
            flags.extend(["-isystem", str(backward)])
        return flags
    raise SystemExit(f"missing C++ headers under {prefix}/include/c++")


def write_cxx_wrapper(repo_root: Path, prefix: Path) -> None:
    wrapper = prefix / "bin" / "musl-g++"
    specs = repo_root / "target" / "cpython-musl" / "toolchain" / "lib" / "musl-gcc.specs"
    flags = " ".join(include_flags(prefix))
    wrapper.write_text(
        "#!/bin/sh\n"
        f'exec "${{REALGXX:-g++}}" "$@" -specs "{specs}" {flags} -L "{prefix / "lib"}"\n'
    )
    wrapper.chmod(0o755)


def check_prefix(repo_root: Path, prefix: Path) -> None:
    lib_dir = prefix / "lib"
    if not lib_dir.is_dir():
        raise SystemExit(f"missing {lib_dir}")
    if not any((lib_dir / name).is_file() for name in ["libc++.a", "libstdc++.a"]):
        raise SystemExit(f"missing libc++.a or libstdc++.a in {lib_dir}")

    build_dir = repo_root / "target" / "cxx-musl-check"
    build_dir.mkdir(parents=True, exist_ok=True)
    source = build_dir / "check.cpp"
    obj = build_dir / "check.o"
    source.write_text(
        "#include <array>\n"
        "#include <cmath>\n"
        "static std::array<int, 2> v = {1, 2};\n"
        "int alloy_cxx_check(void) { return v[0] + (int)std::sqrt(4.0); }\n"
    )
    cxx = os.environ.get("CXX", str(prefix / "bin" / "musl-g++"))
    command = [cxx, *include_flags(prefix), "-O2", "-fPIC", "-c", str(source), "-o", str(obj)]
    result = subprocess.run(command, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    if result.returncode != 0:
        sys.stderr.write("+ " + " ".join(command) + "\n")
        sys.stderr.write(result.stdout[-4000:])
        sys.stderr.write(result.stderr[-4000:])
        raise SystemExit("C++ runtime preflight failed")


def probe_sysroots() -> list[Path]:
    candidates = [
        Path("/usr") / TARGET_TRIPLE,
        Path("/usr/local") / TARGET_TRIPLE,
        Path("/opt") / TARGET_TRIPLE,
        Path("/opt/musl"),
    ]
    gxx = shutil.which(f"{TARGET_TRIPLE}-g++") or shutil.which("musl-g++")
    if gxx:
        # Usually .../bin/x86_64-linux-musl-g++.
        candidates.insert(0, Path(gxx).resolve().parents[1])
    return candidates


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--prefix", type=Path, default=None)
    parser.add_argument("--from-sysroot", type=Path, default=None)
    parser.add_argument("--llvm-project", type=Path, default=None)
    parser.add_argument(
        "--download-muslcc",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="download https://musl.cc/x86_64-linux-musl-cross.tgz if no local runtime is found",
    )
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--clean", action="store_true")
    parser.add_argument("--copy", action="store_true", help="copy files instead of symlinking for --from-sysroot")
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[1]
    prefix = args.prefix or repo_root / "target" / "cxx-musl"

    if args.check:
        check_prefix(repo_root, prefix)
        print(f"ok: {prefix}", flush=True)
        return 0

    if args.llvm_project is not None:
        build_from_llvm(repo_root, args.llvm_project.resolve(), prefix, args.clean)
        return 0

    sysroot = args.from_sysroot
    if sysroot is None:
        for candidate in probe_sysroots():
            try:
                detect_runtime(candidate)
            except SystemExit:
                continue
            sysroot = candidate
            break
    if sysroot is None:
        if args.download_muslcc:
            install_from_muslcc(repo_root, prefix, args.clean)
            return 0
        raise SystemExit(
            "no musl C++ runtime found. Provide one with "
            "`python3 scripts/build_cxx_musl.py --from-sysroot /path/to/x86_64-linux-musl`, "
            "or build LLVM runtimes with "
            "`python3 scripts/build_cxx_musl.py --llvm-project third_party/llvm-project`."
        )

    install_from_sysroot(repo_root, sysroot.resolve(), prefix, symlink=not args.copy)
    return 0


if __name__ == "__main__":
    sys.exit(main())
