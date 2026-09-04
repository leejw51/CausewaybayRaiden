import { describe, expect, it } from "vitest";

import { GH, GW, layout } from "../../src/engine/layout";

/**
 * The display rule the whole game is built on: a fixed virtual height, a
 * uniform scale, and a width that follows the window. Getting this wrong does
 * not just look bad — `viewLeft`/`viewRight` decide how far the ship may fly.
 */
describe("layout", () => {
  it("shows exactly the playfield in a 3:4 window", () => {
    const l = layout(GW * 3, GH * 3);
    expect(l.scale).toBe(3);
    expect(l.vw).toBe(GW);
    expect(l.px).toBe(0);
    expect(l.viewLeft).toBe(0);
    expect(l.viewRight).toBe(GW);
  });

  it("scales by height alone, never by width", () => {
    for (const w of [400, 800, 1600, 3000]) {
      expect(layout(w, 512).scale).toBe(2);
    }
  });

  it("shows more world on either side of a wide window", () => {
    const l = layout(1600, 768); // scale 3, so ~533 virtual pixels across
    expect(l.vw).toBeGreaterThan(GW);
    expect(l.viewLeft).toBeLessThan(0);
    expect(l.viewRight).toBeGreaterThan(GW);
    // Symmetric to within the rounding of a single pixel.
    expect(Math.abs(-l.viewLeft - (l.viewRight - GW))).toBeLessThanOrEqual(1);
  });

  it("crops the sides of a window taller than 3:4 rather than letterboxing", () => {
    const l = layout(300, 900); // much taller than 3:4
    expect(l.vw).toBe(GW);
    expect(l.viewLeft).toBeGreaterThan(0);
    expect(l.viewRight).toBeLessThan(GW);
  });

  it("keeps the visible span inside the canvas", () => {
    for (const [w, h] of [
      [320, 480],
      [1920, 1080],
      [1024, 768],
      [390, 844],
      [2560, 1440],
      [800, 2000],
    ]) {
      const l = layout(w, h);
      expect(l.viewRight).toBeGreaterThan(l.viewLeft);
      expect(l.viewLeft).toBeGreaterThanOrEqual(-l.px);
      expect(l.viewRight).toBeLessThanOrEqual(l.vw - l.px);
    }
  });

  it("the visible span covers the window it was measured for", () => {
    for (const [w, h] of [
      [1920, 1080],
      [1440, 900],
      [1024, 768],
    ]) {
      const l = layout(w, h);
      // The span is the range that is *fully* on screen, rounded inward at
      // both ends, so it can fall short by up to a virtual pixel per side.
      const covered = (l.viewRight - l.viewLeft) * l.scale;
      expect(covered).toBeLessThanOrEqual(w + 1);
      expect(covered).toBeGreaterThanOrEqual(w - 2 * l.scale - 2);
    }
  });

  it("survives a degenerate window", () => {
    for (const [w, h] of [
      [0, 0],
      [-5, -5],
      [1, 1],
      [Number.NaN, 600],
    ]) {
      const l = layout(w, h);
      expect(Number.isFinite(l.scale)).toBe(true);
      expect(l.scale).toBeGreaterThan(0);
      expect(l.vw).toBeGreaterThanOrEqual(GW);
      expect(l.viewRight).toBeGreaterThan(l.viewLeft);
    }
  });
});
