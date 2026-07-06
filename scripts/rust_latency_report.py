#!/usr/bin/env python3

import argparse
import json
import os
import re
import signal
import subprocess
import sys
import tempfile
import time
from pathlib import Path


RESULTS_SCHEMA_VERSION = 3
TOTAL_DUR_RE = re.compile(r'"total_dur\(ms\)":\s*([0-9]+(?:\.[0-9]+)?)')
PROGRESS_INTERVAL_SECONDS = 5
DEFAULT_TEST_TIMEOUT_SECONDS = 10 * 60
BACKENDS = ["rust", "wasm-c", "wasm-py", "musl-c", "musl-py"]

WORD_COUNT_ROWS = [
    ("10MB", "10 * 1024 * 1024"),
    ("100MB", "100 * 1024 * 1024"),
    ("300MB", "300 * 1024 * 1024"),
]
PARALLEL_SORT_ROWS = [
    ("1MB", "1 * 1024 * 1024"),
    ("25MB", "25 * 1024 * 1024"),
    ("50MB", "50 * 1024 * 1024"),
]
LONG_CHAIN_ROWS = [
    ("1MB", "1 * 1024 * 1024"),
    ("64MB", "64 * 1024 * 1024"),
    ("256MB", "256 * 1024 * 1024"),
]

PARALLEL_COLUMNS = ["C1", "C3", "C5"]
LONG_CHAIN_COLUMNS = ["n5", "n10", "n15"]

WORD_COUNT_CONFIGS = {
    "C1": "isol_config/map_reduce_large_c1.json",
    "C3": "isol_config/map_reduce_large_c3.json",
    "C5": "isol_config/map_reduce_large_c5.json",
}
PARALLEL_SORT_CONFIGS = {
    "C1": "isol_config/parallel_sort_c1.json",
    "C3": "isol_config/parallel_sort_c3.json",
    "C5": "isol_config/parallel_sort_c5.json",
}
LONG_CHAIN_CONFIGS = {
    "n5": "isol_config/long_chain_n5.json",
    "n10": "isol_config/long_chain_n10.json",
    "n15": "isol_config/long_chain_n15.json",
}

CONFIG_TEMPLATES = {
    "Word Count": {
        "wasm-c": "isol_config/wasmtime_wordcount_c3.json",
        "wasm-py": "isol_config/wasmtime_cpython_wordcount_c3.json",
        "musl-c": "isol_config/musl_wordcount_c3.json",
        "musl-py": "isol_config/musl_cpython_wordcount_c3.json",
    },
    "Parallel Sort": {
        "wasm-c": "isol_config/wasmtime_parallel_sort_c3.json",
        "wasm-py": "isol_config/wasmtime_cpython_parallel_sort_c3.json",
        "musl-c": "isol_config/musl_parallel_sort_c3.json",
        "musl-py": "isol_config/musl_cpython_parallel_sort_c3.json",
    },
    "Long Chain": {
        "wasm-c": "isol_config/wasmtime_longchain.json",
        "wasm-py": "isol_config/wasmtime_cpython_functionchain_n10.json",
        "musl-c": "isol_config/musl_longchain.json",
        "musl-py": "isol_config/musl_cpython_functionchain_n10.json",
    },
}


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def eval_size_expr(expr: str) -> int:
    parts = [part.strip() for part in expr.split("*")]
    value = 1
    for part in parts:
        if not part:
            raise ValueError(f"invalid data size expression: {expr}")
        value *= int(part)
    return value


def empty_results() -> dict:
    return {
        "Word Count": {
            backend: {
                row: {column: "N/A" for column in PARALLEL_COLUMNS}
                for row, _ in WORD_COUNT_ROWS
            }
            for backend in BACKENDS
        },
        "Parallel Sort": {
            backend: {
                row: {column: "N/A" for column in PARALLEL_COLUMNS}
                for row, _ in PARALLEL_SORT_ROWS
            }
            for backend in BACKENDS
        },
        "Long Chain": {
            backend: {
                row: {column: "N/A" for column in LONG_CHAIN_COLUMNS}
                for row, _ in LONG_CHAIN_ROWS
            }
            for backend in BACKENDS
        },
    }


