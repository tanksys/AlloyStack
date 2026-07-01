#include <alloy/args.h>
#include <int_utils.h>
#include <stdio.h>
#include <stdlib.h>

int main(int argc, char **argv)
{
    int id, sorter_num, merger_num, rc = 0;
    char slot[32];
    as_buffer_t input = AS_BUFFER_INIT;
    struct int_vec pivots = {0}, values = {0};
    struct int_vec *parts = NULL;

    if (as_arg_int(argc, argv, "id", &id) ||
        as_arg_int(argc, argv, "sorter_num", &sorter_num) ||
        as_arg_int(argc, argv, "merger_num", &merger_num) ||
        id < 0 || id >= sorter_num || merger_num <= 0)
        return 2;

    if (merger_num > 1) {
        snprintf(slot, sizeof(slot), "pivot_%d", id);
        if (as_buffer_take(slot, &input) ||
            int_vec_parse_buffer(&input, &pivots)) {
            rc = 3;
            goto cleanup;
        }
        as_buffer_release(&input);
    }
    snprintf(slot, sizeof(slot), "sorter_%d", id);
    if (as_buffer_take(slot, &input) ||
        int_vec_parse_buffer(&input, &values)) {
        rc = 4;
        goto cleanup;
    }
    as_buffer_release(&input);

    parts = calloc((size_t)merger_num, sizeof(*parts));
    if (!parts) {
        rc = 5;
        goto cleanup;
    }
    for (size_t i = 0; i < values.len; ++i) {
        int partition = 0;
        while ((size_t)partition < pivots.len &&
               values.data[i] >= pivots.data[partition])
            partition++;
        if (int_vec_push(&parts[partition], values.data[i])) {
            rc = 6;
            goto cleanup;
        }
    }
    for (int i = 0; i < merger_num; ++i) {
        snprintf(slot, sizeof(slot), "merger_%d_%d", id, i);
        if (publish_int_vec(slot, parts[i].data, parts[i].len)) {
            rc = 7;
            goto cleanup;
        }
    }

cleanup:
    if (input._allocation)
        as_buffer_release(&input);
    if (parts) {
        for (int i = 0; i < merger_num; ++i)
            int_vec_free(&parts[i]);
    }
    free(parts);
    int_vec_free(&values);
    int_vec_free(&pivots);
    return rc;
}
