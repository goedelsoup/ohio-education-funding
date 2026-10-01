/**
 * Every page two levels deep says where it sits, the same way (#594).
 *
 * Seven Library templates put a `p.crumb` above the `h1`. County, House and Senate pages put their
 * up-link in a note under the lede; a district's four tabs and `/scenario/reach` had none at all;
 * and at 375px the header shows only `Menu`, so on a phone those pages named no section anywhere.
 *
 * The rule, read off the build so that a template added later is held to it too: every page whose
 * address has two segments or more opens with a `.crumb` before its `h1`, and the crumb's last link
 * is the page's parent.
 *
 * # The parent
 *
 * The address with its last segment dropped — `/wiki/source/<slug>` under `/wiki/source`, a
 * district's tab under its dashboard. Two families have no page at that address: there is no
 * `/county` and no `/district`, and their parents are the lists a reader reaches them from.
 */

import { existsSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";

import { parseHTML } from "linkedom";
import { expect, test } from "vitest";

import { DIST, pages } from "./artefact.ts";

/** The two families whose parent is not the address above them. */
const LISTED_AT: Record<string, string> = { "/county": "/counties", "/district": "/districts" };

const deep = pages()
  .map((file) => ({ file, path: `/${relative(DIST, file).replace(/\.html$/, "")}` }))
  .filter(({ path }) => path.split("/").length > 2);

function parentOf(path: string): string {
  const up = path.slice(0, path.lastIndexOf("/"));
  if (existsSync(join(DIST, `${up}.html`))) return up;
  const listed = LISTED_AT[up];
  if (!listed) throw new Error(`${path}: no page at ${up} and no list it is reached from`);
  return listed;
}

test("the sweep reaches every family", () => {
  // A walk that stopped finding district tabs would pass on the rest.
  const families = new Set(deep.map(({ path }) => path.split("/")[1]));
  for (const family of ["county", "district", "house", "senate", "scenario", "wiki"]) {
    expect(families, family).toContain(family);
  }
  expect(deep.length).toBeGreaterThan(2_500);
});

test("every page two levels deep has a crumb whose last link is its parent", () => {
  const faults: string[] = [];
  for (const { file, path } of deep) {
    const { document } = parseHTML(readFileSync(file, "utf8"));
    const crumb = document.querySelector("main .crumb");
    const h1 = document.querySelector("main h1");
    if (!crumb) {
      faults.push(`${path}: no crumb`);
      continue;
    }
    // `compareDocumentPosition` 4 is FOLLOWING: the heading comes after the crumb.
    if (!h1 || !(crumb.compareDocumentPosition(h1) & 4)) faults.push(`${path}: crumb is not above the h1`);
    const links = crumb.querySelectorAll("a");
    const last = links[links.length - 1]?.getAttribute("href");
    const want = parentOf(path);
    if (last !== want) faults.push(`${path}: crumb ends at ${last ?? "nothing"}, parent is ${want}`);
  }
  expect(faults.slice(0, 20)).toEqual([]);
});
