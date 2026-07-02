#ifndef ALLOY_ASBUFFER_H
#define ALLOY_ASBUFFER_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct as_buffer {
    /* Public byte range. Reserved fields must not be modified by callers. */
    void *data;
    size_t len;
    size_t capacity;
    uintptr_t _allocation;
} as_buffer_t;

#define AS_BUFFER_INIT { NULL, 0, 0, 0 }

/*
 * All functions return zero on success and a negative errno value on failure.
 * publish transfers ownership to the named slot and clears the producer handle.
 * take transfers ownership to the consumer, which must call release exactly once.
 */
int as_buffer_alloc(size_t capacity, as_buffer_t *out);
int as_buffer_publish(const char *slot, as_buffer_t *buffer, size_t len);
int as_buffer_take(const char *slot, as_buffer_t *out);
int as_buffer_release(as_buffer_t *buffer);

#ifdef __cplusplus
}
#endif

#endif
