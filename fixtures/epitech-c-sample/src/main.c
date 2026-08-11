#include <stdio.h>
#include <string.h>

static int add(int left, int right)
{
    return left + right;
}

int main(int argc, char **argv)
{
    if (argc == 2 && strcmp(argv[1], "--self-test") == 0) {
        if (add(20, 22) != 42) {
            fprintf(stderr, "self-test failed\n");
            return 84;
        }
        puts("self-test passed");
        return 0;
    }

    puts("CodeHarbor sample ready");
    return 0;
}
