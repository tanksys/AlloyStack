import pyas
import sys
import time

buffer_size = 500000
flag_size = 2
length_size = 20

def get_now():
    return time.time()

def take_data_buffer(slot_name, size):
    take_buffer = getattr(pyas, "take_buffer", None)
    if take_buffer is not None:
        return take_buffer(slot_name)
    get_buffer_len = getattr(pyas, "buffer_len", None)
    if get_buffer_len is not None:
        size = get_buffer_len(slot_name)
    else:
        length_buffer = bytearray(length_size)
        pyas.access_buffer(slot_name + ".len", length_buffer)
        size = int(bytes(length_buffer).rstrip(b"\x00"))
    buffer = bytearray(size)
    pyas.access_buffer(slot_name, buffer)
    return buffer

def publish_data_buffer(slot_name, encoded_data):
    buffer = pyas.buffer_register(slot_name, len(encoded_data))
    buffer[:] = encoded_data
    setattr(__import__("__main__"), slot_name, buffer)
    if (
        getattr(pyas, "buffer_len", None) is None
        and getattr(pyas, "take_buffer", None) is None
    ):
        length_slot = slot_name + ".len"
        length_buffer = pyas.buffer_register(length_slot, length_size)
        encoded_len = str(len(encoded_data)).encode()
        length_buffer[:len(encoded_len)] = encoded_len
        setattr(__import__("__main__"), length_slot, length_buffer)

def mapper(my_id, reducer_num):
    # print("python: mapper {} start, reducer_num is {}".format(my_id, reducer_num), flush=True)
    for i in range(reducer_num):
        slot_name = "flag_{}_{}".format(my_id, i)
        setattr(__import__("__main__"), slot_name, pyas.buffer_register(slot_name, flag_size))
        encoded_data = "0".encode()
        getattr(__import__("__main__"), slot_name)[:1] = encoded_data

    print("compute{}start1: {}".format(my_id, get_now()))
    input_file = "fake_data_{}.txt".format(my_id)
    # print("python: Reading from {}".format(input_file), flush=True)
    words_count = {}
    with open(input_file, "r") as f:
        print("read{}start1: {}".format(my_id, get_now()))
        data = f.readlines()
        print("read{}end1: {}".format(my_id, get_now()))
        for line in data:
            words = line.split()
            words = [word.lower() for word in words]
            for word in words:
                if word in words_count:
                    words_count[word] += 1
                else:
                    words_count[word] = 1
    # print("python: mapper {} words counted: {}".format(my_id, len(words_count)), flush=True)

    slot_data = ["" for _ in range(reducer_num)]
    for word, count in words_count.items():
        slot = hash(word) % reducer_num
        slot_data[slot] += "{} {}\n".format(word, count)

    for i, data in enumerate(slot_data):
        slot_name = "buffer_{}_{}".format(my_id, i)
        encoded_data = data.encode()
        publish_data_buffer(slot_name, encoded_data)
        # print("python: mapper {} pass {} size: {}".format(my_id, slot_name, len(encoded_data)), flush=True) # important
    print("compute{}end1: {}".format(my_id, get_now()))
    for i in range(reducer_num):
        slot_name = "flag_{}_{}".format(my_id, i)
        encoded_data = "1".encode()
        getattr(__import__("__main__"), slot_name)[:1] = encoded_data

    # print("python: mapper {} finished!".format(my_id), flush=True)

def reducer(my_id, mapper_num):
    while True:
        flag = 0
        for i in range(mapper_num):
            slot_name = "flag_{}_{}".format(i, my_id)
            buffer = bytearray(flag_size)
            pyas.access_buffer(slot_name, buffer)
            flag += int(buffer[0]) - 48
        # print("python: reducer {} wait for flag, flag is {}".format(my_id, flag), flush=True)
        if flag == mapper_num:
            break

    # print("python: reducer {} start, mapper_num is {}".format(my_id, mapper_num), flush=True)
    print("compute{}start2: {}".format(my_id, get_now()))
    data = ""
    print("trans{}start: {}".format(my_id, get_now()))
    for i in range(mapper_num):
        slot_name = "buffer_{}_{}".format(i, my_id)
        buffer = take_data_buffer(slot_name, buffer_size)
        slot_data = str(buffer, "utf-8").rstrip("\x00")
        print("python: reducer {} recv {} size: {}".format(my_id, slot_name, len(slot_data)), flush=True) # important
        data += slot_data
    print("trans{}end: {}".format(my_id, get_now()))
    counter = {}
    for line in data.split("\n"):
        if not line:
            continue
        word, count = line.split()
        count = int(count)
        if word in counter:
            counter[word] += count
        else:
            counter[word] = count
    output_file = "reducer_{}.txt".format(my_id)
    print("read{}start2: {}".format(my_id, get_now()))
    with open(output_file, "w") as f:
        for word, count in counter.items():
            f.write("{} {}\n".format(word, count))
    print("read{}end2: {}".format(my_id, get_now()))
    print("compute{}end2: {}".format(my_id, get_now()))
    # print("python: reducer {} finished!".format(my_id), flush=True)

def main():
    my_id = int(sys.argv[1])
    mapper_num = int(sys.argv[2])
    reducer_num = int(sys.argv[3])

    mapper(my_id, reducer_num)
    reducer(my_id, mapper_num)

if __name__ == '__main__':
    main()
