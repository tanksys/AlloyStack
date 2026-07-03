#!/usr/bin/env python3

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path


BACKENDS = ["rust", "wasm-c", "wasm-py", "musl-c", "musl-py"]
DATA_SIZES = ["4*1024", "64*1024", "1024*1024", "16*1024*1024", "256*1024*1024"]
WORKLOADS = ["longchain", "ps", "wc"]

TOTAL_DUR_RE = re.compile(r'"total_dur\(ms\)":\s*([0-9]+(?:\.[0-9]+)?)')
TRANS_DATA_RE = re.compile(r"trans data time:\s*([0-9]+(?:\.[0-9]+)?)(µs|ms)")
TRANSFER_COST_NS_RE = re.compile(r"transfer cost:\s*([0-9]+(?:\.[0-9]+)?)\s*ns")
RUST_COST_NS_RE = re.compile(r"data size:\s*[0-9]+\s*bytes,\s*cost\s*([0-9]+(?:\.[0-9]+)?)\s*ns")
CURRENTLY_CLEAN_AFTER_CELL = False


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def default_config() -> dict:
    profile = "release"
    asvisor = f"target/{profile}/asvisor"

    data_transfer_cells = {
        size: {backend: None for backend in BACKENDS}
        for size in DATA_SIZES
    }

    for size in DATA_SIZES:
        data_transfer_cells[size]["rust"] = {
            "command": f"printf '%s\\n' '{size}' > user/data_size.config && just pass_args >/dev/null && {asvisor} --files isol_config/pass_complex_args.json --metrics all",
            "parser": "rust_cost_ns_ms",
            "cleanup": [
                "cargo clean --manifest-path user/func_a/Cargo.toml",
                "cargo clean --manifest-path user/func_b/Cargo.toml",
            ],
        }
        data_transfer_cells[size]["wasm-c"] = {
            "config": "isol_config/wasmtime_trans_data.json",
            "config_args": {"data_size": "{data_size_bytes}"},
            "command": f"just wasm_func wasmtime_trans_data >/dev/null && {asvisor} --files {{config_path}} --metrics all",
            "parser": "transfer_cost_ns_ms",
            "cleanup": [
                "cargo clean --manifest-path user/wasmtime_trans_data/Cargo.toml",
            ],
        }
        data_transfer_cells[size]["wasm-py"] = {
            "config": "isol_config/wasmtime_cpython_transfer.json",
            "config_args": {"data_size": "{data_size_bytes}"},
            "command": f"just wasm_func wasmtime_cpython >/dev/null && {asvisor} --files {{config_path}} --metrics all",
            "parser": "transfer_cost_ns_ms",
            "cleanup": [
                "cargo clean --manifest-path user/wasmtime_cpython/Cargo.toml",
            ],
        }
        data_transfer_cells[size]["musl-c"] = {
            "config": "isol_config/musl_trans_data.json",
            "config_args": {"data_size": "{data_size_bytes}"},
            "command": f"just musl_func musl_trans_data >/dev/null && {asvisor} --files {{config_path}} --metrics all",
            "parser": "transfer_cost_ns_ms",
            "cleanup": [
                "cargo clean --manifest-path user/musl_trans_data/Cargo.toml",
            ],
        }
        data_transfer_cells[size]["musl-py"] = {
            "config": "isol_config/musl_cpython_transfer.json",
            "config_args": {"data_size": "{data_size_bytes}"},
            "command": f"env -u CC cargo build --release --manifest-path user/musl_cpython/Cargo.toml >/dev/null && {asvisor} --files {{config_path}} --metrics all",
            "parser": "transfer_cost_ns_ms",
            "cleanup": [
                "cargo clean --manifest-path user/musl_cpython/Cargo.toml",
            ],
        }

    end_to_end_cells = {
        "longchain": {
            "rust": {
                "command": f"just long_chain >/dev/null && {asvisor} --files isol_config/long_chain_n15.json --metrics total-dur",
                "parser": "total_dur_ms",
                "cleanup": [
                    "cargo clean --manifest-path user/array_sum/Cargo.toml",
                ],
            },
            "wasm-c": {
                "command": f"just c_long_chain >/dev/null && {asvisor} --files isol_config/wasmtime_longchain.json --metrics total-dur",
                "parser": "total_dur_ms",
                "cleanup": [
                    "cargo clean --manifest-path user/wasmtime_longchain/Cargo.toml",
                ],
            },
            "wasm-py": {
                "command": f"just python_long_chain >/dev/null && {asvisor} --files isol_config/wasmtime_cpython_functionchain_n5.json --metrics total-dur",
                "parser": "total_dur_ms",
                "cleanup": [
                    "cargo clean --manifest-path user/wasmtime_cpython_func/Cargo.toml",
                ],
            },
            "musl-c": {
                "command": f"just musl_long_chain >/dev/null && {asvisor} --files isol_config/musl_longchain.json --metrics total-dur",
                "parser": "total_dur_ms",
                "cleanup": [
                    "cargo clean --manifest-path user/musl_longchain/Cargo.toml",
                ],
            },
            "musl-py": {
                "command": f"just musl_cpython >/dev/null && {asvisor} --files isol_config/musl_cpython_functionchain_n5.json --metrics total-dur",
                "parser": "total_dur_ms",
                "cleanup": [
                    "cargo clean --manifest-path user/musl_cpython/Cargo.toml",
                    "rm -f target/release/libmusl_cpython.so target/release/libmusl_cpython_0.so target/release/libmusl_cpython_1.so target/release/libmusl_cpython_2.so target/release/libmusl_cpython_3.so target/release/libmusl_cpython_4.so",
                ],
            },
        },
        "ps": {
            "rust": {
                "command": f"just parallel_sort >/dev/null && {asvisor} --files isol_config/parallel_sort_c3.json --metrics total-dur",
                "parser": "total_dur_ms",
                "cleanup": [
                    "cargo clean --manifest-path user/file_reader/Cargo.toml",
                    "cargo clean --manifest-path user/sorter/Cargo.toml",
                    "cargo clean --manifest-path user/splitter/Cargo.toml",
                    "cargo clean --manifest-path user/merger/Cargo.toml",
                ],
            },
            "wasm-c": {
                "command": f"just c_parallel_sort >/dev/null && {asvisor} --files isol_config/wasmtime_parallel_sort_c3.json --metrics total-dur",
                "parser": "total_dur_ms",
                "cleanup": [
                    "cargo clean --manifest-path user/wasmtime_sorter/Cargo.toml",
                    "cargo clean --manifest-path user/wasmtime_spliter/Cargo.toml",
                    "cargo clean --manifest-path user/wasmtime_merger/Cargo.toml",
                    "cargo clean --manifest-path user/wasmtime_checker/Cargo.toml",
                ],
            },
            "wasm-py": {
                "command": f"just python_parallel_sort >/dev/null && {asvisor} --files isol_config/wasmtime_cpython_parallel_sort_c1.json --metrics total-dur",
                "parser": "total_dur_ms",
                "cleanup": [
                    "cargo clean --manifest-path user/wasmtime_cpython_parallel_sort/Cargo.toml",
                ],
            },
            "musl-c": {
                "command": f"just musl_parallel_sort >/dev/null && {asvisor} --files isol_config/musl_parallel_sort_c3.json --metrics total-dur",
                "parser": "total_dur_ms",
                "cleanup": [
                    "cargo clean --manifest-path user/musl_sorter/Cargo.toml",
                    "cargo clean --manifest-path user/musl_splitter/Cargo.toml",
                    "cargo clean --manifest-path user/musl_merger/Cargo.toml",
                    "cargo clean --manifest-path user/musl_checker/Cargo.toml",
                ],
            },
            "musl-py": {
                "command": f"just musl_cpython >/dev/null && {asvisor} --files isol_config/musl_cpython_parallel_sort_c1.json --metrics total-dur",
                "parser": "total_dur_ms",
                "cleanup": [
                    "cargo clean --manifest-path user/musl_cpython/Cargo.toml",
                    "rm -f target/release/libmusl_cpython.so target/release/libmusl_cpython_0.so target/release/libmusl_cpython_1.so target/release/libmusl_cpython_2.so target/release/libmusl_cpython_3.so target/release/libmusl_cpython_4.so",
                ],
            },
        },
        "wc": {
            "rust": {
                "command": f"just map_reduce >/dev/null && {asvisor} --files isol_config/map_reduce_large_c3.json --metrics total-dur",
                "parser": "total_dur_ms",
                "cleanup": [
                    "cargo clean --manifest-path user/mapper/Cargo.toml",
                    "cargo clean --manifest-path user/reducer/Cargo.toml",
                ],
            },
            "wasm-c": {
                "command": f"just c_wordcount >/dev/null && {asvisor} --files isol_config/wasmtime_wordcount_c3.json --metrics total-dur",
                "parser": "total_dur_ms",
                "cleanup": [
                    "cargo clean --manifest-path user/wasmtime_mapper/Cargo.toml",
                    "cargo clean --manifest-path user/wasmtime_reducer/Cargo.toml",
                ],
            },
            "wasm-py": {
                "command": f"just python_wordcount >/dev/null && {asvisor} --files isol_config/wasmtime_cpython_wordcount_c1.json --metrics total-dur",
                "parser": "total_dur_ms",
                "cleanup": [
                    "cargo clean --manifest-path user/wasmtime_cpython_wordcount/Cargo.toml",
                ],
            },
            "musl-c": {
                "command": f"just musl_wordcount >/dev/null && {asvisor} --files isol_config/musl_wordcount_c3.json --metrics total-dur",
                "parser": "total_dur_ms",
                "cleanup": [
                    "cargo clean --manifest-path user/musl_mapper/Cargo.toml",
                    "cargo clean --manifest-path user/musl_reducer/Cargo.toml",
                ],
            },
            "musl-py": {
                "command": f"just musl_cpython >/dev/null && {asvisor} --files isol_config/musl_cpython_wordcount_c1.json --metrics total-dur",
                "parser": "total_dur_ms",
                "cleanup": [
                    "cargo clean --manifest-path user/musl_cpython/Cargo.toml",
                    "rm -f target/release/libmusl_cpython.so target/release/libmusl_cpython_0.so target/release/libmusl_cpython_1.so target/release/libmusl_cpython_2.so target/release/libmusl_cpython_3.so target/release/libmusl_cpython_4.so",
                ],
            },
        },
    }

    return {
        "profile": profile,
        "unset_cc": True,
        "prepare": [
            "./scripts/sync_python_workloads.sh",
        ],
        "data_transfer_latency": {
            "columns": BACKENDS,
            "rows": DATA_SIZES,
            "cells": data_transfer_cells,
        },
        "end_to_end_latency": {
            "columns": BACKENDS,
            "rows": WORKLOADS,
            "cells": end_to_end_cells,
        },
    }


