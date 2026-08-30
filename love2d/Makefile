# CAUSEWAYBAY RAIDEN — LÖVE 11.5 game
# Run from repo root with `make start`, or here with `make start`.
#
#   make start    open the window
#   make test     run the in-engine suite
#   make check    everything CI runs
#   make love     build the .love archive
#   make app      build a double-clickable, signed macOS .app
#   make notarize send the .app to Apple and staple the ticket

.DEFAULT_GOAL := help
.PHONY: help start test format fmt fmt-check lint check love verify-archive \
        love-smoke app app-icon app-sign app-smoke notarize gatekeeper clean

LOVE      ?= love
LUAJIT    ?= luajit
STYLUA    ?= stylua
LUA_SRC   := main.lua conf.lua src/world.lua src/gfx.lua src/audio.lua src/display.lua src/stage.lua src/save.lua src/test.lua src/balance.lua

# The version of record lives in one file at the repo root, so a tag has a
# single number to be checked against. `make version` up there prints it.
VERSION := $(shell sed -n '1p' $(CURDIR)/../VERSION | tr -d ' \t\r\n')

help:
	@printf '%s\n' \
		'make help      list targets' \
		'make start     launch CAUSEWAYBAY RAIDEN (love .)' \
		'make test      run game tests' \
		'make lint      byte-compile every Lua file' \
		'make fmt-check fail if stylua would reformat anything' \
		'make check     lint + test (what CI runs)' \
		'make format    format Lua with stylua' \
		'make love      build build/$(APP_NAME).love' \
		'make app       build a signed macOS .app with LÖVE inside (macOS only)' \
		'make notarize  notarize and staple the .app (needs Apple credentials)' \
		'make clean     remove build/'

# Fail with a usable message rather than a stack trace three targets deep.
.PHONY: require-love require-luajit
require-love:
	@command -v $(LOVE) > /dev/null 2>&1 || { \
		echo "LÖVE not found. Install it (brew install love) or set LOVE=." >&2; \
		exit 1; }

require-luajit:
	@command -v $(LUAJIT) > /dev/null 2>&1 || { \
		echo "LuaJIT not found. Install it (brew install luajit) or set LUAJIT=." >&2; \
		exit 1; }

# ------------------------------------------------------------------- run/test

start: require-love
	$(LOVE) . $(ARGS)

# The suite runs inside the engine because the game is the engine: loading a
# sprite needs a graphics context, so there is no honest headless shortcut.
# On a machine with no display, run it under xvfb — which is what CI does.
test: require-love
	$(LOVE) . -- --test

# --------------------------------------------------------------------- checks

