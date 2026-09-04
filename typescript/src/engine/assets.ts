/**
 * The art, loaded in the order the wasm draw list refers to it.
 *
 * `mkassets` writes `manifest.json` from the same table the Rust `Sprite` enum
 * is built from, so the order here cannot drift from the order over there —
 * and `checkOrder` proves it at boot rather than letting a mismatch show up as
 * the wrong bug on screen.
 */
export type Art = {
  images: (HTMLImageElement | null)[];
  names: string[];
  /** Art that is repeated to fill the screen: backgrounds and parallax. */
  tiled: Set<string>;
};

type Manifest = { sprites: string[]; tiled: string[] };

function loadImage(src: string): Promise<HTMLImageElement | null> {
  return new Promise((resolve) => {
    const img = new Image();
    // A single missing file should cost one sprite, not the whole game.
    img.onload = () => resolve(img);
    img.onerror = () => resolve(null);
    img.src = src;
  });
}

export async function loadArt(base = "./art/"): Promise<Art> {
  const res = await fetch(`${base}manifest.json`);
  if (!res.ok) throw new Error(`art manifest: ${res.status}`);
  const manifest = (await res.json()) as Manifest;
  const images = await Promise.all(manifest.sprites.map((n) => loadImage(`${base}${n}.png`)));
  return { images, names: manifest.sprites, tiled: new Set(manifest.tiled) };
}

/**
 * Fail loudly if the manifest and the wasm module disagree about which index
 * means which picture.
 */
export function checkOrder(art: Art, wasmNames: string[]): void {
  if (art.names.length !== wasmNames.length) {
    throw new Error(`art manifest has ${art.names.length} entries, wasm expects ${wasmNames.length}`);
  }
  for (let i = 0; i < wasmNames.length; i += 1) {
    if (art.names[i] !== wasmNames[i]) {
      throw new Error(`art index ${i} is "${art.names[i]}" but wasm calls it "${wasmNames[i]}"`);
    }
  }
}
