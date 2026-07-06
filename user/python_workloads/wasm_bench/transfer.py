import sys
import time

import pyas


def parse_size() -> int:
    if len(sys.argv) >= 2:
        return int(sys.argv[1])
    return 4 * 1024


def main() -> None:
    buffer_size = parse_size()
    slot_name = "slot_1"
    producer = pyas.buffer_register(slot_name, buffer_size)

    start = time.time_ns()
    take_buffer = getattr(pyas, "take_buffer", None)
    if take_buffer is not None:
        consumer = take_buffer(slot_name)
    else:
        consumer = bytearray(buffer_size)
        pyas.access_buffer(slot_name, consumer)
    end = time.time_ns()

    if consumer is not None and buffer_size > 0:
        _ = consumer[0]
    if producer is not None and buffer_size > 0:
        producer[0] = 1

    print(f"transfer cost: {end - start} ns size: {buffer_size}")


main()