def run_command(
    command: str,
    cwd: Path,
    unset_cc: bool,
    *,
    progress_label: str | None = None,
    interactive: bool = False,
    timeout_seconds: int | None = None,
) -> str:
    env = os.environ.copy()
    if unset_cc:
        env.pop("CC", None)

    if interactive:
        completed = subprocess.run(["bash", "-lc", command], cwd=cwd, env=env, text=True)
        if completed.returncode != 0:
            raise RuntimeError(f"command failed: {command}")
        return ""

    process = subprocess.Popen(
        ["bash", "-lc", command],
        cwd=cwd,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        start_new_session=True,
    )
    started_at = time.monotonic()
    while True:
        elapsed = time.monotonic() - started_at
        if timeout_seconds is not None and elapsed >= timeout_seconds:
            os.killpg(process.pid, signal.SIGTERM)
            try:
                output, _ = process.communicate(timeout=5)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                output, _ = process.communicate()
            detail = output.strip()
            if detail:
                detail += "\n"
            raise RuntimeError(
                f"{detail}command timed out after {timeout_seconds}s"
            )
        wait_seconds = PROGRESS_INTERVAL_SECONDS
        if timeout_seconds is not None:
            wait_seconds = min(wait_seconds, timeout_seconds - elapsed)
        try:
            output, _ = process.communicate(timeout=wait_seconds)
            break
        except subprocess.TimeoutExpired:
            if progress_label:
                elapsed = int(time.monotonic() - started_at)
                print(
                    f"[progress] {progress_label}: running for {elapsed}s",
                    file=sys.stderr,
                    flush=True,
                )

    if process.returncode != 0:
        raise RuntimeError(output.strip() or f"command failed: {command}")
    return output


def run_prepare(cwd: Path, unset_cc: bool) -> None:
    commands = [
        "sudo mount fs_images/fatfs.img image_content 2>/dev/null || true",
        "just asvisor >/dev/null",
        "just all_libos >/dev/null",
        "just map_reduce >/dev/null",
        "just parallel_sort >/dev/null",
        "just all_c_wasm >/dev/null",
        "just all_py_wasm >/dev/null",
        "just all_musl_c >/dev/null",
        "just all_py_musl >/dev/null",
        "sudo -E ./scripts/sync_python_workloads.sh",
    ]
    for command in commands:
        print(f"[prepare] {command}", file=sys.stderr, flush=True)
        run_command(
            command,
            cwd,
            unset_cc,
            progress_label=f"prepare: {command}",
            interactive=command.startswith("sudo "),
        )
    ensure_image_mounted(cwd, unset_cc)
    run_command("sync", cwd, unset_cc, progress_label="sync filesystem image")
    unmount_image(cwd, unset_cc)


def ensure_image_mounted(cwd: Path, unset_cc: bool) -> None:
    try:
        run_command("mountpoint -q image_content", cwd, unset_cc)
    except RuntimeError as exc:
        raise RuntimeError(
            "image_content is not mounted; run `sudo mount fs_images/fatfs.img image_content` "
            "or run without skip_prepare=1"
        ) from exc


def mount_image(cwd: Path, unset_cc: bool) -> None:
    try:
        run_command("mountpoint -q image_content", cwd, unset_cc)
        return
    except RuntimeError:
        pass
    run_command(
        "sudo mount fs_images/fatfs.img image_content",
        cwd,
        unset_cc,
        progress_label="mount filesystem image",
        interactive=True,
    )
    ensure_image_mounted(cwd, unset_cc)


def unmount_image(cwd: Path, unset_cc: bool) -> None:
    try:
        run_command("mountpoint -q image_content", cwd, unset_cc)
    except RuntimeError:
        return
    run_command(
        "sudo umount image_content",
        cwd,
        unset_cc,
        progress_label="unmount filesystem image",
        interactive=True,
    )


