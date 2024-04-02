#include <stdlib.h>
#include <stdio.h>


int memLeak(int numIn){
    int *i = malloc(32);
    i[0] = 5;

    int retVal = i[0] + i[1] + numIn;


    free(i);

    return retVal;
}


int main() {
    int i = 10;
    int x = memLeak(i);


    for(int k = 0; k < 10000; k++) {
        x = memLeak(x);
    }


    printf("Final X: %i\n", x);

    return 0;
}