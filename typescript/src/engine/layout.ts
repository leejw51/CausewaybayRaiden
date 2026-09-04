/**
 * Where the 192x256 playfield sits inside the window.
 *
 * The rule, unchanged from `love2d/src/display.lua`: one fixed virtual
 * *height* and a uniform scale, so pixels are never stretched. The virtual
 * *width* then follows the window's shape, and the canvas covers the whole
 * window with no bars and no letterbox.
 *
 * A window wider than 3:4 shows more of the same world on either side of the
 * playfield, and the ship may fly out there. A window taller than 3:4 crops
 * the playfield's sides instead. `viewLeft` and `viewRight` say, in playfield
 * coordinates, what is actually on screen either way.
 *
 * Pure arithmetic on purpose: this is the one piece of presentation logic that
 * changes what the game lets you do, so it is worth being able to test.
 */
export type Layout = {
  /** Window pixels per playfield pixel. */
  scale: number;
  /** Width of the virtual canvas, in playfield pixels. Never below 192. */
  vw: number;
  /** Where the playfield's left edge sits inside that canvas. */
  px: number;
  /** Leftmost visible playfield x. Negative on a wide window. */
  viewLeft: number;
  /** Rightmost visible playfield x. Past 192 on a wide window. */
  viewRight: number;
};

export const GW = 192;
export const GH = 256;

/** A window measurement that can be divided by: never zero, never NaN. */
function side(v: number): number {
  return Number.isFinite(v) ? Math.max(1, Math.floor(v)) : 1;
}

export function layout(winW: number, winH: number): Layout {
  const w = side(winW);
  const h = side(winH);
  const scale = Math.max(1, h / GH);
  const vw = Math.max(GW, Math.ceil(w / scale));
  const px = Math.floor((vw - GW) / 2);
  // Zero when the canvas fits, negative when a tall window crops the sides.
  const cx = Math.floor((w - vw * scale) / 2);
  const ox = cx + px * scale;
  let viewLeft = Math.max(-px, Math.ceil(-ox / scale));
  let viewRight = Math.min(vw - px, Math.floor((w - ox) / scale));
  // A window barely wider than nothing would leave nothing to centre text in
  // and nowhere for the ship to fly. Keep a sliver either way.
  if (viewRight - viewLeft < 8) {
    const mid = (viewLeft + viewRight) / 2;
    viewLeft = Math.max(-px, mid - 4);
    viewRight = Math.min(vw - px, viewLeft + 8);
  }
  // `+ 0` turns a negative zero back into a plain one; it reads as -0 in a
  // failure message and compares unequal under Object.is.
  return { scale, vw, px, viewLeft: viewLeft + 0, viewRight: viewRight + 0 };
}
