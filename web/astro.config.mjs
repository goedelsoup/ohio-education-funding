// @ts-check
import { defineConfig } from "astro/config";

/**
 * Astro, in static mode, over Vite.
 *
 * The output is a directory of plain files with no server behind it — the same deployment
 * property the hand-rolled `tsc` build had, which is the one static hosting wants. What Vite
 * adds is a module graph: `src/scripts/main.ts` is bundled, hashed, and preloaded rather than
 * shipped as nine separate network requests.
 *
 * The feed is *not* part of that graph. `public/data/bundle.json` is copied verbatim, so
 * regenerating it stays a `cargo run` redirect and does not require rebuilding the site.
 */
export default defineConfig({
  output: "static",
  // Emitted as `dist/index.html` rather than `dist/index/index.html`. The page routes on the
  // URL fragment, so there is only ever one document.
  build: { format: "file" },
  vite: {
    build: {
      // The formula lives in this bundle and gets read by people checking the arithmetic.
      sourcemap: true,
    },
  },
});