def parse_total_dur(output: str) -> float:
    match = TOTAL_DUR_RE.search(output)
    if not match:
        raise ValueError("missing total_dur(ms)")
    return float(match.group(1))


def gen_data_command(workload: str, workers: int, size_expr: str) -> str:
    if workload == "Word Count":
        return f"sudo -E ./scripts/gen_data.py {workers} '{size_expr}' 0 0"
    if workload == "Parallel Sort":
        return f"sudo -E ./scripts/gen_data.py 0 0 {workers} '{size_expr}'"
    raise ValueError(f"unsupported generated-data workload: {workload}")


def clean_workload_data(cwd: Path, unset_cc: bool, workload: str) -> None:
    if workload == "Word Count":
        pattern = "image_content/fake_data_*.txt"
    elif workload == "Parallel Sort":
        pattern = "image_content/sort_data_*.txt"
    else:
        return
    command = f"sudo -E bash -lc 'rm -f {pattern}'"
    run_command(command, cwd, unset_cc, progress_label=f"clean {workload} input data", interactive=True)


def clean_generated_data(cwd: Path, unset_cc: bool) -> None:
    mount_image(cwd, unset_cc)
    command = (
        "sudo -E bash -lc "
        "'rm -f image_content/*.imd "
        "image_content/fake_data_*.txt "
        "image_content/sort_data_*.txt'"
    )
    try:
        run_command(
            command,
            cwd,
            unset_cc,
            progress_label="clean generated data",
            interactive=True,
        )
        run_command("sync", cwd, unset_cc, progress_label="sync cleaned data")
    finally:
        unmount_image(cwd, unset_cc)


def verify_generated_data(cwd: Path, workload: str, workers: int, size_expr: str) -> None:
    if workload == "Word Count":
        prefix = "fake_data_"
    elif workload == "Parallel Sort":
        prefix = "sort_data_"
    else:
        return

    files = sorted((cwd / "image_content").glob(f"{prefix}*.txt"))
    if len(files) != workers:
        names = ", ".join(path.name for path in files[:10])
        raise RuntimeError(
            f"{workload} generated {len(files)} files, expected {workers}; files=[{names}]"
        )

    expected_total = eval_size_expr(size_expr)
    sizes = [path.stat().st_size for path in files]
    actual_total = sum(sizes)
    if any(size <= 0 for size in sizes):
        raise RuntimeError(f"{workload} generated empty input file")
    if actual_total < expected_total:
        raise RuntimeError(
            f"{workload} generated {actual_total} bytes, expected at least {expected_total}"
        )


def rust_config(workload: str, column: str) -> str:
    if workload == "Word Count":
        return WORD_COUNT_CONFIGS[column]
    if workload == "Parallel Sort":
        return PARALLEL_SORT_CONFIGS[column]
    if workload == "Long Chain":
        return LONG_CHAIN_CONFIGS[column]
    raise ValueError(f"unsupported workload: {workload}")


