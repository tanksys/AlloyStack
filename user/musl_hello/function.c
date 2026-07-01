#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <unistd.h>

static const char *find_id(int argc, char **argv)
{
    const char prefix[] = "--id=";
    for (int i = 1; i < argc; ++i) {
        if (strncmp(argv[i], prefix, sizeof(prefix) - 1) == 0)
            return argv[i] + sizeof(prefix) - 1;
    }
    return "unknown";
}

int main(int argc, char **argv)
{
    const char *id = find_id(argc, argv);
    char path[64];
    char *payload = malloc(256 * 1024);
    char readback[128] = {0};
    struct stat st;

    if (!payload)
        return 10;
    memset(payload, 'A', 256 * 1024 - 1);
    payload[256 * 1024 - 1] = '\0';
    char *grown = realloc(payload, 512 * 1024);
    if (!grown) {
        free(payload);
        return 18;
    }
    payload = grown;
    memset(payload + 256 * 1024, 'B', 256 * 1024);

    if (snprintf(path, sizeof(path), "musl-%s.txt", id) < 0) {
        free(payload);
        return 11;
    }

    FILE *file = fopen(path, "w+");
    if (!file) {
        free(payload);
        return 12;
    }
    if (fwrite(payload, 1, 127, file) != 127 || fflush(file) != 0) {
        fclose(file);
        free(payload);
        return 13;
    }
    if (fseek(file, 0, SEEK_SET) != 0 ||
        fread(readback, 1, 127, file) != 127 ||
        memcmp(payload, readback, 127) != 0) {
        fclose(file);
        free(payload);
        return 14;
    }
    if (fstat(fileno(file), &st) != 0 || st.st_size != 127) {
        fclose(file);
        free(payload);
        return 15;
    }
    if (fclose(file) != 0) {
        free(payload);
        return 16;
    }

    printf("musl function id=%s file=%s size=%ld\n", id, path, (long)st.st_size);

    errno = 0;
    if (syscall(SYS_getpid) != -1 || errno != ENOSYS) {
        free(payload);
        return 17;
    }

    free(payload);
    return 0;
}
