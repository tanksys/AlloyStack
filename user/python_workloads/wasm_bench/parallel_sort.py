import pyas
import sys
import time
import heapq

buffer_size = 60000000
flag_size = 2

def get_now():
    return time.time()

def take_data_buffer(slot_name, size):
    take_buffer = getattr(pyas, "take_buffer", None)
    if take_buffer is not None:
        return take_buffer(slot_name)
    buffer = bytearray(size)
    pyas.access_buffer(slot_name, buffer)
    return buffer

setattr(__import__("__main__"), "trans_start_times", 0)
def get_trans_start_time():
    setattr(__import__("__main__"), "trans_start_times", getattr(__import__("__main__"), "trans_start_times") + 1)
    return getattr(__import__("__main__"), "trans_start_times")

setattr(__import__("__main__"), "compute_start_times", 0)
def get_compute_start_time():
    setattr(__import__("__main__"), "compute_start_times", getattr(__import__("__main__"), "compute_start_times") + 1)
    return getattr(__import__("__main__"), "compute_start_times")

setattr(__import__("__main__"), "trans_end_times", 0)
def get_trans_end_time():
    setattr(__import__("__main__"), "trans_end_times", getattr(__import__("__main__"), "trans_end_times") + 1)
    return getattr(__import__("__main__"), "trans_end_times")

setattr(__import__("__main__"), "compute_end_times", 0)
def get_compute_end_time():
    setattr(__import__("__main__"), "compute_end_times", getattr(__import__("__main__"), "compute_end_times") + 1)
    return getattr(__import__("__main__"), "compute_end_times")

setattr(__import__("__main__"), "trans_start_times_checker", 0)
def get_trans_start_time_checker():
    setattr(__import__("__main__"), "trans_start_times_checker", getattr(__import__("__main__"), "trans_start_times_checker") + 1)
    return getattr(__import__("__main__"), "trans_start_times_checker")

setattr(__import__("__main__"), "compute_start_times_checker", 0)
def get_compute_start_time_checker():
    setattr(__import__("__main__"), "compute_start_times_checker", getattr(__import__("__main__"), "compute_start_times_checker") + 1)
    return getattr(__import__("__main__"), "compute_start_times_checker")

setattr(__import__("__main__"), "trans_end_times_checker", 0)
def get_trans_end_time_checker():
    setattr(__import__("__main__"), "trans_end_times_checker", getattr(__import__("__main__"), "trans_end_times_checker") + 1)
    return getattr(__import__("__main__"), "trans_end_times_checker")

setattr(__import__("__main__"), "compute_end_times_checker", 0)
def get_compute_end_time_checker():
    setattr(__import__("__main__"), "compute_end_times_checker", getattr(__import__("__main__"), "compute_end_times_checker") + 1)
    return getattr(__import__("__main__"), "compute_end_times_checker")