def empty_results(config: dict) -> dict:
    data_transfer = {
        row: {column: "N/A" for column in config["data_transfer_latency"]["columns"]}
        for row in config["data_transfer_latency"]["rows"]
    }
    end_to_end = {
        row: {column: "N/A" for column in config["end_to_end_latency"]["columns"]}
        for row in config["end_to_end_latency"]["rows"]
    }
    return {
        "data_transfer_latency": data_transfer,
        "end_to_end_latency": end_to_end,
    }


def merge_dict(base: dict, override: dict) -> dict:
    result = dict(base)
    for key, value in override.items():
        if isinstance(value, dict) and isinstance(result.get(key), dict):
            result[key] = merge_dict(result[key], value)
        else:
            result[key] = value
    return result


def load_config(path: str | None) -> dict:
    config = default_config()
    if not path:
        return config
    with open(path, "r", encoding="utf-8") as fh:
        override = json.load(fh)
    return merge_dict(config, override)


def run_command(command: str, cwd: Path, unset_cc: bool, interactive: bool = False) -> str:
    env = os.environ.copy()
    if unset_cc:
        env.pop("CC", None)
    if interactive:
        completed = subprocess.run(
            ["bash", "-lc", command],
            cwd=cwd,
            env=env,
            text=True,
        )
        output = ""
    else:
        completed = subprocess.run(
            ["bash", "-lc", command],
            cwd=cwd,
            env=env,
            capture_output=True,
            text=True,
        )
        output = completed.stdout + completed.stderr
    if completed.returncode != 0:
        raise RuntimeError(output.strip() or f"command failed: {command}")
    return output


