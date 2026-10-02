/**
 * The built site, read as files.
 *
 * # Why a suite that reads `dist/` is not an e2e suite
 *
 * Twenty-three assertions in this repository are about the *artefact* rather than about the page:
 * the CSP the host applies and `vite preview` does not, the `og:` tags whose only consumers are
 * unfurlers, `robots.txt`, the MathML a browser has already repaired by the time it is in the DOM.
 * Each of them was written inside the Playwright suite because that is where the built site was
 * available — and each paid for a Chromium launch and a preview server to do what `readFileSync`
 * does.
 *
 * They also could not run under `pnpm test:unit`, which is the half that matters: Playwright shards
 * by file, so every one of them was queued behind the browser tests in the same file.
 *
 * So they live here, in their own Vitest project, and the only thing they need is that something
 * has built the site first.
 *
 * # The ordering hazard, which is the same one `check:dist` documents
 *
 * A suite that reads `dist/` is reading *some* build, and a stale one is the previous commit. This
 * project therefore runs **after** `build`, never as part of `test:unit` — see `web/mise.toml` and
 * the `dist` step in `ci.yml`. The guard below catches the case where nothing has built at all,
 * which is the failure a newcomer hits; it cannot catch a stale build, and neither can
 * `check-dist-links.ts`. Ordering is what holds that, and it is stated in both places that own it.
 */

import { existsSync, readdirSync } from "node:fs";
import { join } from "node:path";

/** The build output, which every assertion in this project reads and none of them writes. */
export const DIST = join(import.meta.dirname, "../../dist");

if (!existsSync(DIST)) {
  throw new Error(
    `No build to read: ${DIST} does not exist. This project reports on the built site — run \`pnpm build\` first, or \`mise run //web:gate\`, which orders it for you.`,
  );
}

/**
 * Every HTML document in the build, absolute paths, depth-first.
 *
 * Eleven copies of this walk were inlined in the tests that moved here, byte for byte identical.
 * They are one function now — a sweep over the build is the shape of nearly every assertion in
 * this project, and the thing each test is actually about is what it looks for in the bytes.
 *
 * Only `.html`: the endpoints that emit JSON, CSV and PNG are checked by the tests that care about
 * them, by name, because "every JSON file in the build" is not a question anything here asks.
 */
export const pages = (dir: string = DIST): string[] =>
  readdirSync(dir, { withFileTypes: true }).flatMap((entry) =>
    entry.isDirectory()
      ? pages(join(dir, entry.name))
      : entry.name.endsWith(".html")
        ? [join(dir, entry.name)]
        : [],
  );

/*
 * # Parsing less than the page
 *
 * A full linkedom parse of every page is most of what this project costs (#646): 17.6s a sweep
 * over the build, and six tests made one each. Most of them read a small part of what they parse,
 * so these two cut the input down to the part read — and each is written so that what the caller
 * reads comes out byte-identical, not merely close. A narrowed sweep that passes because it no
 * longer sees the defect is the failure to fear here, so the tests that use them carry a bite
 * test on a doctored real page as well as a count of what they found.
 */

/**
 * The page up to and including `</head>`: the `<title>` and every `<meta>`, and nothing of the
 * body. Throws on a page with no `</head>` rather than returning a fragment that would read as a
 * page with no title.
 */
export function head(html: string, file = "a page"): string {
  const end = html.indexOf("</head>");
  if (end < 0) throw new Error(`${file} has no </head>`);
  return html.slice(0, end + "</head>".length);
}

/** One tag, with `>` allowed inside a quoted attribute value. */
const TAG = /<(?:[^>"']|"[^"]*"|'[^']*')*>/g;

/**
 * The page with every inline `<svg>` reduced to its text.
 *
 * A chart's marks — the rects, paths and their coordinates — are about half of what a parse of
 * the build spends its time on, and no sweep that reads text reads them. Its text it does read:
 * `body.textContent` carries every axis foot and direct label, and a cell holding a sparkline
 * counts its words. So the tags go and the text stays, in place, and `textContent` of every
 * element that held a drawing is what it was. That holds because no drawing in the build nests
 * another, holds a comment, or carries a `<style>` or `<script>` whose text a sweep would
 * otherwise remove — and `artefact.spec.ts` holds each of those over the build.
 *
 * Not for a sweep that reads a drawing's elements or attributes: those parse the page whole.
 */
export const drawingsAsText = (html: string): string =>
  html.replace(/<svg\b[\s\S]*?<\/svg>/g, (svg) => svg.replace(TAG, ""));
