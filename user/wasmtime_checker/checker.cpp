#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <iostream>
#include <vector>
#include <cstring>
#include <stdlib.h>
#include <cctype>
#include <fstream>
#include <sstream>
#include <algorithm>
#include <unordered_map>
using namespace std;

__attribute__((import_module("env"), import_name("access_buffer"))) void access_buffer(void *slot_name, int name_size, void *buffer, int buffer_size);

#define MAX_ARRAY_LENGTH 8000000
#define MAX_BUFFER_SIZE 80000000

int result[MAX_ARRAY_LENGTH];

int main(int argc, char* argv[]) {
    int id = atoi(argv[1]);
    int sorter_num = atoi(argv[2]);
    int merger_num = atoi(argv[3]);
    printf("checker_%d start!\n", id);
    int bufferSize = MAX_BUFFER_SIZE;
    int index = 0;
    for (int i = 0; i < merger_num; i++) {
        string slot_name = "checker_" + to_string(i);
        string buffer = string(MAX_BUFFER_SIZE, 0);
        access_buffer((void*)slot_name.c_str(), slot_name.length(), (void*)buffer.c_str(), MAX_BUFFER_SIZE);
        istringstream iss(buffer);
        int num;
        while (iss >> num) {
            result[index++] = num;
        }
    }
    // printf("result_array: ");
    // for (int i = 0; i < index-1; i++) {
    //     if (result[i] > result[i+1]) {
    //         printf("sort error!\n");
    //         printf("%d %d\n", result[i], result[i + 1]);
    //         return 0;
    //     }
    //     printf("%d ", result[i]);
    // }
    // printf("%d\n", result[index-1]);
    printf("checker_%d all finished!\n", id);
    return 0;
}