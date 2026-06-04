BINDIR := $(CURDIR)/bin
BINNAME := clip

SRC := $(shell find . -type f -name '*.go')

.PHONY: all
all: build

.PHONY: build
build: $(BINDIR)/$(BINNAME) tidy

$(BINDIR)/$(BINNAME): $(SRC)
	go build -o $@ $^

.PHONY: tidy
tidy:
	go mod tidy

.PHONY: clean
clean:
	@rm -rf $(BINDIR)
