#ifndef ALLOY_ARGS_H
#define ALLOY_ARGS_H

#include <limits.h>
#include <stdlib.h>
#include <string.h>

static inline const char *as_arg_value(int argc, char **argv, const char *key)
{
    size_t key_len = strlen(key);
    for (int i = 1; i < argc; ++i) {
        if (argv[i][0] == '-' && argv[i][1] == '-' &&
            strncmp(argv[i] + 2, key, key_len) == 0 &&
            argv[i][key_len + 2] == '=')
            return argv[i] + key_len + 3;
    }
    return NULL;
}

static inline int as_arg_int(int argc, char **argv, const char *key, int *out)
{
    const char *value = as_arg_value(argc, argv, key);
    char *end;
    long parsed;

    if (!value || !out)
        return -1;
    parsed = strtol(value, &end, 10);
    if (*value == '\0' || *end != '\0' || parsed < INT_MIN || parsed > INT_MAX)
        return -1;
    *out = (int)parsed;
    return 0;
}

#endif
