import pyas
import sys
import time
import os

def get_now():
    return time.time()

buffer_size = 256 * 1024 * 1024

def take_data_buffer(slot_name, size):
    take_buffer = getattr(pyas, "take_buffer", None)
    if take_buffer is not None:
        return take_buffer(slot_name)
    buffer = bytearray(size)
    pyas.access_buffer(slot_name, buffer)
    return buffer

def func_inner(func_num, func_n):
    # print("python: func {} start".format(func_num), flush=True)

    if func_num == 0:
        to_name = "func_{}".format(func_num)
        setattr(__import__("__main__"), to_name, pyas.buffer_register(to_name, buffer_size))
        encoded_data = os.urandom(buffer_size)
        getattr(__import__("__main__"), to_name)[:len(encoded_data)] = encoded_data
        # print(getattr(__import__("__main__"), to_name), flush=True)
    elif func_num == func_n - 1:
        from_name = "func_{}".format(func_num - 1)
        buffer = take_data_buffer(from_name, buffer_size)
        # print(buffer, flush=True)
    else:
        from_name = "func_{}".format(func_num - 1)
        to_name = "func_{}".format(func_num)
        buffer = pyas.buffer_register(to_name, buffer_size)
        if buffer == None:
            print("find None! id: {}".format(func_num), flush=True)
        else:
            print("id: {} ok!".format(func_num), flush=True)
        setattr(__import__("__main__"), to_name, buffer)
        source = take_data_buffer(from_name, buffer_size)
        buffer[:len(source)] = source

    # print("python: func {} finished!".format(func_num), flush=True)


def main():
    func_num = int(sys.argv[1])
    for i in range(func_num):
        func_inner(i, func_num)

if __name__ == '__main__':
    main()
