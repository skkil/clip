BINDIR := $(CURDIR)/bin
BINNAME := clip

.PHONY: all
all: build

.PHONY: build
build: $(BINDIR)/$(BINNAME) tidy

$(BINDIR)/$(BINNAME):
	sqlc generate
	go build -o $@ ./cmd/...

.PHONY: tidy
tidy:
	go mod tidy

.PHONY: clean
clean:
	@rm -rf $(BINDIR)

.PHONY: migrate
migrate:
	@set -a; . ./.env; set +a; atlas schema apply --env local
