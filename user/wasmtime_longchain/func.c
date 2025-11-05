#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>

__attribute__((import_module("env"), import_name("buffer_register"))) void buffer_register(void *slot_name, int name_size, void *buffer, int buffer_size);
__attribute__((import_module("env"), import_name("access_buffer"))) void access_buffer(void *slot_name, int name_size, void *buffer, int buffer_size);

int main(int argc, char* argv[]) {
    int id = atoi(argv[1]);
    int func_num = atoi(argv[2]);
    printf("func.c recieve: id: %d, func_num: %d\n", id, func_num);

    char slot_name[20];
    int bufferSize = 2;
    char *buffer = (char *)malloc(bufferSize * sizeof(char));
    
    if (func_num == 0) {
        sprintf(slot_name, "buffer_%d_%d", func_num, id);
        buffer[0] = '0';
        buffer[1] = '\0';
        buffer_register(slot_name, strlen(slot_name), buffer, bufferSize);
    } else {
        sprintf(slot_name, "buffer_%d_%d", func_num-1, id);
        access_buffer(slot_name, strlen(slot_name), buffer, bufferSize);
        sprintf(slot_name, "buffer_%d_%d", func_num, id);
        buffer[0] += 1;
        buffer_register(slot_name, strlen(slot_name), buffer, bufferSize);
    }
    free(buffer);
    printf("func_%d finished!\n", func_num);
    return 0;
}