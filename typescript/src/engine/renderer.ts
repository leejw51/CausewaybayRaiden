/**
 * The cabinet glass: the game canvas on a full-screen quad, through a CRT pass.
 *
 * Everything the game draws is already rasterised at the virtual resolution by
 * `paint.ts`. Three.js is here for the one thing canvas cannot do cheaply — a
 * shader over the finished frame — and for the nearest-neighbour upscale that
 * keeps the pixel art crisp at any window size.
 *
 * The pass is deliberately gentle: a slight barrel curve, chromatic fringing,
 * scanlines, a vignette, plus the screen shake and the white-out flash the
 * simulation asks for. LÖVE shook the blit by whole pixels; doing it here
 * means the shake can be sub-pixel and does not tear a hole at the edge.
 */
import * as THREE from "three";

const VERT = `
varying vec2 vUv;
void main() {
  vUv = uv;
  gl_Position = vec4(position.xy, 0.0, 1.0);
}
`;

const FRAG = `
uniform sampler2D tGame;
uniform float uTime;
uniform vec2 uRes;
uniform vec2 uScreen;
uniform float uShake;
uniform float uFlash;
varying vec2 vUv;

// Fill the screen with the game texture, cropping the overhang rather than
// letterboxing or stretching. layout() normally sizes the canvas to the
// window's own shape, so this is a no-op; it earns its keep on a window taller
// than 3:4, where the canvas cannot get any narrower than the 192-wide
// playfield and the sides have to be cut instead.
//
// Cover shrinks the sampled range on the axis with the overhang. Expanding it
// past 0..1 is contain, which is the bar-on-two-sides fit this game has
// never wanted.
vec2 coverUv(vec2 uv) {
  float ratio = (uScreen.x / max(1.0, uScreen.y)) / (uRes.x / uRes.y);
  vec2 t = uv;
  // Screen wider than the art: fill the width, crop top and bottom.
  if (ratio > 1.0) t.y = (uv.y - 0.5) / ratio + 0.5;
  // Screen taller: fill the height, crop the sides.
  else t.x = (uv.x - 0.5) * max(0.001, ratio) + 0.5;
  return t;
}

void main() {
  vec2 uv = vUv;
  uv += vec2(uShake * sin(uTime * 71.0), uShake * cos(uTime * 57.0));
  vec2 cc = uv - 0.5;
  float r2 = dot(cc, cc);
  uv = uv + cc * r2 * 0.06;
  vec2 tex = coverUv(uv);

  if (tex.x < 0.0 || tex.x > 1.0 || tex.y < 0.0 || tex.y > 1.0) {
    gl_FragColor = vec4(0.015, 0.012, 0.03, 1.0);
    return;
  }

  float ca = 0.0018 + uFlash * 0.004;
  float r = texture2D(tGame, tex + vec2(ca, 0.0)).r;
  float g = texture2D(tGame, tex).g;
  float b = texture2D(tGame, tex - vec2(ca, 0.0)).b;
  vec3 col = vec3(r, g, b);
  col += col * col * 0.18;
  // Kept light on purpose. LÖVE drew a 7% black line on every other row and
  // no vignette at all; a heavier pass here would darken the neon Causeway
  // Bay plate the whole stage is read against.
  float scan = sin(tex.y * uRes.y * 3.14159) * 0.055;
  col -= scan;
  float vig = smoothstep(1.55, 0.22, r2 * 1.15);
  col *= vig;
  col *= vec3(1.04, 0.98, 1.07);
  col += vec3(0.55, 0.62, 0.80) * uFlash * 0.45;
  gl_FragColor = vec4(col, 1.0);
}
`;

export type Screen = {
  /** Window size in CSS pixels, as last measured. */
  size: () => { w: number; h: number };
  render: (game: HTMLCanvasElement, time: number, shake: number, flash: number) => void;
  onResize: (fn: () => void) => void;
  dispose: () => void;
};

