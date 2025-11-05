#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
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
using namespace std;

__attribute__((import_module("env"), import_name("buffer_register"))) void buffer_register(void *slot_name, int name_size, void *buffer, int buffer_size);

#define MAX_ARRAY_LENGTH 8000000
#define MAX_BUFFER_SIZE 80000000

// 比较函数，用于 qsort
int compare(const void *a, const void *b) {
    return (*(int *)a - *(int *)b); // 升序排序
}

int arrays[MAX_ARRAY_LENGTH];

char nc(FILE *stream) {
  static char buf[1<<20], *p1 = buf, *p2 = buf;
  return p1 == p2 && (p2 = (p1 = buf) + fread(buf, 1, 1 << 20, stream), p1 == p2) ? EOF : *p1 ++;
}

int readfile(FILE *stream) {
  int x = 0, ch = nc(stream);
  for (; ch < '0' || ch > '9'; ch = nc(stream));
  for (; ch >= '0' && ch <= '9'; ch = nc(stream))
    x = (x << 1) + (x << 3) + (ch ^ 48);
  return x;
}

int main(int argc, char* argv[]) {
    int id = atoi(argv[1]);
    int sorter_num = atoi(argv[2]);
    int merger_num = atoi(argv[3]);
    printf("sorter_%d start!\n", id);
    int bufferSize = MAX_BUFFER_SIZE;
    char input_file[30];
    sprintf(input_file, "sort_data_%d.txt", id);
    FILE *file = fopen(input_file, "r");
    if (!file) {
        perror("Failed to open input file\n");
        exit(EXIT_FAILURE);
    }
    int index = 0;
    char number[10];
    while (arrays[index++] = readfile(file));
    printf("sorter_index: %d\n", index);
    fclose(file);

    sort(arrays, arrays + index);

    if (merger_num > 1 && id == 0) {
        int pivot[merger_num-1];
        for (int i = 0; i < merger_num-1; i++) {
            int idx = (i+1) * index / merger_num;
            pivot[i] = arrays[idx];
        }
        string pivot_buffer;
        for (int i = 0; i < merger_num-1; i++) {
            pivot_buffer.append(to_string(pivot[i]) + " ");
        }
        pivot_buffer.resize(MAX_BUFFER_SIZE, '\0');
        for (int k = 0; k < sorter_num; k++) {
            string slot_name = "pivot_" + to_string(k);
            // printf("pivotname: %s\n", slot_name);
            buffer_register((void*)slot_name.c_str(), slot_name.length(), (void*)pivot_buffer.c_str(), MAX_BUFFER_SIZE);
        }
    }

    string slot_name = "sorter_" + to_string(id);
    string buffer;
    for (int i = 0; i < index; i++) {
        buffer.append(to_string(arrays[i]) + " "); 
    }
    buffer.resize(MAX_BUFFER_SIZE, '\0');
    buffer_register((void*)slot_name.c_str(), slot_name.length(), (void*)buffer.c_str(), MAX_BUFFER_SIZE);
    return 0;
}