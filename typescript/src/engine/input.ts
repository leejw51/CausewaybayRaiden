/**
 * Keyboard and touch, folded into the single word the wasm module reads.
 *
 * Low bits are what is held down this frame; high bits are what was *pressed*
 * this frame. Menus want the edge (one press, one step) and flying wants the
 * level (hold to move, hold to fire), so both travel together and the game
 * picks whichever it needs.
 *
 * Fullscreen and mute never reach the simulation — they are the shell's.
 */
export const B_LEFT = 1 << 0;
export const B_RIGHT = 1 << 1;
export const B_UP = 1 << 2;
export const B_DOWN = 1 << 3;
export const B_SHOOT = 1 << 4;
export const H_LEFT = 1 << 8;
export const H_RIGHT = 1 << 9;
export const H_UP = 1 << 10;
export const H_DOWN = 1 << 11;
export const H_CONFIRM = 1 << 12;
export const H_BACK = 1 << 13;
export const H_BOMB = 1 << 14;
export const H_COIN = 1 << 15;

/** Logical buttons, as the key map and the touch pad both name them. */
export type Button =
  | "left"
  | "right"
  | "up"
  | "down"
  | "shoot"
  | "bomb"
  | "confirm"
  | "back"
  | "coin"
  | "full"
  | "mute";

const KEYS: Record<string, Button> = {
  arrowleft: "left",
  arrowright: "right",
  arrowup: "up",
  arrowdown: "down",
  a: "left",
  d: "right",
  w: "up",
  s: "down",
  // Shot doubles as the menu confirm, exactly as it does on the cabinet.
  z: "shoot",
  j: "shoot",
  " ": "shoot",
  enter: "confirm",
  x: "bomb",
  k: "bomb",
  shift: "bomb",
  escape: "back",
  p: "back",
  c: "coin",
  f: "full",
  f11: "full",
  m: "mute",
};

/** Buttons that also count as "confirm" when pressed. */
const CONFIRMS = new Set<Button>(["shoot", "confirm"]);

const HELD: Partial<Record<Button, number>> = {
  left: B_LEFT,
  right: B_RIGHT,
  up: B_UP,
  down: B_DOWN,
  shoot: B_SHOOT,
};

const HIT: Partial<Record<Button, number>> = {
  left: H_LEFT,
  right: H_RIGHT,
  up: H_UP,
  down: H_DOWN,
  bomb: H_BOMB,
  back: H_BACK,
  coin: H_COIN,
};

export type Input = {
  /** Consume the frame's bits. Edges are cleared by reading. */
  read: () => number;
  /** Fullscreen and mute presses since the last call. */
  takeShellKeys: () => { full: boolean; mute: boolean };
  bindTouch: (root: HTMLElement) => void;
  dispose: () => void;
};

export function createInput(target: Window = window): Input {
  const down = new Set<Button>();
  const hit = new Set<Button>();
  const fingers = new Map<number, Button>();
  let fullHit = false;
  let muteHit = false;

  function press(b: Button, on: boolean): void {
    if (!on) {
      down.delete(b);
      return;
    }
    if (!down.has(b)) {
      hit.add(b);
      if (b === "full") fullHit = true;
      if (b === "mute") muteHit = true;
    }
    down.add(b);
  }

  function lookup(ev: KeyboardEvent): Button | undefined {
    return KEYS[ev.key.toLowerCase()];
  }

  const onKeyDown = (ev: KeyboardEvent) => {
    // Leave the browser's own chords alone: Cmd+R must still reload.
    if (ev.metaKey || ev.ctrlKey || ev.altKey) return;
    const b = lookup(ev);
    if (!b) return;
    ev.preventDefault();
    if (!ev.repeat) press(b, true);
  };

  const onKeyUp = (ev: KeyboardEvent) => {
    const b = lookup(ev);
    if (!b) return;
    ev.preventDefault();
    press(b, false);
  };

  // A window that loses focus mid-press would otherwise hold that key forever.
  const onBlur = () => down.clear();

  target.addEventListener("keydown", onKeyDown as EventListener);
  target.addEventListener("keyup", onKeyUp as EventListener);
  target.addEventListener("blur", onBlur);

  function bindTouch(root: HTMLElement): void {
    const keyOf = (el: EventTarget | null): Button | undefined => {
      if (!(el instanceof Element)) return undefined;
      const name = el.closest("button[data-key]")?.getAttribute("data-key");
      return (name as Button) ?? undefined;
    };

    const start = (ev: TouchEvent) => {
      ev.preventDefault();
      for (const t of Array.from(ev.changedTouches)) {
        const b = keyOf(t.target) ?? keyOf(ev.target);
        if (!b || fingers.has(t.identifier)) continue;
        fingers.set(t.identifier, b);
        press(b, true);
      }
    };
    const end = (ev: TouchEvent) => {
      ev.preventDefault();
      for (const t of Array.from(ev.changedTouches)) {
        const b = fingers.get(t.identifier);
        if (!b) continue;
        fingers.delete(t.identifier);
        // Two fingers can hold the same button; release on the last one.
        if (![...fingers.values()].includes(b)) press(b, false);
      }
    };

    root.addEventListener("touchstart", start, { passive: false });
    root.addEventListener("touchend", end, { passive: false });
    root.addEventListener("touchcancel", end, { passive: false });
    root.addEventListener("touchmove", (ev) => ev.preventDefault(), { passive: false });

    // Mouse and pen, for a desktop browser emulating a phone.
    root.querySelectorAll<HTMLButtonElement>("button[data-key]").forEach((btn) => {
      const b = btn.dataset.key as Button | undefined;
      if (!b) return;
      const on = (ev: PointerEvent) => {
        if (ev.pointerType === "touch") return;
        ev.preventDefault();
        press(b, true);
      };
      const off = (ev: PointerEvent) => {
        if (ev.pointerType === "touch") return;
        ev.preventDefault();
        press(b, false);
      };
      btn.addEventListener("pointerdown", on);
      btn.addEventListener("pointerup", off);
      btn.addEventListener("pointerleave", off);
      btn.addEventListener("pointercancel", off);
    });
  }

  function read(): number {
    let bits = 0;
    for (const b of down) bits |= HELD[b] ?? 0;
    for (const b of hit) {
      bits |= HIT[b] ?? 0;
      if (CONFIRMS.has(b)) bits |= H_CONFIRM;
    }
    hit.clear();
    return bits;
  }

  return {
    read,
    takeShellKeys: () => {
      const out = { full: fullHit, mute: muteHit };
      fullHit = false;
      muteHit = false;
      return out;
    },
    bindTouch,
    dispose: () => {
      target.removeEventListener("keydown", onKeyDown as EventListener);
      target.removeEventListener("keyup", onKeyUp as EventListener);
      target.removeEventListener("blur", onBlur);
    },
  };
}
