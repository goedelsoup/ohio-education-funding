/**
 * Every Explained topic is reached from the pages where readers meet its words (#717).
 *
 * The bar lists every topic on every page, so a link anywhere in the document proves nothing. What
 * counts is a link inside `main`: a data page's "What is this?" note or glossary definition, and a
 * wiki node's "In plain terms" line.
 */

import { expect, test } from "vitest";

import { builtPages } from "../../src/lib/appearances.ts";
import { TOPICS } from "../../src/lib/explained/registry.ts";
import * as routes from "../../src/lib/routes.ts";
import { DIST } from "./artefact.ts";

const built = builtPages(DIST).map((p) => ({ file: p.file, main: p.html.match(/<main[\s\S]*<\/main>/)?.[0] ?? "" }));

/** The built files whose `main` links a topic. */
function inbound(slug: string, pages = built): string[] {
  const href = new RegExp(`href="${routes.explained(slug)}(?:\\.html)?(?:#[^"]*)?"`);
  return pages.filter((p) => href.test(p.main)).map((p) => p.file);
}

const isWiki = (file: string) => file.startsWith("wiki/");
const isData = (file: string) => !isWiki(file) && !file.startsWith("explained/") && !file.startsWith("explained.");

test("every topic is linked from a data page and from a wiki node", () => {
  const missing = TOPICS.flatMap((t) => {
    const from = inbound(t.slug);
    return [
      ...(from.some(isData) ? [] : [`${t.slug}: no data page`]),
      ...(from.some(isWiki) ? [] : [`${t.slug}: no wiki node`]),
    ];
  });
  expect(missing).toEqual([]);
});

test("a link in the bar alone does not count", () => {
  // The same pages with every `main` emptied: the bar still links every topic, and none counts.
  const bare = built.map((p) => ({ ...p, main: "" }));
  expect(TOPICS.flatMap((t) => inbound(t.slug, bare))).toEqual([]);
});
