/**
 * No page in the built site carries a relative `href`.
 *
 * # Why this runs over `dist/` and not over the route table
 *
 * `tests/unit/links.spec.ts` checks the links this repository *generates*, by asking
 * `resolveTarget` about every target the corpus writes. That is the right check for that layer and
 * it is fast enough to run on every save, but it is blind by construction to any anchor that
 * reaches a page some other way — a hand-written `<a>` in a component, a href built inline in a
 * template, a rewrite that silently stops firing. Forty broken relative hrefs shipped while that
 * suite was green, and the fix that closed the hole in it does not close this one.
 *
 * So this asks the opposite question, of the actual artefact: given the HTML that will be
 * uploaded, is there anything in it that a static host cannot serve? Every route on this site is
 * absolute from the root and `build.format` is `"file"`, so a relative href is never correct here
 * — `<a href="ocg-white-paper-013.md">` on `/wiki/source/x` resolves to
 * `/wiki/source/ocg-white-paper-013.md`, which is a 404 that looks in the markup like a link.
 *
 * Runs after `build` rather than in the unit suite, which runs before it: scanning a stale `dist/`
 * would report on the previous commit's output and is worse than not scanning at all.
 */

import { readFileSync, readdirSync } from "node:fs";
import { join, relative, resolve } from "node:path";

const DIST = resolve(process.cwd(), "dist");

/** Absolute from the root, off the site, or a fragment. Anything else resolves against the page. */
const ACCEPTABLE = /^(\/|https?:|mailto:|tel:|#|data:)/;

function* htmlFiles(dir: string): Generator<string> {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) yield* htmlFiles(path);
    else if (entry.name.endsWith(".html")) yield path;
  }
}

const offenders: string[] = [];
let pages = 0;
let anchors = 0;

for (const file of htmlFiles(DIST)) {
  pages += 1;
  const html = readFileSync(file, "utf8");
  for (const match of html.matchAll(/\s(?:href|src)="([^"]*)"/g)) {
    const value = match[1]!;
    anchors += 1;
    if (value === "" || ACCEPTABLE.test(value)) continue;
    offenders.push(`  ${relative(DIST, file)}\n    ${value}`);
  }
}

if (offenders.length > 0) {
  console.error(
    `\n${offenders.length} relative link${offenders.length === 1 ? "" : "s"} in the built site.\n\n` +
      `Each one resolves against the page it is on rather than against the site root, which on a\n` +
      `static host is a 404 that looks exactly like a working link in the markup.\n\n` +
      offenders.join("\n") +
      `\n\nIf a corpus target is behind this, the shapes resolveTarget places are listed above it\n` +
      `in src/lib/corpus.ts. If a template is, the routes belong in src/lib/routes.ts.\n`,
  );
  process.exit(1);
}

console.log(`${anchors} links across ${pages} built pages, all absolute or off-site`);
