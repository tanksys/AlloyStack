#include <alloy/args.h>
#include <alloy/asbuffer.h>
#include <ctype.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define BUCKETS 1024
#define WORD_SIZE 256

struct word_entry {
    struct word_entry *next;
    char *word;
    unsigned count;
    unsigned partition;
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
    int id, reducer_num, rc = 0;
    char path[64], word[WORD_SIZE];
    FILE *file = NULL;
    size_t *sizes = NULL, *offsets = NULL;
    as_buffer_t *buffers = NULL;

    if (!table || as_arg_int(argc, argv, "id", &id) ||
        as_arg_int(argc, argv, "reducer_num", &reducer_num) ||
        reducer_num <= 0)
        return 2;
    snprintf(path, sizeof(path), "fake_data_%d.txt", id);
    file = fopen(path, "r");
    if (!file) {
        free(table);
        return 3;
    }
    while (fscanf(file, "%255s", word) == 1) {
        uint64_t hash;
        size_t bucket;
        struct word_entry *entry;
        for (char *p = word; *p; ++p)
            *p = (char)tolower((unsigned char)*p);
        hash = word_hash(word);
        bucket = hash % BUCKETS;
        for (entry = table[bucket]; entry; entry = entry->next) {
            if (strcmp(entry->word, word) == 0) {
                entry->count++;
                break;
            }
        }
        if (!entry) {
            entry = malloc(sizeof(*entry));
            if (!entry || !(entry->word = strdup(word))) {
                free(entry);
                rc = 4;
                goto cleanup;
            }
            entry->count = 1;
            entry->partition = hash % (unsigned)reducer_num;
            entry->next = table[bucket];
            table[bucket] = entry;
        }
    }
    fclose(file);
    file = NULL;

    sizes = calloc((size_t)reducer_num, sizeof(*sizes));
    offsets = calloc((size_t)reducer_num, sizeof(*offsets));
    buffers = calloc((size_t)reducer_num, sizeof(*buffers));
    if (!sizes || !offsets || !buffers) {
        rc = 5;
        goto cleanup;
    }
    for (size_t i = 0; i < BUCKETS; ++i)
        for (struct word_entry *entry = table[i]; entry; entry = entry->next)
            sizes[entry->partition] +=
                (size_t)snprintf(NULL, 0, "%s %u\n", entry->word, entry->count);

    for (int i = 0; i < reducer_num; ++i) {
        if (as_buffer_alloc(sizes[i] + 1, &buffers[i])) {
            rc = 6;
            goto cleanup;
        }
    }
    for (size_t i = 0; i < BUCKETS; ++i) {
        for (struct word_entry *entry = table[i]; entry; entry = entry->next) {
            unsigned p = entry->partition;
            offsets[p] += (size_t)snprintf(
                (char *)buffers[p].data + offsets[p],
                buffers[p].capacity - offsets[p],
                "%s %u\n",
                entry->word,
                entry->count);
        }
    }
    for (int i = 0; i < reducer_num; ++i) {
        char slot[32];
        ((char *)buffers[i].data)[offsets[i]] = '\0';
        snprintf(slot, sizeof(slot), "buffer_%d_%d", i, id);
        if (as_buffer_publish(slot, &buffers[i], offsets[i] + 1)) {
            rc = 7;
            goto cleanup;
        }
    }
cleanup:
    if (file)
        fclose(file);
    if (buffers) {
        for (int i = 0; i < reducer_num; ++i)
            if (buffers[i]._allocation)
                as_buffer_release(&buffers[i]);
    }
    free(buffers);
    free(offsets);
    free(sizes);
    free_table(table);
    free(table);
    return rc;
}
