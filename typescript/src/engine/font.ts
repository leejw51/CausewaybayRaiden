/**
 * The game's 8x8 arcade font, baked into an atlas.
 *
 * The bitmap comes from the wasm module, which is also what measures strings,
 * so text the game centred is text the browser draws at the same width. LÖVE
 * drew each lit pixel as its own rectangle; a page full of menu text is
 * thousands of those, so here each glyph is blitted once from a pre-rendered
 * atlas, with one atlas per colour actually used.
 */
export type Font = {
  draw: (
    ctx: CanvasRenderingContext2D,
    text: string,
    x: number,
    y: number,
    scale: number,
    rgb: readonly [number, number, number],
  ) => void;
  /** Width in pixels — the same fixed cell the wasm side measures with. */
  width: (text: string, scale: number) => number;
};

const CELL = 8;
const LIMIT = 64;

export function createFont(charset: string, bitmap: Uint8Array): Font {
  const index = new Map<string, number>();
  for (let i = 0; i < charset.length; i += 1) index.set(charset[i], i);
  const n = charset.length;

  // The master atlas: white glyphs on transparent, one 8x8 cell each.
  const base = document.createElement("canvas");
  base.width = Math.max(1, n * CELL);
  base.height = CELL;
  const bctx = base.getContext("2d");
  if (bctx) {
    bctx.fillStyle = "#fff";
    for (let g = 0; g < n; g += 1) {
      for (let row = 0; row < CELL; row += 1) {
        const bits = bitmap[g * CELL + row] ?? 0;
        if (bits === 0) continue;
        for (let col = 0; col < CELL; col += 1) {
          if (bits & (1 << (7 - col))) bctx.fillRect(g * CELL + col, row, 1, 1);
        }
      }
    }
  }

  const tinted = new Map<string, HTMLCanvasElement>();

  function atlasFor(rgb: readonly [number, number, number]): HTMLCanvasElement {
    const q = rgb.map((v) => Math.max(0, Math.min(15, Math.round(v * 15))));
    const key = q.join(",");
    const hit = tinted.get(key);
    if (hit) return hit;
    const out = document.createElement("canvas");
    out.width = base.width;
    out.height = base.height;
    const ctx = out.getContext("2d");
    if (ctx) {
      ctx.imageSmoothingEnabled = false;
      ctx.drawImage(base, 0, 0);
      // Glyphs are pure white, so painting the colour straight through their
      // alpha is exactly a multiply — no third pass needed here.
      ctx.globalCompositeOperation = "source-in";
      ctx.fillStyle = `rgb(${(q[0] / 15) * 255} ${(q[1] / 15) * 255} ${(q[2] / 15) * 255})`;
      ctx.fillRect(0, 0, out.width, out.height);
      ctx.globalCompositeOperation = "source-over";
    }
    if (tinted.size >= LIMIT) tinted.clear();
    tinted.set(key, out);
    return out;
  }

  function draw(
    ctx: CanvasRenderingContext2D,
    text: string,
    x: number,
    y: number,
    scale: number,
    rgb: readonly [number, number, number],
  ): void {
    const atlas = atlasFor(rgb);
    const step = CELL * scale;
    let px = x;
    for (const ch of text) {
      const g = index.get(ch);
      if (g !== undefined && g !== 0) {
        ctx.drawImage(atlas, g * CELL, 0, CELL, CELL, px, y, step, step);
      }
      px += step;
    }
  }

  function width(text: string, scale: number): number {
    return [...text].length * CELL * scale;
  }

  return { draw, width };
}
