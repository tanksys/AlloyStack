#include <alloy/args.h>
#include <alloy/asbuffer.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define BUCKETS 1024

struct word_entry {
    struct word_entry *next;
    char *word;
    unsigned long count;
};

static uint64_t word_hash(const char *word)
{
    uint64_t hash = UINT64_C(1469598103934665603);
    while (*word) {
        hash ^= (unsigned char)*word++;
        hash *= UINT64_C(1099511628211);
    }
    return hash;
}

static void free_table(struct word_entry **table)
{
    for (size_t i = 0; i < BUCKETS; ++i) {
        struct word_entry *entry = table[i];
        while (entry) {
            struct word_entry *next = entry->next;
            free(entry->word);
            free(entry);
            entry = next;
        }
    }
}

int main(int argc, char **argv)
{
    struct word_entry **table = calloc(BUCKETS, sizeof(*table));
    int id, mapper_num, rc = 0;
    unsigned long total = 0, unique = 0;

    if (!table || as_arg_int(argc, argv, "id", &id) ||
        as_arg_int(argc, argv, "mapper_num", &mapper_num) ||
        mapper_num <= 0)
        return 2;

    for (int mapper = 0; mapper < mapper_num; ++mapper) {
        char slot[32], *save = NULL, *line;
        as_buffer_t buffer = AS_BUFFER_INIT;
        snprintf(slot, sizeof(slot), "buffer_%d_%d", id, mapper);
        if (as_buffer_take(slot, &buffer)) {
            rc = 3;
            goto cleanup;
        }
        if (!buffer.len || ((char *)buffer.data)[buffer.len - 1] != '\0') {
            as_buffer_release(&buffer);
            rc = 4;
            goto cleanup;
        }
        for (line = strtok_r(buffer.data, "\n", &save); line;
             line = strtok_r(NULL, "\n", &save)) {
            char word[256];
            unsigned count;
            size_t bucket;
            struct word_entry *entry;
            if (sscanf(line, "%255s %u", word, &count) != 2) {
                as_buffer_release(&buffer);
                rc = 5;
                goto cleanup;
            }
            bucket = word_hash(word) % BUCKETS;
            for (entry = table[bucket]; entry; entry = entry->next)
                if (strcmp(entry->word, word) == 0)
                    break;
            if (!entry) {
                entry = malloc(sizeof(*entry));
                if (!entry || !(entry->word = strdup(word))) {
                    free(entry);
                    as_buffer_release(&buffer);
                    rc = 6;
                    goto cleanup;
                }
                entry->count = 0;
                entry->next = table[bucket];
                table[bucket] = entry;
                unique++;
            }
            entry->count += count;
            total += count;
        }
        if (as_buffer_release(&buffer)) {
            rc = 7;
            goto cleanup;
        }
    }
    printf("musl reducer_%d unique=%lu total=%lu\n", id, unique, total);

cleanup:
    free_table(table);
    free(table);
    return rc;
}
