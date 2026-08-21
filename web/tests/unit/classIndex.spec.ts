/**
 * Every node appears in its own class's README index.
 *
 * # Why this is a test
 *
 * The corpus keeps two kinds of index. `.yidam/corpus/README.md` holds a generated one — a REGEN
 * block rebuilt by `edfund-connect index`, which CI checks for staleness — and every class
 * directory holds a hand-written one, a table of its instances with a line of description each.
 * The generated index has never been wrong. The hand-written ones drifted quietly: a review found
 * fifteen nodes across seven classes missing from their own class table, including all five
 * biennia of the Bridge decade, in a README that discussed those five biennia at length two
 * sections below the table that omitted them.
 *
 * Nothing could have caught it. The nodes were valid, linked, rendered and indexed; the only thing
 * wrong was that the directory's own front page did not know they existed.
 *
 * # Why a table row and not a mention
 *
 * A class README may cite a node anywhere in its prose, and several do. Being cited is not being
 * indexed — the question this answers is "does the class's index list its members", and the index
 * is the table. `legislation/README.md` carries a second table of vetoed provisions and rows there
 * count too: they are still the README listing the node, and requiring a specific table would mean
 * teaching this test the shape of each one.
 *
 * # Why zero rather than a ratchet
 *
 * Because adding a row costs a line and there is no class of node that legitimately cannot be
 * listed. `legislationCoverage.spec.ts` is a ratchet because most of the acts it names may never
 * be written; this is a zero because nothing stands between a node and its own index.
 */

import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

import { expect, test } from "vitest";

const CORPUS = join(import.meta.dirname, "../../../.yidam/corpus");

/** Every `<class>/<slug>` in the corpus, and the class READMEs to check them against. */
function classes(): { name: string; readme: string; slugs: string[] }[] {
  return readdirSync(CORPUS, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => ({
      name: entry.name,
      readme: readFileSync(join(CORPUS, entry.name, "README.md"), "utf8"),
      slugs: readdirSync(join(CORPUS, entry.name))
        .filter((file) => file.endsWith(".yml"))
        .map((file) => file.replace(/\.yml$/, "")),
    }));
}

/** Whether a README lists a node in one of its tables, as distinct from citing it in prose. */
function listed(readme: string, slug: string): boolean {
  return readme
    .split("\n")
    .some((line) => line.startsWith("|") && line.includes(`(${slug}.yml)`));
}

test("every class README lists every node in its directory", () => {
  const missing: string[] = [];
  for (const { name, readme, slugs } of classes()) {
    for (const slug of slugs) {
      if (!listed(readme, slug)) missing.push(`${name}/${slug}`);
    }
  }
  expect(missing).toEqual([]);
});

test("and every table in one has a header, because a headerless table is not a table", () => {
  // `metric/README.md` split its index across a paragraph and gave the second half no header row,
  // so nine rows rendered as literal pipe characters on GitHub and two of the class's nodes went
  // missing from an index nobody could see was an index. Markdown will not tell you.
  const broken: string[] = [];
  for (const { name, readme } of classes()) {
    const lines = readme.split("\n");
    for (let i = 0; i < lines.length; i += 1) {
      const row = lines[i]!.startsWith("|");
      const previous = i > 0 && lines[i - 1]!.startsWith("|");
      if (!row || previous) continue;
      if (!/^\|[-| :]+\|$/.test(lines[i + 1] ?? "")) {
        broken.push(`${name}/README.md:${i + 1} opens a table with no separator row under it`);
      }
    }
  }
  expect(broken).toEqual([]);
});

test("and lists nothing that is not there, which is the other way a table rots", () => {
  const ghosts: string[] = [];
  for (const { name, readme, slugs } of classes()) {
    const held = new Set(slugs);
    for (const line of readme.split("\n")) {
      if (!line.startsWith("|")) continue;
      for (const [, slug] of line.matchAll(/\(([a-z0-9-]+)\.yml\)/g)) {
        if (!held.has(slug!)) ghosts.push(`${name}/${slug} is listed and does not exist`);
      }
    }
  }
  expect(ghosts).toEqual([]);
});
