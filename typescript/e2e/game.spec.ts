import { expect, test, type Page } from "@playwright/test";

/**
 * The parts only a real browser can answer for: that the wasm module loads,
 * that WebGL draws something, that the keyboard walks the cabinet from the
 * boot logo to a running game, and that a run is still there after a reload.
 *
 * The screen the game is on is published as `data-screen` on the document —
 * pixels cannot be read back through a shader pass reliably enough to assert
 * on, and asserting on a screenshot would fail on every unrelated art change.
 */
const READY = '[data-raiden="ready"]';

async function boot(page: Page): Promise<void> {
  await page.goto("/");
  await page.waitForSelector(READY, { timeout: 30_000 });
  await expect(page.locator("html")).toHaveAttribute("data-screen", /boot|title/);
}

/** One key press, given a frame either side to be noticed. */
async function tap(page: Page, key: string): Promise<void> {
  await page.keyboard.press(key);
  await page.waitForTimeout(120);
}

async function screen(page: Page): Promise<string | null> {
  return page.locator("html").getAttribute("data-screen");
}

test("boots into the attract screen", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (err) => errors.push(String(err)));
  page.on("console", (msg) => {
    if (msg.type() === "error") errors.push(msg.text());
  });

  await boot(page);
  await tap(page, "Escape");
  await expect(page.locator("html")).toHaveAttribute("data-screen", "title");
  expect(errors).toEqual([]);
});

test("draws something other than a blank screen", async ({ page }) => {
  await boot(page);
  await tap(page, "Escape");
  await page.waitForTimeout(500);

  // Sample the WebGL canvas: a frame that drew nothing would be one flat
  // colour, and the title screen is anything but.
  const spread = await page.evaluate(() => {
    const view = document.querySelector("canvas") as HTMLCanvasElement;
    const copy = document.createElement("canvas");
    copy.width = 64;
    copy.height = 64;
    const ctx = copy.getContext("2d");
    if (!ctx) return -1;
    ctx.drawImage(view, 0, 0, 64, 64);
    const { data } = ctx.getImageData(0, 0, 64, 64);
    const seen = new Set<number>();
    for (let i = 0; i < data.length; i += 4) {
      seen.add((data[i] >> 4) * 256 + (data[i + 1] >> 4) * 16 + (data[i + 2] >> 4));
    }
    return seen.size;
  });
  expect(spread).toBeGreaterThan(8);
});

test("the frame is not gamma-crushed", async ({ page }) => {
  await boot(page);
  await tap(page, "Escape");
  await page.waitForTimeout(700);

  // A guard against one specific, silent bug: declaring the game canvas an
  // sRGB texture makes WebGL hardware-decode it to linear light on every
  // read, and this renderer's shader writes its result straight to the
  // framebuffer without encoding back. Nothing errors. The whole game just
  // arrives about 2.3x too dark, mid grey landing near a fifth of its
  // brightness, and only a side-by-side against the LÖVE build shows it.
  //
  // Mean luminance over the title screen measured 15.9 crushed and 37.3
  // correct, so the floor sits between them with room for art changes either
  // side. The ceiling only catches the mirror-image mistake, encoding twice.
  const luma = await page.evaluate(() => {
    const view = document.querySelector("canvas") as HTMLCanvasElement;
    const copy = document.createElement("canvas");
    copy.width = 160;
    copy.height = 200;
    const ctx = copy.getContext("2d");
    if (!ctx) return -1;
    ctx.drawImage(view, 0, 0, copy.width, copy.height);
    const { data } = ctx.getImageData(0, 0, copy.width, copy.height);
    let sum = 0;
    for (let i = 0; i < data.length; i += 4) {
      sum += 0.2126 * data[i] + 0.7152 * data[i + 1] + 0.0722 * data[i + 2];
    }
    return sum / (data.length / 4);
  });
  expect(luma).toBeGreaterThan(26);
  expect(luma).toBeLessThan(120);
});

test("the keyboard walks from the title into a game", async ({ page }) => {
  await boot(page);
  await tap(page, "Escape");
  expect(await screen(page)).toBe("title");
  await tap(page, "Enter");
  expect(await screen(page)).toBe("story");
  await tap(page, "Enter");
  expect(await screen(page)).toBe("map");
  await tap(page, "Enter");
  expect(await screen(page)).toBe("rank");
  await tap(page, "Enter");
  expect(await screen(page)).toBe("play");

  // Pause and resume, so the in-game keys are wired too.
  await tap(page, "Escape");
  expect(await screen(page)).toBe("pause");
  await tap(page, "Escape");
  expect(await screen(page)).toBe("play");
});

