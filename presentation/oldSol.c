#include <stdlib.h>
#include <stdio.h>

int main() {
    char *c = malloc(10240);

    c[0] = 'c';

    int i;
    for (i = 0; i < 10; i++) { 
        printf("%c\n", c[i]);
    }

    free(c);

    return 0;
}