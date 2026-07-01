#include <alloy/args.h>
#include <alloy/asbuffer.h>
#include <stdio.h>

int main(int argc, char **argv)
{
    int id, stage, chain_len;
    char input_slot[32], output_slot[32];
    as_buffer_t buffer = AS_BUFFER_INIT;
    int rc;

    if (as_arg_int(argc, argv, "id", &id) ||
        as_arg_int(argc, argv, "func_num", &stage) ||
        as_arg_int(argc, argv, "chain_len", &chain_len) ||
        stage < 0 || stage >= chain_len)
        return 2;

    if (stage == 0) {
        rc = as_buffer_alloc(2, &buffer);
        if (rc)
            return 3;
        ((char *)buffer.data)[0] = '0';
        ((char *)buffer.data)[1] = '\0';
    } else {
        snprintf(input_slot, sizeof(input_slot), "buffer_%d_%d", stage - 1, id);
        rc = as_buffer_take(input_slot, &buffer);
        if (rc || buffer.len != 2)
            return 4;
        ((char *)buffer.data)[0]++;
    }

    if (stage + 1 == chain_len) {
        rc = (((char *)buffer.data)[0] == '0' + stage) ? 0 : 5;
        if (as_buffer_release(&buffer))
            return 6;
        printf("musl longchain result=%c\n", '0' + stage);
        return rc;
    }

    snprintf(output_slot, sizeof(output_slot), "buffer_%d_%d", stage, id);
    rc = as_buffer_publish(output_slot, &buffer, 2);
    if (rc) {
        as_buffer_release(&buffer);
        return 7;
    }
    return 0;
}
