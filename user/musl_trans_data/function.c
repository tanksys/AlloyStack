#include <alloy/asbuffer.h>
#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/time.h>

static uint64_t now_ns(void)
{
    struct timeval tv;
    gettimeofday(&tv, NULL);
    return (uint64_t)tv.tv_sec * 1000000000ull +
           (uint64_t)tv.tv_usec * 1000ull;
}

static size_t find_data_size(int argc, char **argv)
{
    const char prefix[] = "--data_size=";
    for (int i = 1; i < argc; ++i) {
        if (strncmp(argv[i], prefix, sizeof(prefix) - 1) == 0) {
            char *end = NULL;
            unsigned long long value = strtoull(
                argv[i] + sizeof(prefix) - 1, &end, 10);
            if (end && *end == '\0')
                return (size_t)value;
        }
    }
    return 4 * 1024;
}

int main(int argc, char **argv)
{
    const char *slot = "slot_1";
    as_buffer_t producer = AS_BUFFER_INIT;
    as_buffer_t consumer = AS_BUFFER_INIT;
    size_t data_size = find_data_size(argc, argv);
    uint64_t start_ns;
    uint64_t end_ns;
    int rc;

    rc = as_buffer_alloc(data_size, &producer);
    if (rc)
        return -rc;
    if (data_size)
        memset(producer.data, 'a', data_size);

    start_ns = now_ns();
    rc = as_buffer_publish(slot, &producer, data_size);
    if (rc) {
        as_buffer_release(&producer);
        return -rc;
    }
    rc = as_buffer_take(slot, &consumer);
    end_ns = now_ns();
    if (rc)
        return -rc;

    if (data_size && consumer.data)
        (void)((volatile unsigned char *)consumer.data)[0];

    printf("transfer cost: %llu ns size: %zu\n",
           (unsigned long long)(end_ns - start_ns), data_size);

    rc = as_buffer_release(&consumer);
    if (rc)
        return -rc;
    return 0;
}
