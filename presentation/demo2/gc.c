#include <stdio.h>
#include <stdlib.h>

typedef struct {
    int y;
} C;

int main() {
    C *x;

    for (long long i = 0; i < 100000000000; i++) {
        x = (C *)malloc(sizeof(C)); // Allocate memory
        if (x == NULL) {
            printf("Memory allocation failed\n");
            exit(1);
        }
        x->y = 2;
        // Note: Memory for 'x' is never deallocated, simulating a memory leak
    }

    return 0;
}
