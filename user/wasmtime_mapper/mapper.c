#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <unistd.h>

__attribute__((import_module("env"), import_name("buffer_register"))) void buffer_register(void *slot_name, int name_size, void *buffer, int buffer_size);

#define MAX_WORD_LENGTH 100
#define MAX_WORDS 8000
#define MAX_SLOT_NUM 100
#define MAX_BUFFER_SIZE 8000


void to_lowercase(char *str) {
    for (int i = 0; str[i]; i++) {
        str[i] = tolower(str[i]);
    }
}
int main(int argc, char* argv[]) {
    int id = atoi(argv[1]);
    int reducer_num = atoi(argv[2]);
    printf("mapper.c recieve: id: %d, reducer_num: %d\n", id, reducer_num);

    char input_file[30];
    sprintf(input_file, "fake_data_%d.txt", id);
    FILE *file = fopen(input_file, "r");
    if (!file) {
        perror("Failed to open input file\n");
        exit(EXIT_FAILURE);
    }

    printf("mapper_%d input file: %s\n", id, input_file);
    
    int count[MAX_WORDS] = {0};
    char *words[MAX_WORDS];
    char word[MAX_WORD_LENGTH];
    int word_index = 0;
    
    char line[1024];
    while (fgets(line, sizeof(line), file)) {
        char *token = strtok(line, " \n"); // 以空格和换行作为分隔符
        while (token != NULL) {
            char word[MAX_WORD_LENGTH];
            strncpy(word, token, MAX_WORD_LENGTH);
            word[MAX_WORD_LENGTH - 1] = '\0'; // 确保字符串以 null 结尾

            to_lowercase(word);
            int found = 0;

            for (int i = 0; i < word_index; i++) {
                if (strcmp(words[i], word) == 0) {
                    count[i]++;
                    found = 1;
                    break;
                }
            }
            
            if (!found) {
                words[word_index] = strdup(word);
                count[word_index]++;
                word_index++;
            }

            token = strtok(NULL, " \n"); //读取下一个单词
        }
    }

    fclose(file);
    printf("mapper_%d_index: %d\n", id, word_index);

    char *slot_name[MAX_SLOT_NUM];
    char *buffer[MAX_SLOT_NUM];
    char slot[20];
    int bufferSize = MAX_BUFFER_SIZE;
    int slot_index = 0;
    
    for (int i = 0; i < word_index; i++) {
        int partition_index = i % reducer_num;
        sprintf(slot, "buffer_%d_%d", partition_index, id);
        int found = 0;
        for (int j = 0; j < slot_index; j++) {
            if (strcmp(slot_name[j], slot) == 0) {
                found = 1;
                sprintf(buffer[j] + strlen(buffer[j]), "%s: %d\n", words[i], count[i]);
                break;
            }
        }
        
        if (!found) {
            slot_name[slot_index] = strdup(slot);
            buffer[slot_index] = (char *)malloc(bufferSize * sizeof(char));
            if (buffer[slot_index] == NULL) {
                printf("alloc mem failed\n");
                return 1;
            }
            memset(buffer[slot_index], 0, bufferSize * sizeof(char));
            sprintf(buffer[slot_index], "%s: %d\n", words[i], count[i]);
            // printf("slot: %s; buffer: %s\n", slot_name[slot_index], buffer[slot_index]);
            slot_index++;
        }
        // printf("found: %d; word: %s; count: %d; slot_name: %s; buffer: %s\n", found, words[i], count[i], slot, buffer[slot_index]);
        free(words[i]);
    }

    for (int i = 0; i < slot_index; i++) {
        buffer_register(slot_name[i], strlen(slot_name[i]), buffer[i], bufferSize);
    }
    
    for (int i = 0; i < slot_index; i++) {
        free(slot_name[i]); // 释放 strdup 分配的内存
        free(buffer[i]);    // 释放 buffer
    }
    write(1, "mapper end!\n", sizeof("mapper end!\n"));
    // printf("mapper_%d finished!\n", id);
    return 0;
}