def run_prepare_command(command: str, cwd: Path, unset_cc: bool) -> None:
    try:
        run_command(command, cwd, unset_cc)
        return
    except RuntimeError as exc:
        message = str(exc)
        if "rerun this script with sudo" not in message:
            raise

    sudo_command = f"sudo -E {command}"
    run_command(sudo_command, cwd, unset_cc, interactive=True)


def run_cleanup_commands(
    commands: list[str],
    cwd: Path,
    unset_cc: bool,
    context: dict[str, str],
) -> None:
    for command in commands:
        run_command(command.format(**context), cwd, unset_cc)


def build_temp_config(
    cwd: Path,
    config_path: str,
    config_args: dict[str, str],
    context: dict[str, str],
) -> str:
    source_path = cwd / config_path
    with open(source_path, "r", encoding="utf-8") as fh:
        config = json.load(fh)
    for group in config.get("groups", []):
        args = group.get("args", {})
        for key, value in config_args.items():
            args[key] = value.format(**context)
    with tempfile.NamedTemporaryFile(
        "w",
        suffix=".json",
        prefix="latency-report-",
        delete=False,
        dir="/tmp",
        encoding="utf-8",
    ) as fh:
        json.dump(config, fh)
        fh.flush()
        return fh.name


def eval_data_size(expr: str) -> int:
    parts = [part.strip() for part in expr.split("*")]
    value = 1
    for part in parts:
        if not part:
            raise ValueError(f"invalid data size expression: {expr}")
        value *= int(part)
    return value