def generated_config(
    cwd: Path,
    temp_dir: Path,
    workload: str,
    backend: str,
    column: str,
    size_expr: str,
) -> str:
    with open(cwd / CONFIG_TEMPLATES[workload][backend], "r", encoding="utf-8") as fh:
        config = json.load(fh)

    size_bytes = str(eval_size_expr(size_expr))
    amount = int(column[1:])

    if workload == "Word Count":
        group = config["groups"][0]
        if backend == "musl-py":
            app_names = [f"wordcount_{idx}" for idx in range(amount)]
            config["apps"] = [
                [name, f"libmusl_cpython_{idx}.so"]
                for idx, name in enumerate(app_names)
            ]
            group["list"] = app_names
        else:
            group["list"] = [group["list"][0]] * amount
            if len(config["groups"]) > 1:
                reducer_group = config["groups"][1]
                reducer_group["list"] = [reducer_group["list"][0]] * amount
        for item in config["groups"]:
            item["args"]["mapper_num"] = str(amount)
            item["args"]["reducer_num"] = str(amount)
    elif workload == "Parallel Sort":
        group = config["groups"][0]
        if backend == "musl-py":
            app_names = [f"parallel_sort_{idx}" for idx in range(amount)]
            config["apps"] = [
                [name, f"libmusl_cpython_{idx}.so"]
                for idx, name in enumerate(app_names)
            ]
            group["list"] = app_names
        else:
            for item in config["groups"]:
                first = item["list"][0]
                if "checker" not in first:
                    item["list"] = [first] * amount
        for item in config["groups"]:
            item["args"]["sorter_num"] = str(amount)
            item["args"]["merger_num"] = str(amount)
    elif workload == "Long Chain":
        if backend.endswith("-py"):
            config["groups"][0]["args"]["func_num"] = str(amount)
            config["groups"][0]["args"]["data_size"] = size_bytes
        else:
            app_name = config["groups"][0]["list"][0]
            config["groups"] = [
                {
                    "list": [app_name],
                    "args": {
                        "func_num": str(stage),
                        "chain_len": str(amount),
                        "data_size": size_bytes,
                    },
                }
                for stage in range(amount)
            ]
    else:
        raise ValueError(f"unsupported workload: {workload}")

    path = temp_dir / (
        f"{workload.lower().replace(' ', '_')}-{backend}-{column}-{size_bytes}.json"
    )
    with open(path, "w", encoding="utf-8") as fh:
        json.dump(config, fh, indent=2)
        fh.write("\n")
    return str(path)


def collect_cell(
    cwd: Path,
    unset_cc: bool,
    workload: str,
    backend: str,
    row_label: str,
    column: str,
    config_path: str,
    repeat: int,
    timeout_seconds: int,
) -> str:
    progress_label = (
        f"{workload} runtime={backend}, row={row_label}, column={column}"
    )
    print(f"[start] {progress_label}", file=sys.stderr, flush=True)
    started_at = time.monotonic()

    try:
        # The LibOS opens fs_images/fatfs.img directly. Never let it race with
        # the kernel FAT driver operating on the mounted image.
        unmount_image(cwd, unset_cc)
        values = []
        for run_id in range(1, repeat + 1):
            run_label = f"{progress_label}, run={run_id}/{repeat}"
            output = run_command(
                f"target/release/asvisor --files {config_path} --metrics total-dur",
                cwd,
                unset_cc,
                progress_label=run_label,
                timeout_seconds=timeout_seconds,
            )
            values.append(parse_total_dur(output))

        value = sum(values) / len(values)
        elapsed = time.monotonic() - started_at
        print(
            f"[done] {progress_label}: {value:.3f} ms in {elapsed:.1f}s",
            file=sys.stderr,
            flush=True,
        )
        return f"{value:.3f}"
    except Exception as exc:
        error = str(exc).splitlines()[-1]
        elapsed = time.monotonic() - started_at
        print(
            f"[failed] {progress_label}: {error} after {elapsed:.1f}s",
            file=sys.stderr,
            flush=True,
        )
        return f"N/A ({error})"


def prepare_generated_data(
    cwd: Path,
    unset_cc: bool,
    workload: str,
    row_label: str,
    size_expr: str,
    column: str,
) -> None:
    workers = int(column[1:])
    label = f"{workload} row={row_label}, column={column}"
    mount_image(cwd, unset_cc)
    try:
        clean_workload_data(cwd, unset_cc, workload)
        run_command(
            gen_data_command(workload, workers, size_expr),
            cwd,
            unset_cc,
            progress_label=f"generate data: {label}",
            interactive=True,
        )
        run_command("sync", cwd, unset_cc, progress_label=f"sync data: {label}")
        verify_generated_data(cwd, workload, workers, size_expr)
    finally:
        unmount_image(cwd, unset_cc)


