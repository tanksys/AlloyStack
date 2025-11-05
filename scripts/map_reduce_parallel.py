#!/usr/bin/python3

import subprocess
import re

# 并行执行 ./scripts/rust_end_to_end.sh map_reduce c3

concur_num = 10


def test_map_reduce_parallel():
    import concurrent.futures

    def run_script():
        command = f"./target/release/msvisor --files isol_config/map_reduce_large_c3.json --metrics total-dur 2>&1"
        result = subprocess.run(command, shell=True,
                                capture_output=True, text=True)
        return result.stdout, result.stderr

    latencies = []

    with concurrent.futures.ThreadPoolExecutor() as executor:
        futures = [executor.submit(run_script)
                   for _ in range(concur_num)]

        for future in concurrent.futures.as_completed(futures):
            stdout, stderr = future.result()
            # 从 stdout 中提取 'Total Dur (ms): 37.825' 中的数字
            for line in stdout.splitlines():
                match = re.search(r'"total_dur\(ms\)": ([\d.]+)', line)
                if match:
                    duration = float(match.group(1))
                    # print("Total Duration (ms):", duration)
                    latencies.append(duration)
            # print("Output:", stdout)
            if stderr:
                print("Error:", stderr)

    latencies = sorted(latencies)
    print("Latencies:", latencies)
    print("p99:", latencies[-2])


if __name__ == '__main__':
    import sys
    concur_num = int(sys.argv[1])
    test_map_reduce_parallel()
