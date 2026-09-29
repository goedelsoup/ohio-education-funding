/**
 * The corpus links back to the data (#551), and the list it does so from is the build's own.
 *
 * `src/lib/appearances.json` is computed from one build and rendered by the next — see
 * `src/lib/appearances.ts` for why. So two things can go wrong, and each has a test: the committed
 * inversion can stop matching the pages it was read off, and a node page can stop rendering it.
 */

import { readFileSync } from "node:fs";
import { join } from "node:path";

import { parseHTML } from "linkedom";
import { expect, test } from "vitest";

import { APPEARANCES_PATH, appearancesOf, builtPages, type Appearances } from "../../src/lib/appearances.ts";
import { exemplarIrns, loadCorpus } from "../../src/lib/corpus.ts";
import { count } from "../../src/lib/format.ts";
import { DIST } from "./artefact.ts";

const committed = JSON.parse(readFileSync(APPEARANCES_PATH, "utf8")) as Appearances;
const built = builtPages(DIST);

test("the committed inversion is the one this build would produce", () => {
  // If this fails, a page outside the corpus changed which nodes it links or what it is called:
  // run `pnpm appearances && pnpm build` after the build and commit the JSON.
  expect(Object.keys(committed).length).toBeGreaterThan(50);
  expect(appearancesOf(built, exemplarIrns())).toEqual(committed);
});

test("the freshness check bites on one real page losing one link", () => {
  // Cleveland's outcome page charts the performance index against comparable-poverty districts and
  // links the metric's node from that card. Take the link out of that one page.
  const file = "district/043786/outcome.html";
  const page = built.find((p) => p.file === file)!;
  const link = /<a href="\/wiki\/metric\/performance-index"[^>]*>([\s\S]*?)<\/a>/g;
  expect(page.html).toMatch(link);
  const doctored = built.map((p) => (p === page ? { file, html: page.html.replace(link, "$1") } : p));

  const before = committed["metric/performance-index"]!.find((place) => place.route === "/district/[irn]/outcome")!;
  const after = appearancesOf(doctored, exemplarIrns())["metric/performance-index"]!.find(
    (place) => place.route === "/district/[irn]/outcome",
  )!;
  expect(after.pages).toBe(before.pages - 1);
  expect(after.example.href).not.toBe("/district/043786/outcome");
});

test("every node page renders its places, and no other node page has the card", () => {
  let rendered = 0;
  const wrong: string[] = [];
  for (const node of loadCorpus().nodes) {
    const html = readFileSync(join(DIST, `wiki/${node.id}.html`), "utf8");
    const { document } = parseHTML(html);
    const card = document.querySelector("#on-the-site");
    const places = committed[node.id];
    if (!places) {
      if (card) wrong.push(`${node.id}: a card with nothing to list`);
      continue;
    }
    rendered += 1;
    if (!card) {
      wrong.push(`${node.id}: no card`);
      continue;
    }
    // Build-time and inert: the list is in the bytes, so it reads the same with scripts off.
    if (card.querySelector("script")) wrong.push(`${node.id}: a script in the card`);
    const rows = [...card.querySelectorAll("tbody tr")];
    const hrefs = rows.map((row) => row.querySelector("th a")?.getAttribute("href"));
    const expected = places.map((place) => place.example.href);
    if (JSON.stringify(hrefs) !== JSON.stringify(expected)) {
      wrong.push(`${node.id}: rows ${JSON.stringify(hrefs)}, places ${JSON.stringify(expected)}`);
    }
    // Every section named is linked where the citation sits.
    for (const [i, place] of places.entries()) {
      const anchors = [...(rows[i]?.querySelectorAll("td a") ?? [])].map((a) => a.getAttribute("href"));
      const want = place.sections.map((section) => `${place.example.href}#${section.id}`);
      if (JSON.stringify(anchors) !== JSON.stringify(want)) {
        wrong.push(`${node.id} ${place.route}: sections ${JSON.stringify(anchors)}`);
      }
    }
  }
  expect(rendered).toBe(Object.keys(committed).length);
  expect(wrong).toEqual([]);
});

test("a count the reader is shown is the count in the manifest", () => {
  // The performance index is linked from nearly every district's outcome page and not all of them.
  const place = committed["metric/performance-index"]!.find((p) => p.route === "/district/[irn]/outcome")!;
  expect(place.pages).toBeLessThan(place.of);
  const html = readFileSync(join(DIST, "wiki/metric/performance-index.html"), "utf8");
  const { document } = parseHTML(html);
  const text = document.querySelector("#on-the-site")!.textContent!.replace(/\s+/g, " ");
  expect(text).toContain(`on ${count(place.pages)} of the ${count(place.of)} pages like this one`);
});