def sorter(my_id, sorter_num, merger_num):
    # print("python: sorter {} start".format(my_id), flush=True)

    # flag_register
    for i in range(sorter_num):
        slot_name = "sorter_flag_{}_{}".format(my_id, i)
        setattr(__import__("__main__"), slot_name, pyas.buffer_register(slot_name, flag_size))
        encoded_data = "0".encode()
        getattr(__import__("__main__"), slot_name)[:1] = encoded_data

    print("compute{}start{}: {}".format(my_id, get_compute_start_time(), get_now()))
    nums = []
    with open("sort_data_{}.txt".format(my_id), "r", errors='ignore') as f:
        print("read{}start: {}".format(my_id, get_now()))
        data = f.readlines()
        print("read{}end: {}".format(my_id, get_now()))
        for line in data:
            nums.extend(map(int, line.split(',')))
    nums.sort()

    if my_id == 0 and merger_num > 1:
        pivot = [nums[(i + 1) * len(nums) // merger_num] for i in range(merger_num - 1)]
        for i in range(sorter_num):
            slot_name = "pivot_{}".format(i)
            setattr(__import__("__main__"), slot_name, pyas.buffer_register(slot_name, buffer_size))
            encoded_data = " ".join(map(str, pivot)).encode()
            # print("python: sorter {} pass {} size: {}".format(my_id, slot_name, len(encoded_data)), flush=True) # important
            getattr(__import__("__main__"), slot_name)[:len(encoded_data)] = encoded_data

    slot_name = "sorter_{}".format(my_id)
    setattr(__import__("__main__"), slot_name, pyas.buffer_register(slot_name, buffer_size))
    encoded_data = " ".join(map(str, nums)).encode()
    # print("python: sorter {} pass {} size: {}".format(my_id, slot_name, len(encoded_data)), flush=True) # important
    getattr(__import__("__main__"), slot_name)[:len(encoded_data)] = encoded_data
    print("compute{}end{}: {}".format(my_id, get_compute_end_time(), get_now()))

    # print("python: sorter {} finished!".format(my_id), flush=True)

    # flag_finish
    for i in range(sorter_num):
        slot_name = "sorter_flag_{}_{}".format(my_id, i)
        encoded_data = "1".encode()
        getattr(__import__("__main__"), slot_name)[:1] = encoded_data

def spilter(my_id, sorter_num, merger_num):
    # flag_access
    while True:
        flag = 0
        for i in range(sorter_num):
            slot_name = "sorter_flag_{}_{}".format(i, my_id)
            buffer = bytearray(flag_size)
            pyas.access_buffer(slot_name, buffer)
            flag += int(buffer[0]) - 48
        # print("python: spilter {} wait for flag, flag is {}".format(my_id, flag), flush=True)
        if flag == sorter_num:
            break
    # print("python: spliter {} start".format(my_id), flush=True)

    # flag_register
    for i in range(sorter_num):
        slot_name = "spilter_flag_{}_{}".format(my_id, i)
        setattr(__import__("__main__"), slot_name, pyas.buffer_register(slot_name, flag_size))
        encoded_data = "0".encode()
        getattr(__import__("__main__"), slot_name)[:1] = encoded_data

    print("compute{}start{}: {}".format(my_id, get_compute_start_time(), get_now()))
    pivot = []
    if merger_num > 1:
        slot_name = "pivot_{}".format(my_id)
        print("trans{}start{}: {}".format(my_id, get_trans_start_time(), get_now()))
        buffer = take_data_buffer(slot_name, buffer_size)
        data = str(buffer, "utf-8").rstrip("\x00")
        # print("python: spilter {} recv {} size: {}".format(my_id, slot_name, len(encoded_data)), flush=True)
        print("trans{}end{}: {}".format(my_id, get_trans_end_time(), get_now()))
        pivot = data.split()
        pivot = list(map(int, pivot))

    slot_name = "sorter_{}".format(my_id)
    print("trans{}start{}: {}".format(my_id, get_trans_start_time(), get_now()))
    buffer = take_data_buffer(slot_name, buffer_size)
    data = str(buffer, "utf-8").rstrip("\x00")
    # print("python: spilter {} recv {} size: {}".format(my_id, slot_name, len(encoded_data)), flush=True)
    print("trans{}end{}: {}".format(my_id, get_trans_end_time(), get_now()))
    nums = data.split()
    nums = list(map(int, nums))

    slot_data = [[] for _ in range(merger_num)]
    for num in nums:
        row = 0
        for i in range(len(pivot)):
            if num >= pivot[i]:
                row = i
                break
        slot_data[row].append(num)

    for i in range(merger_num):
        slot_name = "merger_{}_{}".format(my_id, i)
        setattr(__import__("__main__"), slot_name, pyas.buffer_register(slot_name, buffer_size))
        encoded_data = " ".join(map(str, slot_data[i])).encode()
        # print("python: spilter {} pass {} size: {}".format(my_id, slot_name, len(encoded_data)), flush=True) # important
        getattr(__import__("__main__"), slot_name)[:len(encoded_data)] = encoded_data
    print("compute{}end{}: {}".format(my_id, get_compute_end_time(), get_now()))
    # print("python: spliter {} finished!".format(my_id), flush=True)

    # flag_finish
    for i in range(sorter_num):
        slot_name = "spilter_flag_{}_{}".format(my_id, i)
        encoded_data = "1".encode()
        getattr(__import__("__main__"), slot_name)[:1] = encoded_data

def merger(my_id, sorter_num, merger_num):
    # flag_access
    while True:
        flag = 0
        for i in range(sorter_num):
            slot_name = "spilter_flag_{}_{}".format(i, my_id)
            buffer = bytearray(flag_size)
            pyas.access_buffer(slot_name, buffer)
            flag += int(buffer[0]) - 48
        # print("python: merger {} wait for flag, flag is {}".format(my_id, flag), flush=True)
        if flag == sorter_num:
            break
    # print("python: merger {} start!".format(my_id), flush=True)

    # flag_register
    for i in range(merger_num):
        slot_name = "merger_flag_{}_{}".format(my_id, i)
        setattr(__import__("__main__"), slot_name, pyas.buffer_register(slot_name, flag_size))
        encoded_data = "0".encode()
        getattr(__import__("__main__"), slot_name)[:1] = encoded_data
        # print("python: merger {} pass {} buffer: {}".format(my_id, slot_name, getattr(__import__("__main__"), slot_name)))

    print("compute{}start{}: {}".format(my_id, get_compute_start_time(), get_now()))
    min_heap = []
    for i in range(sorter_num):
        slot_name = "merger_{}_{}".format(i, my_id)
        print("trans{}start{}: {}".format(my_id, get_trans_start_time(), get_now()))
        buffer = take_data_buffer(slot_name, buffer_size)
        _data = str(buffer, "utf-8").rstrip("\x00")
        # print("python: merger {} recv {} size: {}".format(my_id, slot_name, len(encoded_data)), flush=True)
        print("trans{}end{}: {}".format(my_id, get_trans_end_time(), get_now()))
        data = _data.split()
        for x in data:
            heapq.heappush(min_heap, int(x))

    slot_name = "checker_{}".format(my_id)
    final_data = []
    while min_heap:
        final_data.append(heapq.heappop(min_heap))

    setattr(__import__("__main__"), slot_name, pyas.buffer_register(slot_name, buffer_size))
    encoded_data = " ".join(map(str, final_data)).encode()
    # print("python: merger {} pass {} size: {}".format(my_id, slot_name, len(encoded_data)), flush=True) # important
    getattr(__import__("__main__"), slot_name)[:len(encoded_data)] = encoded_data
    print("compute{}end{}: {}".format(my_id, get_compute_end_time(), get_now()))
    # print("python: merger {} finished!".format(my_id), flush=True)

    # flag_finish
    for i in range(merger_num):
        slot_name = "merger_flag_{}_{}".format(my_id, i)
        encoded_data = "1".encode()
        getattr(__import__("__main__"), slot_name)[:1] = encoded_data
        # print("python: merger {} pass {} buffer: {}".format(my_id, slot_name, getattr(__import__("__main__"), slot_name)))

def checker(my_id, sorter_num, merger_num):
    # print("python: checker {} start".format(my_id), flush=True)

    if my_id != 0:
        while True:
            flag = 0
            slot_name = "checker_flag_{}".format(my_id)
            buffer = bytearray(flag_size)
            pyas.access_buffer(slot_name, buffer)
            # print("python: checker {} recv {} buffer: {}".format(my_id, slot_name, buffer))
            flag += int(buffer[0]) - 48
            if flag == 1:
                return
    # flag_access
    while True:
        flag = 0
        for i in range(merger_num):
            slot_name = "merger_flag_{}_{}".format(i, my_id)
            buffer = bytearray(flag_size)
            pyas.access_buffer(slot_name, buffer)
            # print("python: checker {} recv {} buffer: {}".format(my_id, slot_name, buffer))
            flag += int(buffer[0]) - 48
        # print("python: checker {} wait for flag, flag is {}".format(my_id, flag), flush=True)
        if flag == merger_num:
            break
    print("compute{}start{}:checker {}".format(my_id, get_compute_start_time_checker(), get_now()))
    result = []
    for i in range(sorter_num):
        slot_name = "checker_{}".format(i)
        print("trans{}start{}:checker {}".format(my_id, get_trans_start_time_checker(), get_now()))
        buffer = take_data_buffer(slot_name, buffer_size)
        _data = str(buffer, "utf-8").rstrip("\x00")
        # print("python: checker {} recv {} size: {}".format(my_id, slot_name, len(encoded_data)), flush=True)
        print("trans{}end{}:checker {}".format(my_id, get_trans_end_time_checker(), get_now()))
        data = map(int, _data.split())
        result.extend(data)

    print("compute{}end{}:checker {}".format(my_id, get_compute_end_time_checker(), get_now()))

    # flag_finish
    for i in range(1, merger_num):
        slot_name = "checker_flag_{}".format(i)
        setattr(__import__("__main__"), slot_name, pyas.buffer_register(slot_name, flag_size))
        encoded_data = "1".encode()
        getattr(__import__("__main__"), slot_name)[:1] = encoded_data
    print("compute{}start{}:checker {}".format(my_id, get_compute_start_time_checker(), get_now()))
    for i in range(1, len(result)):
        if result[i] < result[i-1]:
            print("python: checker Error: {} < {}".format(result[i], result[i-1]))
            exit(1)
    print("compute{}end{}:checker {}".format(my_id, get_compute_end_time_checker(), get_now()))
    # print("python: checker {} finished!".format(my_id), flush=True)

def main():
    my_id = int(sys.argv[1])
    sorter_num = int(sys.argv[2])
    merger_num = int(sys.argv[3])

    sorter(my_id, sorter_num, merger_num)
    spilter(my_id, sorter_num, merger_num)
    merger(my_id, sorter_num, merger_num)
    checker(my_id, sorter_num, merger_num)

if __name__ == '__main__':
    main()
