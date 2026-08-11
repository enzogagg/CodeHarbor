#include <SFML/Graphics.h>
#include <stdio.h>
#include <string.h>

static int run_self_test(void)
{
    sfVideoMode mode = {64, 64, 32};
    sfRenderWindow *window = sfRenderWindow_create(mode, "CodeHarbor CSFML sample", sfClose, NULL);

    if (window == NULL) {
        fprintf(stderr, "CSFML self-test failed: window did not open\n");
        return 84;
    }

    sfRenderWindow_clear(window, sfBlack);
    sfRenderWindow_display(window);
    sfRenderWindow_close(window);
    sfRenderWindow_destroy(window);
    puts("CSFML self-test passed");
    return 0;
}

int main(int argc, char **argv)
{
    if (argc == 2 && strcmp(argv[1], "--self-test") == 0) {
        return run_self_test();
    }

    puts("CodeHarbor CSFML sample ready");
    return 0;
}
