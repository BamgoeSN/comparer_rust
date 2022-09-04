CC = g++
CFLAGS = -O2 -Wall -lm -static -std=gnu++20
SOURCES = $(wildcard *.cpp)
# CC = gcc
# CFLAGS = -O2 -Wall -lm -static -std=gnu11
# SOURCES = $(wildcard *.c)

TARGET = wr.exe

all: $(TARGET)

$(TARGET): $(SOURCES)
	$(CC) $^ -o $@ $(CFLAGS)

clear:
	rm *.exe