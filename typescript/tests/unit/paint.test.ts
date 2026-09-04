import { describe, expect, it } from "vitest";

import { CMD_LEN, TAG, walk } from "../../src/engine/paint";

/**
 * The draw list is the contract between the Rust game and the browser. Both
 * ends carry the same table of command lengths; if they ever disagree the
 * renderer reads a colour as a coordinate, so the walk is checked here and
 * again from the Rust side in `crates/raiden-core/tests/play.rs`.
 */
function cmd(tag: number, ...rest: number[]): number[] {
  const body = [tag, ...rest];
  const want = CMD_LEN[tag];
  // Pad out to the declared length with an opaque colour.
  while (body.length < want) body.push(1);
  return body.slice(0, want);
}

describe("draw list", () => {
  it("knows the length of every tag it defines", () => {
    for (const tag of Object.values(TAG)) {
      expect(CMD_LEN[tag]).toBeGreaterThan(0);
    }
    expect(CMD_LEN).toHaveLength(Object.values(TAG).length + 1);
  });

  it("visits each command once, in order", () => {
    const buf = Float32Array.from([
      ...cmd(TAG.tiled, 3, -12),
      ...cmd(TAG.rect, 0, 0, 192, 256),
      ...cmd(TAG.sprite, 0, 96, 214, 0, 1, 1, 20, 20),
      ...cmd(TAG.text, 0, 4, 4, 1),
    ]);
    const seen: number[] = [];
    expect(walk(buf, (tag) => seen.push(tag))).toBe(true);
    expect(seen).toEqual([TAG.tiled, TAG.rect, TAG.sprite, TAG.text]);
  });

  it("reports offsets a reader can index from", () => {
    const buf = Float32Array.from([...cmd(TAG.rect, 5, 6, 7, 8), ...cmd(TAG.circle, 1, 2, 3)]);
    const at: number[] = [];
    walk(buf, (_tag, i, len) => at.push(i, len));
    expect(at).toEqual([0, CMD_LEN[TAG.rect], CMD_LEN[TAG.rect], CMD_LEN[TAG.circle]]);
  });

  it("stops on a truncated command rather than reading past it", () => {
    const buf = Float32Array.from([...cmd(TAG.rect, 1, 2, 3, 4), TAG.sprite, 0, 1]);
    const seen: number[] = [];
    expect(walk(buf, (tag) => seen.push(tag))).toBe(false);
    expect(seen).toEqual([TAG.rect]);
  });

  it("stops on a tag it does not recognise", () => {
    const buf = Float32Array.from([...cmd(TAG.rect, 1, 2, 3, 4), 99, 0, 0, 0]);
    const seen: number[] = [];
    expect(walk(buf, (tag) => seen.push(tag))).toBe(false);
    expect(seen).toEqual([TAG.rect]);
  });

  it("accepts an empty frame", () => {
    const seen: number[] = [];
    expect(walk(new Float32Array(0), (tag) => seen.push(tag))).toBe(true);
    expect(seen).toEqual([]);
  });
});
