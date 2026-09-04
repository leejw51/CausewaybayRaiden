/**
 * The shell around the wasm game.
 *
 * Its whole job is the four things a browser will not delegate: a canvas, a
 * keyboard, Web Audio, and local storage. Every decision about what happens
 * on screen is made in `crates/raiden-core`.
 *
 * One frame:
 *
 *   1. measure the window and tell the game how much of the world is visible;
 *   2. hand it this frame's input word and let it simulate and lay out;
 *   3. read its draw list straight out of wasm memory and paint it;
 *   4. put that canvas through the CRT pass and play whatever it asked for.
 */
import init, { Game, font_bitmap, font_charset, sprite_names } from "./wasm/raiden_core.js";

import { createChip } from "./audio/chip";
import { checkOrder, loadArt } from "./engine/assets";
import { createFont } from "./engine/font";
import { createInput } from "./engine/input";
import { GH, layout } from "./engine/layout";
import { createPainter } from "./engine/paint";
import { createScreen } from "./engine/renderer";
import { loadProgress, saveProgress } from "./game/store";

/** A frame longer than this is a tab that was in the background, not a lag spike. */
const MAX_STEP = 0.05;

function isTouchUi(): boolean {
  if (window.matchMedia?.("(any-pointer: coarse)").matches) return true;
  if ((navigator.maxTouchPoints ?? 0) > 0) return true;
  return "ontouchstart" in window && window.matchMedia?.("(hover: none)").matches === true;
}

/** Something went wrong before there was a game to show it in. */
function fail(view: HTMLCanvasElement, message: string): void {
  const ctx = view.getContext("2d");
  if (!ctx) return;
  view.width = 384;
  view.height = 512;
  ctx.fillStyle = "#07060e";
  ctx.fillRect(0, 0, view.width, view.height);
  ctx.fillStyle = "#dc6645";
  ctx.font = "16px ui-monospace, monospace";
  ctx.textAlign = "center";
  ctx.fillText("CAUSEWAYBAY RAIDEN", view.width / 2, view.height / 2 - 12);
  ctx.fillStyle = "#66dbf0";
  ctx.font = "12px ui-monospace, monospace";
  ctx.fillText(message, view.width / 2, view.height / 2 + 12);
}

async function boot(): Promise<void> {
  const view = document.querySelector<HTMLCanvasElement>("#view");
  if (!view) throw new Error("missing #view");
  const touch = document.querySelector<HTMLElement>("#touch");
  if (isTouchUi()) document.documentElement.classList.add("touch");

  let wasm: Awaited<ReturnType<typeof init>>;
  try {
    wasm = await init();
  } catch (err) {
    fail(view, `wasm did not load: ${String(err)}`);
    return;
  }

  const art = await loadArt().catch((err) => {
    fail(view, `art did not load: ${String(err)}`);
    return null;
  });
  if (!art) return;

  try {
    checkOrder(art, sprite_names().split("\n"));
  } catch (err) {
    fail(view, String(err));
    return;
  }

  const font = createFont(font_charset(), font_bitmap());
  const painter = createPainter(art, font);
  const screen = createScreen(view);
  const input = createInput();
  if (touch) input.bindTouch(touch);
  const chip = createChip();

  const game = new Game(Date.now() % 2 ** 32);
  const progress = loadProgress();
  game.set_progress(progress.hiscore, progress.cursor, progress.cleared);
  let savedVersion = game.progress_version();

  // Audio cannot start before a gesture, and any gesture will do.
  const wake = () => chip.ensure();
  for (const ev of ["pointerdown", "keydown", "touchstart"] as const) {
    window.addEventListener(ev, wake, { passive: true });
  }

  const toggleFullscreen = (): void => {
    const stage = view.parentElement ?? view;
    if (!document.fullscreenElement) void stage.requestFullscreen?.().catch(() => {});
    else void document.exitFullscreen().catch(() => {});
  };
  view.addEventListener("dblclick", toggleFullscreen);
  view.addEventListener("touchstart", (ev) => ev.preventDefault(), { passive: false });
  view.addEventListener("touchmove", (ev) => ev.preventDefault(), { passive: false });
  view.addEventListener("contextmenu", (ev) => ev.preventDefault());

  // The screen the game is on, published on the document so a test (or a
  // person with the dev tools open) can see it without reading pixels back
  // through the CRT pass. Written only when it changes.
  const SCREENS = [
    "boot", "title", "story", "map", "rank",
    "play", "pause", "continue", "gameover", "ending",
  ];
  let shownState = -1;

  let last = performance.now();
  const frame = (now: number): void => {
    const dt = Math.min(MAX_STEP, Math.max(0, (now - last) / 1000));
    last = now;

    const { w, h } = screen.size();
    const view3 = layout(w, h);
    painter.resize(view3.vw);
    game.set_view(view3.viewLeft, view3.viewRight);

    const shell = input.takeShellKeys();
    if (shell.full) toggleFullscreen();
    if (shell.mute) chip.toggleMute();

    game.update(dt, input.read());

    // Views over wasm memory, rebuilt each frame: the heap can move under us
    // when the game allocates, which would leave a stale view pointing at
    // whatever is there now.
    const len = game.commands_len();
    const cmds = new Float32Array(wasm.memory.buffer, game.commands_ptr(), len);
    // The font is uppercase-only, as it was on the cabinet.
    const pool = game.text_pool().toUpperCase().split("\n");
    painter.paint(cmds, pool, view3.px);

    const cues = new Uint8Array(wasm.memory.buffer, game.audio_ptr(), game.audio_len());
    if (cues.length > 0) chip.play(cues.slice());

    screen.render(painter.canvas, now / 1000, game.shake(), game.flash());

    const state = game.state();
    if (state !== shownState) {
      shownState = state;
      document.documentElement.dataset.screen = SCREENS[state] ?? String(state);
    }

    const version = game.progress_version();
    if (version !== savedVersion) {
      savedVersion = version;
      saveProgress({
        hiscore: game.hiscore(),
        cursor: game.map_cursor(),
        cleared: game.cleared(),
      });
    }

    requestAnimationFrame(frame);
  };

  // Tell the world the cabinet is up: the end-to-end tests wait on this
  // rather than on a pixel, which cannot be read back through the CRT pass.
  document.documentElement.dataset.raiden = "ready";
  document.documentElement.style.setProperty("--gh", `${GH}px`);
  requestAnimationFrame(frame);
}

void boot();
