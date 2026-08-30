# CAUSEWAYBAY RAIDEN
# Usage: make / make help / make start / make test / make check / make app

.DEFAULT_GOAL := help
.PHONY: help version start test lint fmt-check format check love love-smoke \
        app notarize gatekeeper clean

GAME := love2d

# One number, in one file, so a `v0.1.0` tag has something to be checked
# against. The release workflow refuses to build when the two disagree.
VERSION := $(shell sed -n '1p' VERSION | tr -d ' \t\r\n')

help:
	@printf '%s\n' \
		'CAUSEWAYBAY RAIDEN $(VERSION)' \
		'' \
		'make help      list targets' \
		'make version   print the version of record' \
		'make start     launch CAUSEWAYBAY RAIDEN (love love2d)' \
		'make test      run game tests' \
		'make lint      byte-compile every Lua file' \
		'make fmt-check fail if stylua would reformat anything' \
		'make check     lint + test (what CI runs)' \
		'make format    format Lua with stylua' \
		'make love      build the .love archive' \
		'make app       build a signed macOS .app with LÖVE inside' \
		'make notarize  notarize and staple the .app (needs Apple credentials)' \
		'make clean     remove build output'

# Printed, and sanity-checked while we are here: a tag comparison against a
# malformed number would pass for the wrong reason.
version:
	@echo "$(VERSION)" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$$' || { \
		echo "VERSION is not a semantic version: '$(VERSION)'" >&2; exit 1; }
	@echo "$(VERSION)"

start test lint fmt-check format check love love-smoke app notarize gatekeeper clean:
	$(MAKE) -C $(GAME) $@
