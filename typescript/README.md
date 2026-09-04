# CAUSEWAYBAY RAIDEN — web

The LÖVE game in [`../love2d/`](../love2d/), ported to run in a browser.

Rust holds the game and compiles to wasm. TypeScript is the shell around it:
a canvas, a keyboard, Web Audio, local storage. Three.js draws the finished
frame through a CRT pass. Cloudflare serves it.

```bash
make start     # http://localhost:5290
make check     # lint, both test suites, a production build
make deploy    # wrangler deploy
```

## How it is put together

```
crates/raiden-core/   the game: simulation, menus, HUD, draw list  -> wasm
crates/mkassets/      shrinks love2d/assets into public/art        -> build tool
src/engine/           canvas, layout, input, the three.js CRT pass
src/audio/chip.ts     the 8-bit synth, ported from src/audio.lua
src/game/store.ts     progress, in local storage
```

Every decision about what happens on screen is made in Rust. A frame crosses
into JavaScript as two views into wasm memory — a flat list of tagged draw
commands and a pool of strings the text commands index into — so a frame with
a thousand sparks in it costs two reads rather than a thousand calls. The
command layouts are documented in
[`crates/raiden-core/src/draw.rs`](crates/raiden-core/src/draw.rs).

The browser owns the four things it will not delegate:

| the shell does | because |
| --- | --- |
| rasterises the draw list onto a 2D canvas | Rust has no canvas |
| runs that canvas through a WebGL shader | shake, scanlines, flash |
| synthesises every sound | Web Audio is the only sound hardware here |
| reads the keyboard, saves progress | ditto |

### Display

The rule is the LÖVE one, in [`src/engine/layout.ts`](src/engine/layout.ts):
one fixed virtual height of 256 and a uniform scale, so pixels are never
stretched. The virtual width then follows the window's shape and the canvas
covers it edge to edge — no bars, no letterbox. A window wider than 3:4 shows
more of the same world on either side of the 192-wide playfield, and the ship
may fly out there. A window taller than 3:4 crops the sides instead.

### Art

`love2d/assets` is 155 MB of 1024-square studio renders. The desktop game
redraws each one into a canvas of a few dozen pixels at launch and keeps only
that; `make art` does the same shrink once, at build time, with the same
chroma-key thresholds, and `public/art` comes out at about 1.5 MB. The sizes
live in [`crates/raiden-core/src/sprites.rs`](crates/raiden-core/src/sprites.rs),
which is also where the draw list's art indices come from — the build tool
writes `manifest.json` from that same table, and the loader checks the two
agree before a mismatch can turn into the wrong sprite on screen.

Generated art is not committed. `make start` and `make build` produce it.

## Controls

| key | action |
| --- | --- |
| Arrows / WASD | move |
| Z / Space / J | shot (hold), and confirm in the menus |
| Enter | confirm |
| X / K / Shift | bomb |
| C | insert coin |
| P / Esc | pause, and back out of a menu |
| F / F11, or double-click | fullscreen |
| M | mute |

On a touch device an on-screen pad appears instead.

Hold the shot to focus and move slower. The tiny cyan pixel is your hitbox.

## Testing

The point of putting the game in Rust is that it can be played without a
browser: `crates/raiden-core/tests/play.rs` runs stages at a fixed seed and a
fixed frame time and asserts on what happens — that the boss arrives, that a
bomb inside the death window saves the ship, that an agent runs out of tokens
and leaves its badge behind, that nothing grows without bound over a minute of
play. LÖVE could only ever test this inside a window on a machine with a GPU.

```bash
make test      # the Rust suite, then the browser-side one
make e2e       # Playwright: wasm loads, WebGL draws, the keys walk the cabinet
make e2e-dist  # the same suite, against the bundle Cloudflare will serve
```

Two of the end-to-end tests measure the rendered frame rather than the DOM,
because the faults they guard against are silent — nothing throws, the game
just looks wrong, and only a side-by-side against the LÖVE build shows it:

- **not gamma-crushed.** Declaring the game canvas an sRGB texture makes WebGL
  decode it to linear light on every read, and this renderer's shader writes
  its result straight out without encoding back. Mean luminance measured 15.9
  crushed against 37.3 correct.
- **no letterboxing.** On a window taller than 3:4 the canvas cannot get any
  narrower than the playfield, so the shader has to crop the sides to fill it.
  Getting that fit backwards puts a bar over an eighth of the height at each
  end; the test measures the dark margin, which is 1% correct and 17% wrong.

`e2e-dist` is the one that covers the gap between the dev server and the real
thing: a `.wasm` served as anything but `application/wasm` fails streaming
instantiation, an asset Vite resolved from source never reaches `dist`, and a
base path can work under Vite and nowhere else. It builds, starts `wrangler
dev` — Cloudflare's own runtime, locally, no account needed — and runs the
suite against that.

## Deploying

```bash
npx wrangler login
make deploy
```

`wrangler.jsonc` serves `dist/` as static assets — 61 files, about 1.7 MB, of
which 138 KB is the wasm module. There is no server side, no build step on
Cloudflare's end, and no bindings: it is a static site with a game in it, so
the free plan covers it.

`make e2e-dist` runs the whole suite against that bundle through `wrangler dev`
first, which needs no account at all.
