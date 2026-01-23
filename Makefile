# Makefile for nirify
# A native settings application for the niri Wayland compositor

PREFIX ?= /usr/local
DESTDIR ?=
BINDIR = $(DESTDIR)$(PREFIX)/bin
DATADIR = $(DESTDIR)$(PREFIX)/share

.PHONY: all build install uninstall clean

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