def parse_metric(output: str, parser_name: str) -> float:
    if parser_name == "total_dur_ms":
        match = TOTAL_DUR_RE.search(output)
        if not match:
            raise ValueError("missing total_dur(ms)")
        return float(match.group(1))

    if parser_name == "trans_data_ms":
        match = TRANS_DATA_RE.search(output)
        if not match:
            raise ValueError("missing trans data time")
        value = float(match.group(1))
        unit = match.group(2)
        return value / 1000.0 if unit == "µs" else value

    if parser_name == "transfer_cost_ns_ms":
        match = TRANSFER_COST_NS_RE.search(output)
        if not match:
            raise ValueError("missing transfer cost")
        return float(match.group(1)) / 1_000_000.0

    if parser_name == "rust_cost_ns_ms":
        match = RUST_COST_NS_RE.search(output)
        if not match:
            raise ValueError("missing rust transfer cost")
        return float(match.group(1)) / 1_000_000.0

    raise ValueError(f"unsupported parser: {parser_name}")


def collect_table(section: dict, cwd: Path, unset_cc: bool) -> dict:
    rows = section["rows"]
    columns = section["columns"]
    cells = section["cells"]
    results: dict[str, dict[str, str]] = {}

    for row in rows:
        results[row] = {}
        for column in columns:
            cell = cells.get(row, {}).get(column)
            if cell is None:
                results[row][column] = "N/A"
                continue
            if "value_ms" in cell:
                results[row][column] = f"{float(cell['value_ms']):.3f}"
                continue
            temp_config_path = None
            context = {
                "data_size": row,
                "data_size_bytes": row,
                "workload": row,
                "backend": column,
            }
            try:
                data_size_bytes = str(eval_data_size(row)) if "*" in row else row
                context["data_size_bytes"] = data_size_bytes
                command = cell["command"]
                if "config" in cell:
                    temp_config_path = build_temp_config(
                        cwd,
                        cell["config"],
                        cell.get("config_args", {}),
                        context,
                    )
                    context["config_path"] = temp_config_path
                output = run_command(
                    command.format(**context),
                    cwd,
                    unset_cc,
                )
                value = parse_metric(output, cell["parser"])
                results[row][column] = f"{value:.3f}"
            except Exception as exc:
                results[row][column] = f"N/A ({str(exc).splitlines()[-1]})"
            finally:
                if CURRENTLY_CLEAN_AFTER_CELL and cell is not None:
                    cleanup_commands = cell.get("cleanup", [])
                    if cleanup_commands:
                        try:
                            run_cleanup_commands(cleanup_commands, cwd, unset_cc, context)
                        except Exception as exc:
                            results[row][column] = (
                                f"{results[row][column]} | cleanup failed: "
                                f"{str(exc).splitlines()[-1]}"
                            )
                if temp_config_path:
                    try:
                        os.unlink(temp_config_path)
                    except OSError:
                        pass
    return results


