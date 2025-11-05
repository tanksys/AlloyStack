#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

__attribute__((import_module("env"), import_name("buffer_register"))) void buffer_register(void *slot_name, int name_size, void *buffer, int buffer_size);

#define MAX_ARRAY_LENGTH 20000000
#define MAX_BUFFER_SIZE 200000000

// 比较函数，用于 qsort
int compare(const void *a, const void *b) {
    return (*(int *)a - *(int *)b); // 升序排序
}

int array[MAX_ARRAY_LENGTH];

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
    // char line[1024];
    // while (fgets(line, sizeof(line), file)) {
    //     char *token = strtok(line, " \n"); // 以空格和换行作为分隔符
    //     while (token != NULL) {
    //         array[index++] = atoi(token);
    //         token = strtok(NULL, " \n"); //读取下一个单词
    //     }
    // }
    // time_t now;
    // time(&now);
    // printf("%ld read start\n", now);
    // write(1, "read start\n", sizeof("read start\n"));
    char number[10];
    while (array[index++] = readfile(file));
    // time(&now);
    // printf("%ld read finished\n", now);
    // write(1, "read finished\n", sizeof("read finished\n"));
    // while (fscanf(file, "%s", number) != EOF) {
    //     array[index] = atoi(number);
    //     // printf("array[%d]: %d\n", index, array[index]);
    //     index++;
    // }
    // printf("sorter_%d read finished!\n", id);
    // printf("index: %d\n", index);
    fclose(file);
    // qsort(array, index, sizeof(int), compare);

    // printf("sorter_%d sort finished!\n", id);

    if (merger_num > 1 && id == 0) {
        int pivot[merger_num-1];
        for (int i = 0; i < merger_num-1; i++) {
            int idx = (i+1) * index / merger_num;
            pivot[i] = array[idx];
        }
        char *buffer;
        buffer = (char *)malloc(bufferSize * sizeof(char));
        if (buffer == NULL) {
            perror("malloc error");
            return 1;
        }
        memset(buffer, 0, bufferSize * sizeof(char));
        buffer[0] = '\0';
        for (int i = 0; i < merger_num-1; i++) {
            char temp[12];
            snprintf(temp, sizeof(temp), "%d ", pivot[i]);
            strcat(buffer, temp); // 追加到 buffer
        }
        buffer[strlen(buffer) - 1] = '\0';
        for (int k = 0; k < sorter_num; k++) {
            char slot_name[20];
            sprintf(slot_name, "pivot_%d", k);
            // printf("pivotname: %s\n", slot_name);
            buffer_register(slot_name, strlen(slot_name), buffer, bufferSize);
        }
        free(buffer);
    }

    // printf("sorter_%d pivot finished!\n", id);
    // time(&now);
    // printf("%ld before alloc\n", now);
    // write(1, "alloc start\n", sizeof("alloc start\n"));
    char slot_name[20];
    sprintf(slot_name, "sorter_%d", id);
    char *buffer;
    buffer = (char *)malloc(bufferSize * sizeof(char));
    if (buffer == NULL) {
        perror("malloc error");
        return 1;
    }
    // time(&now);
    // printf("%ld alloc finished\n", now);
    // write(1, "alloc finished\n", sizeof("alloc finished\n"));
    memset(buffer, 0, bufferSize * sizeof(char));
    // time(&now);
    // printf("%ld memset finished\n", now);
    // write(1, "memset finished\n", sizeof("memset finished\n"));
    // buffer[0] = '\0'; // 初始化为空字符串
    char *ptr = buffer;
    for (int i = 0; i < index; i++) {
        char temp[12]; // 临时缓冲区，注意要足够大以容纳最大整数和一个空格
        snprintf(temp, sizeof(temp), "%d ", array[i]); // 将整数转换为字符串，并加上空格
        strncpy(ptr, temp, strlen(temp));
        ptr += strlen(temp);
        // strcat(buffer, temp); // 追加到 buffer
    }
    *ptr++ = '\0';
    // buffer[strlen(buffer) - 1] = '\0';
    // write(1, "buffer make finished\n", sizeof("buffer make finished\n"));
    buffer_register(slot_name, strlen(slot_name), buffer, bufferSize);
    // write(1, "buffer register finished\n", sizeof("buffer register finished\n"));
    free(buffer);
    // printf("sorter_%d all finished!\n", id);
    // write(1, "all finished\n", sizeof("all finished\n"));
    return 0;
}