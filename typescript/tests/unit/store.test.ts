import { beforeEach, describe, expect, it } from "vitest";

import { DEFAULT_PROGRESS, KEY, loadProgress, saveProgress } from "../../src/game/store";

function fakeStorage(seed: Record<string, string> = {}) {
  const map = new Map(Object.entries(seed));
  return {
    getItem: (k: string) => map.get(k) ?? null,
    setItem: (k: string, v: string) => {
      map.set(k, v);
    },
    map,
  };
}

describe("progress store", () => {
  let store: ReturnType<typeof fakeStorage>;

  beforeEach(() => {
    store = fakeStorage();
  });

  it("starts at the factory high score", () => {
    expect(loadProgress(store)).toEqual(DEFAULT_PROGRESS);
  });

  it("round-trips a run", () => {
    const p = { hiscore: 123_456, cursor: 2, cleared: 0b011 };
    expect(saveProgress(p, store)).toBe(true);
    expect(loadProgress(store)).toEqual(p);
  });

  it("ignores a hand-edited or corrupt record", () => {
    for (const raw of ["", "{", "null", '"nope"', "[1,2,3]"]) {
      const s = fakeStorage({ [KEY]: raw });
      expect(loadProgress(s)).toEqual(DEFAULT_PROGRESS);
    }
  });

  it("clamps values that would reach past the map", () => {
    const s = fakeStorage({
      [KEY]: JSON.stringify({ hiscore: -10, cursor: 99, cleared: 0xffff }),
    });
    const p = loadProgress(s);
    expect(p.hiscore).toBe(0);
    expect(p.cursor).toBe(2);
    expect(p.cleared).toBe(0b111);
  });

  it("drops non-numbers rather than storing NaN", () => {
    const s = fakeStorage({
      [KEY]: JSON.stringify({ hiscore: "lots", cursor: null, cleared: undefined }),
    });
    expect(loadProgress(s)).toEqual(DEFAULT_PROGRESS);
  });

  it("carries on when storage refuses to write", () => {
    const broken = {
      getItem: () => null,
      setItem: () => {
        throw new Error("QuotaExceededError");
      },
    };
    expect(saveProgress(DEFAULT_PROGRESS, broken)).toBe(false);
    expect(loadProgress(broken)).toEqual(DEFAULT_PROGRESS);
  });
});
