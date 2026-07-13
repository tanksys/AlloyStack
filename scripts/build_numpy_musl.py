#!/usr/bin/env python3
"""Build NumPy as built-in CPython extension modules for musl CPython.

This follows the Unikraft lib-python-numpy approach: NumPy C extensions are
compiled into a static archive and imported through flat built-in module names
used by the importfix Python shims.
"""

from __future__ import annotations

import argparse
import concurrent.futures
import datetime
import os
import re
import shutil
import subprocess
import sys
import tarfile
import urllib.request
from pathlib import Path


NUMPY_VERSION = "1.25.0"
NUMPY_URL = (
    f"https://github.com/numpy/numpy/releases/download/"
    f"v{NUMPY_VERSION}/numpy-{NUMPY_VERSION}.tar.gz"
)

LOG_FILE: Path | None = None


def init_log(repo_root: Path) -> None:
    global LOG_FILE
    log_dir = repo_root / "target" / "logs"
    log_dir.mkdir(parents=True, exist_ok=True)
    LOG_FILE = log_dir / "build_numpy_musl.log"
    LOG_FILE.write_text(
        f"[{datetime.datetime.now().isoformat(timespec='seconds')}] "
        "start NumPy musl build\n"
    )


def log(message: str) -> None:
    if LOG_FILE is None:
        return
    with LOG_FILE.open("a", encoding="utf-8") as fh:
        fh.write(message)
        if not message.endswith("\n"):
            fh.write("\n")

BUILTIN_MODULES = [
    ("numpy_core__multiarray_umath", "PyInit__multiarray_umath"),
    ("numpy_core__multiarray_tests", "PyInit__multiarray_tests"),
    ("numpy_core__operand_flag_tests", "PyInit__operand_flag_tests"),
    ("numpy_core__rational_tests", "PyInit__rational_tests"),
    ("numpy_core__simd", "PyInit__simd"),
    ("numpy_core__struct_ufunc_tests", "PyInit__struct_ufunc_tests"),
    ("numpy_core__umath_tests", "PyInit__umath_tests"),
    ("numpy_fft__pocketfft_internal", "PyInit__pocketfft_internal"),
    ("numpy_linalg_lapack_lite", "PyInit_lapack_lite"),
    ("numpy_linalg__umath_linalg", "PyInit__umath_linalg"),
    ("numpy_random_bit_generator", "PyInit_bit_generator"),
    ("numpy_random__bounded_integers", "PyInit__bounded_integers"),
    ("numpy_random__common", "PyInit__common"),
    ("numpy_random__generator", "PyInit__generator"),
    ("numpy_random__mt19937", "PyInit__mt19937"),
    ("numpy_random_mtrand", "PyInit_mtrand"),
    ("numpy_random__pcg64", "PyInit__pcg64"),
    ("numpy_random__philox", "PyInit__philox"),
    ("numpy_random__sfc64", "PyInit__sfc64"),
]


