#include <stdio.h>
#include <stdlib.h>
#include <string.h>

__attribute__((import_module("env"), import_name("access_buffer"))) void access_buffer(void *slot_name, int name_size, void *buffer, int buffer_size);

#define MAX_ARRAY_LENGTH 20000000
#define MAX_BUFFER_SIZE 200000000

int result[MAX_ARRAY_LENGTH];

int main(int argc, char* argv[]) {
    int id = atoi(argv[1]);
    int sorter_num = atoi(argv[2]);
    int merger_num = atoi(argv[3]);
    printf("checker_%d start!\n", id);
    int bufferSize = MAX_BUFFER_SIZE;
    int index = 0;
    for (int i = 0; i < merger_num; i++) {
        char slot_name[20];
        sprintf(slot_name, "checker_%d", i);
        // printf("pivotname: %s\n", slot_name);
        char *buffer;
        buffer = (char *)malloc(bufferSize * sizeof(char));
        if (buffer == NULL) {
            perror("malloc error");
            return 1;
        }
        memset(buffer, 0, bufferSize * sizeof(char));
        buffer[0] = '\0'; // 初始化为空字符串
        access_buffer(slot_name, strlen(slot_name), buffer, bufferSize);
        char *ptr = buffer;
        int num;
        while (sscanf(ptr, "%d", &num) == 1) {
            result[index] = num;
            index++;
            // 移动指针到下一个数字
            while (*ptr && *ptr != ' ') {
                ptr++;
            }
            if (*ptr == ' ') {
                ptr++;
            }
        }
        free(buffer);
    }
    printf("result_array: ");
    for (int i = 0; i < index-1; i++) {
        if (result[i] > result[i+1]) {
            printf("sort error!\n");
            return 0;
        }
        printf("%d ", result[i]);
    }
    printf("%d\n", result[index-1]);
    printf("checker_%d all finished!\n", id);
    return 0;
}