/**
 * The typeface the preview cards are set in.
 *
 * # Why a font file exists at all, when the site has none
 *
 * `app.css` sets `ui-sans-serif, system-ui, …` and declares no `@font-face`, so the site itself
 * ships no font: every reader sees it in whatever their platform calls a UI sans. That is a good
 * default for a document and an impossible one for a card, because a card is rasterized here. There
 * is no reader's machine involved — satori has to be handed actual outlines, and "system-ui" is not
 * a thing a build server has.
 *
 * So a card is the one surface on this site with a chosen typeface. Inter, because it is the face
 * `ui-sans-serif` most nearly resolves to on the platforms most readers are on, and because its
 * figures are the same width, which matters when the largest thing on the card is a dollar amount.
 *
 * # Why it is not committed
 *
 * `@fontsource/inter` is a devDependency carrying the Latin subset as `.woff` — 30 KB per weight,
 * already stripped of the scripts this site never sets. Committing a `.ttf` instead would put a
 * binary in a repository that has none, and would put the SIL licence somewhere nobody maintains.
 * The package brings its own.
 *
 * satori accepts `ttf`, `otf` and `woff`, and **not** `woff2` — which is the file beside these in
 * the same directory and the one every other tool wants. Resolving the subpath explicitly rather
 * than globbing the directory is what keeps that mistake from being a silent fallback.
 */

import { readFileSync } from "node:fs";
import { createRequire } from "node:module";

/** Resolved through the export map, so pnpm's non-flat `node_modules` is not walked by hand. */
const require = createRequire(import.meta.url);

/** What satori wants: a name, the bytes, a weight, a style. */
export interface LoadedFont {
  name: string;
  data: Buffer;
  weight: 400 | 600;
  style: "normal";
}

let cached: LoadedFont[] | null = null;

/**
 * The two weights the cards use, read once.
 *
 * Memoized because the card endpoints render roughly a thousand images in one build and every one
 * of them needs these buffers. Reading 60 KB from disk a thousand times is not the end of the
 * world, but it is a thousand syscalls in service of nothing.
 */
export function fonts(): LoadedFont[] {
  if (cached) return cached;
  cached = [
    {
      name: "Inter",
      data: readFileSync(require.resolve("@fontsource/inter/files/inter-latin-400-normal.woff")),
      weight: 400,
      style: "normal",
    },
    {
      name: "Inter",
      data: readFileSync(require.resolve("@fontsource/inter/files/inter-latin-600-normal.woff")),
      weight: 600,
      style: "normal",
    },
  ];
  return cached;
}
