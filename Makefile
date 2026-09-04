# CAUSEWAYBAY RAIDEN
# Usage: make / make help / make start / make test / make check / make app

.DEFAULT_GOAL := help
.PHONY: help version start test lint fmt-check format check love love-smoke \
        app notarize gatekeeper clean \
        web web-start web-stop web-build web-art web-wasm web-test web-e2e \
        web-e2e-dist \
        web-check web-e2e-dist web-deploy web-clean

GAME := love2d
WEB  := typescript

# One number, in one file, so a `v0.1.0` tag has something to be checked
# against. The release workflow refuses to build when the two disagree.
VERSION := $(shell sed -n '1p' VERSION | tr -d ' \t\r\n')

help:
	@printf '%s\n' \
		'CAUSEWAYBAY RAIDEN $(VERSION)' \
		'' \
		'make help      list targets' \
		'make version   print the version of record' \
		'' \
		'  desktop (LÖVE, love2d/)' \
		'make start     launch CAUSEWAYBAY RAIDEN (love love2d)' \
		'make test      run game tests' \
		'make lint      byte-compile every Lua file' \
		'make fmt-check fail if stylua would reformat anything' \
		'make check     lint + test (what CI runs)' \
		'make format    format Lua with stylua' \
		'make love      build the .love archive' \
		'make app       build a signed macOS .app with LÖVE inside' \
		'make notarize  notarize and staple the .app (needs Apple credentials)' \
		'make clean     remove build output' \
		'' \
		'  web (Rust to wasm + TypeScript, typescript/)' \
		'make web-start serve it on http://localhost:5290' \
		'make web-stop  stop that server' \
		'make web-build production bundle in typescript/dist' \
		'make web-test  Rust suite + browser-side suite' \
		'make web-e2e   Playwright, against a real browser' \
		'make web-e2e-dist  the same suite, against the Cloudflare bundle' \
		'make web-check everything CI runs for the web build' \
		'make web-deploy build, then wrangler deploy to Cloudflare' \
		'make web-clean remove web build output'

# Printed, and sanity-checked while we are here: a tag comparison against a
# malformed number would pass for the wrong reason.
version:
	@echo "$(VERSION)" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$$' || { \
		echo "VERSION is not a semantic version: '$(VERSION)'" >&2; exit 1; }
	@echo "$(VERSION)"

start test lint fmt-check format check love love-smoke app notarize gatekeeper clean:
	$(MAKE) -C $(GAME) $@

# The web build lives in its own directory with its own Makefile, and its
# targets are reachable from here under a `web-` prefix so the two ports never
# argue over a name like `test` or `clean`.
web: web-start

web-start web-stop web-build web-art web-wasm web-test web-e2e web-e2e-dist \
web-check web-deploy web-clean:
	$(MAKE) -C $(WEB) $(patsubst web-%,%,$@)
