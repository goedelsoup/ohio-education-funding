/**
 * The site mark, read from `public/favicon.svg`.
 *
 * One source for the icon in every size: the SVG is served to browser tabs as-is and rasterized to
 * PNG by the two icon routes. Exporting a PNG by hand and committing it beside the SVG would make
 * their agreement a promise somebody has to keep, and the way that promise breaks — the tab and the
 * home screen showing different marks — is invisible to whoever broke it.
 */

import { readFileSync } from "node:fs";
import { resolve } from "node:path";

/**
 * Resolved against the working directory, matching `src/lib/feed.ts`.
 *
 * `import.meta.url` is the obvious choice and the wrong one for the same reason it is wrong there:
 * Astro bundles this module into the output tree before running it, so at build time the URL points
 * at the build product rather than at the source. The working directory is `web/` under every way
 * this is invoked, and the second candidate covers being run from the repository root.
 */
const CANDIDATES = ["public/favicon.svg", "web/public/favicon.svg"];

let cached: string | null = null;

export function mark(): string {
  if (cached) return cached;
  for (const candidate of CANDIDATES) {
    try {
      cached = readFileSync(resolve(process.cwd(), candidate), "utf8");
      return cached;
    } catch {
      continue;
    }
  }
  throw new Error(
    `Could not read the site mark from any of ${CANDIDATES.join(", ")} relative to ` +
      `${process.cwd()}. The icon routes rasterize it, so it is not optional.`,
  );
}

/**
 * The same drawing, prepared to be inlined into a page rather than served as a file.
 *
 * Two things are removed, and both would be bugs in a document.
 *
 * The XML comment, because it says "edit the generator, not this file" to whoever opens
 * `favicon.svg` — and repeating that in the `<head>`-adjacent markup of 3,493 pages says it to
 * nobody.
 *
 * The `<style>` block, which matters more. It carries the dark face behind
 * `prefers-color-scheme`, which answers the operating system — and this site's rule is that
 * `data-theme` beats the operating system in BOTH directions. Inlined as-is, a reader who chose
 * light on a dark machine would get a light page with a dark mark in the corner of it. Stripped,
 * the fills fall to `app.css`, which styles them from the theme tokens and is therefore right in
 * all three states. Its class selectors would also have leaked into the document, where `.state`
 * and `.key` are names another rule could plausibly want.
 *
 * What is kept is `role="img"` and the label, which is what gives the brand link its accessible
 * name now that it has no text: the same words the link used to spell out.
 */
export function inlineMark(): string {
  const raw = mark();
  return raw.slice(raw.indexOf("<svg")).replace(/[ \t]*<style>[\s\S]*?<\/style>\n?/, "");
}
