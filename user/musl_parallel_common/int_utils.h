#ifndef ALLOY_INT_UTILS_H
#define ALLOY_INT_UTILS_H

#include <alloy/asbuffer.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>

struct int_vec {
    int *data;
    size_t len;
    size_t capacity;
};

static int int_vec_push(struct int_vec *vec, int value)
{
    if (vec->len == vec->capacity) {
        size_t capacity = vec->capacity ? vec->capacity * 2 : 256;
        int *data;
        if (capacity < vec->capacity || capacity > SIZE_MAX / sizeof(*data))
            return -1;
        data = realloc(vec->data, capacity * sizeof(*data));
        if (!data)
            return -1;
        vec->data = data;
        vec->capacity = capacity;
    }
    vec->data[vec->len++] = value;
    return 0;
}

static void int_vec_free(struct int_vec *vec)
{
    free(vec->data);
    vec->data = NULL;
    vec->len = 0;
    vec->capacity = 0;
}

static int int_vec_parse_file(FILE *file, struct int_vec *out)
{
    int ch, value = 0, in_number = 0;
    while ((ch = fgetc(file)) != EOF) {
        if (ch >= '0' && ch <= '9') {
            if (value > (INT_MAX - (ch - '0')) / 10)
                return -1;
            value = value * 10 + ch - '0';
            in_number = 1;
        } else if (in_number) {
            if (int_vec_push(out, value))
                return -1;
            value = 0;
            in_number = 0;
        }
    }
    if (in_number && int_vec_push(out, value))
        return -1;
    return 0;
}

static int int_vec_parse_buffer(const as_buffer_t *buffer, struct int_vec *out)
{
    const unsigned char *bytes = buffer->data;
    int value = 0, in_number = 0;
    if (!buffer->len || bytes[buffer->len - 1] != '\0')
        return -1;
    for (size_t i = 0; i + 1 < buffer->len; ++i) {
        int ch = bytes[i];
        if (ch >= '0' && ch <= '9') {
            if (value > (INT_MAX - (ch - '0')) / 10)
                return -1;
            value = value * 10 + ch - '0';
            in_number = 1;
        } else if (in_number) {
            if (int_vec_push(out, value))
                return -1;
            value = 0;
            in_number = 0;
        }
    }
    if (in_number && int_vec_push(out, value))
        return -1;
    return 0;
}

static int publish_int_vec(const char *slot, const int *values, size_t len)
{
    as_buffer_t buffer = AS_BUFFER_INIT;
    size_t capacity, used = 0;
    int rc;

    if (len > (SIZE_MAX - 1) / 12)
        return -1;
    capacity = len * 12 + 1;
    rc = as_buffer_alloc(capacity, &buffer);
    if (rc)
        return rc;
    for (size_t i = 0; i < len; ++i) {
        int written = snprintf(
            (char *)buffer.data + used, capacity - used, "%d ", values[i]);
        if (written < 0 || (size_t)written >= capacity - used) {
            as_buffer_release(&buffer);
            return -1;
        }
        used += (size_t)written;
    }
    ((char *)buffer.data)[used++] = '\0';
    rc = as_buffer_publish(slot, &buffer, used);
    if (rc)
        as_buffer_release(&buffer);
    return rc;
}

static int int_compare(const void *left, const void *right)
{
    int a = *(const int *)left;
    int b = *(const int *)right;
    return (a > b) - (a < b);
}

#endif
