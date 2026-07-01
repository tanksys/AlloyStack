#include <alloy/args.h>
#include <int_utils.h>
#include <stdio.h>
#include <stdlib.h>

struct heap_node {
    int value;
    int source;
    size_t index;
};

static void heap_down(struct heap_node *heap, int size, int index)
{
    for (;;) {
        int smallest = index, left = index * 2 + 1, right = left + 1;
        if (left < size && heap[left].value < heap[smallest].value)
            smallest = left;
        if (right < size && heap[right].value < heap[smallest].value)
            smallest = right;
        if (smallest == index)
            return;
        struct heap_node tmp = heap[index];
        heap[index] = heap[smallest];
        heap[smallest] = tmp;
        index = smallest;
    }
}

int main(int argc, char **argv)
{
    int id, sorter_num, merger_num, rc = 0, heap_size = 0;
    char slot[32];
    struct int_vec *sources = NULL, result = {0};
    struct heap_node *heap = NULL;

    if (as_arg_int(argc, argv, "id", &id) ||
        as_arg_int(argc, argv, "sorter_num", &sorter_num) ||
        as_arg_int(argc, argv, "merger_num", &merger_num) ||
        sorter_num <= 0 || id < 0 || id >= merger_num)
        return 2;
    sources = calloc((size_t)sorter_num, sizeof(*sources));
    heap = malloc((size_t)sorter_num * sizeof(*heap));
    if (!sources || !heap) {
        rc = 3;
        goto cleanup;
    }
    for (int i = 0; i < sorter_num; ++i) {
        as_buffer_t buffer = AS_BUFFER_INIT;
        snprintf(slot, sizeof(slot), "merger_%d_%d", i, id);
        if (as_buffer_take(slot, &buffer) ||
            int_vec_parse_buffer(&buffer, &sources[i])) {
            if (buffer._allocation)
                as_buffer_release(&buffer);
            rc = 4;
            goto cleanup;
        }
        as_buffer_release(&buffer);
        if (sources[i].len) {
            heap[heap_size++] = (struct heap_node){
                .value = sources[i].data[0], .source = i, .index = 0};
        }
    }
    for (int i = heap_size / 2; i-- > 0;)
        heap_down(heap, heap_size, i);
    while (heap_size) {
        struct heap_node node = heap[0];
        if (int_vec_push(&result, node.value)) {
            rc = 5;
            goto cleanup;
        }
        node.index++;
        if (node.index < sources[node.source].len) {
            node.value = sources[node.source].data[node.index];
            heap[0] = node;
        } else {
            heap[0] = heap[--heap_size];
        }
        if (heap_size)
            heap_down(heap, heap_size, 0);
    }
    snprintf(slot, sizeof(slot), "checker_%d", id);
    if (publish_int_vec(slot, result.data, result.len))
        rc = 6;

cleanup:
    if (sources) {
        for (int i = 0; i < sorter_num; ++i)
            int_vec_free(&sources[i]);
    }
    free(sources);
    free(heap);
    int_vec_free(&result);
    return rc;
}
