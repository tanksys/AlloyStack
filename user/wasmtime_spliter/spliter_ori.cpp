#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/time.h>

__attribute__((import_module("env"), import_name("buffer_register"))) void buffer_register(void *slot_name, int name_size, void *buffer, int buffer_size);
__attribute__((import_module("env"), import_name("access_buffer"))) void access_buffer(void *slot_name, int name_size, void *buffer, int buffer_size);

// #define MAX_ARRAY_LENGTH 152221
// #define MAX_BUFFER_SIZE 1024*1024+152221

// #define MAX_ARRAY_LENGTH 7611001
// #define MAX_BUFFER_SIZE 50*1024*1024+7611001

#define MAX_ARRAY_LENGTH 3805441
#define MAX_BUFFER_SIZE 25*1024*1024+3805441

int sorter_array[MAX_ARRAY_LENGTH]; 

void get_time() {
    timeval tv{};
    gettimeofday(&tv, nullptr);
    printf("%lld.%06lld\n", tv.tv_sec, tv.tv_usec);
}

int main(int argc, char* argv[]) {
    get_time();
    int id = atoi(argv[1]);
    int sorter_num = atoi(argv[2]);
    int merger_num = atoi(argv[3]);
    get_time();
    // printf("spliter_%d start!\n", id);
    char slot_name[20];
    int bufferSize = MAX_BUFFER_SIZE;
    char *ptr;
    int num;
    
    // access pivot buffer
    int pivot_array[10];
    int pivot_index = 0;
    if (merger_num > 1) {
        sprintf(slot_name, "pivot_%d", id);
        char *pivot_buffer;
        pivot_buffer = (char *)malloc(bufferSize * sizeof(char));
        memset(pivot_buffer, 0, bufferSize * sizeof(char));
        pivot_buffer[0] = '\0'; // 初始化为空字符串
        access_buffer(slot_name, strlen(slot_name), pivot_buffer, bufferSize);
        ptr = pivot_buffer;
        // printf("pivot_buffer: %s", pivot_buffer);
        while (sscanf(ptr, "%d", &num) == 1) {
            pivot_array[pivot_index] = num;
            pivot_index++;
            // 移动指针到下一个数字
            while (*ptr && *ptr != ' ') {
                ptr++;
            }
            if (*ptr == ' ') {
                ptr++;
            }
        }
        free(pivot_buffer);
        // printf("spliter_%d pivot access finished!\n", id);
    }

    // access sorter buffer
    sprintf(slot_name, "sorter_%d", id);
    char *sorter_buffer;
    sorter_buffer = (char *)malloc(bufferSize * sizeof(char));
    memset(sorter_buffer, 0, bufferSize * sizeof(char));
    sorter_buffer[0] = '\0'; // 初始化为空字符串
    get_time();
    access_buffer(slot_name, strlen(slot_name), sorter_buffer, bufferSize);
    get_time();
    int sorter_index = 0;
    ptr = sorter_buffer;
    get_time();
    while (sscanf(ptr, "%d", &num) == 1) {
        sorter_array[sorter_index] = num;
        sorter_index++;
        // 移动指针到下一个数字
        while (*ptr && *ptr != ' ') {
            ptr++;
        }
        if (*ptr == ' ') {
            ptr++;
        }
    }
    // free(ptr);
    printf("sorter_index: %d\n", sorter_index);
    get_time();
    // free(sorter_buffer);
    // printf("spliter_%d sorter access finished!\n", id);

    // trans to merger
    // int array[merger_num][MAX_ARRAY_LENGTH];
    get_time();
    int **array = (int **)malloc(merger_num * sizeof(int*));
    for (int i = 0; i < merger_num; i++) {
        array[i] = (int *)malloc(MAX_ARRAY_LENGTH * sizeof(int));
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
        array[row][array_index[row]] = sorter_array[i];
        array_index[row]++;
    }
    printf("merger_index_before_register: %d\n", array_index[0]);
    get_time();
    get_time();
    printf("array0: %d :%d\n", (void*)(array_index), *array_index);
    for (int i = 0; i < merger_num; i++) {
        printf("i(%d) = %d array0: %d :%d\n", (void*)&i, i, (void*)(array_index), *array_index);
        char slot_name[20];
        sprintf(slot_name, "merger_%d_%d", id, i);
        char *merger_buffer = (char *)malloc(bufferSize * sizeof(char));
        // memset(merger_buffer, 0, bufferSize * sizeof(char));
        merger_buffer[0] = '\0'; // 初始化为空字符串
        char *merger_ptr = merger_buffer;
        printf("array0: %d :%d\n", (void*)(array_index), *array_index);
        printf("i: %d; array_index_before_for: %d\n", i, array_index[i]);
        printf("%d: %d\n", (void*)(array_index+1), *(array_index+1));
        printf("array_index[0]: %d\n", array_index[0]);
        for (int j = 0; j < array_index[i]; j++) {
            char temp[12]; // 临时缓冲区，注意要足够大以容纳最大整数和一个空格
            snprintf(temp, sizeof(temp), "%d ", array[i][j]); // 将整数转换为字符串，并加上空格
            // printf("array[i][j]: %d\n", array[i][j]);
            // strcat(buffer, temp); // 追加到 buffer
            strncpy(merger_ptr, temp, strlen(temp));
            merger_ptr += strlen(temp);
        }
        // 去掉最后一个多余的空格
        // buffer[strlen(buffer) - 1] = '\0';
        *merger_ptr++ = '\0';
        buffer_register(slot_name, strlen(slot_name), merger_buffer, MAX_BUFFER_SIZE);
        // buffer_register(slot_name, strlen(slot_name), buffer, bufferSize);
        // free(buffer);
        // free(array[i]);
    }
    get_time();
    // free(array_index);
    // printf("spliter_%d all finished!\n", id);
    // get_time();
    return 0;
}