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

__attribute__((import_module("env"), import_name("buffer_register"))) void buffer_register(void *slot_name, int name_size, void *buffer, int buffer_size);
__attribute__((import_module("env"), import_name("access_buffer"))) void access_buffer(void *slot_name, int name_size, void *buffer, int buffer_size);

#define MAX_ARRAY_LENGTH 8000000
#define MAX_BUFFER_SIZE 80000000

int sorter_array[MAX_ARRAY_LENGTH]; 

int main(int argc, char* argv[]) {
    int id = atoi(argv[1]);
    int sorter_num = atoi(argv[2]);
    int merger_num = atoi(argv[3]);
    printf("spliter_%d start!\n", id);
    string slot_name;
    int bufferSize = MAX_BUFFER_SIZE;
    
    // access pivot buffer
    int pivot_array[10];
    int pivot_index = 0;
    if (merger_num > 1) {
        slot_name = "pivot_" + to_string(id);
        string pivot_buffer = string(MAX_BUFFER_SIZE, 0);
        access_buffer((void*)slot_name.c_str(), slot_name.length(), (void*)pivot_buffer.c_str(), MAX_BUFFER_SIZE);
        // printf("pivot_buffer: %s", pivot_buffer);
        istringstream iss(pivot_buffer);
        int num;
        while (iss >> num) {
            pivot_array[pivot_index++] = num;
        }
        printf("spliter_%d pivot access finished!\n", id);
    }

    // access sorter buffer
    slot_name = "sorter_" + to_string(id);
    string sorter_buffer = string(MAX_BUFFER_SIZE, 0);
    access_buffer((void*)slot_name.c_str(), slot_name.length(), (void*)sorter_buffer.c_str(), MAX_BUFFER_SIZE);
    int sorter_index = 0;
    istringstream iss(sorter_buffer);
    int num;
    while (iss >> num) {
        sorter_array[sorter_index++] = num;
    }
    printf("access_sorter_index: %d\n", sorter_index);
    printf("spliter_%d sorter access finished!\n", id);

    // trans to merger
    // int array[merger_num][MAX_ARRAY_LENGTH];
    int **arrays = (int **)malloc(merger_num * sizeof(int*));
    for (int i = 0; i < merger_num; i++) {
        arrays[i] = (int *)malloc(MAX_ARRAY_LENGTH * sizeof(int));
    }
    int array_index[merger_num];
    memset(array_index, 0, merger_num * sizeof(int));
    for (int i = 0; i < sorter_index; i++) {
        int row = 0;
        for (int j = 0; j < pivot_index; j++) {
            if (sorter_array[i] >= pivot_array[j]) {
                row++;
            } else {
                break;
            }
        }
        arrays[row][array_index[row]] = sorter_array[i];
        array_index[row]++;
    }
    printf("array_index[0]: %d\n", array_index[0]);
    for (int i = 0; i < merger_num; i++) {
        string slot_name = "merger_" + to_string(id) + "_" + to_string(i);
        string buffer;
        for (int j = 0; j < array_index[i]; j++) {
            buffer.append(to_string(arrays[i][j]) + " "); 
        }
        buffer.resize(MAX_BUFFER_SIZE, '\0');
        buffer_register((void*)slot_name.c_str(), slot_name.length(), (void*)buffer.c_str(), MAX_BUFFER_SIZE);
        free(arrays[i]);
    }
    printf("spliter_%d all finished!\n", id);
    return 0;
}