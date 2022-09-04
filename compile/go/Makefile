GO = go build
SOURCES = $(wildcard *.go)
TARGET = cr.exe

all: $(TARGET)

$(TARGET): $(SOURCES)
	$(GO) -o $@ $^

clear:
	rm *.exe