CC=gcc

.PHONY: clean all

all: servers

servers:
	$(CC) 1.c -o 1 -lpthread
	$(CC) 2.c -o 2 -lpthread
	$(CC) 3.c -o 3

clean:
	rm 1 2 3