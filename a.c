#include <stdio.h>

int main()
{
	for (int i = 0; i < 1000000; i++) {
		printf("%d,Name%d Lastname%d,%d\n", i, i, i, i);
	}
}
