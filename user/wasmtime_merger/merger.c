#include <stdio.h>
#include <stdlib.h>
#include <string.h>

__attribute__((import_module("env"), import_name("buffer_register"))) void buffer_register(void *slot_name, int name_size, void *buffer, int buffer_size);
__attribute__((import_module("env"), import_name("access_buffer"))) void access_buffer(void *slot_name, int name_size, void *buffer, int buffer_size);

#define MAX_ARRAY_LENGTH 20000000
#define MAX_BUFFER_SIZE 200000000

int result[MAX_ARRAY_LENGTH];

typedef struct {
    int value;  // 存储的值
    int arrayIndex;  // 数组索引
    int elementIndex; // 元素索引
} HeapNode;

// 最小堆的比较函数
int compare(const void *a, const void *b) {
    return ((HeapNode *)a)->value - ((HeapNode *)b)->value;
}

void heapifyDown(HeapNode *minHeap, int heapSize, int index) {
    int smallest = index;
    int left = 2 * index + 1;
    int right = 2 * index + 2;

    // 比较当前节点与其左子节点
    if (left < heapSize && minHeap[left].value < minHeap[smallest].value) {
        smallest = left;
    }

    // 比较当前节点与其右子节点
    if (right < heapSize && minHeap[right].value < minHeap[smallest].value) {
        smallest = right;
    }

    // 如果最小值不是当前节点，则交换并继续下沉
    if (smallest != index) {
        HeapNode temp = minHeap[index];
        minHeap[index] = minHeap[smallest];
        minHeap[smallest] = temp;

        // 递归调用下沉操作
        heapifyDown(minHeap, heapSize, smallest);
    }
}

int main(int argc, char* argv[]) {
    int id = atoi(argv[1]);
    int sorter_num = atoi(argv[2]);
    int merger_num = atoi(argv[3]);
    printf("merger_%d start!\n", id);
    int bufferSize = MAX_BUFFER_SIZE;

    // access buffer
    // int array[sorter_num][MAX_ARRAY_LENGTH];
    int **array = (int **)malloc(sorter_num * sizeof(int*));
    for (int i = 0; i < sorter_num; i++) {
        array[i] = (int *)malloc(MAX_ARRAY_LENGTH * sizeof(int));
    }
    int index[sorter_num];
    // int *index = (int *)malloc(sorter_num * sizeof(int));
    memset(index, 0, sizeof(index));
    for (int i = 0; i < sorter_num; i++) {
        char slot_name[20];
        sprintf(slot_name, "merger_%d_%d", i, id);
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
            array[i][index[i]] = num;
            index[i]++;
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

    // merge
    int resultIndex = 0;
    HeapNode *minHeap = (HeapNode *)malloc(sorter_num * sizeof(HeapNode));
    int heapSize = 0;
    // 初始化最小堆
    for (int i = 0; i < sorter_num; i++) {
        if (index[i] > 0) {  // 确保数组非空
            minHeap[heapSize].value = array[i][0];
            minHeap[heapSize].arrayIndex = i;
            minHeap[heapSize].elementIndex = 0;
            heapSize++;
        }
    }
    // 构建初始最小堆
    // qsort(minHeap, heapSize, sizeof(HeapNode), compare);
    // 使用下沉操作调整整个堆以确保最小堆性质
    for (int i = (heapSize - 2) / 2; i >= 0; i--) {
        heapifyDown(minHeap, heapSize, i);
    }
    while (heapSize > 0) {
        // 获取最小元素
        HeapNode minNode = minHeap[0];
        result[resultIndex++] = minNode.value;

        // 替换最小值的元素
        if (minNode.elementIndex + 1 < index[minNode.arrayIndex]) {
            minNode.elementIndex++;
            minNode.value = array[minNode.arrayIndex][minNode.elementIndex];
            // 更新堆
            minHeap[0] = minNode;
            // qsort(minHeap, heapSize, sizeof(HeapNode), compare); // 重新构建堆
        } else {
            // 如果该数组没有更多元素，则用最后一个元素替换掉
            minHeap[0] = minHeap[--heapSize];
        }
        // 执行下沉操作以恢复堆的性质
        heapifyDown(minHeap, heapSize, 0);
    }
    free(minHeap);

    // rigister mergered array
    
    char slot_name[20];
    sprintf(slot_name, "checker_%d", id);
    char *buffer;
    buffer = (char *)malloc(bufferSize * sizeof(char));
    if (buffer == NULL) {
        perror("malloc error");
        return 1;
    }
    memset(buffer, 0, bufferSize * sizeof(char));
    buffer[0] = '\0'; // 初始化为空字符串
    char *ptr = buffer;
    for (int i = 0; i < resultIndex; i++) {
        char temp[12]; // 临时缓冲区，注意要足够大以容纳最大整数和一个空格
        snprintf(temp, sizeof(temp), "%d ", result[i]); // 将整数转换为字符串，并加上空格
        // strcat(buffer, temp); // 追加到 buffer
        strncpy(ptr, temp, strlen(temp));
        ptr += strlen(temp);
    }
    // 去掉最后一个多余的空格
    // buffer[strlen(buffer) - 1] = '\0';
    *ptr++ = '\0';
    buffer_register(slot_name, strlen(slot_name), buffer, bufferSize);
    free(buffer);
    

    printf("merger_%d all finished!\n", id);
    return 0;
}