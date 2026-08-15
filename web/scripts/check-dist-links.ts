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
 *
 * # And no fragment names an id that is not in the page it points at
 *
 * The second half is newer and closes the same hole one level down. A fragment written inline in a
 * template is enumerated by nothing, exactly like the `routes.parameter("state-share-percentage")`
 * call that shipped a 404 on 609 pages — and a fragment fails *worse* than that call did. A bad
 * path 404s and is visible; a bad fragment resolves, serves the right document, and silently
 * leaves the reader at the top of it having been promised a section four cards down.
 *
 * Several sections on these routes are conditional — `#denominators` is absent for 177 of 609
 * districts because the two valuations agree within 5% — so this is not a check that could be done
 * by reading the templates. It has to ask the built page.
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

/** Every `id` in a built page, by the site path that serves it. Filled on the first pass. */
const identifiers = new Map<string, Set<string>>();
/** Every internal fragment link: where it was written, and what it points at. */
const fragments: { from: string; path: string; id: string }[] = [];

/** `dist/district/043786.html` is served as `/district/043786`; `build.format` is "file". */
const servedAs = (file: string): string => "/" + relative(DIST, file).replace(/\.html$/, "");

for (const file of htmlFiles(DIST)) {
  pages += 1;
  const html = readFileSync(file, "utf8");
  const here = servedAs(file);

  const ids = new Set<string>();
  for (const match of html.matchAll(/\sid="([^"]*)"/g)) ids.add(match[1]!);
  identifiers.set(here, ids);

  for (const match of html.matchAll(/\s(?:href|src)="([^"]*)"/g)) {
    const value = match[1]!;
    anchors += 1;
    if (value === "" || ACCEPTABLE.test(value)) continue;
    offenders.push(`  ${relative(DIST, file)}\n    ${value}`);
  }

  for (const match of html.matchAll(/\shref="(\/[^"]*)#([^"]+)"/g)) {
    fragments.push({ from: here, path: match[1]!, id: match[2]! });
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

const dangling = fragments.filter(({ path, id }) => {
  const ids = identifiers.get(path);
  // A path this site does not build is the other check's business, not this one's.
  return ids !== undefined && !ids.has(id);
});

if (dangling.length > 0) {
  console.error(
    `\n${dangling.length} fragment link${dangling.length === 1 ? "" : "s"} in the built site name\n` +
      `an id the target page does not carry.\n\n` +
      `Unlike a bad path this does not 404. The document is served, the browser finds nothing to\n` +
      `scroll to, and the reader lands at the top of the right page with no sign that the section\n` +
      `they were sent to is missing — so nothing but this reports it.\n\n` +
      `Sections on the district routes are conditional: #denominators is absent wherever the two\n` +
      `valuations agree within 5%. A link to one has to be gated on the same predicate its\n` +
      `renderer uses.\n\n` +
      dangling
        .slice(0, 20)
        .map(({ from, path, id }) => `  ${from}\n    → ${path}#${id}`)
        .join("\n") +
      `\n`,
  );
  process.exit(1);
}

console.log(
  `${anchors} links across ${pages} built pages, all absolute or off-site; ` +
    `${fragments.length} fragment links all resolve to an id in the page they name`,
);