export function createScreen(view: HTMLCanvasElement): Screen {
  const renderer = new THREE.WebGLRenderer({
    canvas: view,
    antialias: false,
    alpha: false,
    powerPreference: "high-performance",
    // Kept so the canvas can be read back: screenshots, and the end-to-end
    // tests that check the game is actually drawing something.
    preserveDrawingBuffer: true,
  });
  renderer.setPixelRatio(1);
  // No colour management on the way out. The shader below writes finished
  // display-space pixels, and three's default sRGB output transform is only
  // correct for a shader that worked in linear light — see the texture setup
  // in `render` for the other half of this.
  renderer.outputColorSpace = THREE.LinearSRGBColorSpace;

  const scene = new THREE.Scene();
  const camera = new THREE.OrthographicCamera(-1, 1, 1, -1, 0, 1);

  let texture: THREE.CanvasTexture | null = null;
  let textureFor: HTMLCanvasElement | null = null;
  let textureW = 0;
  let textureH = 0;

  const uniforms = {
    tGame: { value: null as THREE.Texture | null },
    uTime: { value: 0 },
    uRes: { value: new THREE.Vector2(192, 256) },
    uScreen: { value: new THREE.Vector2(1, 1) },
    uShake: { value: 0 },
    uFlash: { value: 0 },
  };

  const material = new THREE.ShaderMaterial({
    uniforms,
    vertexShader: VERT,
    fragmentShader: FRAG,
  });
  const mesh = new THREE.Mesh(new THREE.PlaneGeometry(2, 2), material);
  scene.add(mesh);

  let w = 1;
  let h = 1;
  const listeners: (() => void)[] = [];

  function measure(): void {
    const vv = window.visualViewport;
    const cssW = Math.max(1, Math.round(vv?.width ?? window.innerWidth));
    const cssH = Math.max(1, Math.round(vv?.height ?? window.innerHeight));
    // Handed to CSS so the layout follows the *visual* viewport on mobile,
    // where the address bar makes `100vh` a lie.
    document.documentElement.style.setProperty("--vvw", `${cssW}px`);
    document.documentElement.style.setProperty("--vvh", `${cssH}px`);
    if (cssW === w && cssH === h) return;
    w = cssW;
    h = cssH;
    renderer.setSize(w, h, false);
    view.style.width = "100%";
    view.style.height = "100%";
    uniforms.uScreen.value.set(w, h);
    for (const fn of listeners) fn();
  }

  measure();
  const onWindowResize = () => measure();
  window.addEventListener("resize", onWindowResize);
  window.addEventListener("orientationchange", onWindowResize);
  window.visualViewport?.addEventListener("resize", onWindowResize);
  window.visualViewport?.addEventListener("scroll", onWindowResize);

  function render(game: HTMLCanvasElement, time: number, shake: number, flash: number): void {
    measure();
    // A widening window changes the game canvas's size but not its identity,
    // and an upload into an already-allocated texture only replaces the part
    // it covers — leaving a strip of the previous, wider frame standing on the
    // right. So the texture is rebuilt whenever the canvas *dimensions* move,
    // not just when the canvas object does.
    if (texture === null || textureFor !== game || textureW !== game.width || textureH !== game.height) {
      texture?.dispose();
      texture = new THREE.CanvasTexture(game);
      texture.magFilter = THREE.NearestFilter;
      texture.minFilter = THREE.NearestFilter;
      texture.generateMipmaps = false;
      // NOT SRGBColorSpace, though the canvas does hold sRGB pixels.
      //
      // Declaring it sRGB makes WebGL2 allocate an SRGB8_ALPHA8 texture, so
      // `texture2D` hardware-decodes to linear light on every read. A material
      // built from three's own shader chunks would encode that back on output
      // and the two would cancel; this is a raw ShaderMaterial, which does no
      // such thing, so the linear value reached the framebuffer as if it were
      // already sRGB and the whole game came out gamma-crushed — mid grey
      // landing near a fifth of its brightness.
      //
      // The canvas is already in display space. Sample it verbatim.
      texture.colorSpace = THREE.NoColorSpace;
      textureFor = game;
      textureW = game.width;
      textureH = game.height;
      uniforms.tGame.value = texture;
    }
    texture.needsUpdate = true;
    uniforms.uRes.value.set(game.width, game.height);
    uniforms.uTime.value = time;
    // Shake arrives in playfield pixels; the shader wants texture units.
    uniforms.uShake.value = Math.min(0.05, shake / Math.max(1, game.width));
    uniforms.uFlash.value = Math.min(1, Math.max(0, flash));
    renderer.render(scene, camera);
  }

  return {
    size: () => ({ w, h }),
    render,
    onResize: (fn) => {
      listeners.push(fn);
    },
    dispose: () => {
      window.removeEventListener("resize", onWindowResize);
      window.removeEventListener("orientationchange", onWindowResize);
      window.visualViewport?.removeEventListener("resize", onWindowResize);
      window.visualViewport?.removeEventListener("scroll", onWindowResize);
      texture?.dispose();
      material.dispose();
      renderer.dispose();
    },
  };
}
