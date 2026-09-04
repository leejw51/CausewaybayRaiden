import { defineConfig } from "vite";

// `base: "./"` so the built bundle works from any path a Cloudflare project
// happens to be served under, including a preview deployment.
export default defineConfig({
  base: "./",
  server: { host: true, port: 5290 },
  build: {
    outDir: "dist",
    assetsDir: "assets",
    sourcemap: false,
    // The wasm module arrives as its own file rather than being inlined; it is
    // far too big for a data URL and caches better on its own.
    assetsInlineLimit: 4096,
  },
});
