/**
 * Progress, kept in the browser.
 *
 * The desktop game appends a line of JSON per event to `~/.causewayraiden`.
 * There is no home directory here, and a replayable log buys nothing a single
 * record does not, so this keeps the same three facts — best score, which
 * stages are cleared, where the walker stands — under one key.
 *
 * Every path through here tolerates storage being unavailable or holding
 * nonsense: a private window, a cleared site, a hand-edited value. Losing a
 * high score is a disappointment; refusing to boot over one is a bug.
 */
export const KEY = "causewaybay-raiden.progress";

export type Progress = {
  hiscore: number;
  /** Which stage the map cursor sits on, 0-based. */
  cursor: number;
  /** One bit per stage, set once cleared. */
  cleared: number;
};

export const DEFAULT_PROGRESS: Progress = { hiscore: 50_000, cursor: 0, cleared: 0 };

type Store = Pick<Storage, "getItem" | "setItem">;

function storage(given?: Store): Store | null {
  if (given) return given;
  try {
    return globalThis.localStorage ?? null;
  } catch {
    // Blocked site data throws on access, not on use.
    return null;
  }
}

function whole(v: unknown, fallback: number, lo: number, hi: number): number {
  const n = typeof v === "number" ? v : Number.NaN;
  if (!Number.isFinite(n)) return fallback;
  return Math.min(hi, Math.max(lo, Math.floor(n)));
}

export function loadProgress(given?: Store): Progress {
  const s = storage(given);
  if (!s) return { ...DEFAULT_PROGRESS };
  try {
    const raw = s.getItem(KEY);
    if (!raw) return { ...DEFAULT_PROGRESS };
    const v = JSON.parse(raw) as Partial<Progress>;
    return {
      hiscore: whole(v.hiscore, DEFAULT_PROGRESS.hiscore, 0, Number.MAX_SAFE_INTEGER),
      cursor: whole(v.cursor, 0, 0, 2),
      cleared: whole(v.cleared, 0, 0, 0b111),
    };
  } catch {
    return { ...DEFAULT_PROGRESS };
  }
}

export function saveProgress(p: Progress, given?: Store): boolean {
  const s = storage(given);
  if (!s) return false;
  try {
    s.setItem(KEY, JSON.stringify(p));
    return true;
  } catch {
    // Full, or refused. The game carries on with what it has in memory.
    return false;
  }
}
