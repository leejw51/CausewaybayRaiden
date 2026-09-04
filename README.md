# CAUSEWAYBAY RAIDEN

Hong Kong vertical shmup. A rust programmer vs the bugs, from Causeway Bay to HKU.

It exists twice, from the same design and the same art:

| | | |
| --- | --- | --- |
| [`love2d/`](love2d/) | LÖVE 11.5 and Lua | the original; ships as a macOS app |
| [`typescript/`](typescript/) | Rust compiled to wasm, TypeScript, three.js | the same game in a browser, on Cloudflare |

## Run

The desktop game needs LÖVE 11.5+ (`brew install love` on macOS):

```bash
make start        # or: love love2d
make test
```

The web game needs Rust and Node:

```bash
make web-start    # http://localhost:5290
make web-test
```

`make help` lists everything.

## Controls

| Key | Action |
|---|---|
| Arrows / WASD | Move |
| Space | Start / continue (shot in play) |
| Z / J | Shot (hold) |
| X / Shift / K | Bomb |
| C | Insert coin |
| P / Esc | Pause |
| F / F11 | Toggle window / fullscreen |
| Up / Down | Choose EASY / NORMAL / HARD |

Hold shot to focus (move slower). The tiny cyan pixel is your hitbox.

After you pick a stage, choose a rank. **NORMAL** is the intended fight. AI agents mostly block enemy bullets. Bosses show HP when you hit them.

## How to play

Shot rank (collect **P**):

1. `PRINTLN`
2. `TRAIT`
3. `ASYNC`
4. `TOKIO`

AI agent pickups (tokens, like vibe coding — they burn out):

- **C** Claude (yellow) — lock-on shots
- **G** Grok (red) — wide burst
- **X** Codex (green) — dual streams

When tokens run out, the agent leaves. Pick up another **C / G / X** to refill or recruit again.

Other pickups:

- **P** power up
- **B** bomb
- **S** points
- **U** UNWRAP — piercing shots
- **N** UNSAFE — extra damage
- Rare **1UP**

World 1 maps: Causeway Bay, MTR Line, HKU Campus.

## Display

The 192×256 playfield is drawn at a uniform pixel scale that fills the window
height, and the visible width follows the window's shape: a 3:4 window shows
exactly the playfield, a wide screen shows more of the same world on either
side (and the ship can fly there). Nothing is stretched or letterboxed. `F`
toggles fullscreen; the window is freely resizable.

## Build and release

The version of record is one line in [`VERSION`](VERSION). A `v<version>` tag
that disagrees with it stops the release build rather than shipping.

```bash
make check    # what CI runs: byte-compile every module, then the suite
make love     # love2d/build/CausewaybayRaiden.love — needs LÖVE to run
make app      # a double-clickable macOS .app with LÖVE 11.5 inside, signed
make notarize # send that .app to Apple and staple the ticket
```

`make app` signs with a Developer ID if the machine has one and falls back to
ad-hoc, which runs locally and nowhere else. Notarising is the step that makes
a *downloaded* copy open without "Apple could not verify…", and it needs
credentials this repository does not carry:

```bash
export APPLE_ID=you@example.com
export APPLE_PASSWORD=abcd-efgh-ijkl-mnop   # app-specific, from appleid.apple.com
export APPLE_TEAM_ID=ABCDE12345
make notarize
```

CI runs the suite on Linux under xvfb — twice, once from the checkout and once
from the packaged `.love`, because only the second can catch a module or a
sprite left out of the archive — and builds the macOS bundle on every push.
Pushing a `v*` tag runs [`release.yml`](.github/workflows/release.yml), which
signs and notarises the app and attaches it, the portable `.love`, and their
checksums to a GitHub release. It needs these repository secrets:

| secret | what it is |
| --- | --- |
| `MACOS_CERTIFICATE_P12_BASE64` | Developer ID Application certificate, `base64` of the `.p12` |
| `MACOS_CERTIFICATE_PASSWORD` | the password that `.p12` was exported with |
| `APPLE_ID` | the Apple ID to notarise as |
| `APPLE_APP_SPECIFIC_PASSWORD` | an app-specific password for it |
| `TEAM_ID` | the Developer Team ID |

Without them the build still runs and falls back to ad-hoc signing, so a fork
gets an unsigned bundle rather than a failure.

## The web port

[`typescript/`](typescript/) is the same game with the parts swapped out that a
browser will not run. Rust holds the simulation, the menus and the HUD, and
compiles to a 136 KB wasm module; TypeScript gives it a canvas, a keyboard, Web
Audio and local storage; three.js puts the finished frame through a CRT shader.
A frame crosses the boundary as a flat list of drawing commands rather than as
a stream of calls.

Two things get better in the move. The art is shrunk at build time by a Rust
tool using the same sizes and the same chroma key `src/gfx.lua` applies at
launch, so 155 MB of source renders become about 1.5 MB of pixels. And the game
becomes testable without a screen: `typescript/crates/raiden-core/tests/play.rs`
plays stages at a fixed seed and a fixed frame time and asserts on what
happens, which the LÖVE build cannot do because loading a sprite there needs a
graphics context.

[`typescript/README.md`](typescript/README.md) has the details.

```bash
make web-start    # play it locally
make web-check    # what CI runs: lint, both suites, a production build
make web-deploy   # wrangler deploy
```

## Credits

Causewaybay AI, 2026. Sprites and backgrounds generated for an MSX-style look. Chiptune and SFX are synthesized at runtime.
