# Makefile for niri-settings
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
	install -Dm755 target/release/niri-settings $(BINDIR)/niri-settings
	install -Dm644 resources/niri-settings.desktop $(DATADIR)/applications/niri-settings.desktop
	install -Dm644 resources/icons/niri-settings.svg $(DATADIR)/icons/hicolor/scalable/apps/niri-settings.svg

uninstall:
	rm -f $(BINDIR)/niri-settings
	rm -f $(DATADIR)/applications/niri-settings.desktop
	rm -f $(DATADIR)/icons/hicolor/scalable/apps/niri-settings.svg

clean:
	cargo clean
