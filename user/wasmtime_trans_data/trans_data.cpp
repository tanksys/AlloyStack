#include <sys/time.h>
#include <stdio.h>
#include <iostream>
#include <vector>
#include <cstring>
#include <stdlib.h>
#include <cctype>
#include <fstream>
#include <algorithm>
#include <unordered_map>
#include <stdint.h>

using namespace std;

__attribute__((import_module("env"), import_name("buffer_register"))) void buffer_register(void *slot_name, int name_size, void *buffer, int buffer_size);
__attribute__((import_module("env"), import_name("access_buffer"))) void access_buffer(void *slot_name, int name_size, void *buffer, int buffer_size);

static uint64_t now_ns() {
    timeval tv{};
    gettimeofday(&tv, nullptr);
    return static_cast<uint64_t>(tv.tv_sec) * 1000000000ull +
           static_cast<uint64_t>(tv.tv_usec) * 1000ull;
}

static size_t parse_size(int argc, char **argv) {
    if (argc < 2) {
        return 4 * 1024;
    }
    char *end = nullptr;
    unsigned long long value = strtoull(argv[1], &end, 10);
    if (end != nullptr && *end == '\0') {
        return static_cast<size_t>(value);
    }
    return 4 * 1024;
}

int main(int argc, char **argv)  {
    size_t buffer_size = parse_size(argc, argv);
    string slot_name = "tmp";
    vector<char> buffer(buffer_size, 'a');
    uint64_t start_ns;
    uint64_t end_ns;

    start_ns = now_ns();
    buffer_register((void*)slot_name.c_str(), slot_name.length(), (void*)buffer.data(), buffer_size);
    access_buffer((void*)slot_name.c_str(), slot_name.length(), (void*)buffer.data(), buffer_size);
    end_ns = now_ns();

    if (buffer_size > 0) {
        volatile char touched = buffer[0];
        (void)touched;
    }
    printf("transfer cost: %llu ns size: %zu\n",
           (unsigned long long)(end_ns - start_ns), buffer_size);
    return 0;
}