test("where you left off survives a reload", async ({ page }) => {
  await boot(page);
  await page.evaluate(() => localStorage.clear());
  await page.reload();
  await page.waitForSelector(READY, { timeout: 30_000 });

  for (const key of ["Escape", "Enter", "Enter"]) await tap(page, key);
  expect(await screen(page)).toBe("map");
  await tap(page, "ArrowRight");

  const stored = await page.evaluate(() =>
    localStorage.getItem("causewaybay-raiden.progress"),
  );
  expect(stored).toBeTruthy();
  expect(JSON.parse(stored as string).cursor).toBe(1);

  await page.reload();
  await page.waitForSelector(READY, { timeout: 30_000 });
  for (const key of ["Escape", "Enter", "Enter"]) await tap(page, key);
  expect(await screen(page)).toBe("map");
  // The walker is back on the node it was left on, so the next game is 1-2.
  await tap(page, "Enter");
  await tap(page, "Enter");
  expect(await screen(page)).toBe("play");
  const stage = await page.evaluate(
    () => JSON.parse(localStorage.getItem("causewaybay-raiden.progress") as string).cursor,
  );
  expect(stage).toBe(1);
});

test("a run keeps score", async ({ page }) => {
  await boot(page);
  for (const key of ["Escape", "Enter", "Enter", "Enter", "Enter"]) await tap(page, key);
  expect(await screen(page)).toBe("play");

  // Hold the shot: the opening wave flies straight into it.
  await page.keyboard.down("z");
  await page.waitForTimeout(6000);
  await page.keyboard.up("z");
  expect(await screen(page)).toBe("play");
});

test("fills a window that is not 3:4, without letterboxing", async ({ page }) => {
  // Far taller than the 3:4 playfield. The canvas cannot get narrower than
  // 192 columns, so the shader has to crop the sides to fill the window —
  // getting that fit backwards shows the whole frame shrunk between two black
  // bars, which is precisely the "no bars, no letterbox" rule the LÖVE build
  // is built around.
  await page.setViewportSize({ width: 600, height: 1200 });
  await boot(page);
  await tap(page, "Escape");
  await page.waitForTimeout(700);

  const margins = await page.evaluate(() => {
    const view = document.querySelector("canvas") as HTMLCanvasElement;
    const copy = document.createElement("canvas");
    copy.width = view.width;
    copy.height = view.height;
    const ctx = copy.getContext("2d");
    if (!ctx) return null;
    ctx.drawImage(view, 0, 0);
    const rowMean = (y: number): number => {
      const { data } = ctx.getImageData(0, y, copy.width, 1);
      let sum = 0;
      for (let i = 0; i < data.length; i += 4) sum += data[i] + data[i + 1] + data[i + 2];
      return sum / (data.length / 4) / 3;
    };
    let top = 0;
    while (top < copy.height && rowMean(top) < 12) top += 1;
    let bottom = copy.height - 1;
    while (bottom > 0 && rowMean(bottom) < 12) bottom -= 1;
    return {
      top: top / copy.height,
      bottom: (copy.height - 1 - bottom) / copy.height,
    };
  });

  // Letterboxing here would leave a bar of about an eighth of the height at
  // each end. What is left is the barrel curve rolling off at the very edge,
  // which measured around 1%.
  expect(margins).not.toBeNull();
  expect(margins!.top).toBeLessThan(0.05);
  expect(margins!.bottom).toBeLessThan(0.05);
});

test("survives being resized mid-game", async ({ page }) => {
  await boot(page);
  for (const key of ["Escape", "Enter", "Enter", "Enter", "Enter"]) await tap(page, key);
  expect(await screen(page)).toBe("play");

  for (const size of [
    { width: 1400, height: 700 },
    { width: 500, height: 900 },
    { width: 900, height: 900 },
  ]) {
    await page.setViewportSize(size);
    await page.waitForTimeout(300);
    // The drawing buffer must follow the window, or the frame ends up
    // stretched or carrying a strip of the previous size.
    const fits = await page.evaluate(() => {
      const view = document.querySelector("canvas") as HTMLCanvasElement;
      const r = view.getBoundingClientRect();
      return Math.abs(view.width - r.width) < 2 && Math.abs(view.height - r.height) < 2;
    });
    expect(fits, `buffer did not follow ${size.width}x${size.height}`).toBe(true);
    expect(await screen(page)).toBe("play");
  }
});
