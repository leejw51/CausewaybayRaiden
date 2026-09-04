/**
 * The draw list, executed.
 *
 * wasm hands over a flat `Float32Array` of tagged commands and a pool of
 * strings; this walks it once onto a 2D canvas at the virtual resolution. That
 * canvas then becomes a texture for the CRT pass in `renderer.ts`, so the
 * pixel art is rasterised exactly once at exactly its own size and only the
 * final blit is scaled.
 *
 * Command layouts are documented on the Rust side in `crates/raiden-core/src/
 * draw.rs`; the lengths below are the same table read from the other end.
 */
import type { Art } from "./assets";
import type { Font } from "./font";
import { createTinter, type Rgb } from "./tint";
import { GH, GW } from "./layout";

export const TAG = {
  sprite: 1,
  rect: 2,
  circle: 3,
  ring: 4,
  text: 5,
  tiled: 6,
  ellipseRing: 7,
  ellipseFill: 8,
} as const;

/** Field count of each command, tag included. Index is the tag. */
export const CMD_LEN = [0, 13, 9, 8, 9, 9, 7, 11, 10];

/**
 * Step through a draw list, calling `visit(tag, at, len)` per command.
 *
 * Returns false if the buffer ends mid-command or carries a tag this build
 * does not know — which means a stale wasm module against a newer shell, or
 * the other way round. The caller draws what it understood and stops there
 * rather than reading numbers as coordinates that were never meant to be.
 */
export function walk(
  cmds: Float32Array,
  visit: (tag: number, at: number, len: number) => void,
): boolean {
  let i = 0;
  while (i < cmds.length) {
    const tag = cmds[i];
    const len = CMD_LEN[tag] ?? 0;
    if (len === 0 || i + len > cmds.length) return false;
    visit(tag, i, len);
    i += len;
  }
  return true;
}

const TAU = Math.PI * 2;

export type Painter = {
  canvas: HTMLCanvasElement;
  resize: (vw: number) => void;
  paint: (cmds: Float32Array, pool: string[], px: number) => void;
};

function css(r: number, g: number, b: number, a: number): string {
  const to = (v: number) => Math.round(Math.max(0, Math.min(1, v)) * 255);
  return `rgba(${to(r)},${to(g)},${to(b)},${Math.max(0, Math.min(1, a))})`;
}

export function createPainter(art: Art, font: Font): Painter {
  const canvas = document.createElement("canvas");
  canvas.width = GW;
  canvas.height = GH;
  const ctx = canvas.getContext("2d", { alpha: false })!;
  ctx.imageSmoothingEnabled = false;
  const tinter = createTinter();
  const patterns = new Map<string, CanvasPattern>();

  function resize(vw: number): void {
    const w = Math.max(GW, Math.round(vw));
    if (canvas.width === w) return;
    canvas.width = w;
    canvas.height = GH;
    ctx.imageSmoothingEnabled = false;
    // Patterns are bound to the context that made them; a resize resets it.
    patterns.clear();
  }

  /** A repeating fill of `index`, tinted, reused between frames. */
  function patternFor(index: number, img: CanvasImageSource, rgb: Rgb): CanvasPattern | null {
    const key = `${index}:${rgb.map((v) => Math.round(v * 15)).join(",")}`;
    const hit = patterns.get(key);
    if (hit) return hit;
    const made = ctx.createPattern(tinter.get(index, img, rgb) as CanvasImageSource, "repeat");
    if (!made) return null;
    if (patterns.size >= 32) patterns.clear();
    patterns.set(key, made);
    return made;
  }

  function paint(cmds: Float32Array, pool: string[], px: number): void {
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.globalAlpha = 1;
    ctx.fillStyle = "#07060e";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.setTransform(1, 0, 0, 1, px, 0);

    walk(cmds, (tag, i, len) => {
      const a = cmds[i + len - 1];
      const r = cmds[i + len - 4];
      const g = cmds[i + len - 3];
      const b = cmds[i + len - 2];

      switch (tag) {
        case TAG.sprite: {
          const idx = cmds[i + 1];
          const img = art.images[idx];
          if (img) {
            ctx.globalAlpha = a;
            ctx.save();
            ctx.translate(cmds[i + 2], cmds[i + 3]);
            const rot = cmds[i + 4];
            if (rot !== 0) ctx.rotate(rot);
            ctx.scale(cmds[i + 5], cmds[i + 6]);
            ctx.drawImage(tinter.get(idx, img, [r, g, b]), -cmds[i + 7], -cmds[i + 8]);
            ctx.restore();
            ctx.globalAlpha = 1;
          }
          break;
        }
        case TAG.rect: {
          ctx.fillStyle = css(r, g, b, a);
          ctx.fillRect(cmds[i + 1], cmds[i + 2], cmds[i + 3], cmds[i + 4]);
          break;
        }
        case TAG.circle: {
          ctx.fillStyle = css(r, g, b, a);
          ctx.beginPath();
          ctx.arc(cmds[i + 1], cmds[i + 2], cmds[i + 3], 0, TAU);
          ctx.fill();
          break;
        }
        case TAG.ring: {
          ctx.strokeStyle = css(r, g, b, a);
          ctx.lineWidth = cmds[i + 4];
          ctx.beginPath();
          ctx.arc(cmds[i + 1], cmds[i + 2], cmds[i + 3], 0, TAU);
          ctx.stroke();
          break;
        }
        case TAG.text: {
          const s = pool[cmds[i + 1]];
          if (s) {
            ctx.globalAlpha = a;
            font.draw(ctx, s, cmds[i + 2], cmds[i + 3], cmds[i + 4], [r, g, b]);
            ctx.globalAlpha = 1;
          }
          break;
        }
        case TAG.tiled: {
          const idx = cmds[i + 1];
          const img = art.images[idx];
          const pat = img ? patternFor(idx, img, [r, g, b]) : null;
          if (pat) {
            const yOff = cmds[i + 2];
            ctx.save();
            // Backgrounds cover the canvas, not the playfield, so this one
            // command steps outside the playfield translation.
            ctx.setTransform(1, 0, 0, 1, 0, yOff);
            ctx.globalAlpha = a;
            ctx.fillStyle = pat;
            ctx.fillRect(0, -yOff, canvas.width, GH);
            ctx.restore();
            ctx.globalAlpha = 1;
          }
          break;
        }
        case TAG.ellipseRing:
        case TAG.ellipseFill: {
          const fill = tag === TAG.ellipseFill;
          ctx.beginPath();
          ctx.ellipse(cmds[i + 1], cmds[i + 2], cmds[i + 3], cmds[i + 4], cmds[i + 5], 0, TAU);
          if (fill) {
            ctx.fillStyle = css(r, g, b, a);
            ctx.fill();
          } else {
            ctx.strokeStyle = css(r, g, b, a);
            ctx.lineWidth = cmds[i + 6];
            ctx.stroke();
          }
          break;
        }
        default:
          break;
      }
    });
    ctx.setTransform(1, 0, 0, 1, 0, 0);
  }

  return { canvas, resize, paint };
}
