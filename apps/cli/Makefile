BINDIR := $(CURDIR)/bin
BINNAME := clip

.PHONY: all
all: build

.PHONY: build
build: tidy
	go build -o $(BINDIR)/$(BINNAME) ./cmd/...

.PHONY: tidy
tidy:
	go mod tidy

.PHONY: clean
clean:
	@rm -rf $(BINDIR)
