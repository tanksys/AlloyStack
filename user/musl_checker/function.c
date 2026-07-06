#include <alloy/args.h>
#include <int_utils.h>
#include <stdio.h>

int main(int argc, char **argv)
{
    int merger_num, previous = 0, have_previous = 0;
    size_t total = 0;

    if (as_arg_int(argc, argv, "merger_num", &merger_num) || merger_num <= 0)
        return 2;
    for (int i = 0; i < merger_num; ++i) {
        char slot[32];
        as_buffer_t buffer = AS_BUFFER_INIT;
        struct int_vec values = {0};
        snprintf(slot, sizeof(slot), "checker_%d", i);
        if (as_buffer_take(slot, &buffer) ||
            int_vec_parse_buffer(&buffer, &values)) {
            if (buffer._allocation)
                as_buffer_release(&buffer);
            int_vec_free(&values);
            return 3;
        }
        for (size_t j = 0; j < values.len; ++j) {
            if (have_previous && values.data[j] < previous) {
                as_buffer_release(&buffer);
                int_vec_free(&values);
                return 4;
            }
            previous = values.data[j];
            have_previous = 1;
            total++;
        }
        as_buffer_release(&buffer);
        int_vec_free(&values);
    }
    printf("musl parallel sort checked %zu values\n", total);
    return 0;
}
