VERSION = 6
PATCHLEVEL = 1
SUBLEVEL = 0
O ?= out

.PHONY: all clean
all:
	mkdir -p "$(O)"
	$(CC) -Wall -Wextra -g fixture.c -o "$(O)/vmlinux"

clean:
	rm -f "$(O)/vmlinux"
