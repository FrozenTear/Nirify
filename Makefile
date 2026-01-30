# Makefile for nirify
# A native settings application for the niri Wayland compositor

PREFIX ?= /usr/local
DESTDIR ?=
BINDIR = $(DESTDIR)$(PREFIX)/bin
DATADIR = $(DESTDIR)$(PREFIX)/share

.PHONY: all build install uninstall clean deploy

all: build

build:
	cargo build --release

install:
	install -Dm755 target/release/nirify $(BINDIR)/nirify
	install -Dm644 nirify.desktop $(DATADIR)/applications/nirify.desktop
	install -Dm644 resources/icons/nirify.svg $(DATADIR)/icons/hicolor/scalable/apps/nirify.svg

uninstall:
	rm -f $(BINDIR)/nirify
	rm -f $(DATADIR)/applications/nirify.desktop
	rm -f $(DATADIR)/icons/hicolor/scalable/apps/nirify.svg

clean:
	cargo clean

# Deploy a new release by tagging and pushing to trigger GitHub Actions
deploy:
	@if [ -n "$$(git status --porcelain)" ]; then \
		echo "Error: Working tree is not clean. Commit or stash changes first."; \
		exit 1; \
	fi
	@VERSION=$$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/'); \
	if git rev-parse "v$$VERSION" >/dev/null 2>&1; then \
		echo "Error: Tag v$$VERSION already exists."; \
		exit 1; \
	fi; \
	echo "Deploying v$$VERSION..."; \
	git tag "v$$VERSION" && \
	git push origin "v$$VERSION" && \
	echo "Tagged and pushed v$$VERSION. Release workflow started."