def render_table(title: str, first_col: str, rows: list[str], columns: list[str], values: dict) -> str:
    lines = [f"## {title}", ""]
    header = [first_col, *columns]
    align = ["---"] * len(header)
    lines.append("| " + " | ".join(header) + " |")
    lines.append("| " + " | ".join(align) + " |")
    for row in rows:
        row_values = [row] + [values[row][column] for column in columns]
        lines.append("| " + " | ".join(row_values) + " |")
    lines.append("")
    return "\n".join(lines)


def write_output(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def load_results(path: Path, config: dict) -> dict:
    if not path.exists():
        raise FileNotFoundError(
            f"missing results file: {path}; run with --collect first"
        )
    with open(path, "r", encoding="utf-8") as fh:
        loaded = json.load(fh)
    results = empty_results(config)
    for section in ("data_transfer_latency", "end_to_end_latency"):
        if section not in loaded:
            continue
        for row, columns in loaded[section].items():
            if row not in results[section]:
                continue
            for column, value in columns.items():
                if column in results[section][row]:
                    results[section][row][column] = value
    return results


def write_results(path: Path, results: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as fh:
        json.dump(results, fh, indent=2)
        fh.write("\n")


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate latency report in Markdown.")
    parser.add_argument(
        "--output",
        default="reports/latency_report.md",
        help="output markdown path",
    )
    parser.add_argument(
        "--results",
        default="reports/latency_report_results.json",
        help="raw results json path",
    )
    parser.add_argument(
        "--config",
        help="optional JSON config overriding commands or fixed values",
    )
    parser.add_argument(
        "--collect",
        action="store_true",
        help="run experiments and refresh the raw results json",
    )
    parser.add_argument(
        "--clean",
        action="store_true",
        help="run configured cleanup commands after each collected cell",
    )
    parser.add_argument(
        "--dump-config",
        action="store_true",
        help="print the default config JSON and exit",
    )
    args = parser.parse_args()

    if args.dump_config:
        json.dump(default_config(), sys.stdout, indent=2)
        sys.stdout.write("\n")
        return 0

    cwd = repo_root()
    config = load_config(args.config)
    global CURRENTLY_CLEAN_AFTER_CELL
    CURRENTLY_CLEAN_AFTER_CELL = args.clean
    output_path = Path(args.output)
    if not output_path.is_absolute():
        output_path = cwd / output_path
    results_path = Path(args.results)
    if not results_path.is_absolute():
        results_path = cwd / results_path

    if args.collect:
        unset_cc = bool(config.get("unset_cc", True))
        for command in config.get("prepare", []):
            run_prepare_command(command, cwd, unset_cc)
        results = {
            "data_transfer_latency": collect_table(
                config["data_transfer_latency"], cwd, unset_cc
            ),
            "end_to_end_latency": collect_table(
                config["end_to_end_latency"], cwd, unset_cc
            ),
        }
        write_results(results_path, results)
    else:
        results = load_results(results_path, config)

    content = "\n\n".join(
        [
            render_table(
                "data_transfer_latency",
                "data_size",
                config["data_transfer_latency"]["rows"],
                config["data_transfer_latency"]["columns"],
                results["data_transfer_latency"],
            ).rstrip(),
            render_table(
                "end_to_end_latency",
                "workload",
                config["end_to_end_latency"]["rows"],
                config["end_to_end_latency"]["columns"],
                results["end_to_end_latency"],
            ).rstrip(),
        ]
    ) + "\n"
    write_output(output_path, content)
    print(output_path)
    return 0


if __name__ == "__main__":
    sys.exit(main())