def run(command: list[str], cwd: Path | None = None) -> None:
    log("+ " + " ".join(command))
    result = subprocess.run(
        command,
        cwd=cwd,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    if result.stdout:
        log(result.stdout)
    if result.returncode != 0:
        location = f"; see {LOG_FILE}" if LOG_FILE else ""
        raise RuntimeError(f"command failed: {' '.join(command)}{location}")


def run_quiet(command: list[str], cwd: Path | None = None) -> None:
    log("+ " + " ".join(command))
    result = subprocess.run(
        command,
        cwd=cwd,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.stdout:
        log(result.stdout)
    if result.stderr:
        log(result.stderr)
    if result.returncode != 0:
        sys.stderr.write("+ " + " ".join(command) + "\n")
        if LOG_FILE:
            sys.stderr.write(f"see {LOG_FILE} for full compiler output\n")
        else:
            if result.stdout:
                sys.stderr.write(result.stdout[-4000:])
            if result.stderr:
                sys.stderr.write(result.stderr[-4000:])
        raise subprocess.CalledProcessError(result.returncode, command)


def output(command: list[str]) -> str:
    return subprocess.check_output(command, text=True).strip()


def read_env_file(path: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    if not path.is_file():
        return values
    for line in path.read_text().splitlines():
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        values[key] = value
    return values


def cxx_runtime_include_flags(repo_root: Path) -> list[str]:
    prefix = Path(os.environ.get("CXX_MUSL_PREFIX", repo_root / "target" / "cxx-musl"))
    env_values = read_env_file(prefix / "cxx-runtime.env")
    raw_flags = env_values.get("CXX_INCLUDE_FLAGS")
    if raw_flags:
        return raw_flags.split()

    libcxx = prefix / "include" / "c++" / "v1"
    if libcxx.is_dir():
        return ["-nostdinc++", "-isystem", str(libcxx)]

    include_root = prefix / "include" / "c++"
    if include_root.is_dir():
        for version in sorted(include_root.iterdir(), reverse=True):
            if not version.is_dir():
                continue
            flags = ["-nostdinc++", "-isystem", str(version)]
            target_dir = version / "x86_64-linux-musl"
            if target_dir.is_dir():
                flags.extend(["-isystem", str(target_dir)])
            backward = version / "backward"
            if backward.is_dir():
                flags.extend(["-isystem", str(backward)])
            return flags

    raise SystemExit(
        "missing musl C++ runtime. Run `python3 scripts/build_cxx_musl.py` "
        "or set CXX_MUSL_PREFIX to a prefix containing include/c++ and static "
        "C++ runtime libraries."
    )


def compiler_supports(compiler: str, flags: list[str], language: str) -> bool:
    command = [
        compiler,
        *flags,
        "-x",
        language,
        "-c",
        os.devnull,
        "-o",
        os.devnull,
    ]
    return subprocess.run(
        command,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    ).returncode == 0


def ensure_numpy_source(repo_root: Path, clean: bool) -> Path:
    cache = repo_root / "target" / "numpy-musl"
    source_root = cache / "src" / f"numpy-{NUMPY_VERSION}"
    archive = cache / "downloads" / f"numpy-{NUMPY_VERSION}.tar.gz"
    patches = repo_root / "third_party" / "lib-python-numpy" / "patches"

    if clean and source_root.exists():
        shutil.rmtree(source_root)

    if source_root.exists():
        return source_root

    archive.parent.mkdir(parents=True, exist_ok=True)
    if not archive.exists():
        print(f"downloading {NUMPY_URL}", flush=True)
        urllib.request.urlretrieve(NUMPY_URL, archive)

    extract_dir = cache / "src"
    extract_dir.mkdir(parents=True, exist_ok=True)
    with tarfile.open(archive) as tar:
        tar.extractall(extract_dir)

    for patch_file in sorted(patches.glob("*.patch")):
        run(["patch", "-d", str(source_root), "-p1", "-i", str(patch_file)])

    return source_root


def parse_sources(repo_root: Path, source_root: Path) -> list[Path]:
    makefile = repo_root / "third_party" / "lib-python-numpy" / "Makefile.uk"
    generated = repo_root / "third_party" / "lib-python-numpy" / "generated" / "numpy"
    source = source_root / "numpy"
    sources: list[Path] = []

    pattern = re.compile(r"LIBPYTHON_NUMPY_SRCS(?:-y|-\$\([^)]*\)) \+= (.+)$")
    for line in makefile.read_text().splitlines():
        match = pattern.match(line.strip())
        if not match:
            continue
        entry = match.group(1)
        if "OPENBLAS" in line:
            continue
        entry = entry.replace("$(LIBPYTHON_NUMPY_SRC)", str(source))
        entry = entry.replace("$(LIBPYTHON_NUMPY_BSRC)", str(generated))
        path = Path(entry)
        if path.name == "arm64_exports.c":
            continue
        if path not in sources:
            sources.append(path)
    return sources


def file_flags(path: Path, compiler: str, language: str) -> list[str] | None:
    name = path.name
    sse41 = ["-msse", "-msse2", "-msse3", "-mssse3", "-msse4.1"]
    sse42 = [*sse41, "-mpopcnt", "-msse4.2"]
    avx2 = [*sse42, "-mavx", "-mf16c", "-mavx2"]
    fma3_avx2 = [*avx2, "-mfma"]
    avx512f = [*avx2, "-mavx512f", "-mno-mmx"]
    avx512_skx = [*avx512f, "-mavx512cd", "-mavx512vl", "-mavx512bw", "-mavx512dq"]
    avx512_icl = [
        *avx512_skx,
        "-mavx512vnni",
        "-mavx512ifma",
        "-mavx512vbmi",
        "-mavx512vbmi2",
        "-mavx512bitalg",
        "-mavx512vpopcntdq",
    ]
    avx512_spr = [*avx512_icl, "-mavx512fp16"]

    if "avx512_spr" in name:
        flags = avx512_spr
    elif "avx512_icl" in name:
        flags = avx512_icl
    elif "avx512_skx" in name:
        flags = avx512_skx
    elif "avx512f" in name:
        flags = avx512f
    elif "fma3.avx2" in name:
        flags = fma3_avx2
    elif "avx2" in name:
        flags = avx2
    elif "sse42" in name:
        flags = sse42
    elif "sse41" in name:
        flags = sse41
    else:
        return []

    if compiler_supports(compiler, flags, language):
        return flags
    print(f"skip unsupported SIMD source: {path.name}", flush=True)
    return None


def object_path(build_dir: Path, source: Path) -> Path:
    safe = re.sub(r"[^A-Za-z0-9_.-]+", "_", str(source))
    return build_dir / "obj" / f"{safe}.o"


def compile_one(
    source: Path,
    obj: Path,
    cc: str,
    cxx: str,
    cflags: list[str],
    cxxflags: list[str],
) -> Path | None:
    if obj.exists() and obj.stat().st_mtime >= source.stat().st_mtime:
        return obj
    language = "c++" if source.suffix in {".cpp", ".cc", ".cxx"} else "c"
    compiler = cxx if language == "c++" else cc
    simd_flags = file_flags(source, compiler, language)
    if simd_flags is None:
        return None
    obj.parent.mkdir(parents=True, exist_ok=True)
    flags = cxxflags if language == "c++" else cflags
    run_quiet([compiler, *flags, *simd_flags, "-c", str(source), "-o", str(obj)])
    return obj


def check_cxx_preflight(cxx: str, cxxflags: list[str], build_dir: Path) -> None:
    source = build_dir / "cxx_preflight.cpp"
    obj = build_dir / "cxx_preflight.o"
    source.write_text(
        "#include <Python.h>\n"
        "#include <array>\n"
        "#include <cmath>\n"
        "static std::array<int, 1> value = {1};\n"
        "int alloy_numpy_cxx_preflight(void) { return value[0] + (int)std::sqrt(4.0); }\n"
    )
    result = subprocess.run(
        [cxx, *cxxflags, "-c", str(source), "-o", str(obj)],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    log("+ " + " ".join([cxx, *cxxflags, "-c", str(source), "-o", str(obj)]))
    if result.stdout:
        log(result.stdout)
    if result.stderr:
        log(result.stderr)
    if result.returncode != 0:
        raise SystemExit(
            "musl NumPy C-extension build needs a musl-compatible C++ "
            "standard library. The current C++ toolchain cannot compile "
            "<array>/<cmath> against AlloyStack musl headers. Use "
            "`python3 scripts/build_numpy_musl.py --minimal` for the smoke "
            "test path, or provide a libc++/musl C++ toolchain via CXX."
        )


def generate_inittab(build_dir: Path) -> Path:
    source = build_dir / "numpy_inittab.c"
    lines = [
        "#include <Python.h>",
        "",
    ]
    for _, init in BUILTIN_MODULES:
        lines.append(f"extern PyObject *{init}(void);")
    lines.extend(
        [
            "",
            "int alloy_numpy_register(void)",
            "{",
            "    struct alloy_numpy_module {",
            "        const char *name;",
            "        PyObject *(*init)(void);",
            "    };",
            "    static const struct alloy_numpy_module modules[] = {",
        ]
    )
    for name, init in BUILTIN_MODULES:
        lines.append(f'        {{"{name}", {init}}},')
    lines.extend(
        [
            "    };",
            "    for (size_t i = 0; i < sizeof(modules) / sizeof(modules[0]); ++i) {",
            "        if (PyImport_AppendInittab(modules[i].name, modules[i].init) < 0)",
            "            return -1;",
            "    }",
            "    return 0;",
            "}",
            "",
        ]
    )
    source.write_text("\n".join(lines))
    return source


def build_archive(repo_root: Path, clean: bool) -> None:
    third_party = repo_root / "third_party" / "lib-python-numpy"
    if not (third_party / "Makefile.uk").is_file():
        raise SystemExit("missing third_party/lib-python-numpy; run git submodule update --init")

    print(f"building NumPy musl archive; log: {LOG_FILE}", flush=True)
    source_root = ensure_numpy_source(repo_root, clean)
    cache = repo_root / "target" / "numpy-musl"
    build_dir = cache / "build"
    lib_dir = cache / "lib"
    include_dir = cache / "include"
    if clean and build_dir.exists():
        shutil.rmtree(build_dir)
    if clean and lib_dir.exists():
        shutil.rmtree(lib_dir)
    build_dir.mkdir(parents=True, exist_ok=True)
    lib_dir.mkdir(parents=True, exist_ok=True)
    include_dir.mkdir(parents=True, exist_ok=True)

    cc = os.environ.get("CC", "cc")
    cxx_runtime_prefix = Path(os.environ.get("CXX_MUSL_PREFIX", repo_root / "target" / "cxx-musl"))
    cxx = os.environ.get("CXX", str(cxx_runtime_prefix / "bin" / "musl-g++"))
    ar = os.environ.get("AR", "ar")
    jobs = int(os.environ.get("JOBS", str(os.cpu_count() or 1)))

    musl_dir = repo_root / "target" / "musl-alloy"
    cpython_dir = repo_root / "target" / "cpython-musl"
    python_include = Path(os.environ.get("PYTHON_INCLUDE", repo_root / "third_party" / "cpython" / "Include"))
    python_config_include = Path(
        os.environ.get("PYTHON_CONFIG_INCLUDE", cpython_dir / "build-submodule")
    )
    gcc_include = output([cc, "-print-file-name=include"])
    cxx_runtime_flags = cxx_runtime_include_flags(repo_root)
    generated = third_party / "generated" / "numpy"
    numpy_source = source_root / "numpy"

    common_flags = [
        "-O2",
        "-fPIC",
        "-ffreestanding",
        "-fno-stack-protector",
        "-fwrapv",
        "-fasynchronous-unwind-tables",
        "-nostdinc",
        "-D_GNU_SOURCE",
        "-D_FILE_OFFSET_BITS=64",
        "-D_LARGEFILE64_SOURCE=1",
        "-D_LARGEFILE_SOURCE=1",
        "-DDYNAMIC_ANNOTATIONS_ENABLED=1",
        "-DHAVE_NPY_CONFIG_H=1",
        "-DNPY_INTERNAL_BUILD=1",
        "-isystem",
        str(musl_dir / "include"),
        "-I",
        str(repo_root / "as_musl" / "include"),
        "-isystem",
        str(python_config_include),
        "-isystem",
        str(python_include),
        "-isystem",
        str(gcc_include),
        "-I",
        str(numpy_source / "core" / "include"),
        "-I",
        str(generated / "core" / "include"),
        "-I",
        str(generated / "distutils" / "include"),
    ]
    for rel in [
        "core",
        "core/include",
        "core/include/numpy",
        "core/src",
        "core/src/common",
        "core/src/multiarray",
        "core/src/npymath",
        "core/src/npysort",
        "core/src/_simd",
        "core/src/umath",
        "random",
    ]:
        common_flags.extend(["-I", str(generated / rel)])
        common_flags.extend(["-iquote", str(generated / rel)])
        common_flags.extend(["-I", str(numpy_source / rel)])

    cflags = [*common_flags, "-std=c11"]
    cxxflags = [*cxx_runtime_flags]
    cxx_common_flags = [flag for flag in common_flags if flag != "-ffreestanding"]
    cxxflags.extend([
        *cxx_common_flags,
        "-std=c++17",
        "-fno-exceptions",
        "-fno-rtti",
        "-fno-use-cxa-atexit",
    ])
    check_cxx_preflight(cxx, cxxflags, build_dir)

    sources = [path for path in parse_sources(repo_root, source_root) if path.is_file()]
    missing = [path for path in parse_sources(repo_root, source_root) if not path.is_file()]
    if missing:
        print(f"warning: {len(missing)} NumPy source files were not found", flush=True)
        for path in missing[:10]:
            print(f"  missing: {path}", flush=True)

    objects: list[Path] = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as executor:
        futures = [
            executor.submit(
                compile_one,
                source,
                object_path(build_dir, source),
                cc,
                cxx,
                cflags,
                cxxflags,
            )
            for source in sources
        ]
        for future in concurrent.futures.as_completed(futures):
            obj = future.result()
            if obj is not None:
                objects.append(obj)

    inittab = generate_inittab(build_dir)
    inittab_obj = build_dir / "obj" / "numpy_inittab.o"
    run([cc, *cflags, "-c", str(inittab), "-o", str(inittab_obj)])
    objects.append(inittab_obj)

    archive = lib_dir / "libnumpy_musl.a"
    if archive.exists():
        archive.unlink()
    run([ar, "rcs", str(archive), *map(str, sorted(objects))])

    # Expose generated headers for downstream C extensions if needed later.
    numpy_include = include_dir / "numpy"
    if numpy_include.exists():
        shutil.rmtree(numpy_include)
    shutil.copytree(generated / "core" / "include" / "numpy", numpy_include)
    print(f"built {archive}", flush=True)


def build_minimal_archive(repo_root: Path, clean: bool) -> None:
    print(f"building minimal NumPy musl archive; log: {LOG_FILE}", flush=True)
    cache = repo_root / "target" / "numpy-musl"
    build_dir = cache / "build-minimal"
    lib_dir = cache / "lib"
    if clean and build_dir.exists():
        shutil.rmtree(build_dir)
    build_dir.mkdir(parents=True, exist_ok=True)
    lib_dir.mkdir(parents=True, exist_ok=True)

    cc = os.environ.get("CC", "cc")
    ar = os.environ.get("AR", "ar")
    source = build_dir / "numpy_stub.c"
    obj = build_dir / "numpy_stub.o"
    archive = lib_dir / "libnumpy_musl.a"
    source.write_text("int alloy_numpy_register(void) { return 0; }\n")
    run([cc, "-O2", "-fPIC", "-c", str(source), "-o", str(obj)])
    if archive.exists():
        archive.unlink()
    run([ar, "rcs", str(archive), str(obj)])
    print(f"built minimal {archive}", flush=True)


def copy_py_files(source: Path, destination: Path) -> None:
    if destination.exists():
        shutil.rmtree(destination)
    destination.mkdir(parents=True, exist_ok=True)
    for path in source.rglob("*"):
        rel = path.relative_to(source)
        target = destination / rel
        if path.is_dir():
            target.mkdir(exist_ok=True)
        elif path.suffix == ".py":
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(path, target)


def disable_numpy_doc_initializers(numpy_root: Path) -> None:
    for rel in [
        "core/_add_newdocs.py",
        "core/_add_newdocs_scalars.py",
    ]:
        target = numpy_root / rel
        if target.exists():
            target.write_text(
                "# Disabled for AlloyStack musl smoke workloads.\n"
                "# These modules only attach docstrings to NumPy objects.\n"
            )


def reduce_numpy_top_level_imports(numpy_root: Path) -> None:
    init_py = numpy_root / "__init__.py"
    if not init_py.is_file():
        return
    text = init_py.read_text()
    replacements = {
        "    from . import lib\n": "    # AlloyStack smoke: skip eager import of lib\n",
        "    from .lib import *\n": "    # AlloyStack smoke: skip eager import of lib symbols\n",
        "    from . import linalg\n": "    # AlloyStack smoke: skip eager import of linalg\n",
        "    from . import fft\n": "    # AlloyStack smoke: skip eager import of fft\n",
        "    from . import polynomial\n": "    # AlloyStack smoke: skip eager import of polynomial\n",
        "    from . import random\n": "    # AlloyStack smoke: skip eager import of random\n",
        "    from . import ctypeslib\n": "    # AlloyStack smoke: skip eager import of ctypeslib\n",
        "    from . import ma\n": "    # AlloyStack smoke: skip eager import of ma\n",
        "    from . import matrixlib as _mat\n": "    # AlloyStack smoke: skip eager import of matrixlib\n",
        "    from .matrixlib import *\n": "    # AlloyStack smoke: skip eager import of matrixlib symbols\n",
        "    core.getlimits._register_known_types()\n": (
            "    # AlloyStack smoke: skip eager dtype limit registration\n"
        ),
        "    _sanity_check()\n": "    # AlloyStack smoke: skip NumPy import-time sanity check\n",
        "    use_hugepage = os.environ.get(\"NUMPY_MADVISE_HUGEPAGE\", None)\n": (
            "    use_hugepage = 0\n"
        ),
        "    core.multiarray._set_madvise_hugepage(use_hugepage)\n": (
            "    # AlloyStack smoke: skip import-time madvise hugepage setup\n"
        ),
        "    __all__.extend(lib.__all__)\n": "    # AlloyStack smoke: lib was not eagerly imported\n",
        "    __all__.extend(_mat.__all__)\n": "    # AlloyStack smoke: matrixlib was not eagerly imported\n",
        "    __all__.extend(['linalg', 'fft', 'random', 'ctypeslib', 'ma'])\n": (
            "    # AlloyStack smoke: optional top-level packages were not eagerly imported\n"
        ),
        "    __all__.remove('Arrayterator')\n": (
            "    # AlloyStack smoke: Arrayterator is provided by numpy.lib, "
            "which was not eagerly imported\n"
        ),
        "    del Arrayterator\n": "    # AlloyStack smoke: Arrayterator was not imported\n",
    }
    for old, new in replacements.items():
        text = text.replace(old, new)
    init_py.write_text(text)


def prepare_rootfs(repo_root: Path, clean: bool) -> Path:
    source_root = ensure_numpy_source(repo_root, clean)
    cache = repo_root / "target" / "numpy-musl"
    rootfs = Path(os.environ.get("NUMPY_MUSL_ROOTFS", cache / "rootfs"))
    numpy_dst = rootfs / "Lib" / "site-packages" / "numpy"
    numpy_src = source_root / "numpy"
    generated = repo_root / "third_party" / "lib-python-numpy" / "generated" / "numpy"
    importfix = repo_root / "third_party" / "lib-python-numpy" / "importfix" / "numpy"

    copy_py_files(numpy_src, numpy_dst)
    shutil.copy2(generated / "__config__.py", numpy_dst / "__config__.py")
    distutils_dst = numpy_dst / "distutils"
    distutils_dst.mkdir(parents=True, exist_ok=True)
    shutil.copy2(generated / "distutils" / "__config__.py", distutils_dst / "__config__.py")
    shutil.copytree(importfix, numpy_dst, dirs_exist_ok=True)
    disable_numpy_doc_initializers(numpy_dst)
    reduce_numpy_top_level_imports(numpy_dst)
    print(f"prepared {numpy_dst}", flush=True)
    return rootfs


def prepare_minimal_rootfs(repo_root: Path, clean: bool) -> Path:
    cache = repo_root / "target" / "numpy-musl"
    rootfs = Path(os.environ.get("NUMPY_MUSL_ROOTFS", cache / "rootfs"))
    source = repo_root / "user" / "python_workloads" / "numpy_minimal" / "numpy"
    destination = rootfs / "Lib" / "site-packages" / "numpy"
    if clean and destination.exists():
        shutil.rmtree(destination)
    if destination.exists():
        shutil.rmtree(destination)
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copytree(source, destination)
    print(f"prepared minimal {destination}", flush=True)
    return rootfs


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--clean", action="store_true")
    parser.add_argument("--rootfs-only", action="store_true")
    parser.add_argument("--archive-only", action="store_true")
    parser.add_argument(
        "--minimal",
        action="store_true",
        help="build the small pure-Python numpy-compatible smoke-test package",
    )
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[1]
    init_log(repo_root)
    if args.minimal:
        if not args.rootfs_only:
            build_minimal_archive(repo_root, args.clean)
        if not args.archive_only:
            prepare_minimal_rootfs(repo_root, args.clean)
    else:
        if not args.rootfs_only:
            build_archive(repo_root, args.clean)
        if not args.archive_only:
            prepare_rootfs(repo_root, args.clean)
    return 0


if __name__ == "__main__":
    sys.exit(main())
