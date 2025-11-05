#include <unistd.h>

int main()
{
    write(1, "write.c: Hello World!\n", sizeof("write.c: Hello World!\n"));
    return 0;
}