def build_rust_long_chain(
    cwd: Path,
    unset_cc: bool,
    row_label: str,
    size_expr: str,
) -> None:
    config_file = cwd / "user" / "function_chain_data_size.config"
    original = config_file.read_text(encoding="utf-8")
    config_file.write_text(f"{size_expr}\n", encoding="utf-8")
    try:
        run_command(
            "just rust_func array_sum >/dev/null",
            cwd,
            unset_cc,
            progress_label=f"build Rust Long Chain row={row_label}",
        )
    finally:
        config_file.write_text(original, encoding="utf-8")


def collect_results(
    cwd: Path,
    unset_cc: bool,
    repeat: int,
    clean: bool,
    checkpoint_path: Path,
    timeout_seconds: int,
) -> dict:
    results = empty_results()
    matrix = [
        ("Word Count", WORD_COUNT_ROWS, PARALLEL_COLUMNS),
        ("Parallel Sort", PARALLEL_SORT_ROWS, PARALLEL_COLUMNS),
        ("Long Chain", LONG_CHAIN_ROWS, LONG_CHAIN_COLUMNS),
    ]

    with tempfile.TemporaryDirectory(prefix="alloy-latency-") as temp:
        temp_dir = Path(temp)
        for workload, size_rows, columns in matrix:
            for row_label, size_expr in size_rows:
                rust_long_chain_ready = True
                if workload == "Long Chain":
                    try:
                        build_rust_long_chain(cwd, unset_cc, row_label, size_expr)
                    except Exception as exc:
                        rust_long_chain_ready = False
                        error = str(exc).splitlines()[-1]

                for column in columns:
                    data_ready = True
                    if workload != "Long Chain":
                        try:
                            prepare_generated_data(
                                cwd,
                                unset_cc,
                                workload,
                                row_label,
                                size_expr,
                                column,
                            )
                        except Exception as exc:
                            data_ready = False
                            error = str(exc).splitlines()[-1]
                            print(
                                f"[failed] prepare {workload} row={row_label}, "
                                f"column={column}: {error}",
                                file=sys.stderr,
                                flush=True,
                            )

                    for backend in BACKENDS:
                        if not data_ready:
                            value = f"N/A (data preparation failed: {error})"
                        elif (
                            workload == "Long Chain"
                            and backend == "rust"
                            and not rust_long_chain_ready
                        ):
                            value = f"N/A (Rust build failed: {error})"
                        else:
                            try:
                                config_path = (
                                    rust_config(workload, column)
                                    if backend == "rust"
                                    else generated_config(
                                        cwd,
                                        temp_dir,
                                        workload,
                                        backend,
                                        column,
                                        size_expr,
                                    )
                                )
                                value = collect_cell(
                                    cwd,
                                    unset_cc,
                                    workload,
                                    backend,
                                    row_label,
                                    column,
                                config_path,
                                repeat,
                                timeout_seconds,
                                )
                            except Exception as exc:
                                value = f"N/A ({str(exc).splitlines()[-1]})"
                        results[workload][backend][row_label][column] = value
                        write_results(checkpoint_path, results)

                    if clean and workload != "Long Chain":
                        try:
                            clean_generated_data(cwd, unset_cc)
                        except Exception as exc:
                            print(
                                f"[warn] cleanup failed after {workload} "
                                f"row={row_label}, column={column}: "
                                f"{str(exc).splitlines()[-1]}",
                                file=sys.stderr,
                                flush=True,
                            )

    return results


def load_results(path: Path) -> dict:
    if not path.exists():
        raise FileNotFoundError(f"missing results file: {path}; run with collect=1 first")
    with open(path, "r", encoding="utf-8") as fh:
        loaded = json.load(fh)
    metadata = loaded.get("_metadata") if isinstance(loaded, dict) else None
    if not isinstance(metadata, dict) or metadata.get("schema_version") != RESULTS_SCHEMA_VERSION:
        raise RuntimeError(
            f"incompatible or legacy results file: {path}; rerun with collect=1"
        )
    loaded = loaded.get("results", {})
    results = empty_results()
    for section, backends in loaded.items():
        if section not in results:
            continue
        for backend, rows in backends.items():
            if backend not in results[section]:
                continue
            for row, columns in rows.items():
                if row not in results[section][backend]:
                    continue
                for column, value in columns.items():
                    if column in results[section][backend][row]:
                        results[section][backend][row][column] = value
    return results


