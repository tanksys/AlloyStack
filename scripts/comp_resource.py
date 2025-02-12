#!/usr/bin/python3

import os
from datetime import datetime

# 计算箱形图的五个点：


def five_number_summary(data):
    n = len(data)
    q1 = data[n // 4]
    q2 = data[n // 2]
    q3 = data[3 * n // 4]
    return data[0], q1, q2, q3, data[-1]


def comp(name):
    print(name)
    with open(name) as f:
        cpus = []
        user_cpus = []
        sys_cpus = []
        mems = []
        lines = f.readlines()
        tss = []

        for line in lines:
            fields = line.strip().split(' ')
            timestamp = datetime.strptime(
                fields[0] + ' ' + fields[1].replace(',', ''), "%Y-%m-%d %H:%M:%S.%f")

            ts = timestamp.timestamp()
            tss.append(ts)
            if len(tss) == 1:
                continue

                # print(fields)
            try:
                user_cpu = float(fields[2].replace(',', '')) * (tss[-1] - tss[-2])
                sys_cpu = float(fields[3].replace(',', '')) * (tss[-1] - tss[-2])
            except ValueError:
                continue

            cpu = user_cpu + sys_cpu

            mem = float(fields[-1])
            cpus.append(cpu)
            user_cpus.append(user_cpu)
            sys_cpus.append(sys_cpu)
            mems.append(mem)

        cpus.sort()
        mems.sort()
        # print("cpu极值:", cpus[-1]-cpus[0])
        # print("mem极值:", mems[-1]-mems[0])

        cpu_min, cpu_q1, cpu_median, cpu_q3, cpu_max = five_number_summary(
            cpus)
        # mem_min, mem_q1, mem_median, mem_q3, mem_max = five_number_summary(
        #     mems)

        # print(f"执行耗时: {tss[-1]-tss[0]}ms")
        print("cpu consume:", sum(cpus) - cpu_min * len(cpus))
        print("user cpu consume", sum(user_cpus) -
              min(user_cpus) * len(user_cpus))
        print("system cpu consume", sum(sys_cpus) -
              min(sys_cpus) * len(sys_cpus))
        # print("cpu五数概括:", cpu_min, cpu_q1, cpu_median, cpu_q3, cpu_max)

        # print("mem均值:", sum(mems)/len(mems) - min(mems))
        # print("mem五数概括:", mem_min, mem_q1, mem_median, mem_q3, mem_max)
        print()


# comp("as_map_reduce_resouce_c5_10_20.txt")
# comp("as_map_reduce_resouce_c5_10_40.txt")
# comp("as_map_reduce_resouce_c5_10_60.txt")
# comp("as_map_reduce_resouce_c5_10_80.txt")

# comp("faastlane_map_reduce_resouce_c5_10_20.txt")
# comp("faastlane_map_reduce_resouce_c5_10_40.txt")
# comp("faastlane_map_reduce_resouce_c5_10_60.txt")
# comp("faastlane_map_reduce_resouce_c5_10_80.txt")


comp("as_parallel_sort_resouce_c5_25_20.txt")
comp("as_parallel_sort_resouce_c5_25_40.txt")
comp("as_parallel_sort_resouce_c5_25_60.txt")
comp("as_parallel_sort_resouce_c5_25_80.txt")

if os.path.exists("faastlane_parallel_sort_resouce_c5_25_20.txt"):
    comp("faastlane_parallel_sort_resouce_c5_25_20.txt")
    comp("faastlane_parallel_sort_resouce_c5_25_40.txt")
    comp("faastlane_parallel_sort_resouce_c5_25_60.txt")
    comp("faastlane_parallel_sort_resouce_c5_25_80.txt")
