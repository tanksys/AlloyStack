#include <stdio.h>
#include <stdlib.h>
#include <string.h>

__attribute__((import_module("env"), import_name("buffer_register"))) void buffer_register(void *slot_name, int name_size, void *buffer, int buffer_size);
__attribute__((import_module("env"), import_name("access_buffer"))) void access_buffer(void *slot_name, int name_size, void *buffer, int buffer_size);

#define MAX_ARRAY_LENGTH 20000000
#define MAX_BUFFER_SIZE 200000000

int sorter_array[MAX_ARRAY_LENGTH]; 

int main(int argc, char* argv[]) {
    int id = atoi(argv[1]);
    int sorter_num = atoi(argv[2]);
    int merger_num = atoi(argv[3]);
    printf("spliter_%d start!\n", id);
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
        if (pivot_buffer == NULL) {
            perror("malloc error");
            return 1;
        }
        memset(pivot_buffer, 0, bufferSize * sizeof(char));
        pivot_buffer[0] = '\0'; // 初始化为空字符串
        access_buffer(slot_name, strlen(slot_name), pivot_buffer, bufferSize);
        ptr = pivot_buffer;
        printf("pivot_buffer: %s", pivot_buffer);
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
        printf("spliter_%d pivot access finished!\n", id);
    }

    // access sorter buffer
    sprintf(slot_name, "sorter_%d", id);
    char *sorter_buffer;
    sorter_buffer = (char *)malloc(bufferSize * sizeof(char));
    if (sorter_buffer == NULL) {
        perror("malloc error");
        return 1;
    }
    memset(sorter_buffer, 0, bufferSize * sizeof(char));
    sorter_buffer[0] = '\0'; // 初始化为空字符串
    access_buffer(slot_name, strlen(slot_name), sorter_buffer, bufferSize);
    int sorter_index = 0;
    ptr = sorter_buffer;
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
    free(sorter_buffer);
    printf("spliter_%d sorter access finished!\n", id);

    // trans to merger
    // int array[merger_num][MAX_ARRAY_LENGTH];
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

    for (int i = 0; i < merger_num; i++) {
        char slot_name[20];
        sprintf(slot_name, "merger_%d_%d", id, i);
        char *buffer;
        buffer = (char *)malloc(bufferSize * sizeof(char));
        if (buffer == NULL) {
            perror("malloc error");
            return 1;
        }
        memset(buffer, 0, bufferSize * sizeof(char));
        buffer[0] = '\0'; // 初始化为空字符串
        ptr = buffer;
        for (int j = 0; j < array_index[i]; j++) {
            char temp[12]; // 临时缓冲区，注意要足够大以容纳最大整数和一个空格
            snprintf(temp, sizeof(temp), "%d ", array[i][j]); // 将整数转换为字符串，并加上空格
            // strcat(buffer, temp); // 追加到 buffer
            strncpy(ptr, temp, strlen(temp));
            ptr += strlen(temp);
        }
        // 去掉最后一个多余的空格
        // buffer[strlen(buffer) - 1] = '\0';
        *ptr++ = '\0';
        buffer_register(slot_name, strlen(slot_name), buffer, bufferSize);
        free(buffer);
        free(array[i]);
    }
    free(array_index);
    printf("spliter_%d all finished!\n", id);
    return 0;
}