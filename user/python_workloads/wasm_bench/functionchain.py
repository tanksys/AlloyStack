import pyas
import sys
import time

def get_now():
    return time.time()

buffer_size = 256 * 1024 * 1024

def take_data_buffer(slot_name, size):
    get_buffer_len = getattr(pyas, "buffer_len", None)
    if get_buffer_len is not None:
        size = get_buffer_len(slot_name)
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
        buffer = getattr(__import__("__main__"), to_name)
        if buffer_size:
            buffer[0] = 1
            buffer[-1] = 1
        # print(getattr(__import__("__main__"), to_name), flush=True)
    elif func_num == func_n - 1:
        from_name = "func_{}".format(func_num - 1)
        buffer = take_data_buffer(from_name, buffer_size)
        if hasattr(__import__("__main__"), from_name):
            delattr(__import__("__main__"), from_name)
        # print(buffer, flush=True)
    else:
        from_name = "func_{}".format(func_num - 1)
        to_name = "func_{}".format(func_num)
        move_buffer = getattr(pyas, "move_buffer", None)
        if move_buffer is not None:
            move_buffer(from_name, to_name)
            if hasattr(__import__("__main__"), from_name):
                delattr(__import__("__main__"), from_name)
            return
        source = take_data_buffer(from_name, buffer_size)
        capacity = (
            len(source)
            if getattr(pyas, "take_buffer", None) is not None
            else buffer_size
        )
        buffer = pyas.buffer_register(to_name, capacity)
        if buffer == None:
            print("find None! id: {}".format(func_num), flush=True)
        else:
            print("id: {} ok!".format(func_num), flush=True)
        setattr(__import__("__main__"), to_name, buffer)
        buffer[:len(source)] = source
        if hasattr(__import__("__main__"), from_name):
            delattr(__import__("__main__"), from_name)

    # print("python: func {} finished!".format(func_num), flush=True)


def main():
    func_num = int(sys.argv[1])
    for i in range(func_num):
        func_inner(i, func_num)

if __name__ == '__main__':
    main()
