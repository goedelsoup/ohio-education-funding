/**
 * The typeface the preview cards are set in.
 *
 * # Why font files exist at all, when the site ships almost none
 *
 * The text stacks in `tokens/typography.css` name `ui-sans-serif, system-ui, …` with no
 * `@font-face` behind them, so every reader sees the site in whatever their platform calls a UI
 * sans. That is a good default for a document and an impossible one for a card, because a card is
 * rasterized *here*. There is no reader's machine involved — satori has to be handed actual
 * outlines, and "system-ui" is not a thing a build server has.
 *
 * (The note that used to sit here said the site "ships no font". That stopped being true in #202,
 * which added one subsetted maths face for readers whose platform has none. No TEXT face ships,
 * and that is still the rule these cards are the exception to.)
 *
 * # Two faces, because the site has two voices
 *
 * **Inter** for everything that is a figure, a label or a line of apparatus — it is the face
 * `ui-sans-serif` most nearly resolves to on the platforms most readers are on, and its figures are
 * the same width, which matters when the largest thing on the card is a dollar amount.
 *
 * **Source Serif 4** for the headline, and only the headline. #186 made the site's display face a
 * serif and the cards did not follow, so a card and the page it previews disagreed about what the
 * site looks like — the one place a reader sees both is when they click through from a link they
 * were shown.
 *
 * `--font-serif` is a platform stack: Iowan Old Style, Palatino, Charter, Georgia. None of those
 * can be handed to satori, and Charter is not on npm. So the card takes the nearest available face
 * rather than the same one, and it was chosen by rendering the real headline at the real size in
 * three candidates and looking: Newsreader is narrower and higher-contrast than Iowan, Libre
 * Baskerville is much wider and higher-contrast still, and Source Serif 4 sits closest in colour
 * and width. This is the surface #190 predicted could not match a platform stack; what it can do
 * is agree about the *decision*, which is that a headline is set in an old-style serif.
 *
 * The split follows the site's own rule, from `tokens/typography.css`: the serif is for the display
 * face and the lead, "and for nothing that carries a number, sits in a table, or is drawn inside an
 * `<svg>`". The card's figure is a number, so it stays sans — which is what `.tile .v` does on the
 * page, inheriting the body stack with tabular figures.
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

/**
 * The display face's name, as `card.ts` has to spell it in a `fontFamily`.
 *
 * Exported so the renderer and the loader cannot disagree about it — a `fontFamily` satori has no
 * font for does not fail, it silently falls back to the first one loaded, which would put the
 * headline back in Inter and look exactly like a decision.
 */
export const SERIF = "Source Serif 4";

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
    {
      // 600 only. The headline is the one thing set in it and the site's `h1` is semibold.
      name: SERIF,
      data: readFileSync(
        require.resolve("@fontsource/source-serif-4/files/source-serif-4-latin-600-normal.woff"),
      ),
      weight: 600,
      style: "normal",
    },
  ];
  return cached;
}
