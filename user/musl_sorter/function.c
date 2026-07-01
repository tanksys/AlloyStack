#include <alloy/args.h>
#include <int_utils.h>
#include <stdio.h>

int main(int argc, char **argv)
{
    int id, sorter_num, merger_num, rc = 0;
    char path[64], slot[32];
    FILE *file = NULL;
    struct int_vec values = {0};

    if (as_arg_int(argc, argv, "id", &id) ||
        as_arg_int(argc, argv, "sorter_num", &sorter_num) ||
        as_arg_int(argc, argv, "merger_num", &merger_num) ||
        sorter_num <= 0 || merger_num <= 0)
        return 2;
    snprintf(path, sizeof(path), "sort_data_%d.txt", id);
    file = fopen(path, "r");
    if (!file)
        return 3;
    if (int_vec_parse_file(file, &values)) {
        rc = 4;
        goto cleanup;
    }
    fclose(file);
    file = NULL;
    qsort(values.data, values.len, sizeof(*values.data), int_compare);

    if (id == 0 && merger_num > 1) {
        struct int_vec pivots = {0};
        for (int i = 1; i < merger_num; ++i) {
            size_t index = (size_t)i * values.len / (size_t)merger_num;
            if (index >= values.len || int_vec_push(&pivots, values.data[index])) {
                int_vec_free(&pivots);
                rc = 5;
                goto cleanup;
            }
        }
        for (int i = 0; i < sorter_num; ++i) {
            snprintf(slot, sizeof(slot), "pivot_%d", i);
            if (publish_int_vec(slot, pivots.data, pivots.len)) {
                int_vec_free(&pivots);
                rc = 6;
                goto cleanup;
            }
        }
        int_vec_free(&pivots);
    }

    snprintf(slot, sizeof(slot), "sorter_%d", id);
    if (publish_int_vec(slot, values.data, values.len))
        rc = 7;

cleanup:
    if (file)
        fclose(file);
    int_vec_free(&values);
    return rc;
}
