# CAUSEWAYBAY RAIDEN — LÖVE 11.5 game
# Run from repo root with `make start`, or here with `make start`.

.DEFAULT_GOAL := help
.PHONY: help start test format

LOVE      ?= love
STYLUA    ?= stylua
LUA_SRC   := main.lua conf.lua src/world.lua src/gfx.lua src/audio.lua src/display.lua src/stage.lua src/save.lua src/test.lua src/balance.lua

help:
	@printf '%s\n' \
		'make help     list targets' \
		'make start    launch CAUSEWAYBAY RAIDEN (love .)' \
		'make test     run game tests' \
		'make format   format Lua with stylua'

start:
	$(LOVE) .

test:
	$(LOVE) . -- --test

format:
	@if command -v $(STYLUA) >/dev/null 2>&1; then \
		$(STYLUA) $(LUA_SRC); \
	else \
		echo 'stylua not found. Install with:  cargo install stylua'; \
		echo 'or:  brew install stylua'; \
		exit 1; \
	fi