def write_results(path: Path, results: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "_metadata": {
            "schema_version": RESULTS_SCHEMA_VERSION,
            "generated_at": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
            "unit": "ms",
        },
        "results": results,
    }
    temporary_path = path.with_name(f".{path.name}.tmp")
    with open(temporary_path, "w", encoding="utf-8") as fh:
        json.dump(payload, fh, indent=2)
        fh.write("\n")
        fh.flush()
        os.fsync(fh.fileno())
    os.replace(temporary_path, path)


def render_metric_cell(value: str) -> str:
    try:
        return f"{float(value):8.3f}"
    except (TypeError, ValueError):
        return value


def render_runtime_table(
    title: str,
    dimension_name: str,
    data_sizes: list[str],
    dimensions: list[str],
    values: dict,
) -> str:
    lines = [f"## {title}", "", "Unit: ms", ""]
    header = ["runtime", dimension_name, *data_sizes]
    lines.append("| " + " | ".join(header) + " |")
    lines.append(
        "| "
        + " | ".join(["---", "---", *(["---:"] * len(data_sizes))])
        + " |"
    )
    for backend in BACKENDS:
        for dimension in dimensions:
            row_values = [backend, dimension]
            row_values.extend(
                render_metric_cell(values[backend][data_size][dimension])
                for data_size in data_sizes
            )
            lines.append("| " + " | ".join(row_values) + " |")
    lines.append("")
    return "\n".join(lines)


def render_report(results: dict) -> str:
    return "\n\n".join(
        [
            render_runtime_table(
                "Word Count",
                "parallelism",
                [row for row, _ in WORD_COUNT_ROWS],
                PARALLEL_COLUMNS,
                results["Word Count"],
            ).rstrip(),
            render_runtime_table(
                "Parallel Sort",
                "parallelism",
                [row for row, _ in PARALLEL_SORT_ROWS],
                PARALLEL_COLUMNS,
                results["Parallel Sort"],
            ).rstrip(),
            render_runtime_table(
                "Long Chain",
                "chain_len",
                [row for row, _ in LONG_CHAIN_ROWS],
                LONG_CHAIN_COLUMNS,
                results["Long Chain"],
            ).rstrip(),
        ]
    ) + "\n"


def resolve_path(cwd: Path, value: str) -> Path:
    path = Path(value)
    return path if path.is_absolute() else cwd / path


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Generate full-size end-to-end latency report for all runtimes."
    )
    parser.add_argument("--output", default="reports/rust_latency_report.md")
    parser.add_argument("--results", default="reports/rust_latency_report_results.json")
    parser.add_argument("--collect", action="store_true")
    parser.add_argument("--clean", action="store_true")
    parser.add_argument("--repeat", type=int, default=1)
    parser.add_argument(
        "--timeout",
        type=int,
        default=DEFAULT_TEST_TIMEOUT_SECONDS,
        help="maximum seconds for each workflow run (default: 600)",
    )
    parser.add_argument("--skip-prepare", action="store_true")
    parser.add_argument("--keep-cc", action="store_true")
    args = parser.parse_args()

    if args.repeat < 1:
        raise ValueError("--repeat must be >= 1")
    if args.timeout < 1:
        raise ValueError("--timeout must be >= 1")

    cwd = repo_root()
    output_path = resolve_path(cwd, args.output)
    results_path = resolve_path(cwd, args.results)
    unset_cc = not args.keep_cc

    if args.collect:
        if not args.skip_prepare:
            run_prepare(cwd, unset_cc)
        results = collect_results(
            cwd,
            unset_cc,
            args.repeat,
            args.clean,
            results_path,
            args.timeout,
        )
        write_results(results_path, results)
    else:
        results = load_results(results_path)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(render_report(results), encoding="utf-8")
    print(output_path)
    return 0


if __name__ == "__main__":
    sys.exit(main())