# A glob, not a hand-written list. The list in LUA_SRC is what stylua formats
# and it has to be maintained; what compiles must never depend on somebody
# remembering to add a file, because a syntax error in an unlisted module is
# found by whoever launches the game.
lint: require-luajit
	@status=0; \
	for file in *.lua src/*.lua; do \
		$(LUAJIT) -bl "$$file" /dev/null > /dev/null || { \
			echo "  syntax error in $$file" >&2; status=1; }; \
	done; \
	if [ $$status -eq 0 ]; then \
		echo "  every Lua file compiles ($$(ls *.lua src/*.lua | wc -l | tr -d ' ') files)"; \
	fi; \
	exit $$status

format:
	@if command -v $(STYLUA) >/dev/null 2>&1; then \
		$(STYLUA) $(LUA_SRC); \
	else \
		echo 'stylua not found. Install with:  cargo install stylua'; \
		echo 'or:  brew install stylua'; \
		exit 1; \
	fi

fmt: format

fmt-check:
	@if command -v $(STYLUA) >/dev/null 2>&1; then \
		$(STYLUA) --check $(LUA_SRC) && echo "  formatting is clean"; \
	else \
		echo 'stylua not found. Install with:  cargo install stylua'; \
		echo 'or:  brew install stylua'; \
		exit 1; \
	fi

# What CI runs. `fmt-check` is deliberately not in here: the tree is not
# stylua-clean, and a formatting pass over 100 KB of game source is a commit of
# its own rather than a thing to smuggle into a build change. Run
# `make format` once and add it here when you want it enforced.
check: lint test

# -------------------------------------------------------------------- package

LOVE_VERSION ?= 11.5
APP_NAME     := CausewaybayRaiden
DISPLAY_NAME := Causewaybay Raiden
BUNDLE_ID    := com.causewaybay.raiden
LOVE_URL     := https://github.com/love2d/love/releases/download/$(LOVE_VERSION)/love-$(LOVE_VERSION)-macos.zip
BUILD        := build
APP          := $(BUILD)/$(APP_NAME).app
GAME_LOVE    := $(BUILD)/$(APP_NAME).love

# The game as one archive: what every platform with LÖVE installed can run,
# and what goes inside the macOS bundle below.
#
# Staged by name rather than by copying the directory, so the throwaway junk
# that accumulates beside the source — .tmp_save_test, a stray screenshot —
# cannot ride along into a 76 MB download.
love: $(GAME_LOVE)

$(GAME_LOVE): $(wildcard *.lua) $(wildcard src/*.lua) $(wildcard assets/*)
	@rm -rf "$(BUILD)/stage" && mkdir -p "$(BUILD)/stage"
	@cp *.lua "$(BUILD)/stage/"
	@cp -R src assets "$(BUILD)/stage/"
	@rm -f "$(GAME_LOVE)"
	@cd "$(BUILD)/stage" && zip -qr9 "../$(APP_NAME).love" .
	@rm -rf "$(BUILD)/stage"
	@$(MAKE) --no-print-directory verify-archive ARCHIVE="$(GAME_LOVE)"
	@echo "  $(GAME_LOVE) ($$(du -h '$(GAME_LOVE)' | cut -f1))"

# Every module the game requires has to be inside the archive.
#
# A missing one is not a subtle failure — the game dies on the first `require`
# — but it dies on somebody else's machine, because the checkout it was built
# from has the file sitting right there and `make start` never touches the
# archive. Checked here, deterministically, with no window server needed.
verify-archive:
	@test -n "$(ARCHIVE)" || { echo "  set ARCHIVE=" >&2; exit 1; }
	@missing=""; \
	for file in *.lua src/*.lua; do \
		unzip -l "$(ARCHIVE)" 2>/dev/null | grep -qE " $$file$$" || missing="$$missing $$file"; \
	done; \
	for file in conf.lua main.lua; do \
		unzip -l "$(ARCHIVE)" 2>/dev/null | grep -qE " $$file$$" || missing="$$missing $$file"; \
	done; \
	if [ -n "$$missing" ]; then \
		echo "  the archive is missing:$$missing" >&2; \
		exit 1; \
	fi
	@# Assets are the other half of the game and are not `require`d, so a
	@# staging mistake there would only show as a black screen at run time.
	@have=$$(unzip -l "$(ARCHIVE)" | grep -cE ' assets/.*\.png$$'); \
	want=$$(ls assets/*.png | wc -l | tr -d ' '); \
	if [ "$$have" != "$$want" ]; then \
		echo "  the archive carries $$have sprites, the tree has $$want" >&2; \
		exit 1; \
	fi; \
	echo "  archive carries every module and all $$want sprites"

# The archive, run. `verify-archive` proves every file is present; this proves
# they add up to a program, from the zip rather than from the working tree
# where every module always exists. Needs a display, so CI runs it under xvfb.
love-smoke: love require-love
	@$(LOVE) "$(GAME_LOVE)" --test > "$(BUILD)/smoke.log" 2>&1 || { \
		echo "  the archive did not run:" >&2; tail -20 "$(BUILD)/smoke.log" >&2; exit 1; }
	@grep -q '^# all tests passed' "$(BUILD)/smoke.log" || { \
		echo "  the archive ran but the suite did not pass:" >&2; \
		tail -20 "$(BUILD)/smoke.log" >&2; exit 1; }
	@rm -f "$(BUILD)/smoke.log"
	@echo "  the archive starts and passes its own suite"

# ------------------------------------------------------------------ macOS app

# A real double-clickable .app, with LÖVE inside it.
#
# `make love` above produces an archive that needs LÖVE installed. This
# produces something you can hand to a person who has never heard of LÖVE: the
# runtime is downloaded and embedded, the icon is built from the game's own
# ship sprite, and the whole thing is signed and zipped.
#
# Two entitlements are not optional and both are LuaJIT's fault: the hardened
# runtime needs `allow-jit` and `allow-unsigned-executable-memory`, or the app
# dies the moment LuaJIT compiles anything.
#
# Signing uses a Developer ID if the machine has one and falls back to ad-hoc,
# which is enough to run locally and not enough to hand to somebody else
# without them right-clicking Open. Notarising (`make notarize`) is what
# finishes that, and it is deliberately a separate step.
app: love
	@# Single quotes, not backticks: sh would treat `make app` as a command
	@# substitution and re-enter this target on every platform that is not macOS.
	@test "$$(uname -s)" = "Darwin" || { echo "  'make app' is macOS only." >&2; exit 1; }
	@rm -rf "$(APP)" "$(BUILD)/love-macos" "$(BUILD)/$(APP_NAME)-macos.zip"

	@echo "  fetching LÖVE $(LOVE_VERSION)"
	@curl -fsSL -o "$(BUILD)/love-macos.zip" "$(LOVE_URL)"
	@unzip -q -o "$(BUILD)/love-macos.zip" -d "$(BUILD)/love-macos"
	@mv "$(BUILD)/love-macos/love.app" "$(APP)"

	@cp "$(GAME_LOVE)" "$(APP)/Contents/Resources/game.love"

	@# Without these the bundle is still called "LÖVE" everywhere macOS shows a
	@# name, and two different games installed at once collide on the bundle id.
	@plutil -replace CFBundleName -string "$(DISPLAY_NAME)" "$(APP)/Contents/Info.plist"
	@plutil -replace CFBundleDisplayName -string "$(DISPLAY_NAME)" "$(APP)/Contents/Info.plist"
	@plutil -replace CFBundleIdentifier -string "$(BUNDLE_ID)" "$(APP)/Contents/Info.plist"
	@plutil -replace CFBundleShortVersionString -string "$(VERSION)" "$(APP)/Contents/Info.plist"
	@plutil -replace CFBundleVersion -string "$(VERSION)" "$(APP)/Contents/Info.plist"
	@# LÖVE ships these so a .love dropped on the icon runs. This bundle *is*
	@# one game and must not open somebody else's.
	@plutil -remove CFBundleDocumentTypes "$(APP)/Contents/Info.plist" 2>/dev/null || true
	@plutil -remove UTExportedTypeDeclarations "$(APP)/Contents/Info.plist" 2>/dev/null || true
	@plutil -replace CFBundleExecutable -string "love" "$(APP)/Contents/Info.plist"
	@plutil -replace NSHighResolutionCapable -bool true "$(APP)/Contents/Info.plist"

	@$(MAKE) --no-print-directory app-icon
	@$(MAKE) --no-print-directory app-sign

	@# Skipped where there is no window server to draw into — a CI runner has
	@# none, and a smoke test that cannot open a window would fail for a reason
	@# that has nothing to do with the bundle. `verify-archive` still runs
	@# there and catches the same class of fault deterministically.
	@if [ -n "$(SKIP_SMOKE)" ]; then \
		echo "  smoke test skipped (SKIP_SMOKE set)"; \
	else \
		$(MAKE) --no-print-directory app-smoke; \
	fi

	@cd "$(BUILD)" && ditto -c -k --sequesterRsrc --keepParent "$(APP_NAME).app" "$(APP_NAME)-macos.zip"
	@rm -rf "$(BUILD)/love-macos" "$(BUILD)/love-macos.zip"
	@echo
	@du -sh "$(APP)" "$(BUILD)/$(APP_NAME)-macos.zip" | sed 's/^/    /'
	@echo
	@echo "  Open it with: open \"$(APP)\""

# The icon, from the game's own ship. `sips` refuses to upscale cleanly, so
# every size is generated from the 1024×1024 source rather than from each other.
app-icon:
	@test -f assets/player.png || { echo "  no assets/player.png; keeping LÖVE's icon"; exit 0; }
	@rm -rf "$(BUILD)/icon.iconset" && mkdir -p "$(BUILD)/icon.iconset"
	@for size in 16 32 64 128 256 512 1024; do \
		sips -s format png -z $$size $$size assets/player.png \
			--out "$(BUILD)/icon.iconset/icon_$${size}x$${size}.png" > /dev/null 2>&1; \
	done
	@for size in 16 32 128 256 512; do \
		double=$$((size * 2)); \
		cp "$(BUILD)/icon.iconset/icon_$${double}x$${double}.png" \
			"$(BUILD)/icon.iconset/icon_$${size}x$${size}@2x.png" 2>/dev/null || true; \
	done
	@rm -f "$(BUILD)/icon.iconset/icon_1024x1024.png"
	@iconutil -c icns "$(BUILD)/icon.iconset" -o "$(BUILD)/icon.icns"
	@cp "$(BUILD)/icon.icns" "$(APP)/Contents/Resources/OS X AppIcon.icns" 2>/dev/null || true
	@cp "$(BUILD)/icon.icns" "$(APP)/Contents/Resources/Love.icns" 2>/dev/null || true
	@cp "$(BUILD)/icon.icns" "$(APP)/Contents/Resources/GameIcon.icns" 2>/dev/null || true
	@rm -rf "$(BUILD)/icon.iconset" "$(BUILD)/icon.icns"
	@echo "  icon from assets/player.png"

# Signed inside-out: nested binaries first, then the executable, then the
# bundle. Signing the outer bundle first invalidates itself the moment
# anything inside it is signed afterwards.
app-sign:
	@test -d "$(APP)" || { echo "  no $(APP); run 'make app'" >&2; exit 1; }
	@rm -f "$(BUILD)/entitlements.plist"
	@printf '%s\n' \
		'<?xml version="1.0" encoding="UTF-8"?>' \
		'<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">' \
		'<plist version="1.0"><dict>' \
		'  <key>com.apple.security.cs.allow-jit</key><true/>' \
		'  <key>com.apple.security.cs.allow-unsigned-executable-memory</key><true/>' \
		'  <key>com.apple.security.cs.disable-library-validation</key><true/>' \
		'</dict></plist>' \
		> "$(BUILD)/entitlements.plist"
	@# `APPLE_SIGNING_IDENTITY` wins when it is set, because that is what the
	@# release workflow exports after importing the certificate into a keychain
	@# it made for the job. Falling back to a keychain search is for a laptop.
	@identity="$${APPLE_SIGNING_IDENTITY:-}"; \
	if [ -z "$$identity" ] || [ "$$identity" = "-" ]; then \
		identity=$$(security find-identity -v -p codesigning 2>/dev/null \
			| grep "Developer ID Application" | head -1 | awk -F'"' '{print $$2}'); \
	fi; \
	if [ -z "$$identity" ]; then \
		identity="-"; \
		echo "  no Developer ID found - signing ad-hoc (runs here, not elsewhere)"; \
	else \
		echo "  signing as $$identity"; \
	fi; \
	stamp=--timestamp; \
	[ "$$identity" = "-" ] && stamp=""; \
	find "$(APP)/Contents/Frameworks" -type f 2>/dev/null | while read -r file; do \
		if file "$$file" | grep -q "Mach-O"; then \
			codesign --force --options runtime --sign "$$identity" \
				--entitlements "$(BUILD)/entitlements.plist" $$stamp "$$file" || exit 1; \
		fi; \
	done; \
	find "$(APP)/Contents/Frameworks" -maxdepth 1 -name "*.framework" 2>/dev/null | while read -r bundle; do \
		codesign --force --options runtime --sign "$$identity" \
			--entitlements "$(BUILD)/entitlements.plist" $$stamp "$$bundle" || exit 1; \
	done; \
	codesign --force --options runtime --sign "$$identity" \
		--entitlements "$(BUILD)/entitlements.plist" $$stamp \
		"$(APP)/Contents/MacOS/love" || exit 1; \
	codesign --force --options runtime --sign "$$identity" \
		--entitlements "$(BUILD)/entitlements.plist" $$stamp "$(APP)" || exit 1
	@rm -f "$(BUILD)/entitlements.plist"
	@codesign --verify --deep --strict "$(APP)" && echo "  signature verifies"

# Start the thing that was built and see whether it survives boot.
#
# `verify-archive` proves every file is present; this proves they add up to a
# program. It is the only check that runs the *bundle* rather than the
# checkout, and the difference is the whole point: `make start` loads modules
# from the working tree, where everything always exists.
#
# The in-engine suite is the payload, because it exits with a status and needs
# no window server beyond the one the bundle opens for itself. A missing module
# dies on the first `require`, long before the first assertion.
app-smoke:
	@test -d "$(APP)" || { echo "  no $(APP); run 'make app'" >&2; exit 1; }
	@"$(APP)/Contents/MacOS/love" "$(APP)/Contents/Resources/game.love" --test \
		> "$(BUILD)/smoke.log" 2>&1 || { \
		echo "  the bundle did not run:" >&2; \
		tail -20 "$(BUILD)/smoke.log" >&2; \
		exit 1; }
	@grep -q '^# all tests passed' "$(BUILD)/smoke.log" || { \
		echo "  the bundle ran but the suite did not pass:" >&2; \
		tail -20 "$(BUILD)/smoke.log" >&2; \
		exit 1; }
	@rm -f "$(BUILD)/smoke.log"
	@echo "  the bundle starts and passes its own suite"

# ------------------------------------------------------------------ notarize

# Signing is not enough on its own, and this is the part that finishes it.
#
# A Developer ID signature with the hardened runtime and a secure timestamp is
# necessary and not sufficient: since macOS 10.15 anything distributed outside
# the App Store must also be notarised by Apple. Without it, a *downloaded*
# copy is refused and the person who downloaded it gets "Apple could not
# verify…".
#
# Not part of `make app`, for two reasons: it needs credentials this repository
# must not carry, and it uploads the bundle to Apple and waits, which is not
# something a build should do without being asked.
#
#     export APPLE_ID=you@example.com
#     export APPLE_PASSWORD=abcd-efgh-ijkl-mnop   # app-specific
#     export APPLE_TEAM_ID=ABCDE12345
#     make notarize
notarize:
	@test -d "$(APP)" || { echo "  no $(APP); run 'make app' first" >&2; exit 1; }
	@for name in APPLE_ID APPLE_PASSWORD APPLE_TEAM_ID; do \
		eval "value=\$$$$name"; \
		[ -n "$$value" ] || { \
			echo "  $$name is not set." >&2; \
			echo "  Notarising needs an Apple ID, an app-specific password" >&2; \
			echo "  from appleid.apple.com, and the team id. None is kept here." >&2; \
			exit 1; }; \
	done

	@# Ad-hoc cannot be notarised, and finding that out from Apple takes
	@# minutes. The signature says which it is, so ask it first.
	@codesign -dv --verbose=2 "$(APP)" 2>&1 | grep -q "Authority=Developer ID Application" || { \
		echo "  $(APP) is not signed with a Developer ID." >&2; \
		echo "  Apple will refuse an ad-hoc signature; there is nothing to submit." >&2; \
		exit 1; }

	@echo "  submitting to Apple - this takes minutes, not seconds"
	@ditto -c -k --sequesterRsrc --keepParent "$(APP)" "$(BUILD)/notarize.zip"
	@xcrun notarytool submit "$(BUILD)/notarize.zip" \
		--apple-id "$$APPLE_ID" \
		--password "$$APPLE_PASSWORD" \
		--team-id "$$APPLE_TEAM_ID" \
		--wait --timeout 30m
	@rm -f "$(BUILD)/notarize.zip"

	@# Stapled, so the ticket travels with the bundle and Gatekeeper can check
	@# it with no network at all.
	@xcrun stapler staple "$(APP)"
	@xcrun stapler validate "$(APP)"

	@# The distribution zip was made before any of this and carries an
	@# unstapled copy. Rebuilt, or the download is exactly what it was.
	@rm -f "$(BUILD)/$(APP_NAME)-macos.zip"
	@cd "$(BUILD)" && ditto -c -k --sequesterRsrc --keepParent \
		"$(APP_NAME).app" "$(APP_NAME)-macos.zip"

	@$(MAKE) --no-print-directory gatekeeper

# What a person who downloads it will actually get.
#
# `--context context:primary-signature` is the one that matters: plain
# `spctl -a` accepts an un-notarised Developer ID bundle, and Finder does not.
# Asking the wrong question is how "it is signed" gets mistaken for "it opens".
gatekeeper:
	@test -d "$(APP)" || { echo "  no $(APP); run 'make app' first" >&2; exit 1; }
	@codesign --verify --deep --strict "$(APP)" && echo "  signature: valid"
	@if xcrun stapler validate "$(APP)" > /dev/null 2>&1; then \
		echo "  notarised: yes, and stapled"; \
	else \
		echo "  notarised: NO - a downloaded copy will be blocked"; \
	fi
	@spctl -a -vvv -t open --context context:primary-signature "$(APP)" 2>&1 \
		| sed 's/^/  /' || true

clean:
	rm -rf $(BUILD) .tmp_save_test
