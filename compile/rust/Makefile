RUST = rustc
CFLAGS = --edition 2021 -O
SOURCES = $(wildcard *.rs)
TARGET = cr.exe

all: $(TARGET)

$(TARGET): $(SOURCES)
	$(RUST) $(CFLAGS) -o $@ $^

clear:
	rm *.exe