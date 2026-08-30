# CAUSEWAYBAY RAIDEN
# Usage: make / make help / make start / make test / make format

.DEFAULT_GOAL := help
.PHONY: help start test format

GAME := love2d

help:
	@printf '%s\n' \
		'make help     list targets' \
		'make start    launch CAUSEWAYBAY RAIDEN (love love2d)' \
		'make test     run game tests' \
		'make format   format Lua with stylua'

start:
	$(MAKE) -C $(GAME) start

test:
	$(MAKE) -C $(GAME) test

format:
	$(MAKE) -C $(GAME) format
