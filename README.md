# CAUSEWAYBAY RAIDEN

Hong Kong vertical shmup. A rust programmer vs the bugs, from Causeway Bay to HKU.

Made with [LÖVE](https://love2d.org/) 11.5 and Lua. Game source lives in [`love2d/`](love2d/).

## Run

LÖVE 11.5+ required (`brew install love` on macOS).

```bash
make start
```

Tests:

```bash
make test
```

Or from the game folder:

```bash
love love2d
```

## Controls

| Key | Action |
|---|---|
| Arrows / WASD | Move |
| Space | Start / continue (shot in play) |
| Z / J | Shot (hold) |
| X / Shift / K | Bomb |
| C | Insert coin |
| P / Esc | Pause |
| Tab / L | Toggle vertical / horizontal layout |
| F / F11 | Toggle window / fullscreen |

Hold shot to focus (move slower). The tiny cyan pixel is your hitbox.

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

## Layout

- **Vertical** — portrait arcade cabinet (192×256 playfield)
- **Horizontal** — landscape with side bezels

## Credits

Causewaybay AI, 2026. Sprites and backgrounds generated for an MSX-style look. Chiptune and SFX are synthesized at runtime.
